use failure::{Error, format_err};
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ptr;
use std::sync::Arc;
use crate::id::ObjectID;
use super::{StateData, StatePool};
use super::base::{StateInner, state_vtable};

//
// State Reference
//

#[derive(Debug)]
pub struct StateRef<S>
    where S: StateData
{
    inner: UnsafeCell<StateInner>,
    phantom: PhantomData<S>,
}

impl<S> StateRef<S>
    where S: StateData
{
    fn new(obj_id: ObjectID) -> StateRef<S> {
        return StateRef{
            inner: UnsafeCell::new(StateInner{
                obj_id,
                state: ptr::null_mut(),
                vtable: unsafe { state_vtable::<S>() },
            }),
            phantom: PhantomData,
        };
    }

    fn inner_ptr(&self) -> *mut StateInner {
        return self.inner.get();
    }

    fn inner_ref(&self) -> &mut StateInner {
        return unsafe { &mut *self.inner_ptr() };
    }

    pub fn object_id(&self) -> ObjectID {
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
}

impl<S> Drop for StateRef<S>
    where S: StateData
{
    fn drop(&mut self) {
        self.inner_ref().obj_id = ObjectID::invaild();
        self.inner_ref().state = ptr::null_mut();
        self.inner_ref().vtable = ptr::null_mut();
    }
}

//
// State Reference Manager
//

#[derive(Debug, PartialEq)]
enum RefsMapValue {
    Single(*mut StateInner),
    Multi(Vec<*mut StateInner>),
}

#[derive(Debug)]
pub struct StateDispatcher {
    refers: HashMap<ObjectID, RefsMapValue>,
    pool: Option<Arc<StatePool>>,
}

impl StateDispatcher {
    fn new() -> StateDispatcher {
        return StateDispatcher{
            refers: HashMap::with_capacity(1024),
            pool: None,
        };
    }

    pub fn register<S>(&mut self, state: &StateRef<S>)
        where S: StateData
    {
        let inner = state.inner_ptr();
        let obj_id = state.object_id();
        if let Some(value) = self.refers.get_mut(&obj_id) {
            match value {
                RefsMapValue::Single(single) => {
                    let mut multi = Vec::with_capacity(8);
                    multi.push(*single);
                    multi.push(inner);
                    self.refers.insert(obj_id, RefsMapValue::Multi(multi));
                },
                RefsMapValue::Multi(multi) => {
                    multi.push(inner);
                },
            };
        } else {
            self.refers.insert(obj_id, RefsMapValue::Single(inner));
        }
    }

    pub fn unregister<S>(&mut self, state: &StateRef<S>)
        where S: StateData
    {
        let inner = state.inner_ptr();
        let obj_id = state.object_id();
        let mut remove = false;
        if let Some(value) = self.refers.get_mut(&obj_id) {
            match value {
                RefsMapValue::Single(single) => {
                    remove = *single == inner;
                },
                RefsMapValue::Multi(multi) => {
                    multi.remove_item(&inner);
                    remove = multi.is_empty();
                },
            };
        }
        if remove {
            self.refers.remove(&obj_id);
        }
    }

    fn dispatch(&mut self, pool: Arc<StatePool>) {
        self.clear_all();
        self.pool = None;
        pool.for_each(|obj_id, state, vtable| self.update(obj_id, state, vtable));
        self.pool = Some(pool);
    }

    fn update(&mut self, obj_id: ObjectID, state: *mut u8, vtable: *mut u8) {
        if let Some(value) = self.refers.get_mut(&obj_id) {
            match value {
                RefsMapValue::Single(mut single) => unsafe {
                    if (*single).vtable == vtable {
                        (*single).state = state;
                    }
                },
                RefsMapValue::Multi(multi) => unsafe {
                    for iter in multi {
                        if (**iter).vtable == vtable {
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
                },
                RefsMapValue::Multi(multi) => {
                    for idx in 0..multi.len() {
                        unsafe { (*multi[idx]).state = ptr::null_mut() };
                    }
                },
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::mut_ptr;

    #[derive(Debug, Default, PartialEq)]
    struct StateTest {
        num: u32,
        text: String,
    }

    impl StateData for StateTest {}

    #[test]
    fn test_state_ref() {
        let obj_id = ObjectID::from(1234);
        let refer = StateRef::<StateTest>::new(obj_id);
        assert_eq!(refer.object_id(), obj_id);
        assert_eq!(refer.is_empty(), true);
        assert!(refer.state().is_err());

        let mut state = StateTest{
            num: 0xABCD,
            text: String::from("HaHa"),
        };
        refer.inner_ref().state = mut_ptr(&mut state);
        assert_eq!(refer.is_empty(), false);
        assert_eq!(refer.state().unwrap().num, 0xABCD);
        assert_eq!(refer.state().unwrap().text, String::from("HaHa"));
    }

    #[test]
    fn test_state_ref_manager() {
        let mut sd = StateDispatcher::new();
        let re1 = StateRef::<StateTest>::new(ObjectID::from(123));
        let re2 = StateRef::<StateTest>::new(ObjectID::from(456));
        let re3 = StateRef::<StateTest>::new(ObjectID::from(456));
        let vtable = unsafe { state_vtable::<StateTest>() };

        let mut state = StateTest{
            num: 0xABCD,
            text: String::from("HaHa"),
        };

        // register
        sd.register(&re1);
        sd.register(&re2);
        sd.register(&re3);
        assert_eq!(sd.refers[&re1.object_id()], RefsMapValue::Single(re1.inner_ptr()));
        assert_eq!(
            sd.refers[&re2.object_id()],
            RefsMapValue::Multi(vec![re2.inner_ptr(), re3.inner_ptr()]),
        );

        // update
        sd.update(ObjectID::invaild(), mut_ptr(&mut state), vtable);
        assert!(re1.state().is_err());
        assert!(re3.state().is_err());

        sd.update(re1.object_id(), mut_ptr(&mut state), ptr::null_mut());
        assert!(re1.state().is_err());
        assert!(re3.state().is_err());

        sd.update(re1.object_id(), mut_ptr(&mut state), vtable);
        assert_eq!(re1.state().unwrap().num, 0xABCD);

        sd.update(re2.object_id(), mut_ptr(&mut state), vtable);
        assert_eq!(re2.state().unwrap().num, 0xABCD);
        assert_eq!(re3.state().unwrap().num, 0xABCD);

        // clear_all
        sd.clear_all();
        assert!(re1.state().is_err());
        assert!(re2.state().is_err());
        assert!(re3.state().is_err());

        // unregister
        sd.unregister(&re1);
        assert!(sd.refers.get(&re1.object_id()).is_none());
        sd.unregister(&re1);

        sd.unregister(&re2);
        assert_eq!(
            sd.refers[&re3.object_id()],
            RefsMapValue::Multi(vec![re3.inner_ptr()]),
        );
        sd.unregister(&re3);
        assert!(sd.refers.get(&re3.object_id()).is_none());
    }

    #[test]
    fn test_state_ref_and_pool() {
        // pool
        let mut sp = StatePool::new(1024);

        let state1 = sp.make::<StateTest>(ObjectID::from(123));
        state1.num = 1;
        state1.text = String::from("one");

        let state2 = sp.make::<StateTest>(ObjectID::from(456));
        state2.num = 2;
        state2.text = String::from("two");

        // dispatcher
        let mut sd = StateDispatcher::new();

        let re1 = StateRef::<StateTest>::new(ObjectID::from(123));
        sd.register(&re1);
        let re2 = StateRef::<StateTest>::new(ObjectID::from(456));
        sd.register(&re2);
        let re3 = StateRef::<StateTest>::new(ObjectID::from(456));
        sd.register(&re3);
        let re4 = StateRef::<StateTest>::new(ObjectID::from(789));
        sd.register(&re4);

        sd.dispatch(Arc::new(sp));

        // assert
        assert_eq!(re1.state().unwrap(), &StateTest {
            num: 1,
            text: String::from("one"),
        });
        assert_eq!(re2.state().unwrap(), &StateTest {
            num: 2,
            text: String::from("two"),
        });
        assert_eq!(re3.state().unwrap(), &StateTest {
            num: 2,
            text: String::from("two"),
        });
        assert!(re4.state().is_err());
    }
}
