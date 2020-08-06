use crate::character::GdCharacter;
use crate::euclid::approxeq::ApproxEq;
use crate::util::{NodeExt, ResultExt};
use euclid::{UnknownUnit, Vector2D};
use failure::Error;
use gdnative::api::{
    GlobalConstants, Input, InputEvent, InputEventKey, InputEventMouseButton,
    InputEventMouseMotion, InputMap,
};
use gdnative::prelude::*;

pub struct AppInput {
    move_direction: Vector2D<f32, UnknownUnit>,
}

impl AppInput {
    pub fn new() -> AppInput {
        return AppInput {
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

    pub fn handle_events(&mut self, owner: &Node, event: Ref<InputEvent>) -> Result<(), Error> {
        let event = unsafe { event.assume_safe() };
        let _ = self.move_camera(owner, event.clone())?
            || self.move_character(owner, event.clone())?
            || self.jump_character(owner, event.clone())?
            || self.attack(owner, event.clone())?
            || self.switch_mouse_mode(event.clone())?;
        unsafe { owner.scene_tree()? }.set_input_as_handled();
        return Ok(());
    }

    fn switch_mouse_mode(&mut self, event: TRef<'_, InputEvent>) -> Result<bool, Error> {
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

    fn move_camera(&mut self, owner: &Node, event: TRef<'_, InputEvent>) -> Result<bool, Error> {
        if let Some(ev) = event.cast::<InputEventMouseMotion>() {
            let chara =
                unsafe { owner.root_instance_ref::<GdCharacter, Spatial, _>("Root/Character") }?;
            chara
                .map_mut(|chara, _| chara.move_camera(ev.relative()))
                .cast_err()?;
            return Ok(true);
        }
        return Ok(false);
    }

    fn move_character(&mut self, owner: &Node, event: TRef<'_, InputEvent>) -> Result<bool, Error> {
        fn event_to_num(event: TRef<'_, InputEvent>, action: &str, num: f32) -> f32 {
            if event.is_action_pressed(action, false) {
                return num;
            }
            if event.is_action_released(action) {
                return -num;
            }
            return 0.0;
        }

        if event.is_action("move_up") {
            self.move_direction.y += event_to_num(event, "move_up", -1.0);
        } else if event.is_action("move_down") {
            self.move_direction.y += event_to_num(event, "move_down", 1.0);
        } else if event.is_action("move_left") {
            self.move_direction.x += event_to_num(event, "move_left", -1.0);
        } else if event.is_action("move_right") {
            self.move_direction.x += event_to_num(event, "move_right", 1.0);
        } else {
            return Ok(false);
        }

        self.move_direction.x = self.move_direction.x.clamp(-1.0, 1.0);
        self.move_direction.y = self.move_direction.y.clamp(-1.0, 1.0);
        let move_direction = if self.move_direction.approx_eq(&Vector2D::zero()) {
            Vector2D::zero()
        } else {
            self.move_direction.normalize()
        };
        godot_print!("{}", self.move_direction);

        let chara =
            unsafe { owner.root_instance_ref::<GdCharacter, Spatial, _>("Root/Character") }?;
        chara
            .map_mut(|chara, _| chara.move_character(move_direction))
            .cast_err()?;
        return Ok(true);
    }

    fn jump_character(&mut self, owner: &Node, event: TRef<'_, InputEvent>) -> Result<bool, Error> {
        if event.is_action_pressed("jump", false) {
            let chara =
                unsafe { owner.root_instance_ref::<GdCharacter, Spatial, _>("Root/Character") }?;
            chara
                .map_mut(|chara, _| chara.jump_character())
                .cast_err()?;
            return Ok(true);
        }
        return Ok(false);
    }

    fn attack(&mut self, _owner: &Node, event: TRef<'_, InputEvent>) -> Result<bool, Error> {
        if let Some(_) = event.cast::<InputEventMouseButton>() {
            return Ok(true);
        }
        return Ok(false);
    }
}
