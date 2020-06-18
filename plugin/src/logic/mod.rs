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
use crate::sup::LogicObjSuper;
use failure::Error;
use std::time::Duration;

pub trait LogicObj
where
    Self: LogicObjSuper,
{
    fn obj_id(&self) -> ObjID {
        return self._obj_id();
    }
    fn type_id(&self) -> TypeID {
        return self._type_id();
    }
    fn update(&mut self, pool: &mut Box<StatePool>, dura: Duration) -> Result<(), Error>;
}

pub trait Command {
    fn get_name(&self) -> &'static str;
    fn execute(&self, engine: &mut LogicEngine) -> Result<(), Error>;
}
