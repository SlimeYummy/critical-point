#![allow(dead_code)]

mod class_id;
mod obj_id;
mod res_id;

pub use class_id::ClassID;
pub use obj_id::{ObjID, ObjIDGener};
pub use res_id::{FastResID, FastResIDGener, ResID};
