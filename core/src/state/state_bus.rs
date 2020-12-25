use super::base::{StateData, StateDataStatic};
use super::state_pool::{StatePool, StatePoolItem};
use crate::id::{ClassID, FastObjID};
use anyhow::{anyhow, Result};
use std::cell::{RefCell, UnsafeCell};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ptr::{self, NonNull};
use std::rc::Rc;

//
// State Reference
//

#[derive(Debug)]
pub(crate) struct StateRefInner {
    fobj_id: FastObjID,
    class_id: ClassID,
    state: *mut u8,
    binder: StateBinder,
}

#[derive(Debug)]
pub struct StateRef<S>
where
    S: StateData + StateDataStatic,
{
    inner: Box<UnsafeCell<StateRefInner>>,
    phantom: PhantomData<S>,
}

impl<S> !Sync for StateRef<S> {}
impl<S> !Send for StateRef<S> {}

impl<S> Drop for StateRef<S>
where
    S: StateData + StateDataStatic,
{
    fn drop(&mut self) {
        if self.inner_ref().binder.is_valid() {
            self.inner_ref().binder.stop_ref(self);
            self.inner_ref().binder = StateBinder::invalid();
        }
        self.inner_ref().fobj_id = FastObjID::invalid();
        self.inner_ref().state = ptr::null_mut();
    }
}

impl<S> StateRef<S>
where
    S: StateData + StateDataStatic,
{
    pub fn invaild() -> StateRef<S> {
        return StateRef::new();
    }

    pub fn new() -> StateRef<S> {
        return StateRef {
            inner: Box::new(UnsafeCell::new(StateRefInner {
                fobj_id: FastObjID::invalid(),
                class_id: S::id(),
                state: ptr::null_mut(),
                binder: StateBinder::invalid(),
            })),
            phantom: PhantomData,
        };
    }

    pub fn start(&self, fobj_id: FastObjID, binder: StateBinder) -> Result<()> {
        if self.inner_ref().binder.is_valid() {
            return Err(anyhow!("Start twice"));
        }
        if fobj_id.is_invalid() {
            return Err(anyhow!("Invaild FastObjID"));
        }
        if binder.is_invalid() {
            return Err(anyhow!("Invaild StateBinder"));
        }
        self.inner_ref().fobj_id = fobj_id;
        self.inner_ref().binder = binder;
        self.inner_ref().binder.start_ref(self)?;
        return Ok(());
    }

    pub fn new_and_start(fobj_id: FastObjID, binder: StateBinder) -> Result<StateRef<S>> {
        if fobj_id.is_invalid() {
            return Err(anyhow!("Invaild FastObjID"));
        }
        if binder.is_invalid() {
            return Err(anyhow!("Invaild StateBinder"));
        }
        let re = StateRef {
            inner: Box::new(UnsafeCell::new(StateRefInner {
                fobj_id,
                class_id: S::id(),
                state: ptr::null_mut(),
                binder: binder,
            })),
            phantom: PhantomData,
        };
        re.inner_ref().binder.start_ref(&re)?;
        return Ok(re);
    }

    #[inline]
    pub fn fobj_id(&self) -> FastObjID {
        return self.inner_ref().fobj_id;
    }

    #[inline]
    pub fn class_id(&self) -> ClassID {
        return self.inner_ref().class_id;
    }

    #[inline]
    pub fn is_vaild(&self) -> bool {
        return self.inner_ref().fobj_id.is_valid()
            && self.inner_ref().binder.is_valid();
    }

    #[inline]
    pub fn is_invaild(&self) -> bool {
        return self.inner_ref().fobj_id.is_invalid()
            && self.inner_ref().binder.is_invalid();
    }

    #[inline]
    pub fn is_null(&self) -> bool {
        return self.inner_ref().state.is_null();
    }

    #[inline]
    pub fn state(&self) -> Result<&S> {
        if self.inner_ref().state.is_null() {
            return Err(anyhow!("State is null"));
        }
        return Ok(unsafe { &*(self.inner_ref().state as *const S) });
    }

    #[inline]
    fn inner_ptr(&self) -> NonNull<StateRefInner> {
        return unsafe { NonNull::new_unchecked(self.inner.get()) };
    }

    #[inline]
    fn inner_ref(&self) -> &mut StateRefInner {
        return unsafe { &mut *self.inner.get() };
    }
}

//
// State Bus
//

#[derive(Debug, PartialEq)]
pub(crate) enum InnerItem {
    Single(NonNull<StateRefInner>),
    Multi(Vec<NonNull<StateRefInner>>),
}

pub(crate) type StopInners = Vec<(FastObjID, *mut StateRefInner)>;

