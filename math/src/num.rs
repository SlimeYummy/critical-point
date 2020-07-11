use euclid::num::Zero;
use fixed::traits::ToFixed;
use fixed::types::{I20F12, I40F24};
use simba::scalar::{FixedI20F12, FixedI40F24};

pub type Fx32 = FixedI20F12;
pub type Fx64 = FixedI40F24;
pub type Fx = FixedI40F24;

pub fn fx32<N: ToFixed>(num: N) -> Fx32 {
    let mut fixed: Fx32 = Fx32::zero();
    fixed.0 = I20F12::from_num(num);
    return fixed;
}

pub fn fx64<N: ToFixed>(num: N) -> Fx {
    let mut fixed: Fx = Fx::zero();
    fixed.0 = I40F24::from_num(num);
    return fixed;
}

pub fn fx<N: ToFixed>(num: N) -> Fx {
    let mut fixed: Fx = Fx::zero();
    fixed.0 = I40F24::from_num(num);
    return fixed;
}
