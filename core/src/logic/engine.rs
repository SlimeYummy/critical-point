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
use na::Isometry3;
use ncollide3d::pipeline::{CollisionGroups, GeometricQueryType, ProximityEvent, CollisionObjectSlabHandle, ContactEvent};
use std::rc::Rc;

const STATE_POOL_SIZE: usize = 1024 * 1024 * 4;

pub struct LogicEngine {
    id_gener: ObjectIDGener,
    stage: Option<Rc<LogicStage>>,
    characters: HashMap<ObjID, Rc<LogicCharacter>>,
    world: CollisionWorld<Fixed64, Rc<dyn LogicObj>>,
}

impl Drop for LogicEngine {
    fn drop(&mut self) {}
}

impl LogicEngine {
    pub fn new() -> Rc<LogicEngine> {
        return Rc::new(LogicEngine {
            id_gener: ObjectIDGener::new(),
            stage: None,
            characters: HashMap::new(),
            world: CollisionWorld::new(fixed64(0.02)),
        });
    }

    pub fn update(&mut self, dura: Duration) -> Result<Box<StatePool>, Error> {
        self.world.update();
        let proximity_events = self.world.proximity_events();
        for event in proximity_events {
            let coll_obj1 = match self.world.collision_object(event.collider1) {
                Some(coll_obj) => coll_obj,
                None => continue,
            };
            let coll_obj2 = match self.world.collision_object(event.collider2)  {
                Some(coll_obj) => coll_obj,
                None => continue,
            };

            let dyn_obj1 = coll_obj1.data().clone();
            let logic_obj1 = unsafe { &mut *(Rc::as_ptr(&dyn_obj1) as *mut dyn LogicObj) };
            let dyn_obj2 = coll_obj2.data().clone();
            let logic_obj2 = unsafe { &mut *(Rc::as_ptr(&dyn_obj2) as *mut dyn LogicObj) };
            logic_obj1.collide(dyn_obj2);
            logic_obj2.collide(dyn_obj1);
        }

        let mut state_pool = Box::new(StatePool::new(STATE_POOL_SIZE));
        if let Some(stage) = &self.stage {
            let stage = unsafe { &mut *(Rc::as_ptr(&stage) as *mut LogicStage) };
            stage.update(&mut state_pool, dura)?;
        }
        for (_, chara) in self.characters.iter() {
            let chara = unsafe { &mut *(Rc::as_ptr(&chara) as *mut LogicCharacter) };
            chara.update(&mut state_pool, dura)?;
        }

        return Ok(state_pool);
    }
}

impl LogicEngine {
    pub fn command(&mut self, cmd: Command) -> Result<(), Error> {
        match cmd {
            Command::NewStage(cmd) => self.cmd_new_stage(cmd)?,
            Command::NewCharacter(cmd) => self.cmd_new_character(cmd)?,
        };
        return Ok(());
    }

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
            stage.clone(),
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
            chara.clone(),
        );
        self.characters.insert(chara.obj_id(), chara);
        return Ok(());
    }
}
