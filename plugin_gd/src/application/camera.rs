use crate::utils::NodeExt;
use anyhow::{anyhow, Result};
use euclid::default::Vector3D;
use gdnative::api::{Camera, InputEvent, InputEventMouseMotion};
use gdnative::prelude::*;
use na::{Matrix3, Rotation3, UnitQuaternion, Vector2, Vector3, Unit};
use std::f32::consts::{FRAC_PI_2, PI};

pub struct AppCamera {
    path: String,
    camera: Option<Ref<Camera, Shared>>,
    hori: f32,
    vert: f32,
    distance: f32,
    target: Vector3<f32>,
    speed: f32,
}

impl AppCamera {
    pub fn new(path: &str) -> AppCamera {
        return AppCamera {
            path: path.to_string(),
            camera: None,
            hori: 0.0,
            vert: 0.0,
            distance: 2.0,
            target: Vector3::new(0.0, 0.5, 0.0),
            speed: 0.005,
        };
    }

    pub fn init(&mut self, owner: TRef<Node, Shared>) -> Result<()> {
        let camera = unsafe { owner.root_typed_node(&self.path) }?;
        self.camera = Some(camera);
        return Ok(());
    }

    fn camera(&self) -> Result<TRef<Camera, Shared>> {
        return self
            .camera
            .map(|camera| unsafe { camera.assume_safe() })
            .ok_or(anyhow!("uninited"));
    }

    pub fn rotate_camera(
        &mut self,
        owner: TRef<Node, Shared>,
        distance: Vector2<f32>,
    ) -> Result<()> {
        self.hori = self.hori - distance.x * self.speed;
        self.hori = self.hori % (2.0 * PI);
        self.vert = self.vert - distance.y * self.speed;
        self.vert = na::clamp(self.vert, -FRAC_PI_2, FRAC_PI_2);

        let rot: Rotation3<f32> = Rotation3::from_euler_angles(self.vert, self.hori, 0.0);
        let eye: Vector3<f32> = rot * Vector3::new(0.0, 1.5, self.distance);
        let up: Vector3<f32> = rot * Vector3::new(0.0, 1.0, 0.0);

        let look_at: Rotation3<f32> = Rotation3::look_at_rh(&-eye, &up);
        let mat = look_at.matrix();

        let camera: TRef<Camera, Shared> = self.camera()?;
        camera.set_transform(Transform {
            basis: Basis::from_elements([
                Vector3D::new(mat.m11, mat.m21, mat.m31),
                Vector3D::new(mat.m12, mat.m22, mat.m32),
                Vector3D::new(mat.m13, mat.m23, mat.m33),
            ]),
            origin: Vector3D::new(
                self.target.x + eye.x,
                self.target.y + eye.y,
                self.target.z + eye.z,
            ),
        });

        return Ok(());
    }

    #[allow(dead_code)]
    pub fn global_rotation(&self) -> Result<Rotation3<f32>> {
        let camera: TRef<Camera, Shared> = self.camera()?;
        let transform = camera.global_transform();
        let elements = transform.basis.elements;
        let rotation = Rotation3::from_matrix(&Matrix3::from_vec(vec![
            elements[0].x,
            elements[1].x,
            elements[2].x,
            elements[0].y,
            elements[1].y,
            elements[2].y,
            elements[0].z,
            elements[1].z,
            elements[2].z,
        ]));
        return Ok(rotation);
    }

    #[allow(dead_code)]
    pub fn global_unit_quaternion(&self) -> Result<UnitQuaternion<f32>> {
        let rotation = self.global_rotation()?;
        let quaternion = UnitQuaternion::from_rotation_matrix(&rotation);
        return Ok(quaternion);
    }

    #[allow(dead_code)]
    pub fn global_direction(&self) -> Result<Unit<Vector3<f32>>> {
        let rotation = self.global_rotation()?;
        let direction = rotation.transform_vector(&-Vector3::z_axis());
        return Ok(Unit::new_normalize(direction));
    }
}
