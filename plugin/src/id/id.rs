#![allow(dead_code)]

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

pub struct ObjectIDGener {
    counter: u64,
}

impl ObjectIDGener {
    pub fn new() -> ObjectIDGener {
        return ObjectIDGener { counter: 100000 };
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
pub struct TypeID(pub(super) u32);

impl Eq for TypeID {}

impl From<u32> for TypeID {
    fn from(num: u32) -> TypeID {
        return TypeID(num);
    }
}

impl From<TypeID> for u32 {
    fn from(id: TypeID) -> u32 {
        return id.0;
    }
}

impl Default for TypeID {
    fn default() -> TypeID {
        return TypeID(0xFFFF_FFFF);
    }
}

impl TypeID {
    pub fn invaild() -> TypeID {
        return TypeID(0xFFFF_FFFF);
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
        let mut gener = ObjectIDGener::new();

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
    #[should_panic]
    fn test_object_id_panic() {
        let mut gener = ObjectIDGener::new();
        gener.counter = 0xFFFF_FFFF_FFFF_FFFF;
        let _ = gener.gen();
    }
}
