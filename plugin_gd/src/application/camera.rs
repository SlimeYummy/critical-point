use super::operation::OpMoveCamera;
use crate::utils::NodeExt;
use anyhow::{anyhow, Result};
use euclid::default::Vector3D;
use gdnative::api::Camera;
use gdnative::prelude::*;
use na::{Matrix3, Rotation2, Rotation3, Vector3};
use std::f32::consts::{FRAC_PI_2, PI};

pub struct AppCamera {
    path: String,
    camera: Option<Ref<Camera, Shared>>,
    horizontal: f32,
    vertical: f32,
    distance: f32,
    target: Vector3<f32>,
    speed: f32,
}

impl AppCamera {
    pub fn new(path: &str) -> AppCamera {
        return AppCamera {
            path: path.to_string(),
            camera: None,
            horizontal: 0.0,
            vertical: 0.0,
            distance: 2.0,
            target: Vector3::new(0.0, 1.0, 0.0),
            speed: 0.005,
        };
    }

    pub fn init(&mut self, owner: TRef<Node>) -> Result<()> {
        if self.camera.is_some() {
            return Err(anyhow!("Initialized"));
        }
        let camera = unsafe { owner.typed_node(&self.path) }?;
        self.camera = Some(camera);
        return Ok(());
    }

    fn camera(&self) -> Result<TRef<Camera>> {
        return self
            .camera
            .map(|camera| unsafe { camera.assume_safe() })
            .ok_or(anyhow!("Uninitialized"));
    }

    pub fn rotate_camera(&mut self, op: OpMoveCamera) -> Result<()> {
        self.horizontal = self.horizontal - op.speed.x * self.speed;
        self.horizontal = self.horizontal % (2.0 * PI);
        self.vertical = self.vertical - op.speed.y * self.speed;
        self.vertical = na::clamp(self.vertical, -FRAC_PI_2 * 0.9, FRAC_PI_2 * 0.6);

        let rot: Rotation3<f32> = Rotation3::from_euler_angles(self.vertical, self.horizontal, 0.0);
        let eye: Vector3<f32> = rot * Vector3::new(0.0, 0.0, self.distance);
        let up: Vector3<f32> = rot * Vector3::new(0.0, 1.0, 0.0);

        let look_at: Rotation3<f32> = Rotation3::look_at_rh(&-eye, &up);
        let mat = look_at.matrix();

        let camera: TRef<Camera> = self.camera()?;
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
    pub fn rotation3(&self) -> Result<Rotation3<f32>> {
        let camera: TRef<Camera> = self.camera()?;
        let elements = camera.transform().basis.elements;
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

    pub fn rotation2(&self) -> Rotation2<f32> {
        return Rotation2::new(self.horizontal);
    }
}
