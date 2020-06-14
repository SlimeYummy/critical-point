#![allow(dead_code)]

mod state_pool;
mod state_ref;

pub use crate::macros::{state_data, state_owner};
pub use state_pool::StatePool;
pub use state_ref::{StateBinder, StateRef, state_binder_register, state_binder_unregister, state_binder_dispatch};

use crate::id::{ObjID, TypeID};

pub trait StateData {
    fn obj_id(&self) -> ObjID;
    fn type_id(&self) -> TypeID;
    fn lifecycle(&self) -> StateLifecycle;
}

pub trait StateDataStatic {
    fn id() -> TypeID;
}

pub trait StateOwner {}

pub trait StateOwnerStatic {
    fn id() -> TypeID;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StateDataHeader {
    pub(super) obj_id: crate::id::ObjID,
    pub(super) type_id: crate::id::TypeID,
    pub(super) lifecycle: crate::state::StateLifecycle,
}

impl Default for StateDataHeader {
    fn default() -> StateDataHeader {
        return StateDataHeader {
            obj_id: ObjID::invaild(),
            type_id: TypeID::invaild(),
            lifecycle: StateLifecycle::Unknown,
        };
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq)]
pub enum StateLifecycle {
    Unknown,
    Created,
    Updated,
    Destoryed,
}

impl Default for StateLifecycle {
    fn default() -> StateLifecycle { return StateLifecycle::Unknown; }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::{ObjID, TYPE_STAGE};
    use crate::gdnative::{NativeClass, Node};
    use state_ref::STATE_BINDER;

    #[state_data(TYPE_STAGE)]
    #[derive(Debug, Default, PartialEq)]
    struct StateDataTest {
        num: u32,
        text: String,
    }

    #[test]
    fn test_macro_state_data() {
        let mut t = StateDataTest::default();
        t.num = 1000;
        t.text = String::from("...");
        assert_eq!(StateDataTest::id(), TYPE_STAGE);
        assert_eq!(t.type_id(), TypeID::invaild());
        assert_eq!(t.obj_id(), ObjID::invaild());
        assert_eq!(t.lifecycle(), StateLifecycle::Unknown);
    }

    #[state_owner(TYPE_STAGE)]
    #[derive(Debug, Default, NativeClass)]
    #[inherit(Node)]
    struct StateOwnerTest {
        refer: StateRef<StateDataTest>,
        num: u32,
    }

    #[methods]
    impl StateOwnerTest {
        fn new(obj_id: ObjID) -> StateOwnerTest {
            return StateOwnerTest{
                refer: StateRef::new(obj_id),
                .. Default::default()
            };
        }

        fn _init(_owner: Node) -> Self {
            return StateOwnerTest::default();
        }
    }

    #[test]
    fn test_macro_state_owner() {
        {
            let mut owner = StateOwnerTest::default();
            assert!(owner.bind_state().is_err());

            owner.refer = StateRef::new(ObjID::from(1234));
            assert!(owner.bind_state().is_ok());
            STATE_BINDER.with(|binder| {
                assert_eq!(binder.borrow().refers_count(), 1);
            });
        }

        STATE_BINDER.with(|binder| {
            assert_eq!(binder.borrow().refers_count(), 0);
        });
    }

    #[test]
    fn test_state_all() {
        let mut sp = StatePool::new(1024);

        let state1 = sp.make::<StateDataTest>(ObjID::from(123), StateLifecycle::Updated);
        state1.num = 1;
        state1.text = String::from("one");

        let state2 = sp.make::<StateDataTest>(ObjID::from(456), StateLifecycle::Updated);
        state2.num = 2;
        state2.text = String::from("two");

        let mut owner1 = StateOwnerTest::new(ObjID::from(123));
        owner1.bind_state().unwrap();
        let mut owner2 = StateOwnerTest::new(ObjID::from(456));
        owner2.bind_state().unwrap();
        let mut owner3 = StateOwnerTest::new(ObjID::from(456));
        owner3.bind_state().unwrap();
        let mut owner4 = StateOwnerTest::new(ObjID::from(789));
        owner4.bind_state().unwrap();

        state_binder_dispatch(Box::new(sp));

        assert_eq!(owner1.refer.state().unwrap().num, 1);
        assert_eq!(owner1.refer.state().unwrap().text, String::from("one"));
        assert_eq!(owner2.refer.state().unwrap().num, 2);
        assert_eq!(owner2.refer.state().unwrap().text, String::from("two"));
        assert_eq!(owner3.refer.state().unwrap().num, 2);
        assert_eq!(owner3.refer.state().unwrap().text, String::from("two"));
        assert!(owner4.refer.state().is_err());
    }
}
