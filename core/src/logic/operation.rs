use crate::id::ObjID;
use m::Fx;
use na::{Point3, Vector2};

#[derive(Clone, Debug)]
pub enum Operation {
    MoveCharacter(OpMoveCharacter),
}

#[derive(Clone, Debug)]
pub struct OpMoveCharacter {
    pub direction: Vector2<Fx>,
}
