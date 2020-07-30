use super::base::{CollisionHandle, INVAILD_COLLISION_HANDLE};
use super::{LogicObj, LogicObjX, LogicStage};
use crate as core;
use crate::id::{ObjID, CLASS_CHARACTER};
use crate::logic::base::CollideContext;
use crate::logic::{CmdMoveCharacter, CmdNewCharacter, NewContext, StateContext, UpdateContext, CmdJumpCharacter};
use crate::state::{StateDataX, StateLifecycle};
use crate::util::RcCell;
use failure::Error;
use m::{fx, Fx};
use na::{Isometry3, Point3, Vector2, Vector3, Rotation3, Translation3, UnitQuaternion};
use ncollide3d::pipeline::CollisionGroups;
use ncollide3d::query::{Proximity, Ray};
use ncollide3d::shape::{Capsule, ShapeHandle};
use euclid::Vector3D;

#[derive(LogicObjX)]
#[class_id(CLASS_CHARACTER)]
pub struct LogicCharacter {
    obj_id: ObjID,
    lifecycle: StateLifecycle,
    coll_handle: CollisionHandle,
    p_init_isometry: Isometry3<Fx>,
    p_direction: Vector2<Fx>,
    p_is_moving: bool,
    p_speed: Fx,
    v_isometry: Isometry3<Fx>,
    v_on_ground: u32,
    v_gravity_speed: Fx,
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
            p_init_isometry: Isometry3::new(Vector3::new(fx(0), fx(0.7), fx(0)), na::zero()),
            p_direction: cmd.direction,
            p_is_moving: false,
            p_speed: cmd.speed,
            v_isometry: Isometry3::new(
                Vector3::new(cmd.position.x, cmd.position.y, cmd.position.z),
                na::zero(),
            ),
            v_on_ground: 0,
            v_gravity_speed: fx(0),
        });
        let (coll_handle, _) = ctx.new_collision(
            chara.borrow().v_isometry * chara.borrow().p_init_isometry,
            ShapeHandle::new(Capsule::new(fx(0.35), fx(0.5))),
            CollisionGroups::new(),
            chara.clone(),
        );
        chara.borrow_mut().coll_handle = coll_handle;
        return chara;
    }

    pub(crate) fn mov(&mut self, cmd: &CmdMoveCharacter) {
        self.p_direction = cmd.direction;
        self.p_is_moving = cmd.is_moving;
    }

    pub(crate) fn jump(&mut self, _cmd: &CmdJumpCharacter) {
        self.v_on_ground = 0;
        self.v_gravity_speed += fx(8);
    }
}

impl LogicObj for LogicCharacter {
    fn collide(&mut self, ctx: &mut CollideContext) -> Result<(), Error> {
        if ctx.other_coll.data().is::<LogicStage>() {
            if ctx.prev_status == Proximity::Disjoint {
                self.v_on_ground += 1;
            }
            if ctx.new_status == Proximity::Disjoint {
                self.v_on_ground = u32::max(self.v_on_ground - 1, 0);
            }
        }
        return Ok(());
    }

    fn update(&mut self, ctx: &mut UpdateContext) -> Result<(), Error> {
        // gravity
        if self.v_on_ground > 0 {
            self.v_gravity_speed = fx(0);
        } else {
            self.v_gravity_speed += fx(-10) * ctx.duration;
        };
        let gravity_speed = Vector3::new(fx(0), self.v_gravity_speed, fx(0));

        // move
        let mut move_speed = Vector3::new(fx(0), fx(0), fx(0));
        if self.p_is_moving {
            let ray = Ray::<Fx>::new(
                m::is3_to_p3(self.v_isometry),
                Vector3::new(fx(0), fx(-1), fx(0)),
            );
            let mut direction = Vector3::new(self.p_direction.x, fx(0), self.p_direction.y);
            if self.v_on_ground > 0 {
                if let Some(standing) = ctx.interference_with_ray(&ray, &CollisionGroups::new()) {
                    direction = m::direction_on_plane(&standing.inter.normal, &self.p_direction);
                };
            };
            move_speed = direction * self.p_speed;
        }

        // isometry
        let pt = self.v_isometry.translation.vector + (gravity_speed + move_speed) * ctx.duration;
        let translation = Translation3::from(pt);
        let quaternion = UnitQuaternion::rotation_between(
            &Vector3::new(self.p_direction.x, fx(0), self.p_direction.y),
            &Vector3::new(fx(0), fx(0), fx(-1)),
        ).unwrap();
        self.v_isometry = Isometry3::from_parts(translation, quaternion);

        ctx.update_collision(
            self.coll_handle,
            self.v_isometry * self.p_init_isometry,
        )?;
        return Ok({});
    }

    fn state(&mut self, ctx: &mut StateContext) -> Result<(), Error> {
        let state = ctx.make::<StateCharacter>(self.obj_id, self.lifecycle);
        self.lifecycle = StateLifecycle::Updated;
        state.isometry = self.v_isometry;
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
