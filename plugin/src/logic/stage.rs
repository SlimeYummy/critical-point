use crate::cpp::bullet as bt;
use crate::id::{ObjID, TYPE_STAGE};
use crate::logic::{logic_obj, LogicObj};
use crate::state::{state_data, StateData, StateLifecycle, StatePool};
use crate::utils::mut_ptr;
use failure::Error;
use std::mem;
use std::ptr;
use std::time::Duration;

#[logic_obj(TYPE_STAGE)]
pub struct LogicStage {
    pub(super) lifecycle: StateLifecycle,
    pub(super) bullet: Bullet,
}

pub struct Bullet {
    pub(super) shape: bt::btStaticPlaneShape,
    pub(super) body: bt::btRigidBody,
}

impl Drop for LogicStage {
    fn drop(&mut self) {
        unsafe {
            bt::btStaticPlaneShape_btStaticPlaneShape_destructor(mut_ptr(&mut self.bullet.shape));
            bt::btRigidBody_btRigidBody_destructor(&mut self.bullet.body);
        };
    }
}

impl LogicStage {
    pub(super) fn new(obj_id: ObjID) -> Box<LogicStage> {
        let mut stage: Box<LogicStage> = Box::new(LogicStage {
            sup: Self::new_super(obj_id),
            lifecycle: StateLifecycle::Created,
            bullet: unsafe { mem::zeroed() },
        });

        unsafe {
            stage.bullet.shape =
                bt::btStaticPlaneShape::new(&bt::btVector3::new1(&0.0, &1.0, &0.0), 0.0);
            bt::btConcaveShape_setMargin(mut_ptr(&mut stage.bullet.shape), 0.04);

            stage.bullet.body = bt::btRigidBody::new1(
                0.0,
                ptr::null_mut(),
                mut_ptr(&mut stage.bullet.shape),
                &bt::btVector3::new1(&0.0, &0.0, &0.0),
            );

            let mut transform = bt::btTransform::new();
            transform.setIdentity();
            transform.setOrigin(&bt::btVector3::new1(&0.0, &0.0, &0.0));
            bt::btCollisionObject_setWorldTransform(mut_ptr(&mut stage.bullet.body), &transform);
        };
        return stage;
    }
}

impl LogicObj for LogicStage {
    fn update(&mut self, pool: &mut Box<StatePool>, _: Duration) -> Result<(), Error> {
        let _ = pool.make::<StateStage>(self.obj_id(), self.lifecycle);
        self.lifecycle = StateLifecycle::Updated;
        return Ok(());
    }
}

#[state_data(TYPE_STAGE)]
#[derive(Debug, Default)]
pub struct StateStage {}

impl StateData for StateStage {}
