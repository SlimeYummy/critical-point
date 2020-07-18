use super::base::{CollisionHandle, INVAILD_COLLISION_HANDLE};
use super::{LogicObj, LogicObjX, LogicStage};
use crate as core;
use crate::id::{ObjID, CLASS_CHARACTER};
use crate::logic::base::CollideContext;
use crate::logic::{CmdMoveCharacter, CmdNewCharacter, NewContext, StateContext, UpdateContext};
use crate::state::{StateDataX, StateLifecycle};
use crate::util::RcCell;
use failure::Error;
use m::{fx, Fx};
use na::{Isometry3, Point3, Vector2, Vector3};
use ncollide3d::pipeline::CollisionGroups;
use ncollide3d::query::{Proximity, Ray};
use ncollide3d::shape::{Capsule, ShapeHandle};

#[derive(LogicObjX)]
#[class_id(CLASS_CHARACTER)]
pub struct LogicCharacter {
    obj_id: ObjID,
    lifecycle: StateLifecycle,
    coll_handle: CollisionHandle,
    p_direction: Vector2<Fx>,
    p_speed: Fx,
    v_position: Point3<Fx>,
    v_on_ground: u32,
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
            p_direction: cmd.direction,
            p_speed: cmd.speed,
            v_position: cmd.position,
            v_on_ground: 0,
        });
        let (coll_handle, _) = ctx.new_collision(
            Isometry3::new(na::zero(), na::zero()),
            ShapeHandle::new(Capsule::new(fx(0.9), fx(0.5))),
            CollisionGroups::new(),
            chara.clone(),
        );
        chara.borrow_mut().coll_handle = coll_handle;
        return chara;
    }

    pub(crate) fn mov(&mut self, cmd: &CmdMoveCharacter) {
        self.p_direction = cmd.direction;
    }
}

impl LogicObj for LogicCharacter {
    fn collide(&mut self, ctx: &mut CollideContext) -> Result<(), Error> {
        if ctx.other_coll.data().is::<LogicStage>() {
            if ctx.prev_status == Proximity::Disjoint {
                self.v_on_ground += 1;
            }
            if ctx.new_status != Proximity::Disjoint {
                self.v_on_ground -= 1;
            }
            // let stage = other_coll
            //     .data()
            //     .cast::<LogicStage>()
            //     .ok_or_else(|| format_err!("LogicCharacter.collide(LogicStage)"))?;
            // let ray = Ray::<Fx>::new(self.v_position.clone(), Vector3::new(fx(0), fx(-1), fx(0)));
            // if let Some(standing) = other_coll.shape().toi_and_normal_with_ray(
            //     other_coll.position(),
            //     &ray,
            //     fx(1000),
            //     true,
            // ) {
            //     println!("{:?}", self.p_direction);
            //     self.v_direction = m::direction_on_plane(&standing.normal, &self.p_direction);
            // }
        }
        return Ok(());
    }

    fn update(&mut self, ctx: &mut UpdateContext) -> Result<(), Error> {
        let ray = Ray::<Fx>::new(self.v_position.clone(), Vector3::new(fx(0), fx(-1), fx(0)));

        let mut direction = Vector3::new(self.p_direction.x, fx(0), self.p_direction.y);
        if self.v_on_ground != 0 {
            if let Some(standing) = ctx.interference_with_ray(&ray, &CollisionGroups::new()) {
                direction = m::direction_on_plane(&standing.inter.normal, &self.p_direction);
            };
        };

        if self.p_speed != fx(0) {
            let distance = self.p_speed * ctx.duration;
            let transition = direction * distance;
            self.v_position = self.v_position + transition;
        }
        ctx.update_collision(
            self.coll_handle,
            Isometry3::new(m::p3_to_v3(self.v_position), na::zero()),
        )?;
        return Ok({});
    }

    fn state(&mut self, ctx: &mut StateContext) -> Result<(), Error> {
        let state = ctx.make::<StateCharacter>(self.obj_id, self.lifecycle);
        self.lifecycle = StateLifecycle::Updated;
        state.position = self.v_position;
        return Ok(());
    }
}

#[derive(StateDataX, Debug)]
#[class_id(CLASS_CHARACTER)]
pub struct StateCharacter {
    pub obj_id: ObjID,
    pub lifecycle: StateLifecycle,
    pub position: Point3<Fx>,
}

impl Default for StateCharacter {
    fn default() -> StateCharacter {
        return StateCharacter {
            position: Point3::origin(),
            obj_id: ObjID::default(),
            lifecycle: StateLifecycle::default(),
        };
    }
}
