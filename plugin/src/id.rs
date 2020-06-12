//
// Object ID
//

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, PartialOrd)]
pub struct ObjectID(u64);

impl Eq for ObjectID {}

impl From<u64> for ObjectID {
    fn from(num: u64) -> ObjectID {
        return ObjectID(num);
    }
}

impl From<ObjectID> for u64 {
    fn from(id: ObjectID) -> u64 {
        return id.0;
    }
}

impl Default for ObjectID {
    fn default() -> ObjectID { return ObjectID(0xFFFF_FFFF_FFFF_FFFF); }
}

impl ObjectID {
    pub fn invaild() -> ObjectID {
        return ObjectID(0xFFFF_FFFF_FFFF_FFFF);
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

    pub fn gen(&mut self) -> ObjectID {
        let obj_id = ObjectID(self.counter);
        if obj_id.is_vaild() {
            self.counter += 1;
            return obj_id;
        } else {
            panic!("ObjectID exhausted!");
        }
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

        assert_eq!(gener.gen(), ObjectID(100000));
        assert_eq!(gener.gen(), ObjectID(100001));

        assert_eq!(ObjectID::from(1234), ObjectID(1234));
        assert_eq!(ObjectID::from(0xFFFF_FFFF_FFFF_FFFF), ObjectID::invaild());

        assert_eq!(u64::from(gener.gen()), 100002);
        assert_eq!(u64::from(ObjectID::invaild()), 0xFFFF_FFFF_FFFF_FFFF);

        assert_eq!(gener.gen().is_vaild(), true);
        assert_eq!(ObjectID::invaild().is_vaild(), false);
    }

    #[test]
    #[should_panic]
    fn test_object_id_panic() {
        let mut gener = ObjectIDGener::new();
        gener.counter = 0xFFFF_FFFF_FFFF_FFFF;
        let _ = gener.gen();
    }
}
