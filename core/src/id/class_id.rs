use serde::{Deserialize, Serialize};
use strum::{EnumString, IntoStaticStr, ToString};

#[repr(C)]
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
    None = 0,
    StageGeneral = 0x0101,
    CharaGeneral = 0x0201,
    Skill = 0x0301,
    Action = 0x8001,
}

impl Default for ClassID {
    fn default() -> ClassID {
        return ClassID::None;
    }
}

impl ClassID {
    pub fn invalid() -> ClassID {
        return ClassID::None;
    }

    pub fn is_valid(&self) -> bool {
        return self != &ClassID::None;
    }

    pub fn is_invalid(&self) -> bool {
        return self == &ClassID::None;
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

        assert_eq!(ClassID::CharaGeneral.is_character(), true);
        assert_eq!(ClassID::CharaGeneral.is_stage(), false);

        assert_eq!(ClassID::Skill.is_skill(), true);
        assert_eq!(ClassID::Skill.is_stage(), false);
    }
}
