use crate::id::{ObjID, TYPE_STAGE};
use crate::logic::{logic_obj, LogicObj};
use crate::state::{state_data, StateData, StateLifecycle, StatePool};
use crate::utils::Fixed64;
use failure::Error;
use std::time::Duration;
use ncollide3d::shape::{ShapeHandle, Plane};
use na::Vector3;

#[logic_obj(TYPE_STAGE)]
pub struct LogicStage {
    pub(crate) lifecycle: StateLifecycle,
    pub(crate) shape: ShapeHandle<Fixed64>,
}

impl Drop for LogicStage {
    fn drop(&mut self) {}
}

impl LogicStage {
    pub(super) fn new(obj_id: ObjID) -> Box<LogicStage> {
        return Box::new(LogicStage {
            sup: Self::new_super(obj_id),
            lifecycle: StateLifecycle::Created,
            shape: ShapeHandle::new(Plane::new(Vector3::z_axis())),
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
