use crate::id::{ObjID, TypeID};
use crate::state::StateLifecycle;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StateDataSuperField {
    pub obj_id: crate::id::ObjID,
    pub type_id: crate::id::TypeID,
    pub lifecycle: crate::state::StateLifecycle,
}

pub trait StateDataStatic {
    fn id() -> TypeID;
}

pub trait StateDataSuper {
    fn _obj_id(&self) -> ObjID;
    fn _type_id(&self) -> TypeID;
    fn _lifecycle(&self) -> StateLifecycle;
}
