use super::base::LogicStage;
use crate::engine::{LogicObjX, NewContext, StateContext};
use crate::id::{FastObjID, FastResID};
use crate::physic::{PhysicClass, PhysicHandle, PhysicMeta, PhysicTeam, INVAILD_PHYSIC_HANDLE};
use crate::resource::ResStageGeneral;
use crate::state::{StateDataX, StateLifecycle};
use crate::util::RcCell;
use anyhow::Result;
use na::Isometry3;
use std::sync::Arc;

#[derive(StateDataX, Debug, Default)]
#[class_id(StageGeneral)]
pub struct StateStageGeneral {
    pub fobj_id: FastObjID,
    pub lifecycle: StateLifecycle,
    pub fres_id: FastResID,
}

#[derive(LogicObjX)]
#[class_id(StageGeneral)]
pub struct LogicStageGeneral {
    res: Arc<ResStageGeneral>,
    fobj_id: FastObjID,
    lifecycle: StateLifecycle,
    h_world: PhysicHandle,
}

impl Drop for LogicStageGeneral {
    fn drop(&mut self) {}
}

impl LogicStageGeneral {
    pub(crate) fn new(ctx: &mut NewContext<ResStageGeneral>) -> RcCell<LogicStageGeneral> {
        let stage = RcCell::new(LogicStageGeneral {
            res: ctx.res.clone(),
            fobj_id: ctx.fobj_id,
            lifecycle: StateLifecycle::Created,
            h_world: INVAILD_PHYSIC_HANDLE,
        });
        let (h_world, _) = ctx.new_collision(
            Isometry3::new(na::zero(), na::zero()),
            ctx.res.world.handle.clone(),
            PhysicMeta::new(PhysicClass::Stage, PhysicTeam::None),
            stage.clone(),
        );
        stage.borrow_mut().h_world = h_world;
        return stage;
    }
}

impl LogicStage for LogicStageGeneral {
    fn take_state(&mut self, ctx: &mut StateContext) -> Result<()> {
        let state = ctx.make::<StateStageGeneral>(self.fobj_id, self.lifecycle);
        state.fres_id = self.res.fres_id;
        self.lifecycle = StateLifecycle::Updated;
        return Ok(());
    }
}
