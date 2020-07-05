use crate::id::{ObjID, TypeID};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LogicObjSuperField {
    pub obj_id: ObjID,
    pub type_id: TypeID,
}

pub trait LogicObjStatic {
    fn id() -> TypeID;
}

pub trait LogicObjSuper {
    fn _obj_id(&self) -> ObjID;
    fn _type_id(&self) -> TypeID;
}
