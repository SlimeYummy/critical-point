#![allow(dead_code)]

mod state_pool;
mod state_ref;

pub use crate::macros::{state_data, state_owner};
pub use state_pool::StatePool;
pub use state_ref::{StateBinder, StateRef};

use crate::id::{ObjID, TypeID};

pub trait StateOwnerStatic {
    fn id() -> TypeID;
}

pub trait StateData {
    fn obj_id(&self) -> ObjID;
    fn type_id(&self) -> TypeID;
    fn lifecycle(&self) -> StateLifecycle;
}

pub trait StateDataStatic {
    fn id() -> TypeID;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StateHeader {
    pub(super) obj_id: crate::id::ObjID,
    pub(super) type_id: crate::id::TypeID,
    pub(super) lifecycle: crate::state::StateLifecycle,
}

impl Default for StateHeader {
    fn default() -> StateHeader {
        return StateHeader {
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
    use std::cell::RefCell;

    #[state_data(TYPE_STAGE)]
    #[derive(Debug, Default)]
    struct TestData {
        num: u32,
        text: String,
    }

    #[test]
    fn test_macro_state_data() {
        let mut t = TestData::default();
        t.num = 1000;
        t.text = String::from("...");
        assert_eq!(TestData::id(), TYPE_STAGE);
        assert_eq!(t.type_id(), TypeID::invaild());
        assert_eq!(t.obj_id(), ObjID::invaild());
        assert_eq!(t.lifecycle(), StateLifecycle::Unknown);
    }

    thread_local! {
        static STATE_BINDER: RefCell<StateBinder> = RefCell::new(StateBinder::new());
    }

    fn state_binder() -> &mut StateBinder {
        return STATE_BINDER.get_mut();
    }

    #[state_owner(TYPE_STAGE, state_binder())]
    #[derive(Default, NativeClass)]
    #[inherit(Node)]
    struct TestOwner {
        ref1: StateRef<TestData>,
    }

    #[methods]
    impl TestOwner {
        fn _init(_owner: Node) -> Self {
            return TestOwner{
                .. Default::default()
            };
        }
    }

    #[test]
    fn test_macro_state_owner() {
    }
}
