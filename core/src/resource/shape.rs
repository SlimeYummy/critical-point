use super::cache::RestoreContext;
use anyhow::{anyhow, Result};
use derivative::Derivative;
use lazy_static::lazy_static;
use m::{fi, fx_f64, ConeExt, CylinderExt, Fx, HumanBounding};
use na::{Point3, Vector3};
use ncollide3d::shape::{Ball, Capsule, Cuboid, ShapeHandle, TriMesh};
use wavefront_obj::obj::{self, Primitive};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::fs;
use std::io::BufReader;

pub(crate) type ShapeCacheKey = ResShapeAny;
pub(crate) type ShapeCacheValue = ShapeHandle<Fx>;

lazy_static! {
    pub static ref DEFAULT_SHAPE_HANDLE: ShapeHandle<Fx> = ShapeHandle::new(Ball::new(fi(1)));
}

pub fn default_shape_handle() -> ShapeHandle<Fx> {
    return DEFAULT_SHAPE_HANDLE.clone();
}

#[derive(Derivative, Clone, Serialize, Deserialize)]
#[derivative(Debug)]
pub struct ResShape {
    #[derivative(Debug = "ignore")]
    #[serde(skip)]
    #[serde(default = "default_shape_handle")]
    pub handle: ShapeHandle<Fx>,
    #[serde(flatten)]
    pub shape: ResShapeAny,
}

impl ResShape {
    pub(crate) fn restore(&mut self, ctx: &mut RestoreContext) -> Result<()> {
        if let Some(handle) = ctx.find_shape(&self.shape) {
            self.handle = handle;
        } else {
            self.handle = match &mut self.shape {
                ResShapeAny::Ball(ball) => ball.load(),
                ResShapeAny::Cuboid(cuboid) => cuboid.load(),
                ResShapeAny::Capsule(capsule) => capsule.load(),
                ResShapeAny::Cone(cone) => cone.load(),
                ResShapeAny::Cylinder(cylinder) => cylinder.load(),
                ResShapeAny::Human(human) => human.load(),
                ResShapeAny::TriMesh(mesh) => mesh.load()?,
            };
            ctx.insert_shape(self.shape.clone(), self.handle.clone());
        }
        return Ok(());
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResShapeAny {
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
    pub(crate) fn load(&mut self) -> ShapeHandle<Fx> {
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
    pub(crate) fn load(&mut self) -> ShapeHandle<Fx> {
        return ShapeHandle::new(Cuboid::new(Vector3::new(self.x, self.y, self.z)));
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResShapeCapsule {
    pub half_height: Fx,
    pub radius: Fx,
}

impl ResShapeCapsule {
    pub(crate) fn load(&mut self) -> ShapeHandle<Fx> {
        return ShapeHandle::new(Capsule::new(self.half_height, self.radius));
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResShapeCone {
    pub half_height: Fx,
    pub radius: Fx,
}

impl ResShapeCone {
    pub(crate) fn load(&mut self) -> ShapeHandle<Fx> {
        return ShapeHandle::new(ConeExt::new(self.half_height, self.radius));
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResShapeCylinder {
    pub half_height: Fx,
    pub radius: Fx,
}

impl ResShapeCylinder {
    pub(crate) fn load(&mut self) -> ShapeHandle<Fx> {
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
    pub(crate) fn load(&mut self) -> ShapeHandle<Fx> {
        return ShapeHandle::new(HumanBounding::new(
            self.capsule_radius,
            self.capsule_height,
            self.bottom_radius,
        ));
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResShapeTriMesh {
    pub file: String,
}

impl ResShapeTriMesh {
    pub(crate) fn load(&mut self) -> Result<ShapeHandle<Fx>> {
        if self.file.ends_with(".obj") {
            return Self::load_obj(&self.file);
        } else if self.file.ends_with(".glb") {
            return Err(anyhow!("Not implement .glb"));
        } else if self.file.ends_with(".gltf") {
            return Err(anyhow!("Not implement .gltf"));
        }
        return Err(anyhow!("Unknown file format"));
    }

    fn load_obj(file: &str) -> Result<ShapeHandle<Fx>> {
        let buf = fs::read_to_string(file)?;
        let model = wavefront_obj::obj::parse(buf)?;
        
        let mut vertices = Vec::<Point3<Fx>>::new();
        let mut indices = Vec::<Point3<usize>>::new();

        for obj in model.objects {
            for vtx in obj.vertices {
                vertices.push(Point3::new(fx_f64(vtx.x), fx_f64(vtx.y), fx_f64(vtx.z)));
            }
    
            for geo in obj.geometry {
                for shape in geo.shapes {
                    match shape.primitive {
                        Primitive::Triangle(x, y, z) => indices.push(Point3::new(x.0, y.0, z.0)),
                        _ => return Err(anyhow!("Not a trimesh model")),
                    }
                }
            }
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
        let s1 = ResShape {
            handle: default_shape_handle(),
            shape: ResShapeAny::Ball(ResShapeBall { radius: fi(1) }),
        };
        let json = serde_json::to_string(&s1).unwrap();
        let s2 = serde_json::from_str::<ResShape>(&json).unwrap();
        assert_eq!(s1.shape, s2.shape);
    }

    #[test]
    fn test_res_shape_cuboid() {
        let s1 = ResShape {
            handle: default_shape_handle(),
            shape: ResShapeAny::Cuboid(ResShapeCuboid {
                x: fi(1),
                y: fi(2),
                z: fi(3),
            }),
        };
        let json = serde_json::to_string(&s1).unwrap();
        let s2 = serde_json::from_str::<ResShape>(&json).unwrap();
        assert_eq!(s1.shape, s2.shape);
    }

    #[test]
    fn test_res_shape_capsule() {
        let s1 = ResShape {
            handle: default_shape_handle(),
            shape: ResShapeAny::Capsule(ResShapeCapsule {
                half_height: fi(1),
                radius: ff(0.5),
            }),
        };
        let json = serde_json::to_string(&s1).unwrap();
        let s2 = serde_json::from_str::<ResShape>(&json).unwrap();
        assert_eq!(s1.shape, s2.shape);
    }

    #[test]
    fn test_res_shape_trimesh() {
        let s1 = ResShape {
            handle: default_shape_handle(),
            shape: ResShapeAny::TriMesh(ResShapeTriMesh {
                file: "body.obj".to_string(),
            }),
        };
        let json = serde_json::to_string(&s1).unwrap();
        let s2 = serde_json::from_str::<ResShape>(&json).unwrap();
        assert_eq!(s1.shape, s2.shape);

        let mut trimesh = ResShapeTriMesh {
            file: "../test_files/stage-simple.obj".to_string(),
        };
        trimesh.load().unwrap();
    }
}
