use super::base::{CollisionHandle, INVAILD_COLLISION_HANDLE};
use super::{LogicObj, LogicObjX, LogicStage};
use crate as core;
use crate::id::{ObjID, CLASS_CHARACTER};
use crate::logic::base::CollideContext;
use crate::logic::{
    CmdJumpCharacter, CmdMoveCharacter, CmdNewCharacter, NewContext, StateContext, UpdateContext,
};
use crate::state::{StateDataX, StateLifecycle};
use crate::util::RcCell;
use failure::Error;
use m::{fx, Fx};
use na::{Isometry3, Point3, Rotation3, Translation3, UnitQuaternion, Vector2, Vector3};
use ncollide3d::pipeline::CollisionGroups;
use ncollide3d::query::{Proximity, Ray};
use ncollide3d::shape::{Capsule, ShapeHandle};
use serde::{Deserialize, Serialize};

struct Parameter {
    direction: Vector2<Fx>,
    is_moving: bool,
    speed: Fx,
}

struct Variable {
    isometry: Isometry3<Fx>,
    on_ground: u32,
    gravity_speed: Fx,
}

#[derive(LogicObjX)]
#[class_id(CLASS_CHARACTER)]
pub struct LogicCharacter {
    obj_id: ObjID,
    lifecycle: StateLifecycle,
    coll_handle: CollisionHandle,
    // c: ResCharacter,
    p: Parameter,
    v: Variable,
}

impl Drop for LogicCharacter {
    fn drop(&mut self) {}
}

impl LogicCharacter {
    pub(crate) fn new(ctx: &mut NewContext, cmd: &CmdNewCharacter) -> RcCell<LogicCharacter> {
        let chara = RcCell::new(LogicCharacter {
            obj_id: ctx.obj_id,
            lifecycle: StateLifecycle::Created,
            coll_handle: INVAILD_COLLISION_HANDLE,
            // c: ResCharacter {},
            p: Parameter {
                direction: cmd.direction,
                is_moving: false,
                speed: cmd.speed,
            },
            v: Variable {
                isometry: Isometry3::new(
                    Vector3::new(cmd.position.x, cmd.position.y, cmd.position.z),
                    na::zero(),
                ),
                on_ground: 0,
                gravity_speed: fx(0),
            },
        });
        let (coll_handle, _) = ctx.new_collision(
            // chara.borrow().v.isometry * chara.borrow().c.init_isometry,
            Isometry3::new(na::zero(), na::zero()),
            ShapeHandle::new(Capsule::new(fx(0.35), fx(0.5))),
            CollisionGroups::new(),
            chara.clone(),
        );
        chara.borrow_mut().coll_handle = coll_handle;
        return chara;
    }

    pub(crate) fn mov(&mut self, cmd: &CmdMoveCharacter) {
        self.p.direction = cmd.direction;
        self.p.is_moving = cmd.is_moving;
    }

    pub(crate) fn jump(&mut self, _cmd: &CmdJumpCharacter) {
        self.v.on_ground = 0;
        self.v.gravity_speed += fx(8);
    }
}

impl LogicObj for LogicCharacter {
    fn collide(&mut self, ctx: &mut CollideContext) -> Result<(), Error> {
        if ctx.other_coll.data().is::<LogicStage>() {
            if ctx.prev_status == Proximity::Disjoint {
                self.v.on_ground += 1;
            }
            if ctx.new_status == Proximity::Disjoint {
                self.v.on_ground = u32::max(self.v.on_ground - 1, 0);
            }
        }
        return Ok(());
    }

    fn update(&mut self, ctx: &mut UpdateContext) -> Result<(), Error> {
        // gravity
        if self.v.on_ground > 0 {
            self.v.gravity_speed = fx(0);
        } else {
            self.v.gravity_speed += fx(-10) * ctx.duration;
        };
        let gravity_speed = Vector3::new(fx(0), self.v.gravity_speed, fx(0));

        // move
        let mut move_speed = Vector3::new(fx(0), fx(0), fx(0));
        if self.p.is_moving {
            let ray = Ray::<Fx>::new(
                m::is3_to_p3(self.v.isometry),
                Vector3::new(fx(0), fx(-1), fx(0)),
            );
            let mut direction = Vector3::new(self.p.direction.x, fx(0), self.p.direction.y);
            if self.v.on_ground > 0 {
                if let Some(standing) = ctx.interference_with_ray(&ray, &CollisionGroups::new()) {
                    direction = m::direction_on_plane(&standing.inter.normal, &self.p.direction);
                };
            };
            move_speed = direction * self.p.speed;
        }

        // isometry
        let pt = self.v.isometry.translation.vector + (gravity_speed + move_speed) * ctx.duration;
        let translation = Translation3::from(pt);
        let quaternion = UnitQuaternion::rotation_between(
            &Vector3::new(self.p.direction.x, fx(0), self.p.direction.y),
            &Vector3::new(fx(0), fx(0), fx(-1)),
        )
        .unwrap();
        self.v.isometry = Isometry3::from_parts(translation, quaternion);

        // ctx.update_collision(self.coll_handle, self.v.isometry * self.c.init_isometry)?;
        ctx.update_collision(self.coll_handle, self.v.isometry)?;
        return Ok({});
    }

    fn state(&mut self, ctx: &mut StateContext) -> Result<(), Error> {
        let state = ctx.make::<StateCharacter>(self.obj_id, self.lifecycle);
        self.lifecycle = StateLifecycle::Updated;
        state.isometry = self.v.isometry;
        return Ok(());
    }
}

#[derive(StateDataX, Debug)]
#[class_id(CLASS_CHARACTER)]
pub struct StateCharacter {
    pub obj_id: ObjID,
    pub lifecycle: StateLifecycle,
    pub isometry: Isometry3<Fx>,
}

impl Default for StateCharacter {
    fn default() -> StateCharacter {
        return StateCharacter {
            obj_id: ObjID::default(),
            lifecycle: StateLifecycle::default(),
            isometry: Isometry3::identity(),
        };
    }
}
