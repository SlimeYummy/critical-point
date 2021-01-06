use super::base::LogicChara;
use crate::engine::{LogicObjX, NewContext, StateContext};
use crate::id::{FastObjID, FastResID};
use crate::physic::{
    PhysicClass, PhysicHandle, PhysicMeta, PhysicTeam, PhysicWorld, INVAILD_PHYSIC_HANDLE,
};
use crate::resource::ResCharaGeneral;
use crate::state::{StateDataX, StateLifecycle};
use crate::util::RcCell;
use anyhow::Result;
use derivative::Derivative;
use m::{fi, Fx};
use na::{Isometry3, Translation3, UnitQuaternion, Vector3};
use std::sync::Arc;

#[derive(Derivative, StateDataX, Debug)]
#[derivative(Default)]
#[class_id(CharaGeneral)]
pub struct StateCharaGeneral {
    pub fobj_id: FastObjID,
    pub lifecycle: StateLifecycle,
    pub fres_id: FastResID,
    #[derivative(Default(value = "Isometry3::identity()"))]
    pub isometry: Isometry3<Fx>,
}

#[derive(LogicObjX)]
#[class_id(StageGeneral)]
pub struct LogicCharaGeneral {
    res: Arc<ResCharaGeneral>,
    fobj_id: FastObjID,
    lifecycle: StateLifecycle,
    h_collision: PhysicHandle,
}

impl Drop for LogicCharaGeneral {
    fn drop(&mut self) {}
}

impl LogicCharaGeneral {
    pub(crate) fn new(ctx: &mut NewContext<ResCharaGeneral>) -> RcCell<LogicCharaGeneral> {
        let stage = RcCell::new(LogicCharaGeneral {
            res: ctx.res.clone(),
            fobj_id: ctx.fobj_id,
            lifecycle: StateLifecycle::Created,
            h_collision: INVAILD_PHYSIC_HANDLE,
        });
        let (h_collision, _) = ctx.new_collision(
            Isometry3::identity(),
            ctx.res.collision.handle.clone(),
            PhysicMeta::new(PhysicClass::Stage, PhysicTeam::None),
            stage.clone(),
        );
        stage.borrow_mut().h_collision = h_collision;
        return stage;
    }
}

impl LogicChara for LogicCharaGeneral {
    fn update_posotion(&mut self, _world: &mut PhysicWorld) -> Result<()> {
        return Ok(());
    }

    fn resolve_collision(&mut self, _world: &mut PhysicWorld) -> Result<()> {
        return Ok(());
    }

    fn update_skeleton(&mut self, _world: &mut PhysicWorld) -> Result<()> {
        return Ok(());
    }

    fn update(&mut self) -> Result<()> {
        return Ok(());
    }

    fn take_state(&mut self, ctx: &mut StateContext) -> Result<()> {
        let state = ctx.make::<StateCharaGeneral>(self.fobj_id, self.lifecycle);
        state.fres_id = self.res.fres_id;
        state.isometry = Isometry3::from_parts(
            Translation3::from(Vector3::new(fi(0), fi(0), fi(0))),
            UnitQuaternion::identity(),
        );
        self.lifecycle = StateLifecycle::Updated;
        return Ok(());
    }
}
