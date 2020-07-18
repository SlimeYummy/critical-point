use crate::id::{ClassID, ObjID};
use crate::state::{StateData, StateDataStatic, StateLifecycle, StatePool};
use crate::util::{make_err, RcCell};
use failure::Error;
use m::{fx, Fx};
use na::Isometry3;
use ncollide3d::pipeline::{
    self, CollisionGroups, CollisionObjectSlab, CollisionObjectSlabHandle, CollisionWorld,
    FirstInterferenceWithRay, GeometricQueryType,
};
use ncollide3d::query::{Proximity, Ray};
use ncollide3d::shape::ShapeHandle;
use std::mem;

pub(super) type CollisionObject = pipeline::CollisionObject<Fx, RcCell<dyn LogicObj>>;
pub(super) type CollisionHandle = CollisionObjectSlabHandle;

pub(super) const INVAILD_COLLISION_HANDLE: CollisionObjectSlabHandle =
    CollisionObjectSlabHandle(0xFFFF_FFFF_FFFF_FFFF);

//
// Logic Object
//

pub trait LogicObjSuper {
    fn obj_id(&self) -> ObjID;
    fn class_id(&self) -> ClassID;
}

pub trait LogicObjStatic {
    fn id() -> ClassID;
}

pub trait LogicObj
where
    Self: LogicObjSuper,
{
    fn collide(&mut self, ctx: &mut CollideContext) -> Result<(), Error>;
    fn update(&mut self, ctx: &mut UpdateContext) -> Result<(), Error>;
    fn state(&mut self, ctx: &mut StateContext) -> Result<(), Error>;
}

impl dyn LogicObj {
    pub fn is<L>(&self) -> bool
    where
        L: LogicObj + LogicObjStatic,
    {
        return self.class_id() == L::id();
    }

    pub fn cast_ref<L>(&self) -> Option<&L>
    where
        L: LogicObj + LogicObjStatic,
    {
        if self.class_id() == L::id() {
            return Some(unsafe { mem::transmute_copy(&self) });
        } else {
            return None;
        }
    }

    pub fn cast_mut<L>(&mut self) -> Option<&mut L>
    where
        L: LogicObj + LogicObjStatic,
    {
        if self.class_id() == L::id() {
            return Some(unsafe { mem::transmute_copy(&self) });
        } else {
            return None;
        }
    }
}

impl RcCell<dyn LogicObj> {
    pub fn is<L>(&self) -> bool
    where
        L: LogicObj + LogicObjStatic,
    {
        let refer = unsafe { RcCell::as_ref_mut(self) };
        return refer.class_id() == L::id();
    }

    pub fn cast<L>(&self) -> Option<RcCell<L>>
    where
        L: LogicObj + LogicObjStatic,
    {
        let new_self = self.clone();
        mem::forget(new_self);
        if self.borrow().class_id() == L::id() {
            return Some(unsafe { mem::transmute_copy(self) });
        } else {
            return None;
        }
    }
}

//
// contexts
//

pub struct NewContext<'t> {
    world: &'t mut CollisionWorld<Fx, RcCell<dyn LogicObj>>,
    pub obj_id: ObjID,
}

impl<'t> NewContext<'t> {
    pub fn new(
        world: &'t mut CollisionWorld<Fx, RcCell<dyn LogicObj>>,
        obj_id: ObjID,
    ) -> NewContext<'t> {
        return NewContext { world, obj_id };
    }

    pub fn new_collision(
        &mut self,
        position: Isometry3<Fx>,
        shape: ShapeHandle<Fx>,
        groups: CollisionGroups,
        logic_obj: RcCell<dyn LogicObj>,
    ) -> (CollisionHandle, &mut CollisionObject) {
        return self.world.add(
            position,
            shape,
            groups,
            GeometricQueryType::Proximity(fx(0.0)),
            logic_obj,
        );
    }
}

