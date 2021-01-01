use super::base::{ResObj, ResObjX};
use super::cache::{CompileContext, RestoreContext};
use crate::character::CmdNewCharaGeneral;
use crate::engine::Command;
use crate::id::{FastObjID, ObjID, FastResID, ResID};
use crate::stage::CmdNewStageGeneral;
use anyhow::Result;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

lazy_static! {
    static ref DEFAULT_COMMAND: Arc<ResCommand> = Arc::new(ResCommand::default());
}

#[derive(ResObjX, Debug, Default, Clone, Serialize, Deserialize)]
#[class_id(Command)]
pub struct ResCommand {
    pub res_id: ResID,
    #[serde(skip)]
    pub fres_id: FastResID,
    pub commands: Vec<ResCommandAny>,
}

impl ResCommand {
    pub fn empty() -> Arc<ResCommand> {
        return DEFAULT_COMMAND.clone();
    }

    pub fn to_command(&self) -> Vec<Command> {
        return self
            .commands
            .iter()
            .map(|cmd| match cmd {
                ResCommandAny::NewStageGeneral(cmd) => cmd.to_command(),
                ResCommandAny::NewCharaGeneral(cmd) => cmd.to_command(),
            })
            .collect();
    }
}

#[typetag::serde(name = "Command")]
impl ResObj for ResCommand {
    fn compile(&mut self, ctx: &mut CompileContext) -> Result<()> {
        ctx.insert_res_id(&self.res_id)?;
        for cmd in &mut self.commands {
            match cmd {
                ResCommandAny::NewStageGeneral(cmd) => cmd.compile(ctx),
                ResCommandAny::NewCharaGeneral(cmd) => cmd.compile(ctx),
            }?;
        }
        return Ok(());
    }

    fn restore(&mut self, ctx: &mut RestoreContext) -> Result<()> {
        self.fres_id = ctx.get_fres_id(&self.res_id)?;
        for cmd in &mut self.commands {
            match cmd {
                ResCommandAny::NewStageGeneral(cmd) => cmd.restore(ctx),
                ResCommandAny::NewCharaGeneral(cmd) => cmd.restore(ctx),
            }?;
        }
        return Ok(());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResCommandAny {
    NewStageGeneral(ResCmdNewStageGeneral),
    NewCharaGeneral(ResCmdNewCharaGeneral),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResCmdNewStageGeneral {
    pub obj_id: ObjID,
    #[serde(skip)]
    pub fobj_id: FastObjID,
    pub res_id: ResID,
}

impl ResCmdNewStageGeneral {
    fn compile(&mut self, ctx: &mut CompileContext) -> Result<()> {
        ctx.insert_obj_id(&self.obj_id)?;
        return Ok(());
    }

    fn restore(&mut self, ctx: &mut RestoreContext) -> Result<()> {
        self.fobj_id = ctx.get_fobj_id(&self.obj_id)?;
        return Ok(());
    }

    pub fn to_command(&self) -> Command {
        return Command::NewStageGeneral(CmdNewStageGeneral {
            fobj_id: self.fobj_id,
            res_id: self.res_id.clone(),
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResCmdNewCharaGeneral {
    pub obj_id: ObjID,
    #[serde(skip)]
    pub fobj_id: FastObjID,
    pub res_id: ResID,
}

impl ResCmdNewCharaGeneral {
    fn compile(&mut self, ctx: &mut CompileContext) -> Result<()> {
        ctx.insert_obj_id(&self.obj_id)?;
        return Ok(());
    }

    fn restore(&mut self, ctx: &mut RestoreContext) -> Result<()> {
        self.fobj_id = ctx.get_fobj_id(&self.obj_id)?;
        return Ok(());
    }

    pub fn to_command(&self) -> Command {
        return Command::NewCharaGeneral(CmdNewCharaGeneral {
            fobj_id: self.fobj_id,
            res_id: self.res_id.clone(),
        });
    }
}
