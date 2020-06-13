#![allow(dead_code)]

mod id;

pub use id::{ObjID, ObjectIDGener, TypeID};

pub const TYPE_STAGE: TypeID = TypeID(1);
pub const TYPE_CHARACTER: TypeID = TypeID(2);
