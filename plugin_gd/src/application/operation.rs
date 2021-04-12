use core::engine::OpCommand;
use na::Vector2;

#[derive(Debug, Clone)]
pub enum Operation {
    Core(OpCommand),
    ChangeMouseMode(OpChangeMouseMode),
    MoveCamera(OpMoveCamera),
}

#[derive(Debug, Clone)]
pub struct OpChangeMouseMode {}

#[derive(Debug, Clone)]
pub struct OpMoveCamera {
    pub speed: Vector2<f32>,
}

pub fn op_core(cmd: OpCommand) -> Option<Operation> {
    return Some(Operation::Core(cmd));
}
