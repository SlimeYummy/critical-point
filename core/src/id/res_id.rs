use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Hash, Serialize, Deserialize)]
pub struct ResID(String);

impl From<&'static str> for ResID {
    fn from(text: &'static str) -> ResID {
        return ResID(text.to_string());
    }
}

impl From<String> for ResID {
    fn from(text: String) -> ResID {
        return ResID(text);
    }
}

impl From<ResID> for String {
    fn from(id: ResID) -> String {
        return id.0;
    }
}

impl Default for ResID {
    fn default() -> ResID {
        return ResID(String::new());
    }
}

impl ResID {
    pub fn invalid() -> ResID {
        return ResID(String::new());
    }

    pub fn is_valid(&self) -> bool {
        return self.0 != Self::invalid().0;
    }

    pub fn is_invalid(&self) -> bool {
        return self.0 == Self::invalid().0;
    }

    pub fn is_stage(&self) -> bool {
        return self.0.starts_with("Stage");
    }

    pub fn is_character(&self) -> bool {
        return self.0.starts_with("Chara");
    }

    pub fn is_skill(&self) -> bool {
        return self.0.starts_with("Skill");
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Hash, Serialize, Deserialize)]
pub struct FastResID(u64);

impl From<u64> for FastResID {
    fn from(num: u64) -> FastResID {
        return FastResID(num);
    }
}

impl From<FastResID> for u64 {
    fn from(id: FastResID) -> u64 {
        return id.0;
    }
}

impl Default for FastResID {
    fn default() -> FastResID {
        return FastResID(0xFFFF_FFFF_FFFF_FFFF);
    }
}

impl FastResID {
    pub fn invalid() -> FastResID {
        return FastResID(0xFFFF_FFFF_FFFF_FFFF);
    }

    pub fn is_valid(&self) -> bool {
        return self.0 < Self::invalid().0;
    }

    pub fn is_invalid(&self) -> bool {
        return self.0 >= Self::invalid().0;
    }
}

#[derive(Debug)]
pub struct FastResIDGener {
    counter: u64,
}

impl !Sync for FastResIDGener {}
impl !Send for FastResIDGener {}

impl FastResIDGener {
    pub(crate) fn new(start: u64) -> FastResIDGener {
        return FastResIDGener { counter: start };
    }

    pub(crate) fn gen(&mut self) -> FastResID {
        let fobj_id = FastResID(self.counter);
        if fobj_id.is_valid() {
            self.counter += 1;
            return fobj_id;
        } else {
            panic!("FastResID exhausted!");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_res_id() {
        assert_eq!(ResID::from("abcd"), ResID("abcd".to_string()));
        assert_eq!(ResID::from(""), ResID::invalid());

        assert_eq!(ResID::from("789".to_string()), ResID("789".to_string()));
        assert_eq!(ResID::from("".to_string()), ResID::invalid());

        assert_eq!(ResID("xyz".to_string()).is_valid(), true);
        assert_eq!(ResID::invalid().is_valid(), false);
    }

    #[test]
    fn test_fast_res_id() {
        let mut gener = FastResIDGener::new(1);

        assert_eq!(gener.gen(), FastResID(1));
        assert_eq!(gener.gen(), FastResID(2));

        assert_eq!(FastResID::from(1234), FastResID(1234));
        assert_eq!(FastResID::from(0xFFFF_FFFF_FFFF_FFFF), FastResID::invalid());

        assert_eq!(u64::from(gener.gen()), 3);
        assert_eq!(u64::from(FastResID::invalid()), 0xFFFF_FFFF_FFFF_FFFF);

        assert_eq!(gener.gen().is_valid(), true);
        assert_eq!(FastResID::invalid().is_valid(), false);
    }
}
