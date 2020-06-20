use crate::id::{ObjID, ObjectIDGener};
use crate::logic::*;
use crate::state::StatePool;
use crate::utils::{mut_ptr, Fixed64, fixed64};
use failure::{format_err, Error};
use ncollide3d::pipeline::world::CollisionWorld;
use std::collections::HashMap;
use std::mem;
use std::ptr;
use std::time::Duration;

const STATE_POOL_SIZE: usize = 1024 * 1024 * 4;

pub struct LogicEngine {
    id_gener: ObjectIDGener,
    stage: Option<Box<LogicStage>>,
    characters: HashMap<ObjID, Box<LogicCharacter>>,
    world: CollisionWorld<Fixed64, ()>,
}

impl Drop for LogicEngine {
    fn drop(&mut self) {
    }
}

impl LogicEngine {
    pub fn new() -> Box<LogicEngine> {
        return Box::new(LogicEngine {
            id_gener: ObjectIDGener::new(),
            stage: None,
            characters: HashMap::new(),
            world: CollisionWorld::new(fixed64(0.02)),
        });
    }

    pub fn update(&mut self, dura: Duration) -> Result<Box<StatePool>, Error> {
        let mut state_pool = Box::new(StatePool::new(STATE_POOL_SIZE));

        // update stage
        if let Some(stage) = &mut self.stage {
            stage.update(&mut state_pool, dura)?;
        }

        // update character
        for (_, chara) in self.characters.iter_mut() {
            chara.update(&mut state_pool, dura)?;
        }

        return Ok(state_pool);
    }

    pub(super) fn gene_obj_id(&mut self) -> ObjID {
        return self.id_gener.gen();
    }

    pub(super) fn register_stage(&mut self, mut stage: Box<LogicStage>) -> Result<(), Error> {
        if self.stage.is_some() {
            return Err(format_err!("Stage already exists."));
        }
        self.stage = Some(stage);
        return Ok(());
    }

    pub(super) fn register_character(
        &mut self,
        mut chara: Box<LogicCharacter>,
    ) -> Result<(), Error> {
        self.characters.insert(chara.obj_id(), chara);
        return Ok(());
    }
}
