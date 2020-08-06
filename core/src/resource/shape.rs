use crate::util::make_err;
use failure::Error;
use fixed::traits::ToFixed;
use m::{fx, Fx};
use na::{RealField, Vector3};
use ncollide3d::shape::{Ball, Capsule, Cuboid, ShapeHandle};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{self, Debug, Formatter};

#[derive(Default)]
pub struct ShapeCache {
    cache: HashMap<ResShape, ShapeHandle<Fx>>,
}

impl Debug for ShapeCache {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        return f
            .debug_map()
            .entries(self.cache.keys().map(|k| (k, ())))
            .finish();
    }
}

impl ShapeCache {
    fn cache(&mut self, shape: ResShape) -> bool {
        if self.cache.contains_key(&shape) {
            return false;
        }
        let shape_handle = match shape.clone() {
            ResShape::Ball(ball) => ShapeHandle::new(Ball::new(ball.radius)),
            ResShape::Cuboid(cuboid) => {
                ShapeHandle::new(Cuboid::new(Vector3::new(cuboid.x, cuboid.y, cuboid.z)))
            }
            ResShape::Capsule(capsule) => {
                ShapeHandle::new(Capsule::new(capsule.half_height, capsule.radius))
            }
        };
        let res = self.cache.insert(shape, shape_handle);
        debug_assert!(res.is_none());
        return true;
    }

    fn find(&self, shape: ResShape) -> Result<ShapeHandle<Fx>, Error> {
        return match self.cache.get(&shape) {
            Some(shape_handle) => Ok((*shape_handle).clone()),
            None => make_err("ShapeCache::find() => shape not found"),
        };
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum ResShape {
    Ball(ResBall),
    Cuboid(ResCuboid),
    Capsule(ResCapsule),
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ResBall {
    radius: Fx,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ResCuboid {
    x: Fx,
    y: Fx,
    z: Fx,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ResCapsule {
    half_height: Fx,
    radius: Fx,
}
