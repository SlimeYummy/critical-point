#![cfg_attr(rustfmt, rustfmt_skip)]

use crate::fx::{fx, Fx};

pub trait RealExt {
    fn c0() -> Self;
    fn c1() -> Self;
    fn c2() -> Self;
    fn c3() -> Self;
    fn c4() -> Self;
    fn c5() -> Self;
    fn c10() -> Self;
    fn frac2() -> Self;
    fn frac3() -> Self;
    fn frac4() -> Self;
    fn frac5() -> Self;
    fn sqrt2() -> Self;
    fn sqrt3() -> Self;
    fn sqrt5() -> Self;
}

impl RealExt for Fx {
    fn c0() -> Self { fx(0) }
    fn c1() -> Self { fx(1) }
    fn c2() -> Self { fx(2) }
    fn c3() -> Self { fx(3) }
    fn c4() -> Self { fx(4) }
    fn c5() -> Self { fx(5) }
    fn c10() -> Self { fx(10) }
    fn frac2() -> Self { fx(0.5) }
    fn frac3() -> Self { fx(0.3333333333333333) }
    fn frac4() -> Self { fx(0.25) }
    fn frac5() -> Self { fx(0.2) }
    fn sqrt2() -> Self { fx(1.4142135623730951) }
    fn sqrt3() -> Self { fx(1.7320508075688772) }
    fn sqrt5() -> Self { fx(2.23606797749979) }
}

impl RealExt for f32 {
    fn c0() -> Self { 0.0 }
    fn c1() -> Self { 1.0 }
    fn c2() -> Self { 2.0 }
    fn c3() -> Self { 3.0 }
    fn c4() -> Self { 4.0 }
    fn c5() -> Self { 5.0 }
    fn c10() -> Self { 10.0 }
    fn frac2() -> Self { 0.5 }
    fn frac3() -> Self { 0.3333333333333333 }
    fn frac4() -> Self { 0.25 }
    fn frac5() -> Self { 0.2 }
    fn sqrt2() -> Self { 1.4142135623730951 }
    fn sqrt3() -> Self { 1.7320508075688772 }
    fn sqrt5() -> Self { 2.23606797749979 }
}

impl RealExt for f64 {
    fn c0() -> Self { 0.0 }
    fn c1() -> Self { 1.0 }
    fn c2() -> Self { 2.0 }
    fn c3() -> Self { 3.0 }
    fn c4() -> Self { 4.0 }
    fn c5() -> Self { 5.0 }
    fn c10() -> Self { 10.0 }
    fn frac2() -> Self { 0.5 }
    fn frac3() -> Self { 0.3333333333333333 }
    fn frac4() -> Self { 0.25 }
    fn frac5() -> Self { 0.2 }
    fn sqrt2() -> Self { 1.4142135623730951 }
    fn sqrt3() -> Self { 1.7320508075688772 }
    fn sqrt5() -> Self { 2.23606797749979 }
}
