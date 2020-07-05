use euclid::num::Zero;
use fixed::traits::ToFixed;
use fixed::types::{I20F12, I40F24};
use simba::scalar::{FixedI20F12, FixedI40F24};
use na::{Vector2, Vector3};

pub type Fixed32 = FixedI20F12;
pub type Fixed64 = FixedI40F24;

pub fn fixed32<N: ToFixed>(num: N) -> Fixed32 {
    let mut fixed: Fixed32 = Fixed32::zero();
    fixed.0 = I20F12::from_num(num);
    return fixed;
}

pub fn fixed64<N: ToFixed>(num: N) -> Fixed64 {
    let mut fixed: Fixed64 = Fixed64::zero();
    fixed.0 = I40F24::from_num(num);
    return fixed;
}

// normal => (a, b, c)
// plane => ax + by + cz = 0
// y = - (ax + cz) / b
pub fn direction_on_plane(normal: Vector3<Fixed64>, direction: Vector2<Fixed64>) -> Vector3<Fixed64> {
    let a: Fixed64 = normal.x;
    let b: Fixed64 = normal.y;
    let c: Fixed64 = normal.z;
    if b == Fixed64::zero() {
        return Vector3::new(Fixed64::zero(), Fixed64::zero(), Fixed64::zero());
    }
    let x = direction.x;
    let z = direction.y;
    let y = fixed64(-1) * (a * x + c * z) / b;
    return Vector3::new(x, y, z);
}