#[derive(Debug)]
struct StateBusInner {
    inners: HashMap<FastObjID, InnerItem>,
    pool: Option<Box<StatePool>>,
}

#[derive(Debug)]
pub struct StateBus {
    bus: Rc<RefCell<StateBusInner>>,
}

impl !Sync for StateBus {}
impl !Send for StateBus {}

impl StateBus {
    pub fn new() -> StateBus {
        return StateBus {
            bus: Rc::new(RefCell::new(StateBusInner {
                inners: HashMap::with_capacity(1024),
                pool: None,
            })),
        };
    }

    #[inline]
    pub fn refs_count(&self) -> usize {
        return self.bus.borrow().inners.len();
    }

    #[inline]
    pub fn new_binder(&self) -> StateBinder {
        return StateBinder::new(self.bus.clone());
    }

    fn start_ref<S>(&self, re: &StateRef<S>) -> Result<()>
    where
        S: StateData + StateDataStatic,
    {
        if re.fobj_id().is_invalid() {
            return Err(anyhow!("Invaild FastObjID"));
        }

        let inner = re.inner_ptr();
        let fobj_id = re.fobj_id();
        let inners = &mut self.bus.borrow_mut().inners;

        if let Some(value) = inners.get_mut(&fobj_id) {
            match value {
                InnerItem::Single(single) => {
                    let mut multi = Vec::with_capacity(8);
                    multi.push(*single);
                    multi.push(inner);
                    inners.insert(fobj_id, InnerItem::Multi(multi));
                }
                InnerItem::Multi(multi) => {
                    multi.push(inner);
                }
            };
        } else {
            inners.insert(fobj_id, InnerItem::Single(inner));
        }

        return Ok(());
    }

    fn stop_ref<S>(&self, re: &StateRef<S>)
    where
        S: StateData + StateDataStatic,
    {
        let inner = re.inner_ptr();
        let fobj_id = re.fobj_id();
        let inners = &mut self.bus.borrow_mut().inners;

        let mut remove = false;
        if let Some(value) = inners.get_mut(&fobj_id) {
            match value {
                InnerItem::Single(single) => {
                    remove = *single == inner;
                }
                InnerItem::Multi(multi) => {
                    if let Some(pos) = multi.iter().position(|x| *x == inner) {
                        multi.remove(pos);
                    }
                    remove = multi.is_empty();
                }
            };
        }
        if remove {
            inners.remove(&fobj_id);
        }
    }

    pub fn dispatch_states(&self, pool: Box<StatePool>) {
        let mut bus = self.bus.borrow_mut();
        bus.pool = None;
        Self::clear_all_states(&mut *bus);
        pool.for_each(|_, state| Self::update_state(&mut *bus, state));
        bus.pool = Some(pool);
    }

    fn update_state(bus: &mut StateBusInner, item: &StatePoolItem) {
        fn compare(inner: &StateRefInner, item: &StatePoolItem) -> bool {
            return inner.class_id == item.class_id && inner.fobj_id == item.fobj_id;
        }

        if let Some(value) = bus.inners.get_mut(&item.fobj_id) {
            match value {
                InnerItem::Single(mut single) => unsafe {
                    if compare(single.as_ref(), item) {
                        single.as_mut().state = item.state;
                    }
                },
                InnerItem::Multi(multi) => unsafe {
                    for iter in multi {
                        if compare(iter.as_ref(), item) {
                            iter.as_mut().state = item.state;
                        }
                    }
                },
            };
        }
    }

    fn clear_all_states(bus: &mut StateBusInner) {
        for (_, value) in bus.inners.iter_mut() {
            match value {
                InnerItem::Single(mut single) => {
                    unsafe { single.as_mut().state = ptr::null_mut() };
                }
                InnerItem::Multi(multi) => {
                    for idx in 0..multi.len() {
                        unsafe { multi[idx].as_mut().state = ptr::null_mut() };
                    }
                }
            };
        }
    }
}

//
// State Binder
//

#[derive(Debug, Default, Clone)]
pub struct StateBinder(Option<Rc<RefCell<StateBusInner>>>);

impl !Sync for StateBinder {}
impl !Send for StateBinder {}

impl StateBinder {
    #[inline]
    fn new(bus: Rc<RefCell<StateBusInner>>) -> StateBinder {
        return StateBinder(Some(bus));
    }

    #[inline]
    pub fn invalid() -> StateBinder {
        return StateBinder(None);
    }

    #[inline]
    pub fn is_valid(&self) -> bool {
        return self.0.is_some();
    }

    #[inline]
    pub fn is_invalid(&self) -> bool {
        return self.0.is_none();
    }

