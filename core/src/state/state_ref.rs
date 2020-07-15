use super::state_pool::StatePoolItem;
use super::{StateData, StateDataStatic, StatePool};
use crate::id::{ClassID, ObjID};
use crate::util::make_err;
use failure::Error;
use std::cell::{RefCell, UnsafeCell};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ptr;
use std::rc::Rc;

//
// State Reference
//

#[derive(Clone, Copy, Debug)]
pub struct RefInner {
    obj_id: ObjID,
    class_id: ClassID,
    reg_flag: bool,
    state: *mut u8,
}

impl Default for RefInner {
    fn default() -> Self {
        return RefInner {
            obj_id: ObjID::invaild(),
            class_id: ClassID::invaild(),
            reg_flag: false,
            state: ptr::null_mut(),
        };
    }
}

#[derive(Debug, Default)]
pub struct StateRef<S, R = StateLocalReg<S>>
where
    S: StateData + StateDataStatic,
    R: StateReg<S>,
{
    reg: R,
    inner: UnsafeCell<RefInner>,
    phantom: PhantomData<S>,
}

impl<S, R> !Sync for StateRef<S, R> {}
impl<S, R> !Send for StateRef<S, R> {}

impl<S, R> StateRef<S, R>
where
    S: StateData + StateDataStatic,
    R: StateReg<S>,
{
    pub fn new(obj_id: ObjID, reg: R) -> StateRef<S, R> {
        return StateRef {
            reg,
            inner: UnsafeCell::new(RefInner {
                obj_id,
                class_id: S::id(),
                reg_flag: false,
                state: ptr::null_mut(),
            }),
            phantom: PhantomData,
        };
    }

    pub fn change_reg(&mut self, reg: R) -> Result<(), Error> {
        if self.inner_ref().reg_flag {
            return make_err("StateRef::change_reg() => registered");
        }
        self.reg = reg;
        return Ok(());
    }

    pub fn class_id(&self) -> ClassID {
        return self.inner_ref().class_id;
    }

    pub fn obj_id(&self) -> ObjID {
        return self.inner_ref().obj_id;
    }

    pub fn is_empty(&self) -> bool {
        return self.inner_ref().state.is_null();
    }

    pub fn register(&self) -> Result<(), Error> {
        if !self.inner_ref().reg_flag {
            self.inner_ref().reg_flag = true;
            return self.reg.register(self);
        } else {
            return make_err("StateRef::register()");
        }
    }

    pub fn unregister(&self) -> Result<(), Error> {
        if self.inner_ref().reg_flag {
            self.inner_ref().reg_flag = false;
            return self.reg.unregister(self);
        } else {
            return make_err("StateRef::unregister()");
        }
    }

    pub fn state(&self) -> Result<&S, Error> {
        let state = self.inner_ref().state;
        if !state.is_null() {
            return Ok(unsafe { &*(state as *const S) });
        } else {
            return make_err("StateRef::state() => Empty state");
        }
    }

    fn inner_ptr(&self) -> *mut RefInner {
        return self.inner.get();
    }

    fn inner_ref(&self) -> &mut RefInner {
        return unsafe { &mut *self.inner_ptr() };
    }
}

impl<S, R> Drop for StateRef<S, R>
where
    S: StateData + StateDataStatic,
    R: StateReg<S>,
{
    fn drop(&mut self) {
        if self.inner_ref().reg_flag {
            self.inner_ref().reg_flag = false;
            let _ = self.reg.unregister(self);
        }
        self.inner_ref().obj_id = ObjID::invaild();
        self.inner_ref().class_id = ClassID::invaild();
        self.inner_ref().state = ptr::null_mut();
    }
}

//
// State Register
//

pub trait StateReg<S>
where
    S: StateData + StateDataStatic,
{
    fn register<R>(&self, refer: &StateRef<S, R>) -> Result<(), Error>
    where
        R: StateReg<S>;
    fn unregister<R>(&self, refer: &StateRef<S, R>) -> Result<(), Error>
    where
        R: StateReg<S>;
}

#[derive(Debug)]
pub struct StateLocalReg<S>
where
    S: StateData + StateDataStatic,
{
    bus: Rc<RefCell<StateBus>>,
    phantom: PhantomData<S>,
}

impl<S> StateLocalReg<S>
where
    S: StateData + StateDataStatic,
{
    pub fn new(bus: Rc<RefCell<StateBus>>) -> StateLocalReg<S> {
        return StateLocalReg {
            bus,
            phantom: PhantomData,
        };
    }
}

