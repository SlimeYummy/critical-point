use anyhow::{anyhow, Result};
use derivative::Derivative;
use lazy_static::lazy_static;
use serde::de::{Deserializer, Error, MapAccess, Visitor};
use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::result;

lazy_static! {
    pub static ref INPUT_KEY_MOUSE: Vec<(&'static str, i64)> = vec![
        ("Left", 1),
        ("Right", 2),
        ("Middle", 3),
        ("XButton1", 8),
        ("XButton2", 9),
        ("WheelUp", 4),
        ("WheelDown", 5),
        ("WheelLeft", 6),
        ("WheelRight", 7),
        ("", -1),
    ];

    pub static ref INPUT_KEY_KEYBOARD: Vec<(&'static str, i64)> = vec![
        ("Escape", 16777217),
        ("Esc", 16777217),
        ("Tab", 16777218),
        ("Backspace", 16777220),
        ("Enter", 16777221),
        ("KPEnter", 16777222),
        ("Insert", 16777223),
        ("Delete", 16777224),
        ("Pause", 16777225),
        ("PrintScreen", 16777226),
        ("SystemRequest", 16777227),
        ("Clear", 16777228),
        ("Home", 16777229),
        ("End", 16777230),
        ("Left", 16777231),
        ("Up", 16777232),
        ("Right", 16777233),
        ("Down", 16777234),
        ("PageUp", 16777235),
        ("PageDown", 16777236),
        ("Shift", 16777237),
        ("Control", 16777238),
        ("Meta", 16777239),
        ("Alt", 16777240),
        ("CapsLock", 16777241),
        ("NumLock", 16777242),
        ("ScrollLock", 16777243),
        ("F1", 16777244),
        ("F2", 16777245),
        ("F3", 16777246),
        ("F4", 16777247),
        ("F5", 16777248),
        ("F6", 16777249),
        ("F7", 16777250),
        ("F8", 16777251),
        ("F9", 16777252),
        ("F10", 16777253),
        ("F11", 16777254),
        ("F12", 16777255),
        ("F13", 16777256),
        ("F14", 16777257),
        ("F15", 16777258),
        ("F16", 16777259),
        ("KP*", 16777345),
        ("KP/", 16777346),
        ("KP-", 16777347),
        ("KP.", 16777348),
        ("KP+", 16777349),
        ("KP0", 16777350),
        ("KP1", 16777351),
        ("KP2", 16777352),
        ("KP3", 16777353),
        ("KP4", 16777354),
        ("KP5", 16777355),
        ("KP6", 16777356),
        ("KP7", 16777357),
        ("KP8", 16777358),
        ("KP9", 16777359),
        ("Space", 32),
        (" ", 32),
        ("!", 33),
        ("\"", 34),
        ("#", 35),
        ("$", 36),
        ("%", 37),
        ("&", 38),
        ("\'", 39),
        ("(", 40),
        (")", 41),
        ("*", 42),
        ("+", 43),
        (",", 44),
        ("-", 45),
        (".", 46),
        ("/", 47),
        ("0", 48),
        ("1", 49),
        ("2", 50),
        ("3", 51),
        ("4", 52),
        ("5", 53),
        ("6", 54),
        ("7", 55),
        ("8", 56),
        ("9", 57),
        (":", 58),
        (";", 59),
        ("<", 60),
        ("=", 61),
        (">", 62),
        ("?", 63),
        ("@", 64),
        ("A", 65),
        ("B", 66),
        ("C", 67),
        ("D", 68),
        ("E", 69),
        ("F", 70),
        ("G", 71),
        ("H", 72),
        ("I", 73),
        ("J", 74),
        ("K", 75),
        ("L", 76),
        ("M", 77),
        ("N", 78),
        ("O", 79),
        ("P", 80),
        ("Q", 81),
        ("R", 82),
        ("S", 83),
        ("T", 84),
        ("U", 85),
        ("V", 86),
        ("W", 87),
        ("X", 88),
        ("Y", 89),
        ("Z", 90),
        ("[", 91),
        ("\\", 92),
        ("]", 93),
        ("^", 94),
        ("_", 95),
        ("`", 96),
        ("{", 123),
        ("|", 124),
        ("}", 125),
        ("~", 126),
        ("", -1),
    ];

    pub static ref INPUT_KEY_JOYSTICK: Vec<(&'static str, i64)> = vec![
        ("XboxA", 0), // Xbox controller A
        ("XboxB", 1), // Xbox controller B
        ("XboxX", 2), // Xbox controller X
        ("XboxY", 3), // Xbox controller Y
        ("PSCross", 0), // DualShock X
        ("PSCircle", 1), // DualShock circle
        ("PSSquare", 2), // DualShock square
        ("PSTriangle", 3), // DualShock triangle
        ("NintendoB", 0), // Nintendo controller B
        ("NintendoA", 1), // Nintendo controller A
        ("NintendoY", 2), // Nintendo controller Y
        ("NintendoX", 3), // Nintendo controller X
        ("Select", 10),
        ("Start", 11),
        ("DPadUp", 12),
        ("Up", 12),
        ("DPadDown", 13),
        ("Down", 13),
        ("DPadLeft", 14),
        ("Left", 14),
        ("DPadRight", 15),
        ("Right", 15),
        ("LeftShoulder", 4),
        ("LB", 4),
        ("LeftTrigger", 6),
        ("LT", 6),
        ("LeftStickClick", 8),
        ("LSB", 8),
        ("RightShoulder", 5),
        ("RB", 5),
        ("RightTrigger", 7),
        ("RT", 7),
        ("RightStickClick", 9),
        ("RSB", 9),
        ("", -1),
    ];
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InputDevice {
    Unknown,
    Mouse,
    Keyboard,
    GamePad,
}

#[derive(Derivative, Debug, Clone, Copy, Eq)]
#[derivative(PartialEq, Hash)]
pub struct InputKey {
    pub device: InputDevice,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub key_name: &'static str,
    pub key_code: i64,
}

impl Default for InputKey {
    fn default() -> InputKey {
        return InputKey {
            device: InputDevice::Unknown,
            key_name: "",
            key_code: -1,
        };
    }
}

impl InputKey {
    pub fn from_name(device: InputDevice, key_name: &str) -> Result<InputKey> {
        let (name, code) = match device {
            InputDevice::Mouse => INPUT_KEY_MOUSE.iter(),
            InputDevice::Keyboard => INPUT_KEY_KEYBOARD.iter(),
            InputDevice::GamePad => INPUT_KEY_JOYSTICK.iter(),
            _ => return Err(anyhow!("Unknown device")),
        }
        .find(|(name, _)| name.to_lowercase() == key_name.to_lowercase())
        .ok_or(anyhow!("Unknown key name ({})", key_name))?;
        return Ok(InputKey {
            device: device,
            key_name: *name,
            key_code: *code,
        });
    }

    pub fn from_code(device: InputDevice, key_code: i64) -> Result<InputKey> {
        let (name, code) = match device {
            InputDevice::Mouse => INPUT_KEY_MOUSE.iter(),
            InputDevice::Keyboard => INPUT_KEY_KEYBOARD.iter(),
            InputDevice::GamePad => INPUT_KEY_JOYSTICK.iter(),
            _ => return Err(anyhow!("Unknown device")),
        }
        .find(|(_, code)| *code == key_code)
        .ok_or(anyhow!("Unknown key name ({})", key_code))?;
        return Ok(InputKey {
            device: device,
            key_name: *name,
            key_code: *code,
        });
    }
}

impl Serialize for InputKey {
    fn serialize<S: Serializer>(&self, serializer: S) -> result::Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("InputKey", 2)?;
        state.serialize_field("device", &self.device)?;
        state.serialize_field("key", &self.key_name)?;
        return state.end();
    }
}

struct InputKeyVisitor;

impl<'de> Visitor<'de> for InputKeyVisitor {
    type Value = InputKey;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer between -2^31 and 2^31")
    }

    fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> result::Result<InputKey, V::Error> {
        let mut device: InputDevice = InputDevice::Mouse;
        let mut key_name: &str = "";
        while let Some(key) = map.next_key::<String>()? {
            if key == "device" {
                device = map.next_value()?;
            } else if key == "key" {
                key_name = map.next_value()?;
            }
        }
        return InputKey::from_name(device, key_name)
            .map_err(|_| Error::custom(format!("Invalid key ({})", key_name)));
    }
}

impl<'de> Deserialize<'de> for InputKey {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> result::Result<Self, D::Error> {
        deserializer.deserialize_struct("InputKey", &["device", "key"], InputKeyVisitor)
    }
}
