use super::{StateData, StatePool, StateDataHeader};
use crate::id::{ObjID, TypeID};
use failure::{format_err, Error};
use std::cell::{RefCell, UnsafeCell};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ptr;
use crate::state::StateDataStatic;

//
// State Reference
//

#[derive(Clone, Copy, Debug)]
pub struct RefInner {
    obj_id: ObjID,
    type_id: TypeID,
    state: *mut StateDataHeader,
}

impl Default for RefInner {
    fn default() -> Self {
        return RefInner {
            obj_id: ObjID::invaild(),
            type_id: TypeID::invaild(),
            state: ptr::null_mut(),
        };
    }
}

#[derive(Debug, Default)]
pub struct StateRef<S>
where
    S: StateData + StateDataStatic,
{
    inner: UnsafeCell<RefInner>,
    phantom: PhantomData<S>,
}

impl<S> !Sync for StateRef<S> {}
impl<S> !Send for StateRef<S> {}

impl<S> StateRef<S>
where
    S: StateData + StateDataStatic,
{
    pub fn new(obj_id: ObjID) -> StateRef<S> {
        return StateRef {
            inner: UnsafeCell::new(RefInner {
                obj_id,
                type_id: S::id(),
                state: ptr::null_mut(),
            }),
            phantom: PhantomData,
        };
    }

    pub fn type_id(&self) -> TypeID {
        return self.inner_ref().type_id;
    }

    pub fn obj_id(&self) -> ObjID {
        return self.inner_ref().obj_id;
    }

    pub fn is_empty(&self) -> bool {
        return self.inner_ref().state.is_null();
    }

    pub fn state(&self) -> Result<&S, Error> {
        let state = self.inner_ref().state;
        if !state.is_null() {
            return Ok(unsafe { &*(state as *const S) });
        } else {
            return Err(format_err!("Empty state"));
        }
    }

    fn inner_ptr(&self) -> *mut RefInner {
        return self.inner.get();
    }

    fn inner_ref(&self) -> &mut RefInner {
        return unsafe { &mut *self.inner_ptr() };
    }
}

