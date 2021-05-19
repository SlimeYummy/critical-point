use super::base::ResObj;
use super::cache::{CompileContext, RestoreContext};
use crate::character::ArgsCharaHuman;
use crate::derive::def_res;
use crate::id::{ClassID, FastResID, ResID};
use crate::stage::{ArgsStageGeneral, ArgsStageScenery};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[def_res(ClassID::Prefab)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResPrefab {
    pub res_id: ResID,
    #[serde(skip)]
    pub fres_id: FastResID,
    pub items: Vec<ResPrefabItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResPrefabItem {
    pub res_id: ResID,
    #[serde(flatten)]
    pub args: ResPrefabArgs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResPrefabArgs {
    StageGeneral(ArgsStageGeneral),
    StageScenery(ArgsStageScenery),
    CharaHuman(ArgsCharaHuman),
}

#[typetag::serde(name = "Prefab")]
impl ResObj for ResPrefab {
    fn compile(&mut self, ctx: &mut CompileContext) -> Result<()> {
        ctx.insert_res_id(&self.res_id)?;
        return Ok(());
    }

    fn restore(&mut self, ctx: &mut RestoreContext) -> Result<()> {
        self.fres_id = ctx.get_fres_id(&self.res_id)?;
        return Ok(());
    }
}
