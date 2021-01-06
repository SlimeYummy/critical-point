use crate::engine::{LogicObj, StateContext};
use crate::id::{FastObjID, ResID};
use crate::physic::PhysicWorld;
use anyhow::Result;
use serde::{Deserialize, Serialize};

pub trait LogicChara
where
    Self: LogicObj,
{
    fn update_posotion(&mut self, world: &mut PhysicWorld) -> Result<()>;
    fn resolve_collision(&mut self, world: &mut PhysicWorld) -> Result<()>;
    fn update_skeleton(&mut self, world: &mut PhysicWorld) -> Result<()>;
    fn update(&mut self) -> Result<()>;
    fn take_state(&mut self, ctx: &mut StateContext) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CmdNewCharaGeneral {
    pub fobj_id: FastObjID,
    pub res_id: ResID,
}
