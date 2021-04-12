mod config;
mod core_op;
mod key;

use super::operation::{op_core, OpChangeMouseMode, OpMoveCamera, Operation};
use anyhow::{anyhow, Result};
use config::UI_KEY_MOUSE_MODE;
use config::{InputConfig, KeyMouseFunction, KeyMouseFunctionType};
use core::engine::{OpAction, OpCommand};
use core::utils::deserialize;
use gdnative::api::{InputEvent, InputEventKey, InputEventMouseButton, InputEventMouseMotion};
use gdnative::prelude::*;
use key::{InputDevice, InputKey};
use m::{fi, fx_f32, Fx};
use na::{Rotation2, Rotation3, Unit, Vector2};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Debug, Clone)]
struct HoldTimer {
    started_time: Instant,
    triggered: bool,
}

#[derive(Debug)]
pub struct AppInputKeyMouse {
    initialized: bool,
    user_path: PathBuf,
    config: InputConfig,
    keys: HashMap<InputKey, KeyMouseFunction>,

    camera_rotation2: Rotation2<f32>,
    camera_rotation3: Rotation3<f32>,

    move_direction: Vector2<i32>,
    direction_set: HashSet<KeyMouseFunction>,
    holding_map: HashMap<KeyMouseFunction, HoldTimer>,
    ops_buffer: Vec<Operation>,
}

impl AppInputKeyMouse {
    pub fn new<P: AsRef<Path>>(user_path: P) -> AppInputKeyMouse {
        return AppInputKeyMouse {
            initialized: false,
            user_path: PathBuf::from(user_path.as_ref()),
            config: InputConfig::default(),
            keys: HashMap::with_capacity(32),

            camera_rotation2: Rotation2::identity(),
            camera_rotation3: Rotation3::identity(),

            move_direction: na::zero(),
            direction_set: HashSet::with_capacity(4),
            holding_map: HashMap::with_capacity(16),
            ops_buffer: Vec::with_capacity(32),
        };
    }

    pub fn init(&mut self) -> Result<()> {
        if self.initialized {
            return Err(anyhow!("Initialized"));
        }
        self.initialized = true;

        match deserialize(&self.user_path) {
            Ok(config) => self.config = config,
            Err(err) => godot_warn!("{:?}", err),
        }

        for binding in self.config.key_mouse.key_bindings.iter() {
            self.keys.insert(binding.key, binding.function);
        }

        return Ok(());
    }

    pub fn set_camera_rotation(&mut self, rotation2: Rotation2<f32>, rotation3: Rotation3<f32>) {
        self.camera_rotation2 = rotation2;
        self.camera_rotation3 = rotation3;
    }

    pub fn handle_events(&mut self, event: Ref<InputEvent>) -> Option<Operation> {
        let event = unsafe { event.assume_safe() };

        if let Some(event) = event.cast::<InputEventMouseMotion>() {
            return self.handle_camera(event);
        }

        let key: InputKey;
        if let Some(event) = event.cast::<InputEventMouseButton>() {
            key = InputKey::from_code(InputDevice::Mouse, event.button_index()).ok()?;
        } else if let Some(event) = event.cast::<InputEventKey>() {
            key = InputKey::from_code(InputDevice::Keyboard, event.scancode()).ok()?;
        } else {
            return None;
        }
        let pressed = event.is_pressed();

        match self.keys.get(&key).map(|v| *v) {
            None => return self.handle_ui(key, pressed),
            Some(function) => {
                return self
                    .handle_direction(pressed, function)
                    .or_else(|| self.handle_disable_hold(pressed, function))
                    .or_else(|| self.handle_enable_hold(pressed, function));
            }
        }
    }

    fn handle_camera(&self, event: TRef<InputEventMouseMotion>) -> Option<Operation> {
        let relative = event.relative();
        let speed = Vector2::new(relative.x, relative.y);
        let speed = speed.scale(self.config.key_mouse.camera_speed);
        return Some(Operation::MoveCamera(OpMoveCamera { speed }));
    }

    fn handle_ui(&self, key: InputKey, pressed: bool) -> Option<Operation> {
        if !pressed {
            if UI_KEY_MOUSE_MODE.contains(&key) {
                return Some(Operation::ChangeMouseMode(OpChangeMouseMode {}));
            }
        }
        return None;
    }

    fn handle_direction(&mut self, pressed: bool, function: KeyMouseFunction) -> Option<Operation> {
        if function.prop() != KeyMouseFunctionType::Direction {
            return None;
        }

        use KeyMouseFunction::*;
        if pressed {
            if !self.direction_set.contains(&function) {
                self.direction_set.insert(function);
                match function {
                    Up => self.move_direction.y += -1,
                    Down => self.move_direction.y += 1,
                    Left => self.move_direction.x += -1,
                    Right => self.move_direction.x += 1,
                    _ => assert!(function.prop() != KeyMouseFunctionType::Direction),
                };
            }
        } else {
            self.direction_set.remove(&function);
            match function {
                Up => self.move_direction.y -= -1,
                Down => self.move_direction.y -= 1,
                Left => self.move_direction.x -= -1,
                Right => self.move_direction.x -= 1,
                _ => assert!(function.prop() != KeyMouseFunctionType::Direction),
            };
        }

        self.move_direction.x = self.move_direction.x.clamp(-1, 1);
        self.move_direction.y = self.move_direction.y.clamp(-1, 1);
        return None;
    }

