mod character;
mod command;
mod engine;
mod stage;

pub use crate::macros::logic_obj;
pub use character::*;
pub use command::*;
pub use engine::*;
pub use stage::*;

use crate::id::{ObjID, TypeID};
use crate::state::StatePool;
use failure::Error;
use std::time::Duration;
use std::rc::Rc;

pub trait LogicObj {
    fn obj_id(&self) -> ObjID;
    fn type_id(&self) -> TypeID;
    fn collide(&mut self, other: Rc<dyn LogicObj>) -> Result<(), Error>;
    fn update(&mut self, pool: &mut Box<StatePool>, dura: Duration) -> Result<(), Error>;
}
