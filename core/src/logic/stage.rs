use super::base::{CollisionHandle, INVAILD_COLLISION_HANDLE};
use super::{LogicObj, LogicObjX};
use crate as core;
use crate::id::{ObjID, CLASS_STAGE};
use crate::logic::base::CollideContext;
use crate::logic::{NewContext, StateContext, UpdateContext};
use crate::state::{StateDataX, StateLifecycle};
use crate::util::RcCell;
use failure::Error;
use na::{Isometry3, Vector3};
use ncollide3d::pipeline::CollisionGroups;
use ncollide3d::shape::{Plane, ShapeHandle};

#[derive(LogicObjX)]
#[class_id(CLASS_STAGE)]
pub struct LogicStage {
    obj_id: ObjID,
    lifecycle: StateLifecycle,
    coll_handle: CollisionHandle,
}

impl Drop for LogicStage {
    fn drop(&mut self) {}
}

impl LogicStage {
    pub(super) fn new(ctx: &mut NewContext) -> RcCell<LogicStage> {
        let stage = RcCell::new(LogicStage {
            obj_id: ctx.obj_id,
            lifecycle: StateLifecycle::Created,
            coll_handle: INVAILD_COLLISION_HANDLE,
        });
        let (coll_handle, _) = ctx.new_collision(
            Isometry3::new(na::zero(), na::zero()),
            ShapeHandle::new(Plane::new(Vector3::y_axis())),
            CollisionGroups::new(),
            stage.clone(),
        );
        stage.borrow_mut().coll_handle = coll_handle;
        return stage;
    }
}

impl LogicObj for LogicStage {
    fn collide(&mut self, _ctx: &mut CollideContext) -> Result<(), Error> {
        return Ok(());
    }

    fn update(&mut self, _ctx: &mut UpdateContext) -> Result<(), Error> {
        return Ok(());
    }

    fn state(&mut self, ctx: &mut StateContext) -> Result<(), Error> {
        let _ = ctx.make::<StateStage>(self.obj_id, self.lifecycle);
        self.lifecycle = StateLifecycle::Updated;
        return Ok(());
    }
}

#[derive(StateDataX, Debug, Default)]
#[class_id(CLASS_STAGE)]
pub struct StateStage {
    pub obj_id: ObjID,
    pub lifecycle: StateLifecycle,
}
