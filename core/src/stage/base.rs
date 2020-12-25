use crate::engine::LogicObj;
use crate::engine::StateContext;
use crate::id::{FastObjID, ResID};
use anyhow::Result;
use serde::{Deserialize, Serialize};

pub trait LogicStage
where
    Self: LogicObj,
{
    fn take_state(&mut self, ctx: &mut StateContext) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CmdNewStageGeneral {
    pub fobj_id: FastObjID,
    pub res_id: ResID,
}