impl<S> StateReg<S> for StateLocalReg<S>
where
    S: StateData + StateDataStatic,
{
    fn register<R: StateReg<S>>(&self, refer: &StateRef<S, R>) -> Result<(), Error> {
        return self.bus.try_borrow_mut()?.register(refer);
    }

    fn unregister<R: StateReg<S>>(&self, refer: &StateRef<S, R>) -> Result<(), Error> {
        return self.bus.try_borrow_mut()?.unregister(refer);
    }
}

//
// State Iterator
//

// #[derive(Debug)]
// pub struct StateIter<'t> {
//     inner: &'t RefInner,
// }
//
// impl StateIter<'_> {
//     pub fn obj_id(&self) -> ObjID {
//         return self.inner_ref().obj_id;
//     }
//
//     pub fn is_empty(&self) -> bool {
//         return self.inner_ref().state.is_null();
//     }
//
//     pub fn state<S>(&self) -> Result<&S, Error>
//     where
//         S: StateData,
//     {
//         let state = self.inner.state;
//         if !state.is_null() {
//             return Ok(unsafe { &*(state as *const S) });
//         } else {
//             return Err(format_err!("Empty state"));
//         }
//     }
// }

//
// State Bus
//

#[derive(Debug, PartialEq)]
pub enum RefsMapValue {
    Single(*mut RefInner),
    Multi(Vec<*mut RefInner>),
}

#[derive(Debug)]
pub struct StateBus {
    pub refers: HashMap<ObjID, RefsMapValue>,
    pool: Option<Box<StatePool>>,
}

impl StateBus {
    pub fn new() -> StateBus {
        return StateBus {
            refers: HashMap::with_capacity(1024),
            pool: None,
        };
    }

    pub fn refers_count(&self) -> usize {
        return self.refers.len();
    }

    pub fn register<S, R>(&mut self, refer: &StateRef<S, R>) -> Result<(), Error>
    where
        S: StateData + StateDataStatic,
        R: StateReg<S>,
    {
        if refer.class_id().is_invaild() {
            return make_err("Invaild Object ID");
        }
        if refer.obj_id().is_invaild() {
            return make_err("Invaild Object ID");
        }

        let inner = refer.inner_ptr();
        let obj_id = refer.obj_id();
        if let Some(value) = self.refers.get_mut(&obj_id) {
            match value {
                RefsMapValue::Single(single) => {
                    let mut multi = Vec::with_capacity(8);
                    multi.push(*single);
                    multi.push(inner);
                    self.refers.insert(obj_id, RefsMapValue::Multi(multi));
                }
                RefsMapValue::Multi(multi) => {
                    multi.push(inner);
                }
            };
        } else {
            self.refers.insert(obj_id, RefsMapValue::Single(inner));
        }
        return Ok(());
    }

    pub fn unregister<S, R>(&mut self, refer: &StateRef<S, R>) -> Result<(), Error>
    where
        S: StateData + StateDataStatic,
        R: StateReg<S>,
    {
        let inner = refer.inner_ptr();
        let obj_id = refer.obj_id();
        let mut remove = false;
        if let Some(value) = self.refers.get_mut(&obj_id) {
            match value {
                RefsMapValue::Single(single) => {
                    remove = *single == inner;
                }
                RefsMapValue::Multi(multi) => {
                    multi.remove_item(&inner);
                    remove = multi.is_empty();
                }
            };
        }
        if remove {
            self.refers.remove(&obj_id);
        }
        return Ok(());
    }

    pub fn dispatch(&mut self, pool: Box<StatePool>) {
        self.pool = None;
        self.clear_all();
        pool.for_each(|_, state| self.update(state));
        self.pool = Some(pool);
    }

    fn update(&mut self, item: &StatePoolItem) {
        fn compare(inner: &RefInner, item: &StatePoolItem) -> bool {
            return inner.class_id == item.class_id && inner.obj_id == item.obj_id;
        }

        if let Some(value) = self.refers.get_mut(&item.obj_id) {
            match value {
                RefsMapValue::Single(mut single) => unsafe {
                    if compare(&*single, item) {
                        (*single).state = item.state;
                    }
                },
                RefsMapValue::Multi(multi) => unsafe {
                    for iter in multi {
                        if compare(&**iter, item) {
                            (**iter).state = item.state;
                        }
                    }
                },
            };
        }
    }

    fn clear_all(&mut self) {
        for (_, value) in self.refers.iter_mut() {
            match value {
                RefsMapValue::Single(mut single) => {
                    unsafe { (*single).state = ptr::null_mut() };
                }
                RefsMapValue::Multi(multi) => {
                    for idx in 0..multi.len() {
                        unsafe { (*multi[idx]).state = ptr::null_mut() };
                    }
                }
            };
        }
    }
}

