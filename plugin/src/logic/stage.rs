use crate::id::{ObjID, TYPE_STAGE, TypeID};
use crate::logic::{logic_obj, LogicObj};
use crate::state::{state_data, StateData, StateLifecycle, StatePool};
use crate::utils::Fixed64;
use failure::Error;
use na::Vector3;
use ncollide3d::shape::{Plane, ShapeHandle};
use std::time::Duration;

pub struct LogicStage {
    obj_id: ObjID,
    lifecycle: StateLifecycle,
    pub(crate) shape: ShapeHandle<Fixed64>,
}

impl Drop for LogicStage {
    fn drop(&mut self) {}
}

impl LogicStage {
    pub(super) fn new(obj_id: ObjID) -> Rc<LogicStage> {
        return Rc::new(LogicStage {
            obj_id,
            lifecycle: StateLifecycle::Created,
            shape: ShapeHandle::new(Plane::new(Vector3::z_axis())),
        });
    }
}

impl LogicObj for LogicStage {
    fn obj_id(&self) -> ObjID {
        return self.obj_id;
    }

    fn type_id(&self) -> TypeID {
        return TYPE_STAGE;
    }

    fn collide(&mut self, _: Rc<dyn LogicObj>) -> Result<(), Error> {
        return Ok(());
    }

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
