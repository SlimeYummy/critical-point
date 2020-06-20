use crate::id::{ObjID, TYPE_CHARACTER};
use crate::logic::{logic_obj, LogicObj};
use crate::state::{state_data, StateData, StateLifecycle, StatePool};
use crate::utils::{Fixed64, fixed64};
use failure::Error;
use failure::_core::time::Duration;
use ncollide3d::shape::{ShapeHandle, Capsule};

#[logic_obj(TYPE_CHARACTER)]
pub struct LogicCharacter {
    pub(crate) lifecycle: StateLifecycle,
    pub(crate) shape: ShapeHandle<Fixed64>,
}

impl Drop for LogicCharacter {
    fn drop(&mut self) {}
}

impl LogicCharacter {
    pub(super) fn new(obj_id: ObjID) -> Box<LogicCharacter> {
        return Box::new(LogicCharacter {
            sup: Self::new_super(obj_id),
            lifecycle: StateLifecycle::Created,
            shape: ShapeHandle::new(Capsule::new(fixed64(0.85), fixed64(0.4))),
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
pub struct StateCharacter {}

impl StateData for StateCharacter {}

impl Default for StateCharacter {
    fn default() -> Self {
        return StateCharacter {
            sup: Self::default_super(),
        };
    }
}
