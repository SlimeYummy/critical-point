use approx::AbsDiffEq;
use na::RealField;
use num_traits::{One, Zero};

pub fn approx_eq<T: AbsDiffEq>(a: T, b: T) -> bool {
    return a.abs_diff_eq(&b, T::default_epsilon());
}

pub fn approx_ne<T: AbsDiffEq>(a: T, b: T) -> bool {
    return a.abs_diff_ne(&b, T::default_epsilon());
}

pub fn approx_zero<T: AbsDiffEq + Zero>(v: T) -> bool {
    return v.abs_diff_eq(&T::zero(), T::default_epsilon());
}

pub fn approx_one<T: AbsDiffEq + One>(v: T) -> bool {
    return v.abs_diff_eq(&T::one(), T::default_epsilon());
}

pub fn approx_lt<T: RealField>(a: T, b: T) -> bool {
    return a - T::default_epsilon() < b;
}

pub fn approx_le<T: RealField>(a: T, b: T) -> bool {
    return a - T::default_epsilon() <= b;
}

pub fn approx_gt<T: RealField>(a: T, b: T) -> bool {
    return a + T::default_epsilon() > b;
}

pub fn approx_ge<T: RealField>(a: T, b: T) -> bool {
    return a + T::default_epsilon() >= b;
}
