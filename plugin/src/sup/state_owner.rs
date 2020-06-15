use crate::id::{ObjID, TypeID};
use failure::Error;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StateOwnerSuperField {
    pub obj_id: crate::id::ObjID,
    pub type_id: crate::id::TypeID,
    pub once: bool,
    pub offset: i32,
}

pub trait StateOwnerStatic {
    fn id() -> TypeID;
}

pub trait StateOwnerSuper {
    fn _obj_id(&self) -> ObjID;
    fn _type_id(&self) -> TypeID;
    fn _bind_state(&mut self) -> Result<(), Error>;
}
