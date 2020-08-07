#![allow(dead_code)]

use serde::{Deserialize, Serialize};

//
// Object ID
//

#[derive(Clone, Copy, Debug, Hash, PartialEq, PartialOrd)]
pub struct ObjID(u64);

impl Eq for ObjID {}

impl From<u64> for ObjID {
    fn from(num: u64) -> ObjID {
        return ObjID(num);
    }
}

impl From<ObjID> for u64 {
    fn from(id: ObjID) -> u64 {
        return id.0;
    }
}

impl Default for ObjID {
    fn default() -> ObjID {
        return ObjID(0xFFFF_FFFF_FFFF_FFFF);
    }
}

impl ObjID {
    pub fn invaild() -> ObjID {
        return ObjID(0xFFFF_FFFF_FFFF_FFFF);
    }

    pub fn is_vaild(&self) -> bool {
        return self.0 < Self::invaild().0;
    }

    pub fn is_invaild(&self) -> bool {
        return self.0 >= Self::invaild().0;
    }
}

pub struct ObjIDGener {
    counter: u64,
}

impl ObjIDGener {
    pub fn new() -> ObjIDGener {
        return ObjIDGener { counter: 100000 };
    }

    pub fn gen(&mut self) -> ObjID {
        let obj_id = ObjID(self.counter);
        if obj_id.is_vaild() {
            self.counter += 1;
            return obj_id;
        } else {
            panic!("ObjID exhausted!");
        }
    }
}

//
// Type ID
//

#[derive(Clone, Copy, Debug, Hash, PartialEq, PartialOrd)]
pub struct ClassID(pub(super) u32);

impl Eq for ClassID {}

impl From<u32> for ClassID {
    fn from(num: u32) -> ClassID {
        return ClassID(num);
    }
}

impl From<ClassID> for u32 {
    fn from(id: ClassID) -> u32 {
        return id.0;
    }
}

impl Default for ClassID {
    fn default() -> ClassID {
        return ClassID(0xFFFF_FFFF);
    }
}

impl ClassID {
    pub fn invaild() -> ClassID {
        return ClassID(0xFFFF_FFFF);
    }

    pub fn is_vaild(&self) -> bool {
        return self.0 < Self::invaild().0;
    }

    pub fn is_invaild(&self) -> bool {
        return self.0 >= Self::invaild().0;
    }
}

//
// Resource ID
//

#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, PartialOrd, Serialize)]
pub struct ResID(u64);

impl Eq for ResID {}

impl From<u64> for ResID {
    fn from(num: u64) -> ResID {
        return ResID(num);
    }
}

impl From<ResID> for u64 {
    fn from(id: ResID) -> u64 {
        return id.0;
    }
}

impl Default for ResID {
    fn default() -> ResID {
        return ResID(0xFFFF_FFFF_FFFF_FFFF);
    }
}

impl ResID {
    pub fn invaild() -> ResID {
        return ResID(0xFFFF_FFFF_FFFF_FFFF);
    }

    pub fn is_vaild(&self) -> bool {
        return self.0 < Self::invaild().0;
    }

    pub fn is_invaild(&self) -> bool {
        return self.0 >= Self::invaild().0;
    }
}

//
// tests
//

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_id_normal() {
        let mut gener = ObjIDGener::new();

        assert_eq!(gener.gen(), ObjID(100000));
        assert_eq!(gener.gen(), ObjID(100001));

        assert_eq!(ObjID::from(1234), ObjID(1234));
        assert_eq!(ObjID::from(0xFFFF_FFFF_FFFF_FFFF), ObjID::invaild());

        assert_eq!(u64::from(gener.gen()), 100002);
        assert_eq!(u64::from(ObjID::invaild()), 0xFFFF_FFFF_FFFF_FFFF);

        assert_eq!(gener.gen().is_vaild(), true);
        assert_eq!(ObjID::invaild().is_vaild(), false);
    }

    #[test]
    fn test_class_id_normal() {
        assert_eq!(ClassID::from(1234), ClassID(1234));
        assert_eq!(ClassID::from(0xFFFF_FFFF), ClassID::invaild());

        assert_eq!(ClassID(1111).is_vaild(), true);
        assert_eq!(ClassID::invaild().is_vaild(), false);
    }

    #[test]
    fn test_res_id_normal() {
        assert_eq!(ResID::from(1234), ResID(1234));
        assert_eq!(ResID::from(0xFFFF_FFFF_FFFF_FFFF), ResID::invaild());

        assert_eq!(ResID(1111).is_vaild(), true);
        assert_eq!(ResID::invaild().is_vaild(), false);
    }
}
