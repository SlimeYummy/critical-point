use crate::id::{ObjID, TYPE_CHARACTER, TypeID};
use crate::logic::{logic_obj, LogicObj};
use crate::state::{state_data, StateData, StateLifecycle, StatePool};
use crate::utils::{fixed64, Fixed64};
use failure::Error;
use std::time::Duration;
use ncollide3d::shape::{ShapeHandle, Capsule};
use std::rc::Rc;

pub struct LogicCharacter {
    obj_id: ObjID,
    lifecycle: StateLifecycle,
    pub(crate) shape: ShapeHandle<Fixed64>,
}

impl Drop for LogicCharacter {
    fn drop(&mut self) {}
}

impl LogicCharacter {
    pub(super) fn new(obj_id: ObjID) -> Rc<LogicCharacter> {
        return Rc::new(LogicCharacter {
            obj_id,
            lifecycle: StateLifecycle::Created,
            shape: ShapeHandle::new(Capsule::new(fixed64(0.85), fixed64(0.4))),
        });
    }
}

impl LogicObj for LogicCharacter {
    fn obj_id(&self) -> ObjID {
        return self.obj_id;
    }

    fn type_id(&self) -> TypeID {
        return TYPE_CHARACTER;
    }

    fn collide(&mut self, _: Rc<dyn LogicObj>) -> Result<(), Error> {
        return Ok(());
    }

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
