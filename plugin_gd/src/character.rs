use crate::core_ex::{GdSyncCore, SyncStateRef};
use crate::util::{NodeExt, ResultExt};
use core::agent::{SyncLogicAgent, SyncStateReg};
use core::id::{ObjID, CLASS_STAGE};
use core::logic::{OpJumpCharacter, OpMoveCharacter, Operation, StateCharacter};
use core::state::StateOwnerX;
use core::util::try_result;
use euclid::{UnknownUnit, Vector2D, Vector3D};
use failure::Error;
use failure::_core::f32::consts::FRAC_PI_2;
use gdnative::prelude::*;
use na::base::{Vector2, Vector3};
use na::geometry::{Rotation2, Rotation3};
use std::f32::consts::PI;

#[derive(StateOwnerX, NativeClass)]
#[inherit(Spatial)]
#[user_data(LocalCellData<GdCharacter>)]
#[class_id(CLASS_STAGE)]
pub struct GdCharacter {
    obj_id: ObjID,
    state: SyncStateRef<StateCharacter>,
    agent: Option<SyncLogicAgent>,

    camera_hori: f32,
    camera_vert: f32,
    camera_distance: f32,
    camera_target: Vector3<f32>,
    camera_speed: f32,
}

#[methods]
impl GdCharacter {
    fn new(_: &Spatial) -> GdCharacter {
        godot_print!("GdCharacter::new()");

        let obj_id = ObjID::from(100001);
        return GdCharacter {
            obj_id,
            state: SyncStateRef::new(obj_id, SyncStateReg::default()),
            agent: None,
            camera_hori: 0.0,
            camera_vert: 0.0,
            camera_distance: 3.0,
            camera_target: Vector3::new(0.0, 0.5, 0.0),
            camera_speed: 0.005,
        };
    }

    #[export]
    fn _ready(&mut self, owner: &Spatial) {
        godot_print!("GdCharacter::_ready()");

        try_result(|| {
            let core =
                unsafe { owner.root_instance_ref::<GdSyncCore, Node, _>("./Root/SyncCore")? };
            let agent = core.map_mut(|core, _| core.get_agent()).cast_err()?;
            self.agent = Some(agent.clone());
            self.state.set_reg(SyncStateReg::new(&agent))?;
            self.state.register()
        })
        .expect("GdCharacter::_ready()");
    }

    #[export]
    fn _exit_tree(&mut self, _owner: &Spatial) {
        let _ = self.state.unregister();
    }

    #[export]
    fn _physics_process(&mut self, owner: &Spatial, _delta: f64) {
        let state = self.state.state().unwrap();

        let model = unsafe { owner.typed_node_tref::<Spatial, _>("./Model").unwrap() };
        let mat = state
            .isometry
            .rotation
            .to_rotation_matrix()
            .matrix()
            .clone();
        // godot_print!("===== {} ===== {} {} {}", state.isometry.rotation, x, y, z);
        model.set_transform(Transform {
            basis: Basis::from_elements([
                Vector3D::new(
                    mat[(0, 0)].to_f32(),
                    mat[(1, 0)].to_f32(),
                    mat[(2, 0)].to_f32(),
                ),
                Vector3D::new(
                    mat[(0, 1)].to_f32(),
                    mat[(1, 1)].to_f32(),
                    mat[(2, 1)].to_f32(),
                ),
                Vector3D::new(
                    mat[(0, 2)].to_f32(),
                    mat[(1, 2)].to_f32(),
                    mat[(2, 2)].to_f32(),
                ),
            ]),
            origin: Vector3D::zero(),
        });

        owner.set_translation(Vector3D::new(
            state.isometry.translation.vector.x.to_f32(),
            state.isometry.translation.vector.y.to_f32(),
            state.isometry.translation.vector.z.to_f32(),
        ));

        self.update_camera(owner).unwrap();
    }

    // #[export]
    // fn _input(&mut self, _owner: &Spatial, event: Ref<InputEvent>) {
    //     let event = unsafe { event.assume_safe() };
    //     if let Some(motion) = event.cast::<InputEventMouseMotion>() {
    //         let speed = motion.speed();

    //         self.camera_hori = self.camera_hori + speed.x * 0.0001;
    //         self.camera_hori = self.camera_hori % (2.0 * PI);

    //         self.camera_vert = self.camera_vert - speed.y * 0.0001;
    //         self.camera_vert = na::clamp(self.camera_vert, -FRAC_PI_2, FRAC_PI_2);
    //     }
    // }
}

impl GdCharacter {
    pub fn move_camera(&mut self, speed: Vector2D<f32, UnknownUnit>) {
        self.camera_hori = self.camera_hori - speed.x * self.camera_speed;
        self.camera_hori = self.camera_hori % (2.0 * PI);

        self.camera_vert = self.camera_vert - speed.y * self.camera_speed;
        self.camera_vert = na::clamp(self.camera_vert, -FRAC_PI_2, FRAC_PI_2);
    }

    fn update_camera(&mut self, owner: &Spatial) -> Result<(), Error> {
        let rot: Rotation3<f32> =
            Rotation3::from_euler_angles(self.camera_vert, self.camera_hori, 0.0);
        let eye: Vector3<f32> = rot * Vector3::new(0.0, 0.0, self.camera_distance);
        let up: Vector3<f32> = rot * Vector3::new(0.0, 1.0, 0.0);

        let look_at: Rotation3<f32> = Rotation3::look_at_rh(&-eye, &up);
        let mat = look_at.matrix();

        let camera = unsafe { owner.typed_node_tref::<Spatial, _>("./Camera")? };
        camera.set_transform(Transform {
            basis: Basis::from_elements([
                Vector3D::new(mat[(0, 0)], mat[(1, 0)], mat[(2, 0)]),
                Vector3D::new(mat[(0, 1)], mat[(1, 1)], mat[(2, 1)]),
                Vector3D::new(mat[(0, 2)], mat[(1, 2)], mat[(2, 2)]),
            ]),
            origin: Vector3D::new(
                self.camera_target.x + eye.x,
                self.camera_target.y + eye.y,
                self.camera_target.z + eye.z,
            ),
        });

        return Ok(());
    }
}

impl GdCharacter {
    pub fn move_character(&mut self, direction: Vector2D<f32, UnknownUnit>) {
        let camera_rot: Rotation2<f32> = Rotation2::new(-self.camera_hori); // negative, right-handed
        let move_dir: Vector2<f32> = camera_rot * Vector2::new(direction.x, direction.y);
        if let Some(agent) = &self.agent {
            agent.operate(Operation::MoveCharacter(OpMoveCharacter {
                direction: move_dir,
                is_moving: true,
            }));
        }
    }

    pub fn jump_character(&mut self) {
        if let Some(agent) = &self.agent {
            agent.operate(Operation::JumpCharacter(OpJumpCharacter {}));
        }
    }
}
