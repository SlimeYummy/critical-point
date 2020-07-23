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

pub fn fxi32(num: Fx) -> i32 {
    return num.0.to_num::<i32>();
}

pub fn fxu32(num: Fx) -> u32 {
    return num.0.to_num::<u32>();
}

pub fn fxi64(num: Fx) -> i64 {
    return num.0.to_num::<i64>();
}

pub fn fxu64(num: Fx) -> u64 {
    return num.0.to_num::<u64>();
}

pub fn fxf32(num: Fx) -> f32 {
    return num.0.to_num::<f32>();
}

pub fn fxf64(num: Fx) -> f64 {
    return num.0.to_num::<f64>();
}
