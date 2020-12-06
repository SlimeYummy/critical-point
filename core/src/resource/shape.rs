use super::cache::RestoreContext;
use anyhow::{anyhow, Result};
use derivative::Derivative;
use lazy_static::lazy_static;
use m::{fi, fx_f32, ConeExt, CylinderExt, Fx, HumanBounding};
use na::{Point3, Vector3};
use ncollide3d::shape::{Ball, Capsule, Cuboid, ShapeHandle, TriMesh};
use obj::{load_obj, Obj};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::fs::File;
use std::io::BufReader;

pub(crate) type ShapeCacheKey = ResShape;
pub(crate) type ShapeCacheValue = ShapeHandle<Fx>;

lazy_static! {
    pub static ref DEFAULT_SHAPE_HANDLE: ShapeHandle<Fx> = ShapeHandle::new(Ball::new(fi(1)));
}

pub fn default_shape_handle() -> ShapeHandle<Fx> {
    return DEFAULT_SHAPE_HANDLE.clone();
}

#[derive(Derivative, Clone, Serialize, Deserialize)]
#[derivative(Debug)]
pub struct ResShapeEx {
    #[derivative(Debug = "ignore")]
    #[serde(skip)]
    #[serde(default = "default_shape_handle")]
    pub handle: ShapeHandle<Fx>,
    #[serde(flatten)]
    pub shape: ResShape,
}

impl ResShapeEx {
    pub(crate) fn restore(&mut self, ctx: &mut RestoreContext) -> Result<()> {
        if let Some(handle) = ctx.find_shape(&self.shape) {
            self.handle = handle;
        } else {
            self.handle = match &mut self.shape {
                ResShape::Ball(ball) => ball.load(),
                ResShape::Cuboid(cuboid) => cuboid.load(),
                ResShape::Capsule(capsule) => capsule.load(),
                ResShape::Cone(cone) => cone.load(),
                ResShape::Cylinder(cylinder) => cylinder.load(),
                ResShape::Human(human) => human.load(),
                ResShape::TriMesh(mesh) => mesh.load()?,
            };
            ctx.insert_shape(self.shape.clone(), self.handle.clone());
        }
        return Ok(());
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResShape {
    Ball(ResShapeBall),
    Cuboid(ResShapeCuboid),
    Capsule(ResShapeCapsule),
    Cone(ResShapeCone),
    Cylinder(ResShapeCylinder),
    Human(ResShapeHuman),
    TriMesh(ResShapeTriMesh),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResShapeBall {
    pub radius: Fx,
}

impl ResShapeBall {
    pub(super) fn load(&mut self) -> ShapeHandle<Fx> {
        return ShapeHandle::new(Ball::new(self.radius));
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResShapeCuboid {
    pub x: Fx,
    pub y: Fx,
    pub z: Fx,
}

impl ResShapeCuboid {
    pub(super) fn load(&mut self) -> ShapeHandle<Fx> {
        return ShapeHandle::new(Cuboid::new(Vector3::new(self.x, self.y, self.z)));
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResShapeCapsule {
    pub half_height: Fx,
    pub radius: Fx,
}

impl ResShapeCapsule {
    pub(super) fn load(&mut self) -> ShapeHandle<Fx> {
        return ShapeHandle::new(Capsule::new(self.half_height, self.radius));
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResShapeCone {
    pub half_height: Fx,
    pub radius: Fx,
}

impl ResShapeCone {
    pub(super) fn load(&mut self) -> ShapeHandle<Fx> {
        return ShapeHandle::new(ConeExt::new(self.half_height, self.radius));
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResShapeCylinder {
    pub half_height: Fx,
    pub radius: Fx,
}

impl ResShapeCylinder {
    pub(super) fn load(&mut self) -> ShapeHandle<Fx> {
        return ShapeHandle::new(CylinderExt::new(self.half_height, self.radius));
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResShapeHuman {
    pub capsule_radius: Fx,
    pub capsule_height: Fx,
    pub bottom_radius: Fx,
}

impl ResShapeHuman {
    pub(super) fn load(&mut self) -> ShapeHandle<Fx> {
        return ShapeHandle::new(HumanBounding::new(
            self.capsule_radius,
            self.capsule_height,
            self.bottom_radius,
        ));
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResShapeTriMesh {
    pub filename: String,
}

impl ResShapeTriMesh {
    pub(super) fn load(&mut self) -> Result<ShapeHandle<Fx>> {
        if self.filename.ends_with(".obj") {
            return Self::load_obj(&self.filename);
        } else if self.filename.ends_with(".glb") {
            return Err(anyhow!("Not implement .glb"));
        } else if self.filename.ends_with(".gltf") {
            return Err(anyhow!("Not implement .gltf"));
        }
        return Err(anyhow!("Unknown file format"));
    }

    fn load_obj(filename: &str) -> Result<ShapeHandle<Fx>> {
        let file = File::open(filename)?;
        let buf = BufReader::new(file);
        let model: Obj = load_obj(buf)?;

        let mut vertices = Vec::<Point3<Fx>>::new();
        for vtx in model.vertices {
            vertices.push(Point3::new(
                fx_f32(vtx.position[0]),
                fx_f32(vtx.position[1]),
                fx_f32(vtx.position[2]),
            ));
        }

        let mut indices = Vec::<Point3<usize>>::new();
        for idx in 0..(model.indices.len() / 3) {
            indices.push(Point3::new(
                model.indices[3 * idx] as usize,
                model.indices[3 * idx + 1] as usize,
                model.indices[3 * idx + 2] as usize,
            ));
        }

        let mesh = TriMesh::new(vertices, indices, None);
        let handle = ShapeHandle::new(mesh);
        return Ok(handle);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::relative_eq;
    use m::ff;

    #[test]
    fn test_res_shape_ball() {
        let s1 = ResShapeEx {
            handle: default_shape_handle(),
            shape: ResShape::Ball(ResShapeBall { radius: fi(1) }),
        };
        let json = serde_json::to_string(&s1).unwrap();
        let s2 = serde_json::from_str::<ResShapeEx>(&json).unwrap();
        assert_eq!(s1.shape, s2.shape);
    }

    #[test]
    fn test_res_shape_cuboid() {
        let s1 = ResShapeEx {
            handle: default_shape_handle(),
            shape: ResShape::Cuboid(ResShapeCuboid {
                x: fi(1),
                y: fi(2),
                z: fi(3),
            }),
        };
        let json = serde_json::to_string(&s1).unwrap();
        let s2 = serde_json::from_str::<ResShapeEx>(&json).unwrap();
        assert_eq!(s1.shape, s2.shape);
    }

    #[test]
    fn test_res_shape_capsule() {
        let s1 = ResShapeEx {
            handle: default_shape_handle(),
            shape: ResShape::Capsule(ResShapeCapsule {
                half_height: fi(1),
                radius: ff(0.5),
            }),
        };
        let json = serde_json::to_string(&s1).unwrap();
        let s2 = serde_json::from_str::<ResShapeEx>(&json).unwrap();
        assert_eq!(s1.shape, s2.shape);
    }
}