pub struct CollideContext<'t> {
    pub self_coll: &'t CollisionObject,
    pub other_coll: &'t CollisionObject,
    pub prev_status: Proximity,
    pub new_status: Proximity,
}

impl<'t> CollideContext<'t> {
    pub fn new(
        self_coll: &'t CollisionObject,
        other_coll: &'t CollisionObject,
        prev_status: Proximity,
        new_status: Proximity,
    ) -> CollideContext<'t> {
        return CollideContext {
            self_coll,
            other_coll,
            prev_status,
            new_status,
        };
    }
}

pub struct UpdateContext<'t> {
    world: &'t mut CollisionWorld<Fx, RcCell<dyn LogicObj>>,
    pub duration: Fx,
}

impl<'t> UpdateContext<'t> {
    pub fn new(
        world: &'t mut CollisionWorld<Fx, RcCell<dyn LogicObj>>,
        duration: Fx,
    ) -> UpdateContext<'t> {
        return UpdateContext { world, duration };
    }

    pub fn interference_with_ray<'a, 'b>(
        &'a mut self,
        ray: &'b Ray<Fx>,
        groups: &'b CollisionGroups,
    ) -> Option<FirstInterferenceWithRay<'a, Fx, CollisionObjectSlab<Fx, RcCell<dyn LogicObj>>>>
    {
        return self
            .world
            .first_interference_with_ray(ray, fx(1000), groups);
    }

    pub fn update_collision(
        &mut self,
        coll_handle: CollisionHandle,
        isometry: Isometry3<Fx>,
    ) -> Result<(), Error> {
        if let Some(coll_obj) = self.world.get_mut(coll_handle) {
            coll_obj.set_position(isometry);
            return Ok(());
        } else {
            return make_err("NewContext::update_collision() ");
        }
    }
}

pub struct StateContext<'t> {
    pool: &'t mut Box<StatePool>,
}

impl<'t> StateContext<'t> {
    pub fn new(pool: &'t mut Box<StatePool>) -> StateContext<'t> {
        return StateContext { pool };
    }

    pub fn make<S>(&mut self, obj_id: ObjID, lifecycle: StateLifecycle) -> &mut S
    where
        S: StateData + StateDataStatic,
    {
        return self.pool.make(obj_id, lifecycle);
    }
}

#[cfg(test)]
mod tests {
    use super::super::LogicObjX;
    use super::*;
    use crate as core;
    use crate::id::CLASS_STAGE;

    #[derive(LogicObjX)]
    #[class_id(CLASS_STAGE)]
    struct LoTest {
        obj_id: ObjID,
    }

    impl LoTest {
        fn new(obj_id: ObjID) -> LoTest {
            return LoTest { obj_id };
        }
    }

    impl LogicObj for LoTest {
        fn collide(&mut self, _ctx: &mut CollideContext) -> Result<(), Error> {
            return Ok(());
        }

        fn update(&mut self, _ctx: &mut UpdateContext) -> Result<(), Error> {
            return Ok(());
        }

        fn state(&mut self, _ctx: &mut StateContext) -> Result<(), Error> {
            return Ok(());
        }
    }

    #[test]
    fn test_logic_obj() {
        let lo1 = LoTest::new(ObjID::from(123));
        let to1: &dyn LogicObj = &lo1;
        let ct1 = to1.cast_ref::<LoTest>().unwrap();
        assert_eq!(ObjID::from(123), ct1.obj_id());

        let mut lo2 = LoTest::new(ObjID::from(123));
        let to2: &mut dyn LogicObj = &mut lo2;
        let ct2 = to2.cast_ref::<LoTest>().unwrap();
        assert_eq!(ObjID::from(123), ct2.obj_id());

        let lo3 = RcCell::new(LoTest::new(ObjID::from(123)));
        let to3: RcCell<dyn LogicObj> = lo3.clone();
        let ct3 = to3.cast::<LoTest>().unwrap();
        assert_eq!(ObjID::from(123), ct3.borrow().obj_id());
    }
}