    fn handle_disable_hold(&self, pressed: bool, function: KeyMouseFunction) -> Option<Operation> {
        if function.prop() != KeyMouseFunctionType::DisableHold {
            return None;
        }

        if pressed {
            use KeyMouseFunction::*;
            return match function {
                Dash => {
                    let def_dir = match self.config.key_mouse.default_dash_forward {
                        true => Vector2::new(fi(0), fi(-1)),
                        false => Vector2::new(fi(0), fi(1)),
                    };
                    op_core(OpCommand::Dash(self.move_direction2(def_dir)))
                }
                Item1 => op_core(OpCommand::Item1),
                Item2 => op_core(OpCommand::Item2),
                Item3 => op_core(OpCommand::Item3),
                _ => {
                    assert!(function.prop() != KeyMouseFunctionType::Direction);
                    None
                }
            };
        }

        return None;
    }

    fn handle_enable_hold(
        &mut self,
        pressed: bool,
        function: KeyMouseFunction,
    ) -> Option<Operation> {
        if function.prop() != KeyMouseFunctionType::EnableHold {
            return None;
        }

        if pressed {
            let now = Instant::now();
            if !self.holding_map.contains_key(&function) {
                let timer = HoldTimer {
                    started_time: now,
                    triggered: false,
                };
                self.holding_map.insert(function, timer);
            }
            return None;
        }
        
        let action: OpAction;
        if let Some(timer) = self.holding_map.remove(&function) {
            if timer.triggered {
                action = OpAction::HoldEnd;
            } else {
                action = OpAction::Press;
            }
        } else {
            action = OpAction::Press;
        };

        use KeyMouseFunction::*;
        return match function {
            Attack1 => op_core(OpCommand::Attack1(action, self.move_direction2(na::zero()))),
            Attack2 => op_core(OpCommand::Attack2(action, self.move_direction2(na::zero()))),
            Defend => op_core(OpCommand::Defend(action, self.move_direction2(na::zero()))),
            Skill1 => op_core(OpCommand::Skill1(action)),
            Skill2 => op_core(OpCommand::Skill2(action)),
            SkillEx => op_core(OpCommand::SkillEx(action)),
            Interact => op_core(OpCommand::Interact),
            _ => {
                assert!(function.prop() != KeyMouseFunctionType::Direction);
                None
            }
        };
    }

    pub fn tick(&mut self) -> Vec<Operation> {
        let mut ops = Vec::new();

        let move_dir = self.move_direction2(na::zero());
        if self.move_direction.x != 0 || self.move_direction.y != 0 {
            ops.push(Operation::Core(OpCommand::Move(move_dir)));
        }

        let now = Instant::now();
        let hold_duration = self.config.key_mouse.hold_duration;

        for (function, timer) in self.holding_map.iter_mut() {
            let duration = now.duration_since(timer.started_time);
            if !timer.triggered && duration >= hold_duration {
                timer.triggered = true;

                use KeyMouseFunction::*;
                let cmd = match function {
                    Attack1 => OpCommand::Attack1(OpAction::HoldBegin, move_dir),
                    Attack2 => OpCommand::Attack2(OpAction::HoldBegin, move_dir),
                    Defend => OpCommand::Defend(OpAction::HoldBegin, move_dir),
                    Skill1 => OpCommand::Skill1(OpAction::HoldBegin),
                    Skill2 => OpCommand::Skill2(OpAction::HoldBegin),
                    SkillEx => OpCommand::SkillEx(OpAction::HoldBegin),
                    Interact => OpCommand::Interact,
                    _ => {
                        assert!(function.prop() != KeyMouseFunctionType::Direction);
                        continue;
                    }
                };
                ops.push(Operation::Core(cmd));
            }
        }

        return ops;
    }

    fn move_direction2(&self, def_dir: Vector2<Fx>) -> Vector2<Fx> {
        let move_dir = self.move_direction;
        if move_dir.x == 0 && move_dir.y == 0 {
            return def_dir;
        }
        let local_dir = Vector2::new(move_dir.x as f32, move_dir.y as f32);
        let camera_dir = self.camera_rotation2.transform_vector(&local_dir);
        let norm_dir = Unit::new_normalize(camera_dir);
        return Vector2::new(fx_f32(norm_dir.x), fx_f32(norm_dir.y));
    }
}
