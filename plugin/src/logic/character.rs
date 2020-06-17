use crate::cpp::bullet as bt;
use crate::id::{ObjID, TYPE_CHARACTER};
use crate::logic::{logic_obj, LogicObj};
use crate::state::{state_data, StateData, StateLifecycle, StatePool};
use crate::utils::mut_ptr;
use failure::Error;
use failure::_core::time::Duration;
use std::mem;

#[logic_obj(TYPE_CHARACTER)]
pub struct LogicCharacter {
    pub(super) lifecycle: StateLifecycle,
    pub(super) bullet: Bullet,
}

pub struct Bullet {
    pub(super) shape: bt::btBoxShape,
    pub(super) motion: bt::btDefaultMotionState,
    pub(super) body: bt::btRigidBody,
}

impl Drop for LogicCharacter {
    fn drop(&mut self) {
        unsafe {
            bt::btPolyhedralConvexShape_btPolyhedralConvexShape_destructor(mut_ptr(
                &mut self.bullet.shape,
            ));
            bt::btRigidBody_btRigidBody_destructor(&mut self.bullet.body);
        };
    }
}

impl LogicCharacter {
    pub(super) fn new(obj_id: ObjID) -> Box<LogicCharacter> {
        let mut chara = Box::new(LogicCharacter {
            sup: Self::new_super(obj_id),
            lifecycle: StateLifecycle::Created,
            bullet: unsafe { mem::zeroed() },
        });

        unsafe {
            chara.bullet.shape = bt::btBoxShape::new(&bt::btVector3::new1(&1.0, &1.0, &1.0));

            let mut transform = bt::btTransform::new();
            transform.setIdentity();
            transform.setOrigin(&bt::btVector3::new1(&0.0, &100.0, &0.0));
            chara.bullet.motion =
                bt::bthNewDefaultMotionState(&transform, bt::btTransform_getIdentity());

            let mut inertia = bt::btVector3::new1(&0.0, &0.0, &0.0);
            bt::btBoxShape_calculateLocalInertia(
                mut_ptr(&mut chara.bullet.shape),
                5.0,
                &mut inertia,
            );

            chara.bullet.body = bt::btRigidBody::new1(
                5.0,
                mut_ptr(&mut chara.bullet.motion),
                mut_ptr(&mut chara.bullet.shape),
                &inertia,
            );
        };
        return chara;
    }
}

impl LogicObj for LogicCharacter {
    fn update(&mut self, pool: &mut Box<StatePool>, _: Duration) -> Result<(), Error> {
        let state = pool.make::<StateCharacter>(self.obj_id(), self.lifecycle);
        self.lifecycle = StateLifecycle::Updated;
        unsafe {
            state.transform = bt::btTransform::new();
            state.transform.setIdentity();
            bt::btDefaultMotionState_getWorldTransform(
                mut_ptr(&mut self.bullet.motion),
                &mut state.transform,
            );
        };
        return Ok(());
    }
}

#[state_data(TYPE_CHARACTER)]
#[derive(Debug)]
pub struct StateCharacter {
    pub transform: bt::btTransform,
}

impl StateData for StateCharacter {}

impl Default for StateCharacter {
    fn default() -> Self {
        return StateCharacter {
            sup: Self::default_super(),
            transform: unsafe { mem::zeroed() },
        };
    }
}
