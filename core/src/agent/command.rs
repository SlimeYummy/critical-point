use crate::id::ObjID;
use m::Fx;
use na::{Point3, Vector2};

#[derive(Clone, Debug)]
pub enum Command {
    NewStage(CmdNewStage),
    NewCharacter(CmdNewCharacter),
    MoveCharacter(CmdMoveCharacter),
    JumpCharacter(CmdJumpCharacter),
}

#[derive(Clone, Debug)]
pub struct CmdNewStage {}

#[derive(Clone, Debug)]
pub struct CmdNewCharacter {
    pub position: Point3<Fx>,
    pub direction: Vector2<Fx>,
    pub speed: Fx,
    pub is_main: bool,
}

#[derive(Clone, Debug)]
pub struct CmdMoveCharacter {
    pub obj_id: ObjID,
    pub direction: Vector2<Fx>,
    pub is_moving: bool,
}

#[derive(Clone, Debug)]
pub struct CmdJumpCharacter {
    pub obj_id: ObjID,
}

#[derive(Clone, Debug)]
pub struct CmdNewSkill {
    pub obj_id: ObjID,
    pub skill_id: String,
    pub position: Option<Point3<Fx>>,
    pub source_id: Option<ObjID>,
    pub target_id: Option<ObjID>,
}
