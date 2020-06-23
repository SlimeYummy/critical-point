use euclid::num::Zero;
use fixed::traits::ToFixed;
use fixed::types::{I20F12, I40F24};
use simba::scalar::{FixedI20F12, FixedI40F24};

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
