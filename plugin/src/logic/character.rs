use crate::id::{ObjID, TYPE_CHARACTER};
use crate::logic::{logic_obj, LogicObj};
use crate::state::{state_data, StateData, StateLifecycle, StatePool};
use crate::utils::mut_ptr;
use failure::Error;
use failure::_core::time::Duration;
use std::mem;

#[logic_obj(TYPE_CHARACTER)]
pub struct LogicCharacter {
    lifecycle: StateLifecycle,
}

impl Drop for LogicCharacter {
    fn drop(&mut self) {
    }
}

impl LogicCharacter {
    pub(super) fn new(obj_id: ObjID) -> Box<LogicCharacter> {
        return Box::new(LogicCharacter {
            sup: Self::new_super(obj_id),
            lifecycle: StateLifecycle::Created,
        });
    }
}

impl LogicObj for LogicCharacter {
    fn update(&mut self, pool: &mut Box<StatePool>, _: Duration) -> Result<(), Error> {
        let state = pool.make::<StateCharacter>(self.obj_id(), self.lifecycle);
        self.lifecycle = StateLifecycle::Updated;
        return Ok(());
    }
}

#[state_data(TYPE_CHARACTER)]
#[derive(Debug)]
pub struct StateCharacter {
}

impl StateData for StateCharacter {}

impl Default for StateCharacter {
    fn default() -> Self {
        return StateCharacter {
            sup: Self::default_super(),
        };
    }
}
