use crate::id::{ObjID, ObjectIDGener};
use crate::logic::*;
use crate::state::StatePool;
use crate::utils::{fixed64, mut_ptr, Fixed64};
use failure::{format_err, Error};
use na::Isometry3;
use ncollide3d::pipeline::world::CollisionWorld;
use ncollide3d::pipeline::{CollisionGroups, GeometricQueryType};
use std::collections::HashMap;
use std::time::Duration;

const STATE_POOL_SIZE: usize = 1024 * 1024 * 4;

pub struct LogicEngine {
    id_gener: ObjectIDGener,
    stage: Option<Box<LogicStage>>,
    characters: HashMap<ObjID, Box<LogicCharacter>>,
    world: CollisionWorld<Fixed64, ()>,
}

impl Drop for LogicEngine {
    fn drop(&mut self) {}
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
}

impl LogicEngine {
    fn cmd_new_stage(&mut self, _: CmdNewStage) -> Result<(), Error> {
        if self.stage.is_some() {
            return Err(format_err!("Stage already exists."));
        }
        let obj_id = self.id_gener.gen();
        let stage = LogicStage::new(obj_id);
        self.world.add(
            Isometry3::new(na::zero(), na::zero()),
            stage.shape.clone(),
            CollisionGroups::new(),
            GeometricQueryType::Proximity(fixed64(0.0)),
            (),
        );
        self.stage = Some(stage);
        return Ok(());
    }

    fn cmd_new_character(&mut self, cmd: CmdNewCharacter) -> Result<(), Error> {
        let obj_id = self.id_gener.gen();
        let chara = LogicCharacter::new(obj_id);
        self.world.add(
            Isometry3::new(cmd.position, na::zero()),
            chara.shape.clone(),
            CollisionGroups::new(),
            GeometricQueryType::Proximity(fixed64(0.0)),
            (),
        );
        self.characters.insert(chara.obj_id(), chara);
        return Ok(());
    }
}
