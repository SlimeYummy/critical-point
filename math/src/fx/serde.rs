use super::convert::ff;
use super::fx::Fx;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{self, Formatter};

impl Serialize for Fx {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        return serializer.serialize_f64(self.to_f64());
    }
}

impl<'de> Deserialize<'de> for Fx {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct F64Visitor;

        impl<'de> Visitor<'de> for F64Visitor {
            type Value = f64;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                return formatter.write_str("a int or a float");
            }

            fn visit_f32<E: de::Error>(self, value: f32) -> Result<Self::Value, E> {
                return Ok(value as f64);
            }

            fn visit_f64<E: de::Error>(self, value: f64) -> Result<Self::Value, E> {
                return Ok(value);
            }

            fn visit_i32<E: de::Error>(self, value: i32) -> Result<Self::Value, E> {
                return Ok(value as f64);
            }

            fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
                return Ok(value as f64);
            }

            fn visit_u32<E: de::Error>(self, value: u32) -> Result<Self::Value, E> {
                return Ok(value as f64);
            }

            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
                return Ok(value as f64);
            }
        }

        return deserializer.deserialize_f64(F64Visitor).map(|val| ff(val));
    }
}
