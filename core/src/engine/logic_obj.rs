use super::logic_data::DataPool;
use crate::id::{ClassID, ObjID};
use crate::physics::PhysicsEngine;
use anyhow::Result;
use std::mem;

pub trait LogicObjStatic {
    fn id() -> ClassID;
}

pub trait LogicObjSuper {
    fn class_id(&self) -> ClassID;
    fn obj_id(&self) -> ObjID;
}

pub trait LogicObj: LogicObjSuper {
    fn update_prop(&mut self, pool: &mut DataPool) -> Result<()>;
    fn update_state(&mut self, pool: &mut DataPool) -> Result<()>;
}

impl dyn LogicObj {
    #[inline]
    pub fn is<O>(&self) -> bool
    where
        O: LogicObj + LogicObjStatic,
    {
        return self.class_id() == O::id();
    }

    #[inline]
    pub fn cast<O>(&self) -> Option<&O>
    where
        O: LogicObj + LogicObjStatic,
    {
        if self.class_id() == O::id() {
            return Some(unsafe { mem::transmute_copy(&self) });
        } else {
            return None;
        }
    }

    #[inline]
    pub fn cast_mut<O>(&mut self) -> Option<&mut O>
    where
        O: LogicObj + LogicObjStatic,
    {
        if self.class_id() == O::id() {
            return Some(unsafe { mem::transmute_copy(&self) });
        } else {
            return None;
        }
    }
}

pub trait LogicStage: LogicObj {}

pub trait LogicChara: LogicObj {
    fn update_position(&mut self, phy: &mut PhysicsEngine) -> Result<()>;
    fn solve_penetration(&mut self, phy: &mut PhysicsEngine) -> Result<bool>;
    fn update_skeleton(&mut self, phy: &mut PhysicsEngine) -> Result<()>;
}

pub trait LogicHit: LogicObj {}
