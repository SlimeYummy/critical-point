use crate::derive::def_enum;
use serde::{Deserialize, Serialize};
use strum::{EnumString, IntoStaticStr, ToString};

#[def_enum]
#[repr(u16)]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    EnumString,
    ToString,
    IntoStaticStr,
)]
pub enum ClassID {
    Invalid = 0,
    StageGeneral = 0x0101,
    StageScenery = 0x0102,
    CharaHuman = 0x0201,
    Skill = 0x0301,
    HitAttachment = 0x0401,
    HitPathRay = 0x0402,
    Action = 0xFFFE,
    Prefab = 0xFFFF,
}

impl Default for ClassID {
    fn default() -> ClassID {
        return ClassID::Invalid;
    }
}

impl ClassID {
    pub fn invalid() -> ClassID {
        return ClassID::Invalid;
    }

    pub fn is_valid(&self) -> bool {
        return self != &ClassID::Invalid;
    }

    pub fn is_invalid(&self) -> bool {
        return self == &ClassID::Invalid;
    }

    pub fn is_stage(&self) -> bool {
        return ((*self as usize) & 0xFF00) == 0x0100;
    }

    pub fn is_character(&self) -> bool {
        return ((*self as usize) & 0xFF00) == 0x0200;
    }

    pub fn is_skill(&self) -> bool {
        return ((*self as usize) & 0xFF00) == 0x0300;
    }

    pub fn is_action(&self) -> bool {
        return (*self as usize) == 0xFFFE;
    }

    pub fn is_command(&self) -> bool {
        return (*self as usize) == 0xFFFF;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_class_id() {
        assert_eq!(ClassID::default(), ClassID::invalid());
        assert_eq!(ClassID::invalid().is_invalid(), true);
        assert_eq!(ClassID::invalid().is_valid(), false);

        assert_eq!(ClassID::StageGeneral.is_stage(), true);
        assert_eq!(ClassID::StageGeneral.is_character(), false);

        assert_eq!(ClassID::CharaHuman.is_character(), true);
        assert_eq!(ClassID::CharaHuman.is_stage(), false);

        assert_eq!(ClassID::Skill.is_skill(), true);
        assert_eq!(ClassID::Skill.is_stage(), false);

        assert_eq!(ClassID::Prefab.is_command(), true);
        assert_eq!(ClassID::Prefab.is_skill(), false);

        assert_eq!(ClassID::Action.is_action(), true);
        assert_eq!(ClassID::Action.is_command(), false);
    }
}