    #[inline]
    fn start_ref<S>(&self, re: &StateRef<S>) -> Result<()>
    where
        S: StateData + StateDataStatic,
    {
        return match self.0.clone() {
            Some(bus) => StateBus { bus }.start_ref(re),
            None => Err(anyhow!("Invaild BinderRef")),
        };
    }

    #[inline]
    fn stop_ref<S>(&self, re: &StateRef<S>)
    where
        S: StateData + StateDataStatic,
    {
        if let Some(bus) = self.0.clone() {
            StateBus { bus }.stop_ref(re);
        }
    }
}

//
// tests
//

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::StateLifecycle;
    use crate::util::mut_ptr;
    use derive::StateDataX;

    #[derive(StateDataX, Debug, Default, PartialEq)]
    #[class_id(StageGeneral)]
    struct DataTest {
        fobj_id: FastObjID,
        lifecycle: StateLifecycle,
        num: u32,
        text: String,
    }

    #[test]
    fn test_state_ref() {
        let bus = StateBus::new();

        let fobj_id = FastObjID::from(1234);
        let re = StateRef::<DataTest>::new_and_start(fobj_id, bus.new_binder()).unwrap();
        assert_eq!(re.fobj_id(), fobj_id);
        assert!(re.state().is_err());

        let mut state = DataTest {
            num: 0xABCD,
            text: String::from("HaHa"),
            ..DataTest::default()
        };
        re.inner_ref().state = mut_ptr(&mut state);
        assert_eq!(re.is_null(), false);
        assert_eq!(re.state().unwrap().num, 0xABCD);
        assert_eq!(re.state().unwrap().text, String::from("HaHa"));
    }

    #[test]
    fn test_state_bus_register() {
        let bus = StateBus::new();
        {
            let re1 = StateRef::<DataTest>::new_and_start(FastObjID::from(123), bus.new_binder()).unwrap();
            let re2 = StateRef::<DataTest>::new_and_start(FastObjID::from(456), bus.new_binder()).unwrap();
            let re3 = StateRef::<DataTest>::new_and_start(FastObjID::from(456), bus.new_binder()).unwrap();

            assert_eq!(
                bus.bus.borrow().inners[&re1.fobj_id()],
                InnerItem::Single(re1.inner_ptr())
            );
            assert_eq!(
                bus.bus.borrow().inners[&re2.fobj_id()],
                InnerItem::Multi(vec![re2.inner_ptr(), re3.inner_ptr()]),
            );

            bus.stop_ref(&re1);
            assert!(bus.bus.borrow().inners.get(&re1.fobj_id()).is_none());

            bus.stop_ref(&re2);
            assert_eq!(
                bus.bus.borrow().inners[&re3.fobj_id()],
                InnerItem::Multi(vec![re3.inner_ptr()]),
            );
        }
        assert!(bus.bus.borrow().inners.get(&FastObjID::from(456)).is_none());
    }

    #[test]
    fn test_state_bus_update() {
        let mut bus = StateBus::new();

        let re1 = StateRef::<DataTest>::new_and_start(FastObjID::from(123), bus.new_binder()).unwrap();
        let re2 = StateRef::<DataTest>::new_and_start(FastObjID::from(456), bus.new_binder()).unwrap();
        let re3 = StateRef::<DataTest>::new_and_start(FastObjID::from(456), bus.new_binder()).unwrap();

        let mut state = DataTest {
            fobj_id: FastObjID::invalid(),
            lifecycle: StateLifecycle::Updated,
            num: 0xABCD,
            text: String::from("HaHa"),
        };

        let mut item = StatePoolItem {
            state: mut_ptr(&mut state),
            vtable: ptr::null_mut(),
            class_id: DataTest::id(),
            fobj_id: FastObjID::invalid(),
            lifecycle: StateLifecycle::Updated,
        };
        StateBus::update_state(Rc::get_mut(&mut bus.bus).unwrap().get_mut(), &item);
        assert!(re1.state().is_err());
        assert!(re3.state().is_err());

        item.fobj_id = FastObjID::from(123);
        StateBus::update_state(Rc::get_mut(&mut bus.bus).unwrap().get_mut(), &item);
        assert_eq!(re1.state().unwrap().num, 0xABCD);

        item.fobj_id = FastObjID::from(456);
        StateBus::update_state(Rc::get_mut(&mut bus.bus).unwrap().get_mut(), &item);
        assert_eq!(re2.state().unwrap().num, 0xABCD);
        assert_eq!(re3.state().unwrap().num, 0xABCD);

        StateBus::clear_all_states(Rc::get_mut(&mut bus.bus).unwrap().get_mut());
        assert!(re1.state().is_err());
        assert!(re2.state().is_err());
        assert!(re3.state().is_err());
    }
}
