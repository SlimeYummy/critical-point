use crate::id::{ClassID, FastObjID};

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

pub trait StateDataStatic
where
    Self: Default,
{
    fn id() -> ClassID;
    fn init(fobj_id: FastObjID, lifecycle: StateLifecycle) -> Self;
}

pub trait StateData {
    fn class_id(&self) -> ClassID;
    fn fobj_id(&self) -> FastObjID;
    fn lifecycle(&self) -> StateLifecycle;
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;
    use crate::id::{ClassID, FastObjID};

    #[derive(StateDataX, Debug, Default, PartialEq)]
    #[class_id(StageGeneral)]
    struct StateDataTest {
        fobj_id: FastObjID,
        lifecycle: StateLifecycle,
        num: u32,
        text: String,
    }

    #[test]
    fn test_macro_state_data() {
        let mut t = StateDataTest::default();
        t.num = 1000;
        t.text = String::from("...");
        assert_eq!(StateDataTest::id(), ClassID::StageGeneral);
        assert_eq!(t.class_id(), ClassID::StageGeneral);
        assert_eq!(t.fobj_id(), FastObjID::invalid());
        assert_eq!(t.lifecycle(), StateLifecycle::Unknown);
    }
}
