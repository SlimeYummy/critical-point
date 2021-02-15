#![allow(dead_code)]

pub fn bool_false() -> bool {
    return false;
}

pub fn bool_true() -> bool {
    return true;
}

pub mod isometry {
    use m::{Fx, RealExt};
    use na::{Isometry3, Translation3, UnitQuaternion};
    use serde::de::{Deserializer, MapAccess, Visitor};
    use serde::ser::{SerializeStruct, Serializer};
    use std::fmt;

    pub fn serialize<S: Serializer>(
        isometry: &Isometry3<Fx>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut class = serializer.serialize_struct("Isometry3", 2)?;
        let (roll, pitch, yaw) = isometry.rotation.euler_angles();
        let euler = [
            pitch * Fx::frac_180_pi(),
            yaw * Fx::frac_180_pi(),
            roll * Fx::frac_180_pi(),
        ];
        class.serialize_field("rotation", &euler)?;
        class.serialize_field("translation", &isometry.translation)?;
        return class.end();
    }

    struct Isometry3Visitor;

    impl<'de> Visitor<'de> for Isometry3Visitor {
        type Value = Isometry3<Fx>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            return formatter.write_str("Isometry3{ rotation: [], translation: [] }");
        }

        fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> Result<Isometry3<Fx>, V::Error> {
            let mut rotation: UnitQuaternion<Fx> = UnitQuaternion::identity();
            let mut translation: Translation3<Fx> = Translation3::identity();
            while let Some(key) = map.next_key::<String>()? {
                if key == "translation" {
                    translation = map.next_value()?;
                } else if key == "rotation" {
                    let [pitch, yaw, roll] = map.next_value::<[Fx; 3]>()?;
                    rotation = UnitQuaternion::from_euler_angles(
                        roll * Fx::frac_pi_180(),
                        pitch * Fx::frac_pi_180(),
                        yaw * Fx::frac_pi_180(),
                    );
                }
            }
            return Ok(Isometry3::from_parts(translation, rotation));
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Isometry3<Fx>, D::Error> {
        return deserializer.deserialize_struct(
            "Isometry3",
            &["rotation", "translation"],
            Isometry3Visitor,
        );
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use approx::relative_eq;
        use m::fi;
        use na::{RealField, Vector3};
        use serde::{Deserialize, Serialize};

        #[derive(Clone, Debug, Deserialize, Serialize)]
        struct Struct {
            #[serde(with = "super")]
            is: Isometry3<Fx>,
        }

        #[test]
        fn test_serde_isometry() {
            let s1 = Struct {
                is: Isometry3::from_parts(
                    Translation3::from(Vector3::new(fi(1), fi(1), fi(1))),
                    UnitQuaternion::from_euler_angles(
                        Fx::frac_pi_4(),
                        Fx::frac_pi_3(),
                        Fx::frac_pi_2(),
                    ),
                ),
            };
            let json = serde_json::to_string(&s1).unwrap();
            let s2 = serde_json::from_str::<Struct>(&json).unwrap();
            relative_eq!(s1.is, s2.is);
        }
    }
}
