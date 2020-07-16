use crate::id::{ClassID, ObjID};

//
// State Lifecycle
//

#[derive(Clone, Copy, Debug, Hash, PartialEq)]
pub enum StateLifecycle {
    Unknown,
    Created,
    Updated,
    Destoryed,
}

impl Default for StateLifecycle {
    fn default() -> StateLifecycle {
        return StateLifecycle::Unknown;
    }
}

//
// State Data
//

pub trait StateData {
    fn class_id(&self) -> ClassID;
    fn obj_id(&self) -> ObjID;
    fn lifecycle(&self) -> StateLifecycle;
}

pub trait StateDataStatic
where
    Self: Default,
{
    fn id() -> ClassID;
    fn init(obj_id: ObjID, lifecycle: StateLifecycle) -> Self;
}

//
// State Owner
//

pub trait StateOwner {
    fn obj_id(&self) -> ObjID;
    fn class_id(&self) -> ClassID;
}

pub trait StateOwnerStatic {
    fn id() -> ClassID;
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;
    use crate as core;
    use crate::id::{ObjID, CLASS_STAGE};
    use failure::Error;
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

    #[test]
    fn test_macro_state_data() {
        let mut t = StateDataTest::default();
        t.num = 1000;
        t.text = String::from("...");
        assert_eq!(StateDataTest::id(), CLASS_STAGE);
        assert_eq!(t.class_id(), CLASS_STAGE);
        assert_eq!(t.obj_id(), ObjID::invaild());
        assert_eq!(t.lifecycle(), StateLifecycle::Unknown);
    }

    #[derive(StateOwnerX, Debug)]
    #[class_id(CLASS_STAGE)]
    struct StateOwnerTest {
        obj_id: ObjID,
        refer: StateRef<StateDataTest>,
        num: u32,
    }

    impl StateOwnerTest {
        fn new(obj_id: ObjID, binder: StateLocalReg<StateDataTest>) -> StateOwnerTest {
            return StateOwnerTest {
                obj_id,
                refer: StateRef::new(obj_id, binder),
                num: 0,
            };
        }

        fn register(&self) -> Result<(), Error> {
            return self.refer.register();
        }
    }

    #[test]
    fn test_macro_state_owner() {
        let sb = Rc::new(RefCell::new(StateBus::new()));
        let owner = StateOwnerTest::new(ObjID::from(1234), StateLocalReg::new(sb.clone()));
        assert_eq!(StateDataTest::id(), CLASS_STAGE);
        assert_eq!(owner.class_id(), CLASS_STAGE);
        assert_eq!(owner.obj_id(), ObjID::from(1234));
    }
}
