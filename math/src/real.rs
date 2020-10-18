#![cfg_attr(rustfmt, rustfmt_skip)]

use crate::fx::{fx, Fx};

pub trait RealExt {
    fn c_1() -> Self;
    fn c_2() -> Self;
    fn c_3() -> Self;
    fn c_4() -> Self;
    fn c_5() -> Self;
    fn c_10() -> Self;
    fn c_frac_2() -> Self;
    fn c_frac_3() -> Self;
    fn c_frac_4() -> Self;
    fn c_frac_5() -> Self;
    fn c_sqrt_2() -> Self;
    fn c_sqrt_3() -> Self;
    fn c_sqrt_5() -> Self;
}

impl RealExt for Fx {
    fn c_1() -> Self { fx(1) }
    fn c_2() -> Self { fx(2) }
    fn c_3() -> Self { fx(3) }
    fn c_4() -> Self { fx(4) }
    fn c_5() -> Self { fx(5) }
    fn c_10() -> Self { fx(10) }
    fn c_frac_2() -> Self { fx(0.5) }
    fn c_frac_3() -> Self { fx(0.3333333333333333) }
    fn c_frac_4() -> Self { fx(0.25) }
    fn c_frac_5() -> Self { fx(0.2) }
    fn c_sqrt_2() -> Self { fx(1.4142135623730951) }
    fn c_sqrt_3() -> Self { fx(1.7320508075688772) }
    fn c_sqrt_5() -> Self { fx(2.23606797749979) }
}

impl RealExt for f32 {
    fn c_1() -> Self { 1.0 }
    fn c_2() -> Self { 2.0 }
    fn c_3() -> Self { 3.0 }
    fn c_4() -> Self { 4.0 }
    fn c_5() -> Self { 5.0 }
    fn c_10() -> Self { 10.0 }
    fn c_frac_2() -> Self { 0.5 }
    fn c_frac_3() -> Self { 0.3333333333333333 }
    fn c_frac_4() -> Self { 0.25 }
    fn c_frac_5() -> Self { 0.2 }
    fn c_sqrt_2() -> Self { 1.4142135623730951 }
    fn c_sqrt_3() -> Self { 1.7320508075688772 }
    fn c_sqrt_5() -> Self { 2.23606797749979 }
}

impl RealExt for f64 {
    fn c_1() -> Self { 1.0 }
    fn c_2() -> Self { 2.0 }
    fn c_3() -> Self { 3.0 }
    fn c_4() -> Self { 4.0 }
    fn c_5() -> Self { 5.0 }
    fn c_10() -> Self { 10.0 }
    fn c_frac_2() -> Self { 0.5 }
    fn c_frac_3() -> Self { 0.3333333333333333 }
    fn c_frac_4() -> Self { 0.25 }
    fn c_frac_5() -> Self { 0.2 }
    fn c_sqrt_2() -> Self { 1.4142135623730951 }
    fn c_sqrt_3() -> Self { 1.7320508075688772 }
    fn c_sqrt_5() -> Self { 2.23606797749979 }
}
