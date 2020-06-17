use crate::cpp::bullet as bt;
use crate::id::{ObjID, ObjectIDGener};
use crate::logic::*;
use crate::state::StatePool;
use crate::utils::mut_ptr;
use failure::{format_err, Error};
use std::collections::HashMap;
use std::mem;
use std::ptr;
use std::time::Duration;

const STATE_POOL_SIZE: usize = 1024 * 1024 * 4;

pub struct LogicEngine {
    id_gener: ObjectIDGener,
    stage: Option<Box<LogicStage>>,
    characters: HashMap<ObjID, Box<LogicCharacter>>,
    bullet: Bullet,
}

struct Bullet {
    config: bt::btDefaultCollisionConfiguration,
    dispatcher: bt::btCollisionDispatcher,
    broadphase: bt::btDbvtBroadphase,
    solver: bt::btSequentialImpulseConstraintSolver,
    world: bt::btDiscreteDynamicsWorld,
}

impl Drop for LogicEngine {
    fn drop(&mut self) {
        unsafe {
            bt::btDiscreteDynamicsWorld_btDiscreteDynamicsWorld_destructor(&mut self.bullet.world);
            bt::btSequentialImpulseConstraintSolver_btSequentialImpulseConstraintSolver_destructor(
                &mut self.bullet.solver,
            );
            bt::btDbvtBroadphase_btDbvtBroadphase_destructor(&mut self.bullet.broadphase);
            bt::btCollisionDispatcher_btCollisionDispatcher_destructor(&mut self.bullet.dispatcher);
            bt::btDefaultCollisionConfiguration_btDefaultCollisionConfiguration_destructor(
                &mut self.bullet.config,
            );
        };
    }
}

impl LogicEngine {
    pub fn new() -> Box<LogicEngine> {
        let mut engine: Box<LogicEngine> = Box::new(LogicEngine {
            id_gener: ObjectIDGener::new(),
            stage: None,
            characters: HashMap::new(),
            bullet: unsafe { mem::zeroed() },
        });
        unsafe {
            engine.bullet.config = bt::btDefaultCollisionConfiguration::new(
                &bt::bthNewDefaultCollisionConstructionInfo(),
            );
            engine.bullet.dispatcher =
                bt::btCollisionDispatcher::new(mut_ptr(&mut engine.bullet.config));
            engine.bullet.broadphase = bt::btDbvtBroadphase::new(ptr::null_mut());
            engine.bullet.solver = bt::btSequentialImpulseConstraintSolver::new();
            engine.bullet.world = bt::btDiscreteDynamicsWorld::new(
                mut_ptr(&mut engine.bullet.dispatcher),
                mut_ptr(&mut engine.bullet.broadphase),
                mut_ptr(&mut engine.bullet.solver),
                mut_ptr(&mut engine.bullet.config),
            );

            bt::btDiscreteDynamicsWorld_setGravity(
                mut_ptr(&mut engine.bullet.world),
                &bt::btVector3::new1(&0.0, &-10.0, &0.0),
            );
        };
        return engine;
    }

    pub fn update(&mut self, dura: Duration) -> Result<Box<StatePool>, Error> {
        unsafe {
            bt::btDiscreteDynamicsWorld_stepSimulation(
                mut_ptr(&mut self.bullet.world),
                dura.as_secs_f32(),
                60,
                1.0 / 60.0,
            );
        };

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
        unsafe {
            bt::btDiscreteDynamicsWorld_addRigidBody(
                mut_ptr(&mut self.bullet.world),
                &mut stage.bullet.body,
            )
        };
        self.stage = Some(stage);
        return Ok(());
    }

    pub(super) fn register_character(
        &mut self,
        mut chara: Box<LogicCharacter>,
    ) -> Result<(), Error> {
        unsafe {
            bt::btDiscreteDynamicsWorld_addRigidBody(
                mut_ptr(&mut self.bullet.world),
                &mut chara.bullet.body,
            )
        };
        self.characters.insert(chara.obj_id(), chara);
        return Ok(());
    }
}
