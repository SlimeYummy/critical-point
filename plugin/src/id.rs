use failure::{Error, format_err};
use std::convert::{TryFrom, From};
use std::sync::atomic::{AtomicU64, Ordering};

//
// Object ID
//

#[allow(dead_code)]
#[derive(Copy, Clone, Debug, Hash, PartialEq, PartialOrd)]
pub struct ObjectID(u64);

impl Eq for ObjectID {}

impl TryFrom<u64> for ObjectID {
    type Error = Error;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value < Self::invaild().0 {
            return Ok(ObjectID(value));
        } else {
            return Err(format_err!("Invaild ObjectID number."));
        }
    }
}

impl From<ObjectID> for u64 {
    fn from(id: ObjectID) -> u64 { return id.0; }
}

static OBJECT_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

impl ObjectID {
    pub fn new() -> ObjectID {
        let id_num = OBJECT_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        let obj_id = ObjectID(id_num);
        if obj_id.is_vaild() {
            return obj_id;
        } else {
            panic!("ObjectID exhausted!");
        }
    }

    pub fn invaild() -> ObjectID {
        return ObjectID(0xFFFF_FFFF_FFFF);
    }

    pub fn is_vaild(&self) -> bool {
        return self.0 < Self::invaild().0;
    }
}

//
// Type ID
//

#[allow(dead_code)]
#[derive(Copy, Clone, Debug, Hash, PartialEq, PartialOrd)]
pub struct TypeID(u16);

impl Eq for TypeID {}

impl TryFrom<u16> for TypeID {
    type Error = Error;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if value < Self::invaild().0 {
            return Ok(TypeID(value));
        } else {
            return Err(format_err!("Invaild TypeID number."));
        }
    }
}

impl From<TypeID> for u16 {
    fn from(id: TypeID) -> u16 { return id.0; }
}

static TYPE_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

impl TypeID {
    pub fn new() -> TypeID {
        let id_num = TYPE_ID_COUNTER.fetch_add(1, Ordering::Relaxed) as u16;
        let type_id = TypeID(id_num);
        if type_id.is_vaild() {
            return type_id;
        } else {
            panic!("TypeID exhausted!");
        }
    }

    pub fn invaild() -> TypeID {
        return TypeID(0xFFFF);
    }

    pub fn is_vaild(&self) -> bool {
        return self.0 < Self::invaild().0;
    }
}

//
// Union ID
//

#[allow(dead_code)]
#[derive(Copy, Clone, Debug, Hash, PartialEq, PartialOrd)]
pub struct UnionID(u64);

impl Eq for UnionID {}

impl UnionID {
    pub fn new(obj_id: ObjectID, type_id: TypeID) -> UnionID {
        let obj_num = obj_id.0;
        let type_num = type_id.0 as u64;
        return UnionID((type_num << 48) | obj_num);
    }

    pub fn object_id(&self) -> ObjectID {
        return ObjectID(self.0 & ObjectID::invaild().0);
    }

    pub fn type_id(&self) -> TypeID {
        return TypeID((self.0 >> 48) as u16);
    }

    pub fn invaild() -> UnionID {
        return UnionID(0xFFFF_FFFF_FFFF_FFFF);
    }

    pub fn is_vaild(&self) -> bool {
        return self.object_id().is_vaild() && self.type_id().is_vaild();
    }
}

//
// tests
//

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_id() {
        assert_eq!(ObjectID::new(), ObjectID(0));
        assert_eq!(ObjectID::new(), ObjectID(1));

        assert!(ObjectID::try_from(1234).is_ok());
        assert!(ObjectID::try_from(0xFFFF_FFFF_FFFF).is_err());

        assert_eq!(u64::from(ObjectID::new()), 2);
        assert_eq!(u64::from(ObjectID::invaild()), 0xFFFF_FFFF_FFFF);

        assert_eq!(ObjectID::new().is_vaild(), true);
        assert_eq!(ObjectID::invaild().is_vaild(), false);
    }

    #[test]
    #[should_panic]
    fn test_object_id_panic() {
        OBJECT_ID_COUNTER.store(0xFFFF_FFFF_FFFF, Ordering::Relaxed);
        let _ = ObjectID::new();
    }

    #[test]
    fn test_type_id() {
        assert_eq!(TypeID::new(), TypeID(0));
        assert_eq!(TypeID::new(), TypeID(1));

        assert!(TypeID::try_from(1234).is_ok());
        assert!(TypeID::try_from(0xFFFF).is_err());

        assert_eq!(u16::from(TypeID::new()), 2);
        assert_eq!(u16::from(TypeID::invaild()), 0xFFFF);

        assert_eq!(TypeID::new().is_vaild(), true);
        assert_eq!(TypeID::invaild().is_vaild(), false);
    }

    #[test]
    #[should_panic]
    fn test_type_id_panic() {
        TYPE_ID_COUNTER.store(0xFFFF, Ordering::Relaxed);
        let _ = TypeID::new();
    }

    #[test]
    fn test_union_id() {
        OBJECT_ID_COUNTER.store(0x9999, Ordering::Relaxed);
        TYPE_ID_COUNTER.store(0x100, Ordering::Relaxed);

        let obj_id = ObjectID::new();
        let type_id = TypeID::new();
        let union_id = UnionID::new(obj_id, type_id);

        assert_eq!(union_id.0, 0x0100_0000_0000_9999);
        assert_eq!(union_id.object_id(), obj_id);
        assert_eq!(union_id.type_id(), type_id);

        assert_eq!(union_id.is_vaild(), true);
        assert_eq!(UnionID::invaild().is_vaild(), false);
    }
}
