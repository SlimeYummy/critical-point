use crate::id::{ObjID, ObjIDGener};
use crate::logic::base::CollideContext;
use crate::logic::skill::LogicSkill;
use crate::logic::*;
use crate::state::StatePool;
use crate::util::{make_err, RcCell};
use failure::{format_err, Error};
use m::{fx, Fx};
use na::Vector2;
use ncollide3d::pipeline::world::CollisionWorld;
use std::collections::HashMap;

const STATE_POOL_SIZE: usize = 1024 * 1024 * 4;

pub struct LogicEngine {
    duration: Fx,
    id_gener: ObjIDGener,
    stage: Option<RcCell<LogicStage>>,
    main_character: Option<RcCell<LogicCharacter>>,
    characters: HashMap<ObjID, RcCell<LogicCharacter>>,
    skills: HashMap<ObjID, RcCell<LogicSkill>>,
    world: CollisionWorld<Fx, RcCell<dyn LogicObj>>,
}

impl Drop for LogicEngine {
    fn drop(&mut self) {}
}

impl LogicEngine {
    pub fn new(duration: Fx) -> LogicEngine {
        return LogicEngine {
            duration,
            id_gener: ObjIDGener::new(),
            stage: None,
            main_character: None,
            characters: HashMap::with_capacity(64),
            skills: HashMap::with_capacity(1024),
            world: CollisionWorld::new(fx(0.02)),
        };
    }

    pub fn tick(&mut self) -> Result<Box<StatePool>, Error> {
        self.collide()?;
        self.update()?;
        return self.state();
    }

    pub fn collide(&mut self) -> Result<(), Error> {
        self.world.update();
        let proximity_events = self.world.proximity_events();
        for event in proximity_events {
            let coll_obj1 = self
                .world
                .collision_object(event.collider1)
                .ok_or_else(|| format_err!("CollisionWorld.collision_object(1)"))?;
            let coll_obj2 = self
                .world
                .collision_object(event.collider2)
                .ok_or_else(|| format_err!("CollisionWorld.collision_object(2)"))?;

            let mut logic_obj1 = coll_obj1.data().try_borrow_mut()?;
            logic_obj1.collide(&mut CollideContext::new(
                coll_obj1,
                coll_obj2,
                event.prev_status,
                event.new_status,
            ))?;

            let mut logic_obj2 = coll_obj2.data().try_borrow_mut()?;
            logic_obj2.collide(&mut CollideContext::new(
                coll_obj2,
                coll_obj1,
                event.prev_status,
                event.new_status,
            ))?;
        }
        return Ok(());
    }

    fn update(&mut self) -> Result<(), Error> {
        if let Some(stage) = &self.stage {
            let mut ctx = UpdateContext::new(&mut self.world, self.duration);
            stage.try_borrow_mut()?.update(&mut ctx)?;
        }
        for (_, chara) in self.characters.iter() {
            let mut ctx = UpdateContext::new(&mut self.world, self.duration);
            chara.try_borrow_mut()?.update(&mut ctx)?;
        }
        return Ok(());
    }

    fn state(&mut self) -> Result<Box<StatePool>, Error> {
        let mut state_pool = Box::new(StatePool::new(STATE_POOL_SIZE));
        let mut ctx = StateContext::new(&mut state_pool);
        if let Some(stage) = &self.stage {
            stage.try_borrow_mut()?.state(&mut ctx)?;
        }
        for (_, chara) in self.characters.iter() {
            chara.try_borrow_mut()?.state(&mut ctx)?;
        }
        return Ok(state_pool);
    }
}

impl LogicEngine {
    pub fn operate(&mut self, op: &Operation) -> Result<Command, Error> {
        return match op {
            Operation::MoveCharacter(op) => self.op_move_character(op),
            Operation::JumpCharacter(op) => self.op_jump_character(op),
        };
    }

    fn op_move_character(&mut self, op: &OpMoveCharacter) -> Result<Command, Error> {
        if let Some(chara) = &self.main_character {
            return Ok(Command::MoveCharacter(CmdMoveCharacter {
                obj_id: chara.borrow().obj_id(),
                direction: Vector2::new(fx(op.direction.x), fx(op.direction.y)),
                is_moving: op.is_moving,
            }));
        } else {
            return make_err("LogicEngine::op_move_character() => not found");
        }
    }

    fn op_jump_character(&mut self, _op: &OpJumpCharacter) -> Result<Command, Error> {
        if let Some(chara) = &self.main_character {
            return Ok(Command::JumpCharacter(CmdJumpCharacter {
                obj_id: chara.borrow().obj_id(),
            }));
        } else {
            return make_err("LogicEngine::op_jump_character() => not found");
        }
    }
}

impl LogicEngine {
    pub fn command(&mut self, cmd: &Command) -> Result<(), Error> {
        match cmd {
            Command::NewStage(cmd) => self.cmd_new_stage(cmd)?,
            Command::NewCharacter(cmd) => self.cmd_new_character(cmd)?,
            Command::MoveCharacter(cmd) => self.cmd_move_character(cmd)?,
            Command::JumpCharacter(cmd) => self.cmd_jump_character(cmd)?,
        };
        return Ok(());
    }

    fn cmd_new_stage(&mut self, _: &CmdNewStage) -> Result<(), Error> {
        if self.stage.is_some() {
            return make_err("Stage already exists.");
        }
        let obj_id = self.id_gener.gen();
        let mut ctx = NewContext::new(&mut self.world, obj_id);
        self.stage = Some(LogicStage::new(&mut ctx));
        return Ok(());
    }

    fn cmd_new_character(&mut self, cmd: &CmdNewCharacter) -> Result<(), Error> {
        let obj_id = self.id_gener.gen();
        let mut ctx = NewContext::new(&mut self.world, obj_id);
        let chara = LogicCharacter::new(&mut ctx, cmd);
        if cmd.is_main {
            if self.main_character.is_some() {
                return make_err("Main character already exists.");
            } else {
                self.main_character = Some(chara.clone());
            }
        }
        self.characters.insert(obj_id, chara);
        return Ok(());
    }

    fn cmd_move_character(&mut self, cmd: &CmdMoveCharacter) -> Result<(), Error> {
        if let Some(chara) = self.characters.get(&cmd.obj_id) {
            chara.borrow_mut().mov(cmd);
            return Ok(());
        } else {
            return make_err("LogicEngine::cmd_move_character() => not found");
        }
    }

    fn cmd_jump_character(&mut self, cmd: &CmdJumpCharacter) -> Result<(), Error> {
        if let Some(chara) = self.characters.get(&cmd.obj_id) {
            chara.borrow_mut().jump(cmd);
            return Ok(());
        } else {
            return make_err("LogicEngine::cmd_jump_character() => not found");
        }
    }
}
