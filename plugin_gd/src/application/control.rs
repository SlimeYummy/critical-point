use crate::utils::NodeExt;
use anyhow::Result;
use euclid::{UnknownUnit, Vector2D, Vector3D};
use gdnative::api::{
    Camera, GlobalConstants, Input, InputEvent, InputEventKey, InputEventMouseButton,
    InputEventMouseMotion, InputMap,
};
use gdnative::prelude::*;
use na::{Matrix3, Rotation3, Vector3};
use std::f32::consts::{FRAC_PI_2, PI};

pub struct AppControl {
    camera: AppCamera,
    move_direction: Vector2D<f32, UnknownUnit>,
}

impl AppControl {
    pub fn new() -> AppControl {
        return AppControl {
            camera: AppCamera::new(),
            move_direction: Vector2D::zero(),
        };
    }

    pub fn register_events(&mut self) {
        let im = InputMap::godot_singleton();

        let ev = InputEventKey::new();
        ev.set_scancode(GlobalConstants::KEY_F1);
        im.add_action("mouse_mode", 0.5);
        im.action_add_event("mouse_mode", ev);

        let ev = InputEventKey::new();
        ev.set_scancode(GlobalConstants::KEY_W);
        im.add_action("move_up", 0.5);
        im.action_add_event("move_up", ev);

        let ev = InputEventKey::new();
        ev.set_scancode(GlobalConstants::KEY_S);
        im.add_action("move_down", 0.5);
        im.action_add_event("move_down", ev);

        let ev = InputEventKey::new();
        ev.set_scancode(GlobalConstants::KEY_A);
        im.add_action("move_left", 0.5);
        im.action_add_event("move_left", ev);

        let ev = InputEventKey::new();
        ev.set_scancode(GlobalConstants::KEY_D);
        im.add_action("move_right", 0.5);
        im.action_add_event("move_right", ev);

        let ev = InputEventKey::new();
        ev.set_scancode(GlobalConstants::KEY_SPACE);
        im.add_action("jump", 0.5);
        im.action_add_event("jump", ev);
    }

    pub fn handle_events(&mut self, owner: &Node, event: Ref<InputEvent>) -> Result<()> {
        let event = unsafe { event.assume_safe() };
        let _ = self.camera.move_camera(owner, event.clone())?
            || self.attack(owner, event.clone())?
            || self.switch_mouse_mode(event.clone())?;
        unsafe { owner.scene_tree()? }.set_input_as_handled();

        self.camera.global_transform(owner)?;
        return Ok(());
    }

    fn switch_mouse_mode(&mut self, event: TRef<'_, InputEvent>) -> Result<bool> {
        if event.is_action_pressed("mouse_mode", false) {
            let input = Input::godot_singleton();
            if input.get_mouse_mode().0 != Input::MOUSE_MODE_CAPTURED {
                input.set_mouse_mode(Input::MOUSE_MODE_CAPTURED);
            } else {
                input.set_mouse_mode(Input::MOUSE_MODE_VISIBLE);
            }
            return Ok(true);
        }
        return Ok(false);
    }

    // fn move_character(&mut self, owner: &Node, event: TRef<'_, InputEvent>) -> Result<bool> {
    //     fn event_to_num(event: TRef<'_, InputEvent>, action: &str, num: f32) -> f32 {
    //         if event.is_action_pressed(action, false) {
    //             return num;
    //         }
    //         if event.is_action_released(action) {
    //             return -num;
    //         }
    //         return 0.0;
    //     }

    //     if event.is_action("move_up") {
    //         self.move_direction.y += event_to_num(event, "move_up", -1.0);
    //     } else if event.is_action("move_down") {
    //         self.move_direction.y += event_to_num(event, "move_down", 1.0);
    //     } else if event.is_action("move_left") {
    //         self.move_direction.x += event_to_num(event, "move_left", -1.0);
    //     } else if event.is_action("move_right") {
    //         self.move_direction.x += event_to_num(event, "move_right", 1.0);
    //     } else {
    //         return Ok(false);
    //     }

    //     self.move_direction.x = self.move_direction.x.clamp(-1.0, 1.0);
    //     self.move_direction.y = self.move_direction.y.clamp(-1.0, 1.0);
    //     let move_direction = if self.move_direction.approx_eq(&Vector2D::zero()) {
    //         Vector2D::zero()
    //     } else {
    //         self.move_direction.normalize()
    //     };
    //     godot_print!("{:?}", self.move_direction);

    //     let chara =
    //         unsafe { owner.root_instance_ref::<CharaGeneral, Spatial, _>("Root/Character") }?;
    //     chara
    //         .map_mut(|chara, _| chara.move_character(move_direction))
    //         .cast_err()?;
    //     return Ok(true);
    // }

    // fn jump_character(&mut self, owner: &Node, event: TRef<'_, InputEvent>) -> Result<bool> {
    //     if event.is_action_pressed("jump", false) {
    //         let chara =
    //             unsafe { owner.root_instance_ref::<CharaGeneral, Spatial, _>("Root/Character") }?;
    //         chara
    //             .map_mut(|chara, _| chara.jump_character())
    //             .cast_err()?;
    //         return Ok(true);
    //     }
    //     return Ok(false);
    // }

    fn attack(&mut self, _owner: &Node, event: TRef<'_, InputEvent>) -> Result<bool> {
        if let Some(_) = event.cast::<InputEventMouseButton>() {
            return Ok(true);
        }
        return Ok(false);
    }
}

struct AppCamera {
    hori: f32,
    vert: f32,
    distance: f32,
    target: Vector3<f32>,
    speed: f32,
}

impl AppCamera {
    fn new() -> AppCamera {
        return AppCamera {
            hori: 0.0,
            vert: 0.0,
            distance: 2.0,
            target: Vector3::new(0.0, 0.5, 0.0),
            speed: 0.005,
        };
    }

    fn global_transform(&mut self, owner: &Node) -> Result<()> {
        let camera = unsafe { owner.typed_node_tref::<Camera, _>("Character/Camera") }?;
        let transform = camera.global_transform();

        let vec: Vector3<f32> = Vector3::new(0.0, 0.0, -1.0);
        let elements = transform.basis.elements;
        let rot: Rotation3<f32> = Rotation3::from_matrix(&Matrix3::from_vec(vec![
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
        godot_print!("{:?}", rot * vec);
        return Ok(());
    }

    fn move_camera(&mut self, owner: &Node, event: TRef<'_, InputEvent>) -> Result<bool> {
        let ev = match event.cast::<InputEventMouseMotion>() {
            Some(ev) => ev,
            None => return Ok(false),
        };

        let speed = ev.relative();
        self.hori = self.hori - speed.x * self.speed;
        self.hori = self.hori % (2.0 * PI);
        self.vert = self.vert - speed.y * self.speed;
        self.vert = na::clamp(self.vert, -FRAC_PI_2, FRAC_PI_2);

        let rot: Rotation3<f32> = Rotation3::from_euler_angles(self.vert, self.hori, 0.0);
        let eye: Vector3<f32> = rot * Vector3::new(0.0, 1.5, self.distance);
        let up: Vector3<f32> = rot * Vector3::new(0.0, 1.0, 0.0);

        let look_at: Rotation3<f32> = Rotation3::look_at_rh(&-eye, &up);
        let mat = look_at.matrix();

        let camera = unsafe { owner.typed_node_tref::<Camera, _>("Character/Camera") }?;
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

        return Ok(false);
    }
}
