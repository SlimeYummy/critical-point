#![allow(dead_code)]

mod base;
mod state_pool;
mod state_ref;

pub use crate::derive::{StateDataX, StateOwnerX};
pub use base::{StateData, StateDataStatic, StateLifecycle, StateOwner, StateOwnerStatic};
pub use state_pool::StatePool;
pub use state_ref::{StateBus, StateLocalReg, StateRef, StateReg};

#[cfg(test)]
mod tests {
    use super::*;
    use crate as core;
    use crate::id::{ObjID, CLASS_STAGE};
    use std::cell::RefCell;
    use std::rc::Rc;

    // #[state_data(CLASS_STAGE)]
    #[derive(StateDataX, Debug, Default, PartialEq)]
    #[class_id(CLASS_STAGE)]
    struct StateDataTest {
        obj_id: ObjID,
        lifecycle: StateLifecycle,
        num: u32,
        text: String,
    }

    #[derive(StateOwnerX, Debug)]
    #[class_id(CLASS_STAGE)]
    struct StateOwnerTest {
        obj_id: ObjID,
        refer: StateRef<StateDataTest>,
    }

    impl StateOwnerTest {
        fn new(obj_id: ObjID, bus: Rc<RefCell<StateBus>>) -> StateOwnerTest {
            return StateOwnerTest {
                obj_id,
                refer: StateRef::new(obj_id, StateLocalReg::new(bus)),
            };
        }
    }

    #[test]
    fn test_state_all() {
        let sb = Rc::new(RefCell::new(StateBus::new()));
        let mut sp = StatePool::new(1024);

        let state1 = sp.make::<StateDataTest>(ObjID::from(123), StateLifecycle::Updated);
        state1.num = 1;
        state1.text = String::from("one");

        let state2 = sp.make::<StateDataTest>(ObjID::from(456), StateLifecycle::Updated);
        state2.num = 2;
        state2.text = String::from("two");

        let owner1 = StateOwnerTest::new(ObjID::from(123), sb.clone());
        owner1.refer.register().unwrap();
        let owner2 = StateOwnerTest::new(ObjID::from(456), sb.clone());
        owner2.refer.register().unwrap();
        let owner3 = StateOwnerTest::new(ObjID::from(456), sb.clone());
        owner3.refer.register().unwrap();
        let owner4 = StateOwnerTest::new(ObjID::from(789), sb.clone());
        owner4.refer.register().unwrap();

        sb.borrow_mut().dispatch(Box::new(sp));

        assert_eq!(owner1.refer.state().unwrap().num, 1);
        assert_eq!(owner1.refer.state().unwrap().text, String::from("one"));
        assert_eq!(owner2.refer.state().unwrap().num, 2);
        assert_eq!(owner2.refer.state().unwrap().text, String::from("two"));
        assert_eq!(owner3.refer.state().unwrap().num, 2);
        assert_eq!(owner3.refer.state().unwrap().text, String::from("two"));
        assert!(owner4.refer.state().is_err());
    }
}
