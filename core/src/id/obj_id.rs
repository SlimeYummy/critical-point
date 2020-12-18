use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Hash, Serialize, Deserialize)]
pub struct ObjID(String);

impl From<&'static str> for ObjID {
    fn from(text: &'static str) -> ObjID {
        return ObjID(text.to_string());
    }
}

impl From<String> for ObjID {
    fn from(text: String) -> ObjID {
        return ObjID(text);
    }
}

impl From<ObjID> for String {
    fn from(id: ObjID) -> String {
        return id.0;
    }
}

impl Default for ObjID {
    fn default() -> ObjID {
        return ObjID(String::new());
    }
}

impl ObjID {
    pub fn invalid() -> ObjID {
        return ObjID(String::new());
    }

    pub fn is_valid(&self) -> bool {
        return self.0 != Self::invalid().0;
    }

    pub fn is_invalid(&self) -> bool {
        return self.0 == Self::invalid().0;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash, Serialize, Deserialize)]
pub struct FastObjID(u64);

impl From<u64> for FastObjID {
    fn from(num: u64) -> FastObjID {
        return FastObjID(num);
    }
}

impl From<FastObjID> for u64 {
    fn from(id: FastObjID) -> u64 {
        return id.0;
    }
}

impl Default for FastObjID {
    fn default() -> FastObjID {
        return FastObjID(0xFFFF_FFFF_FFFF_FFFF);
    }
}

impl FastObjID {
    pub fn invalid() -> FastObjID {
        return FastObjID(0xFFFF_FFFF_FFFF_FFFF);
    }

    pub fn is_valid(&self) -> bool {
        return self.0 < Self::invalid().0;
    }

    pub fn is_invalid(&self) -> bool {
        return self.0 >= Self::invalid().0;
    }
}

#[derive(Debug)]
pub struct FastObjIDGener {
    counter: u64,
}

impl !Sync for FastObjIDGener {}
impl !Send for FastObjIDGener {}

impl FastObjIDGener {
    pub fn new(start: u64) -> FastObjIDGener {
        return FastObjIDGener { counter: start };
    }

    pub fn gen(&mut self) -> FastObjID {
        let fobj_id = FastObjID(self.counter);
        if fobj_id.is_valid() {
            self.counter += 1;
            return fobj_id;
        } else {
            panic!("FastObjID exhausted!");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_obj_id() {
        assert_eq!(ObjID::from("abcd"), ObjID("abcd".to_string()));
        assert_eq!(ObjID::from(""), ObjID::invalid());

        assert_eq!(ObjID::from("789".to_string()), ObjID("789".to_string()));
        assert_eq!(ObjID::from("".to_string()), ObjID::invalid());

        assert_eq!(ObjID("xyz".to_string()).is_valid(), true);
        assert_eq!(ObjID::invalid().is_valid(), false);
    }

    #[test]
    fn test_fast_obj_id() {
        let mut gener = FastObjIDGener::new(100000);

        assert_eq!(gener.gen(), FastObjID(100000));
        assert_eq!(gener.gen(), FastObjID(100001));

        assert_eq!(FastObjID::from(1234), FastObjID(1234));
        assert_eq!(FastObjID::from(0xFFFF_FFFF_FFFF_FFFF), FastObjID::invalid());

        assert_eq!(u64::from(gener.gen()), 100002);
        assert_eq!(u64::from(FastObjID::invalid()), 0xFFFF_FFFF_FFFF_FFFF);

        assert_eq!(gener.gen().is_valid(), true);
        assert_eq!(FastObjID::invalid().is_valid(), false);
    }
}
