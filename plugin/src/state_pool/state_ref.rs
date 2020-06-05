use failure::{Error, format_err};
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ptr;
use std::sync::Arc;
use crate::id::{ObjectID, TypeID, UnionID};
use super::{StateData, StatePool};

//
// State Reference
//

#[derive(Debug)]
struct StateRefInner {
    union_id: UnionID,
    state: *mut u8,
}

#[derive(Debug)]
pub struct StateRef<S>
    where S: StateData
{
    inner: UnsafeCell<StateRefInner>,
    phantom: PhantomData<S>,
}

impl<S> StateRef<S>
    where S: StateData
{
    fn new(obj_id: ObjectID) -> StateRef<S> {
        return StateRef{
            inner: UnsafeCell::new(StateRefInner{
                union_id: UnionID::new(obj_id, S::type_id()),
                state: ptr::null_mut(),
            }),
            phantom: PhantomData,
        };
    }

    fn inner_ptr(&self) -> *mut StateRefInner {
        return self.inner.get();
    }

    fn inner_ref(&self) -> &mut StateRefInner {
        return unsafe { &mut *self.inner_ptr() };
    }

    pub fn object_id(&self) -> ObjectID {
        return self.inner_ref().union_id.object_id();
    }

    pub fn type_id(&self) -> TypeID {
        return self.inner_ref().union_id.type_id();
    }

    pub fn union_id(&self) -> UnionID {
        return self.inner_ref().union_id;
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
        self.inner_ref().union_id = UnionID::invaild();
        self.inner_ref().state = ptr::null_mut();
    }
}

//
// State Reference Manager
//

#[derive(Debug, PartialEq)]
enum RefsMapValue {
    Single(*mut StateRefInner),
    Multi(Vec<*mut StateRefInner>),
}

#[derive(Debug)]
pub struct StateRefsManager {
    refers: HashMap<UnionID, RefsMapValue>,
    pool: Option<Arc<StatePool>>,
}

impl StateRefsManager {
    fn new() -> StateRefsManager {
        return StateRefsManager{
            refers: HashMap::with_capacity(1024),
            pool: None,
        };
    }

    pub fn register(&mut self, inner: *mut StateRefInner) {
        let union_id = unsafe { (*inner).union_id };
        if let Some(value) = self.refers.get_mut(&union_id) {
            match value {
                RefsMapValue::Single(single) => {
                    let mut multi = Vec::with_capacity(8);
                    multi.push(*single);
                    multi.push(inner);
                    self.refers.insert(union_id, RefsMapValue::Multi(multi));
                },
                RefsMapValue::Multi(multi) => {
                    multi.push(inner);
                },
            };
        } else {
            self.refers.insert(union_id, RefsMapValue::Single(inner));
        }
    }

    pub fn unregister(&mut self, inner: *mut StateRefInner) {
        let mut remove = false;
        let union_id = unsafe { (*inner).union_id };
        if let Some(value) = self.refers.get_mut(&union_id) {
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
            self.refers.remove(&union_id);
        }
    }

    fn set_all(&mut self, pool: Arc<StatePool>) {

    }

    fn set(&mut self, union_id: &UnionID, state: *mut u8) {
        if let Some(value) = self.refers.get_mut(union_id) {
            match value {
                RefsMapValue::Single(mut single) => {
                    unsafe { (*single).state = state };
                },
                RefsMapValue::Multi(multi) => {
                    for iter in multi {
                        unsafe { (**iter).state = state };
                    }
                },
            };
        }
    }

    fn reset_all(&mut self) {
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
    use std::convert::TryFrom;
    use crate::utils::mut_ptr;

    struct StateTest {
        num: u32,
        text: String,
    }

    impl StateData for StateTest {
        fn type_id() -> TypeID { return TypeID::try_from(567).unwrap(); }
    }

    #[test]
    fn test_state_ref() {
        let obj_id = ObjectID::try_from(1234).unwrap();
        let refer = StateRef::<StateTest>::new(obj_id);
        assert_eq!(refer.object_id(), obj_id);
        assert_eq!(refer.type_id(), StateTest::type_id());
        assert_eq!(refer.union_id(), UnionID::new(obj_id, StateTest::type_id()));
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
        let mut mgr = StateRefsManager::new();
        let obj_id1 = ObjectID::try_from(1234).unwrap();
        let refer1 = StateRef::<StateTest>::new(obj_id1);
        let obj_id2 = ObjectID::try_from(5678).unwrap();
        let refer2 = StateRef::<StateTest>::new(obj_id2);
        let refer3 = StateRef::<StateTest>::new(obj_id2);

        let mut state = StateTest{
            num: 0xABCD,
            text: String::from("HaHa"),
        };

        // register
        mgr.register(refer1.inner_ptr());
        mgr.register(refer2.inner_ptr());
        mgr.register(refer3.inner_ptr());
        assert_eq!(mgr.refers[&refer1.union_id()], RefsMapValue::Single(refer1.inner_ptr()));
        assert_eq!(
            mgr.refers[&refer2.union_id()],
            RefsMapValue::Multi(vec![refer2.inner_ptr(), refer3.inner_ptr()]),
        );

        // set
        mgr.set(&UnionID::invaild(), mut_ptr(&mut state));
        assert!(refer1.state().is_err());
        assert!(refer3.state().is_err());

        mgr.set(&refer1.union_id(), mut_ptr(&mut state));
        assert_eq!(refer1.state().unwrap().num, 0xABCD);

        mgr.set(&refer2.union_id(), mut_ptr(&mut state));
        assert_eq!(refer2.state().unwrap().num, 0xABCD);
        assert_eq!(refer3.state().unwrap().num, 0xABCD);

        // reset_all
        mgr.reset_all();
        assert!(refer1.state().is_err());
        assert!(refer2.state().is_err());
        assert!(refer3.state().is_err());

        // unregister
        mgr.unregister(refer1.inner_ptr());
        assert!(mgr.refers.get(&refer1.union_id()).is_none());
        mgr.unregister(refer1.inner_ptr());

        mgr.unregister(refer2.inner_ptr());
        assert_eq!(
            mgr.refers[&refer3.union_id()],
            RefsMapValue::Multi(vec![refer3.inner_ptr()]),
        );
        mgr.unregister(refer3.inner_ptr());
        assert!(mgr.refers.get(&refer3.union_id()).is_none());
    }
}