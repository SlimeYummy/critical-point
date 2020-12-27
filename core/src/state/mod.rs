#![allow(dead_code)]

mod base;
mod state_pool;
mod state_bus;

pub use base::{StateData, StateDataStatic, StateLifecycle};
pub(crate) use derive::StateDataX;
pub use state_pool::StatePool;
pub use state_bus::{StateBinder, StateBus, StateRef};

#[cfg(test)]
mod tests {
    use super::*;
    use crate as core;
    use crate::id::{FastObjID, ObjID};
    use derive::StateDataX;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(StateDataX, Debug, Default, PartialEq)]
    #[class_id(StageGeneral)]
    struct StateDataTest {
        fobj_id: FastObjID,
        lifecycle: StateLifecycle,
        num: u32,
        text: String,
    }

    #[derive(Debug)]
    struct StateOwnerTest {
        obj_id: ObjID,
        fobj_id: FastObjID,
        refer: StateRef<StateDataTest>,
    }

    impl StateOwnerTest {
        fn new(fobj_id: FastObjID, binder: StateBinder) -> StateOwnerTest {
            return StateOwnerTest {
                obj_id: ObjID::invalid(),
                fobj_id,
                refer: StateRef::new_and_start(fobj_id, binder).unwrap(),
            };
        }
    }

    #[test]
    fn test_state_all() {
        let sb = Rc::new(RefCell::new(StateBus::new()));
        let mut sp = StatePool::new(1024);

        let state1 = sp.make::<StateDataTest>(FastObjID::from(123), StateLifecycle::Updated);
        state1.num = 1;
        state1.text = String::from("one");

        let state2 = sp.make::<StateDataTest>(FastObjID::from(456), StateLifecycle::Updated);
        state2.num = 2;
        state2.text = String::from("two");

        let owner1 = StateOwnerTest::new(FastObjID::from(123), sb.borrow().new_binder());
        let owner2 = StateOwnerTest::new(FastObjID::from(456), sb.borrow().new_binder());
        let owner3 = StateOwnerTest::new(FastObjID::from(456), sb.borrow().new_binder());
        let owner4 = StateOwnerTest::new(FastObjID::from(789), sb.borrow().new_binder());

        sb.borrow_mut().dispatch_states(Box::new(sp));

        assert_eq!(owner1.refer.state().unwrap().num, 1);
        assert_eq!(owner1.refer.state().unwrap().text, String::from("one"));
        assert_eq!(owner2.refer.state().unwrap().num, 2);
        assert_eq!(owner2.refer.state().unwrap().text, String::from("two"));
        assert_eq!(owner3.refer.state().unwrap().num, 2);
        assert_eq!(owner3.refer.state().unwrap().text, String::from("two"));
        assert!(owner4.refer.state().is_err());
    }
}
