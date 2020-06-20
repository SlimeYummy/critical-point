use crate::id::{ObjID, TYPE_STAGE};
use crate::logic::{logic_obj, LogicObj};
use crate::state::{state_data, StateData, StateLifecycle, StatePool};
use crate::utils::mut_ptr;
use failure::Error;
use std::mem;
use std::ptr;
use std::time::Duration;

#[logic_obj(TYPE_STAGE)]
pub struct LogicStage {
    lifecycle: StateLifecycle,
}

impl Drop for LogicStage {
    fn drop(&mut self) {
    }
}

impl LogicStage {
    pub(super) fn new(obj_id: ObjID) -> Box<LogicStage> {
        return Box::new(LogicStage {
            sup: Self::new_super(obj_id),
            lifecycle: StateLifecycle::Created,
        });
    }
}

impl LogicObj for LogicStage {
    fn update(&mut self, pool: &mut Box<StatePool>, _: Duration) -> Result<(), Error> {
        let _ = pool.make::<StateStage>(self.obj_id(), self.lifecycle);
        self.lifecycle = StateLifecycle::Updated;
        return Ok(());
    }
}

#[state_data(TYPE_STAGE)]
#[derive(Debug, Default)]
pub struct StateStage {}

impl StateData for StateStage {}