impl<S> Drop for StateRef<S>
where
    S: StateData + StateDataStatic,
{
    fn drop(&mut self) {
        self.inner_ref().obj_id = ObjID::invaild();
        self.inner_ref().type_id = TypeID::invaild();
        self.inner_ref().state = ptr::null_mut();
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
// State Binder
//

#[derive(Debug, PartialEq)]
enum RefsMapValue {
    Single(*mut RefInner),
    Multi(Vec<*mut RefInner>),
}

#[derive(Debug)]
pub struct StateBinder {
    refers: HashMap<ObjID, RefsMapValue>,
    pool: Option<Box<StatePool>>,
}

impl StateBinder {
    pub fn new() -> StateBinder {
        return StateBinder {
            refers: HashMap::with_capacity(1024),
            pool: None,
        };
    }

    pub fn refers_count(&self) -> usize {
        return self.refers.len();
    }

    pub fn register<S>(&mut self, refer: &StateRef<S>) -> Result<(), Error>
    where
        S: StateData + StateDataStatic,
    {
        if refer.type_id().is_invaild() {
            return Err(format_err!("Invaild Object ID"));
        }
        if refer.obj_id().is_invaild() {
            return Err(format_err!("Invaild Object ID"));
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

    pub fn unregister<S>(&mut self, refer: &StateRef<S>)
    where
        S: StateData + StateDataStatic,
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
    }

    pub fn dispatch(&mut self, pool: Box<StatePool>) {
        self.pool = None;
        self.clear_all();
        pool.for_each(|state| self.update(state));
        self.pool = Some(pool);
    }

    fn update(&mut self, state: *mut StateDataHeader) {
        fn compare(inner: &RefInner, header: &StateDataHeader) -> bool {
            return inner.type_id == header.type_id && inner.obj_id == header.obj_id;
        }

        let header = unsafe { &*state };
        if let Some(value) = self.refers.get_mut(&header.obj_id) {
            match value {
                RefsMapValue::Single(mut single) => unsafe {
                    if compare(&*single, header) {
                        (*single).state = state;
                    }
                },
                RefsMapValue::Multi(multi) => unsafe {
                    for iter in multi {
                        if compare(&**iter, header) {
                            (**iter).state = state;
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
// Thread Local
//

thread_local! {
    pub(super) static STATE_BINDER: RefCell<StateBinder> = RefCell::new(StateBinder::new());
}

pub fn state_binder_register<S>(refer: &StateRef<S>) -> Result<(), Error>
where
    S: StateData + StateDataStatic,
{
    let mut result = Ok(());
    STATE_BINDER.with(|binder| {
        result = binder.borrow_mut().register(refer);
    });
    return result;
}

pub fn state_binder_unregister<S>(refer: &StateRef<S>)
where
    S: StateData + StateDataStatic,
{
    STATE_BINDER.with(|binder| {
        binder.borrow_mut().unregister(refer);
    });
}

pub fn state_binder_dispatch(pool: Box<StatePool>) {
    STATE_BINDER.with(|binder| {
        binder.borrow_mut().dispatch(pool);
    });
}

//
// tests
//

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::mut_ptr;
    use crate::macros::state_data;
    use crate::state::StateLifecycle;
    use crate::id::TYPE_STAGE;

    #[state_data(TYPE_STAGE)]
    #[derive(Debug, Default, PartialEq)]
    struct StateTest {
        num: u32,
        text: String,
    }

    #[test]
    fn test_state_ref() {
        let obj_id = ObjID::from(1234);
        let refer = StateRef::<StateTest>::new(obj_id);
        assert_eq!(refer.obj_id(), obj_id);
        assert_eq!(refer.is_empty(), true);
        assert!(refer.state().is_err());

        let mut state = StateTest {
            num: 0xABCD,
            text: String::from("HaHa"),
            ..StateTest::default()
        };
        refer.inner_ref().state = mut_ptr(&mut state);
        assert_eq!(refer.is_empty(), false);
        assert_eq!(refer.state().unwrap().num, 0xABCD);
        assert_eq!(refer.state().unwrap().text, String::from("HaHa"));
    }

    #[test]
    fn test_state_binder() {
        let mut sb = StateBinder::new();
        let re1 = StateRef::<StateTest>::new(ObjID::from(123));
        let re2 = StateRef::<StateTest>::new(ObjID::from(456));
        let re3 = StateRef::<StateTest>::new(ObjID::from(456));

        let mut state = StateTest {
            header: StateDataHeader{
                obj_id: ObjID::invaild(),
                type_id: TYPE_STAGE,
                lifecycle: StateLifecycle::Updated,
            },
            num: 0xABCD,
            text: String::from("HaHa")
        };

        // register
        sb.register(&re1).unwrap();
        sb.register(&re2).unwrap();
        sb.register(&re3).unwrap();
        assert_eq!(
            sb.refers[&re1.obj_id()],
            RefsMapValue::Single(re1.inner_ptr())
        );
        assert_eq!(
            sb.refers[&re2.obj_id()],
            RefsMapValue::Multi(vec![re2.inner_ptr(), re3.inner_ptr()]),
        );

        // update
        sb.update(mut_ptr(&mut state));
        assert!(re1.state().is_err());
        assert!(re3.state().is_err());

        state.header.obj_id = ObjID::from(123);
        sb.update(mut_ptr(&mut state));
        assert_eq!(re1.state().unwrap().num, 0xABCD);

        state.header.obj_id = ObjID::from(456);
        sb.update(mut_ptr(&mut state));
        assert_eq!(re2.state().unwrap().num, 0xABCD);
        assert_eq!(re3.state().unwrap().num, 0xABCD);

        // clear_all
        sb.clear_all();
        assert!(re1.state().is_err());
        assert!(re2.state().is_err());
        assert!(re3.state().is_err());

        // unregister
        sb.unregister(&re1);
        assert!(sb.refers.get(&re1.obj_id()).is_none());
        sb.unregister(&re1);

        sb.unregister(&re2);
        assert_eq!(
            sb.refers[&re3.obj_id()],
            RefsMapValue::Multi(vec![re3.inner_ptr()]),
        );
        sb.unregister(&re3);
        assert!(sb.refers.get(&re3.obj_id()).is_none());
    }
}
