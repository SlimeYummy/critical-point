use crate::derive::def_struct;
use serde::{Deserialize, Serialize};

#[def_struct]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash, Serialize, Deserialize)]
pub struct ObjID(u64);

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
    #[inline]
    pub fn invalid() -> ObjID {
        return ObjID(0xFFFF_FFFF_FFFF_FFFF);
    }

    #[inline]
    pub fn is_valid(&self) -> bool {
        return self.0 < Self::invalid().0;
    }

    #[inline]
    pub fn is_invalid(&self) -> bool {
        return self.0 >= Self::invalid().0;
    }
}

#[derive(Debug)]
pub struct ObjIDGener {
    counter: u64,
}

impl !Sync for ObjIDGener {}
impl !Send for ObjIDGener {}

impl ObjIDGener {
    pub(crate) fn new(start: u64) -> ObjIDGener {
        return ObjIDGener { counter: start };
    }

    pub(crate) fn gen(&mut self) -> ObjID {
        let obj_id = ObjID(self.counter);
        if obj_id.is_valid() {
            self.counter += 1;
            return obj_id;
        } else {
            panic!("ObjID exhausted!");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_obj_id() {
        let mut gener = ObjIDGener::new(100000);

        assert_eq!(gener.gen(), ObjID(100000));
        assert_eq!(gener.gen(), ObjID(100001));

        assert_eq!(ObjID::from(1234), ObjID(1234));
        assert_eq!(ObjID::from(0xFFFF_FFFF_FFFF_FFFF), ObjID::invalid());

        assert_eq!(u64::from(gener.gen()), 100002);
        assert_eq!(u64::from(ObjID::invalid()), 0xFFFF_FFFF_FFFF_FFFF);

        assert_eq!(gener.gen().is_valid(), true);
        assert_eq!(ObjID::invalid().is_valid(), false);
    }
}