//
// tests
//

#[cfg(test)]
mod tests {
    use super::*;
    use crate as core;
    use crate::derive::StateDataX;
    use crate::id::CLASS_STAGE;
    use crate::state::StateLifecycle;
    use crate::util::mut_ptr;

    #[derive(StateDataX, Debug, Default, PartialEq)]
    #[class_id(CLASS_STAGE)]
    struct DataTest {
        obj_id: ObjID,
        lifecycle: StateLifecycle,
        num: u32,
        text: String,
    }

    #[test]
    fn test_state_ref() {
        let sb = Rc::new(RefCell::new(StateBus::new()));

        let obj_id = ObjID::from(1234);
        let refer = StateRef::<DataTest>::new(obj_id, StateLocalReg::new(sb.clone()));
        assert_eq!(refer.obj_id(), obj_id);
        assert_eq!(refer.is_empty(), true);
        assert!(refer.state().is_err());

        let mut state = DataTest {
            num: 0xABCD,
            text: String::from("HaHa"),
            ..DataTest::default()
        };
        refer.inner_ref().state = mut_ptr(&mut state);
        assert_eq!(refer.is_empty(), false);
        assert_eq!(refer.state().unwrap().num, 0xABCD);
        assert_eq!(refer.state().unwrap().text, String::from("HaHa"));
    }

    #[test]
    fn test_state_bus_register() {
        let sb = Rc::new(RefCell::new(StateBus::new()));
        {
            let re1 = StateRef::<DataTest>::new(ObjID::from(123), StateLocalReg::new(sb.clone()));
            let re2 = StateRef::<DataTest>::new(ObjID::from(456), StateLocalReg::new(sb.clone()));
            let re3 = StateRef::<DataTest>::new(ObjID::from(456), StateLocalReg::new(sb.clone()));

            re1.register().unwrap();
            re2.register().unwrap();
            re3.register().unwrap();
            assert_eq!(
                sb.borrow().refers[&re1.obj_id()],
                RefsMapValue::Single(re1.inner_ptr())
            );
            assert_eq!(
                sb.borrow().refers[&re2.obj_id()],
                RefsMapValue::Multi(vec![re2.inner_ptr(), re3.inner_ptr()]),
            );

            sb.borrow_mut().unregister(&re1).unwrap();
            assert!(sb.borrow().refers.get(&re1.obj_id()).is_none());

            sb.borrow_mut().unregister(&re2).unwrap();
            assert_eq!(
                sb.borrow().refers[&re3.obj_id()],
                RefsMapValue::Multi(vec![re3.inner_ptr()]),
            );
        }
        assert!(sb.borrow().refers.get(&ObjID::from(456)).is_none());
    }

    #[test]
    fn test_state_bus_update() {
        let sb = Rc::new(RefCell::new(StateBus::new()));

        let re1 = StateRef::<DataTest>::new(ObjID::from(123), StateLocalReg::new(sb.clone()));
        let re2 = StateRef::<DataTest>::new(ObjID::from(456), StateLocalReg::new(sb.clone()));
        let re3 = StateRef::<DataTest>::new(ObjID::from(456), StateLocalReg::new(sb.clone()));

        re1.register().unwrap();
        re2.register().unwrap();
        re3.register().unwrap();

        let mut state = DataTest {
            obj_id: ObjID::invaild(),
            lifecycle: StateLifecycle::Updated,
            num: 0xABCD,
            text: String::from("HaHa"),
        };

        let mut item = StatePoolItem {
            state: mut_ptr(&mut state),
            vtable: ptr::null_mut(),
            class_id: DataTest::id(),
            obj_id: ObjID::invaild(),
            lifecycle: StateLifecycle::Updated,
        };
        sb.borrow_mut().update(&item);
        assert!(re1.state().is_err());
        assert!(re3.state().is_err());

        item.obj_id = ObjID::from(123);
        sb.borrow_mut().update(&item);
        assert_eq!(re1.state().unwrap().num, 0xABCD);

        item.obj_id = ObjID::from(456);
        sb.borrow_mut().update(&item);
        assert_eq!(re2.state().unwrap().num, 0xABCD);
        assert_eq!(re3.state().unwrap().num, 0xABCD);

        sb.borrow_mut().clear_all();
        assert!(re1.state().is_err());
        assert!(re2.state().is_err());
        assert!(re3.state().is_err());
    }
}
