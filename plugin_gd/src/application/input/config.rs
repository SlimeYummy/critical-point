use super::key::{InputDevice, InputKey};
use core::utils::EnumX;
use lazy_static::lazy_static;
use maplit::hashset;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::Duration;

lazy_static! {
    pub static ref UI_KEY_MOUSE_MODE: HashSet<InputKey> =
        hashset![InputKey::from_name(InputDevice::Keyboard, "F1").unwrap()];
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct InputConfig {
    pub key_mouse: KeyMouseConfig,
    pub game_pad: GamePadConfig,
    pub touch_screen: TouchScreenConfig,
}

//
// keyboard & mouse config
//

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyMouseConfig {
    pub key_bindings: Vec<KeyMouseBinding>,
    pub camera_speed: f32,
    pub hold_duration: Duration,
    pub default_dash_forward: bool,
}

impl Default for KeyMouseConfig {
    fn default() -> KeyMouseConfig {
        use InputDevice::*;
        use KeyMouseFunction::*;

        return KeyMouseConfig {
            key_bindings: vec![
                KeyMouseBinding::new(Up, InputKey::from_name(Keyboard, "W").unwrap()),
                KeyMouseBinding::new(Down, InputKey::from_name(Keyboard, "S").unwrap()),
                KeyMouseBinding::new(Left, InputKey::from_name(Keyboard, "A").unwrap()),
                KeyMouseBinding::new(Right, InputKey::from_name(Keyboard, "D").unwrap()),
                KeyMouseBinding::new(Dash, InputKey::from_name(Keyboard, "Space").unwrap()),
                KeyMouseBinding::new(Attack1, InputKey::from_name(Mouse, "Left").unwrap()),
                KeyMouseBinding::new(Attack2, InputKey::from_name(Mouse, "Middle").unwrap()),
                KeyMouseBinding::new(Defend, InputKey::from_name(Mouse, "Right").unwrap()),
                KeyMouseBinding::new(Skill1, InputKey::from_name(Keyboard, "Q").unwrap()),
                KeyMouseBinding::new(Skill2, InputKey::from_name(Keyboard, "E").unwrap()),
                KeyMouseBinding::new(SkillEx, InputKey::from_name(Keyboard, "F").unwrap()),
                KeyMouseBinding::new(Item1, InputKey::from_name(Keyboard, "1").unwrap()),
                KeyMouseBinding::new(Item2, InputKey::from_name(Keyboard, "2").unwrap()),
                KeyMouseBinding::new(Item3, InputKey::from_name(Keyboard, "3").unwrap()),
                KeyMouseBinding::new(Interact, InputKey::from_name(Keyboard, "Shift").unwrap()),
            ],
            camera_speed: 1.0,
            hold_duration: Duration::from_millis(400),
            default_dash_forward: true,
        };
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyMouseFunctionType {
    Direction,
    EnableHold,
    DisableHold,
}

#[repr(u8)]
#[derive(EnumX, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr_type(u8)]
#[prop_type(KeyMouseFunctionType)]
pub enum KeyMouseFunction {
    #[prop(KeyMouseFunctionType::Direction)]
    Up,
    #[prop(KeyMouseFunctionType::Direction)]
    Down,
    #[prop(KeyMouseFunctionType::Direction)]
    Left,
    #[prop(KeyMouseFunctionType::Direction)]
    Right,
    #[prop(KeyMouseFunctionType::DisableHold)]
    Dash,
    #[prop(KeyMouseFunctionType::EnableHold)]
    Attack1,
    #[prop(KeyMouseFunctionType::EnableHold)]
    Attack2,
    #[prop(KeyMouseFunctionType::EnableHold)]
    Defend,
    #[prop(KeyMouseFunctionType::EnableHold)]
    Skill1,
    #[prop(KeyMouseFunctionType::EnableHold)]
    Skill2,
    #[prop(KeyMouseFunctionType::EnableHold)]
    SkillEx,
    #[prop(KeyMouseFunctionType::DisableHold)]
    Item1,
    #[prop(KeyMouseFunctionType::DisableHold)]
    Item2,
    #[prop(KeyMouseFunctionType::DisableHold)]
    Item3,
    #[prop(KeyMouseFunctionType::DisableHold)]
    Interact,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct KeyMouseBinding {
    pub function: KeyMouseFunction,
    #[serde(flatten)]
    pub key: InputKey,
}

impl KeyMouseBinding {
    pub fn new(function: KeyMouseFunction, key: InputKey) -> KeyMouseBinding {
        return KeyMouseBinding { function, key };
    }
}

//
// game pad config
//

#[derive(Debug, Serialize, Deserialize)]
pub struct GamePadConfig {}

impl Default for GamePadConfig {
    fn default() -> GamePadConfig {
        return GamePadConfig {};
    }
}

//
// touch screen config
//

#[derive(Debug, Serialize, Deserialize)]
pub struct TouchScreenConfig {}

impl Default for TouchScreenConfig {
    fn default() -> TouchScreenConfig {
        return TouchScreenConfig {};
    }
}
