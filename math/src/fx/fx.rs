use approx::{AbsDiffEq, RelativeEq, UlpsEq};
use fixed::traits::ToFixed;
use fixed::types::I32F32;
use num_traits::{Bounded, FromPrimitive, Num, One, Signed, Zero};
use rand::distributions::{Distribution, OpenClosed01, Standard};
use rand::Rng;
use simba::scalar::{ComplexField, Field, RealField, SubsetOf};
use simba::simd::{PrimitiveSimdValue, SimdValue};
use std::cmp::Ordering;
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign,
};

#[derive(Clone, Copy, Eq)]
pub struct Fx(pub(crate) I32F32);

impl Default for Fx {
    fn default() -> Fx {
        Fx(I32F32::from_bits(0))
    }
}

impl Debug for Fx {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Display for Fx {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Hash for Fx {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.0.hash(h);
    }
}

impl PartialEq for Fx {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd for Fx {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for Fx {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl Distribution<Fx> for Standard {
    #[inline]
    fn sample<'a, G: Rng + ?Sized>(&self, rng: &mut G) -> Fx {
        let bits = rng.gen();
        Fx(I32F32::from_bits(bits))
    }
}

impl Distribution<Fx> for OpenClosed01 {
    #[inline]
    fn sample<'a, G: Rng + ?Sized>(&self, rng: &mut G) -> Fx {
        let val: f64 = rng.gen();
        Fx(I32F32::from_num(val))
    }
}

impl PrimitiveSimdValue for Fx {}

impl SimdValue for Fx {
    type Element = Self;
    type SimdBool = bool;

    #[inline(always)]
    fn lanes() -> usize {
        1
    }

    #[inline(always)]
    fn splat(val: Self::Element) -> Self {
        val
    }

    #[inline(always)]
    fn extract(&self, _: usize) -> Self::Element {
        *self
    }

    #[inline(always)]
    unsafe fn extract_unchecked(&self, _: usize) -> Self::Element {
        *self
    }

    #[inline(always)]
    fn replace(&mut self, _: usize, val: Self::Element) {
        *self = val
    }

    #[inline(always)]
    unsafe fn replace_unchecked(&mut self, _: usize, val: Self::Element) {
        *self = val
    }

    #[inline(always)]
    fn select(self, cond: Self::SimdBool, other: Self) -> Self {
        if cond {
            self
        } else {
            other
        }
    }
}

impl Mul for Fx {
    type Output = Self;
    #[inline(always)]
    fn mul(self, rhs: Self) -> Self {
        Self(self.0.saturating_mul(rhs.0))
    }
}

impl Div for Fx {
    type Output = Self;
    #[inline(always)]
    fn div(self, rhs: Self) -> Self {
        if !rhs.is_zero() {
            Self(self.0.saturating_div(rhs.0))
        } else {
            if self.0 > 0 {
                Self::max_value()
            } else if self.0 < 0 {
                Self::min_value()
            } else {
                Self::zero()
            }
        }
    }
}

impl Rem for Fx {
    type Output = Self;
    #[inline(always)]
    fn rem(self, rhs: Self) -> Self {
        if !rhs.is_zero() {
            Self(self.0 % rhs.0)
        } else {
            Self::zero()
        }
    }
}

impl Add for Fx {
    type Output = Self;
    #[inline(always)]
    fn add(self, rhs: Self) -> Self {
        Self(self.0.saturating_add(rhs.0))
    }
}

impl Sub for Fx {
    type Output = Self;
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self {
        Self(self.0.saturating_sub(rhs.0))
    }
}

impl Neg for Fx {
    type Output = Self;
    #[inline(always)]
    fn neg(self) -> Self {
        Self(-self.0)
    }
}

impl MulAssign for Fx {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        self.0 *= rhs.0
    }
}

impl DivAssign for Fx {
    #[inline(always)]
    fn div_assign(&mut self, rhs: Self) {
        self.0 /= rhs.0
    }
}

impl RemAssign for Fx {
    #[inline(always)]
    fn rem_assign(&mut self, rhs: Self) {
        self.0 %= rhs.0
    }
}

impl AddAssign for Fx {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0
    }
}

impl SubAssign for Fx {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0
    }
}

impl Zero for Fx {
    #[inline(always)]
    fn zero() -> Self {
        Self(I32F32::from_num(0))
    }

    #[inline(always)]
    fn is_zero(&self) -> bool {
        self.0 == Self::zero().0
    }
}

impl One for Fx {
    #[inline(always)]
    fn one() -> Self {
        Self(I32F32::from_num(1))
    }
}

impl Num for Fx {
    type FromStrRadixErr = ();
    fn from_str_radix(_str: &str, _radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        unimplemented!()
    }
}

impl Field for Fx {}

impl SubsetOf<Fx> for f64 {
    #[inline]
    fn to_superset(&self) -> Fx {
        Fx(I32F32::from_num(*self))
    }

    #[inline]
    fn from_superset(element: &Fx) -> Option<Self> {
        Some(Self::from_superset_unchecked(element))
    }

    #[inline]
    fn from_superset_unchecked(element: &Fx) -> Self {
        element.0.to_num::<f64>()
    }

    #[inline]
    fn is_in_subset(_: &Fx) -> bool {
        true
    }
}

impl SubsetOf<Fx> for Fx {
    #[inline]
    fn to_superset(&self) -> Fx {
        *self
    }

    #[inline]
    fn from_superset(element: &Fx) -> Option<Self> {
        Some(*element)
    }

    #[inline]
    fn from_superset_unchecked(element: &Fx) -> Self {
        *element
    }

    #[inline]
    fn is_in_subset(_: &Fx) -> bool {
        true
    }
}

impl AbsDiffEq for Fx {
    type Epsilon = Self;
    fn default_epsilon() -> Self::Epsilon {
        Self(I32F32::from_bits(0x16))
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        // This is the impl used in the approx crate.
        if self > other {
            (*self - *other) <= epsilon
        } else {
            (*other - *self) <= epsilon
        }
    }
}

impl RelativeEq for Fx {
    fn default_max_relative() -> Self::Epsilon {
        Self::default_epsilon()
    }

    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        // This is the impl used in the approx crate.
        let abs_diff = (*self - *other).abs();

        if abs_diff <= epsilon {
            return true;
        }

        let abs_self = self.abs();
        let abs_other = other.abs();

        let largest = if abs_other > abs_self {
            abs_other
        } else {
            abs_self
        };

        abs_diff <= largest * max_relative
    }
}

impl UlpsEq for Fx {
    fn default_max_ulps() -> u32 {
        4
    }

    fn ulps_eq(&self, other: &Self, epsilon: Self::Epsilon, max_ulps: u32) -> bool {
        if self.abs_diff_eq(other, epsilon) {
            return true;
        }

        if self.signum() != other.signum() {
            return false;
        }

        let bits1 = self.0.to_bits();
        let bits2 = other.0.to_bits();

        if bits1 > bits2 {
            (bits1 - bits2) <= max_ulps as i64
        } else {
            (bits2 - bits1) <= max_ulps as i64
        }
    }
}

impl Bounded for Fx {
    #[inline]
    fn min_value() -> Self {
        Self(I32F32::MIN)
    }

    #[inline]
    fn max_value() -> Self {
        Self(I32F32::MAX)
    }
}

impl FromPrimitive for Fx {
    fn from_i64(n: i64) -> Option<Self> {
        n.checked_to_fixed().map(Self)
    }
    fn from_u64(n: u64) -> Option<Self> {
        n.checked_to_fixed().map(Self)
    }
    fn from_isize(n: isize) -> Option<Self> {
        n.checked_to_fixed().map(Self)
    }
    fn from_i8(n: i8) -> Option<Self> {
        n.checked_to_fixed().map(Self)
    }
    fn from_i16(n: i16) -> Option<Self> {
        n.checked_to_fixed().map(Self)
    }
    fn from_i32(n: i32) -> Option<Self> {
        n.checked_to_fixed().map(Self)
    }
    fn from_usize(n: usize) -> Option<Self> {
        n.checked_to_fixed().map(Self)
    }
    fn from_u8(n: u8) -> Option<Self> {
        n.checked_to_fixed().map(Self)
    }
    fn from_u16(n: u16) -> Option<Self> {
        n.checked_to_fixed().map(Self)
    }
    fn from_u32(n: u32) -> Option<Self> {
        n.checked_to_fixed().map(Self)
    }
    fn from_f32(n: f32) -> Option<Self> {
        n.checked_to_fixed().map(Self)
    }
    fn from_f64(n: f64) -> Option<Self> {
        n.checked_to_fixed().map(Self)
    }
}

impl Signed for Fx {
    fn abs(&self) -> Self {
        Self(self.0.abs())
    }

    fn abs_sub(&self, other: &Self) -> Self {
        self.abs() - *other
    }

    fn signum(&self) -> Self {
        Self(self.0.signum())
    }

    fn is_positive(&self) -> bool {
        self.0 >= Self::zero().0
    }

    fn is_negative(&self) -> bool {
        self.0 <= Self::zero().0
    }
}

impl ComplexField for Fx {
    type RealField = Self;

    #[inline]
    fn from_real(re: Self::RealField) -> Self {
        re
    }

    #[inline]
    fn real(self) -> Self::RealField {
        self
    }

    #[inline]
    fn imaginary(self) -> Self::RealField {
        Self::zero()
    }

    #[inline]
    fn norm1(self) -> Self::RealField {
        self.abs()
    }

    #[inline]
    fn modulus(self) -> Self::RealField {
        self.abs()
    }

    #[inline]
    fn modulus_squared(self) -> Self::RealField {
        self * self
    }

    #[inline]
    fn argument(self) -> Self::RealField {
        if self >= Self::zero() {
            Self::zero()
        } else {
            Self::pi()
        }
    }

    #[inline]
    fn to_exp(self) -> (Self, Self) {
        if self >= Self::zero() {
            (self, Self::one())
        } else {
            (-self, -Self::one())
        }
    }

    #[inline]
    fn recip(self) -> Self {
        Self::one() / self
    }

    #[inline]
    fn conjugate(self) -> Self {
        self
    }

    #[inline]
    fn scale(self, factor: Self::RealField) -> Self {
        self * factor
    }

    #[inline]
    fn unscale(self, factor: Self::RealField) -> Self {
        self / factor
    }

    #[inline]
    fn floor(self) -> Self {
        Self(self.0.floor())
    }

    #[inline]
    fn ceil(self) -> Self {
        Self(self.0.ceil())
    }

    #[inline]
    fn round(self) -> Self {
        Self(self.0.round())
    }

    #[inline]
    fn trunc(self) -> Self {
        Self(self.0.int())
    }

    #[inline]
    fn fract(self) -> Self {
        Self(self.0.frac())
    }

    #[inline]
    fn abs(self) -> Self {
        Self(self.0.abs())
    }

    #[inline]
    fn signum(self) -> Self {
        Self(self.0.signum())
    }

    #[inline]
    fn mul_add(self, a: Self, b: Self) -> Self {
        self * a + b
    }

    #[inline]
    fn powi(self, _n: i32) -> Self {
        unimplemented!()
    }

    #[inline]
    fn powf(self, _n: Self) -> Self {
        unimplemented!()
    }

    #[inline]
    fn powc(self, _n: Self) -> Self {
        unimplemented!()
    }

    #[inline]
    fn sqrt(self) -> Self {
        Self(cordic::sqrt(self.0))
    }

    #[inline]
    fn try_sqrt(self) -> Option<Self> {
        if self >= Self::zero() {
            Some(self.sqrt())
        } else {
            None
        }
    }

    #[inline]
    fn exp(self) -> Self {
        Self(cordic::exp(self.0))
    }

    #[inline]
    fn exp2(self) -> Self {
        unimplemented!()
    }

    #[inline]
    fn exp_m1(self) -> Self {
        unimplemented!()
    }

    #[inline]
    fn ln_1p(self) -> Self {
        unimplemented!()
    }

    #[inline]
    fn ln(self) -> Self {
        unimplemented!()
    }

    #[inline]
    fn log(self, _base: Self) -> Self {
        unimplemented!()
    }

    #[inline]
    fn log2(self) -> Self {
        unimplemented!()
    }

    #[inline]
    fn log10(self) -> Self {
        unimplemented!()
    }

    #[inline]
    fn cbrt(self) -> Self {
        unimplemented!()
    }

    #[inline]
    fn hypot(self, _other: Self) -> Self::RealField {
        unimplemented!()
    }

    #[inline]
    fn sin(self) -> Self {
        Self(cordic::sin(self.0))
    }

    #[inline]
    fn cos(self) -> Self {
        Self(cordic::cos(self.0))
    }

    #[inline]
    fn tan(self) -> Self {
        Self(cordic::tan(self.0))
    }

    #[inline]
    fn asin(self) -> Self {
        Self(cordic::asin(self.0))
    }

    #[inline]
    fn acos(self) -> Self {
        Self(cordic::acos(self.0))
    }

    #[inline]
    fn atan(self) -> Self {
        Self(cordic::atan(self.0))
    }

    #[inline]
    fn sin_cos(self) -> (Self, Self) {
        let (sin, cos) = cordic::sin_cos(self.0);
        (Self(sin), Self(cos))
    }

    #[inline]
    fn sinh(self) -> Self {
        unimplemented!()
    }

    #[inline]
    fn cosh(self) -> Self {
        unimplemented!()
    }

    #[inline]
    fn tanh(self) -> Self {
        unimplemented!()
    }

    #[inline]
    fn asinh(self) -> Self {
        unimplemented!()
    }

    #[inline]
    fn acosh(self) -> Self {
        unimplemented!()
    }

    #[inline]
    fn atanh(self) -> Self {
        unimplemented!()
    }

    #[inline]
    fn is_finite(&self) -> bool {
        true
    }
}

impl RealField for Fx {
    #[inline]
    fn is_sign_positive(self) -> bool {
        self.0.is_positive()
    }

    #[inline]
    fn is_sign_negative(self) -> bool {
        self.0.is_negative()
    }

    #[inline]
    fn copysign(self, sign: Self) -> Self {
        if sign >= Self::zero() {
            self.abs()
        } else {
            -self.abs()
        }
    }

    #[inline]
    fn max(self, other: Self) -> Self {
        if self >= other {
            self
        } else {
            other
        }
    }

    #[inline]
    fn min(self, other: Self) -> Self {
        if self < other {
            self
        } else {
            other
        }
    }

    #[inline]
    fn clamp(self, min: Self, max: Self) -> Self {
        if self < min {
            min
        } else if self > max {
            max
        } else {
            self
        }
    }

    #[inline]
    fn atan2(self, other: Self) -> Self {
        Self(cordic::atan2(self.0, other.0))
    }

    /// Archimedes' constant.
    #[inline]
    fn pi() -> Self {
        Self(I32F32::PI)
    }

    /// 2.0 * pi.
    #[inline]
    fn two_pi() -> Self {
        Self::pi() + Self::pi()
    }

    /// pi / 2.0.
    #[inline]
    fn frac_pi_2() -> Self {
        Self(I32F32::FRAC_PI_2)
    }

    /// pi / 3.0.
    #[inline]
    fn frac_pi_3() -> Self {
        Self(I32F32::FRAC_PI_3)
    }

    /// pi / 4.0.
    #[inline]
    fn frac_pi_4() -> Self {
        Self(I32F32::FRAC_PI_4)
    }

    /// pi / 6.0.
    #[inline]
    fn frac_pi_6() -> Self {
        Self(I32F32::FRAC_PI_6)
    }

    /// pi / 8.0.
    #[inline]
    fn frac_pi_8() -> Self {
        Self(I32F32::FRAC_PI_8)
    }

    /// 1.0 / pi.
    #[inline]
    fn frac_1_pi() -> Self {
        Self(I32F32::FRAC_1_PI)
    }

    /// 2.0 / pi.
    #[inline]
    fn frac_2_pi() -> Self {
        Self(I32F32::FRAC_2_PI)
    }

    /// 2.0 / sqrt(pi).
    #[inline]
    fn frac_2_sqrt_pi() -> Self {
        Self(I32F32::FRAC_2_SQRT_PI)
    }

    /// Euler's number.
    #[inline]
    fn e() -> Self {
        Self(I32F32::E)
    }

    /// log2(e).
    #[inline]
    fn log2_e() -> Self {
        Self(I32F32::LOG2_E)
    }

    /// log10(e).
    #[inline]
    fn log10_e() -> Self {
        Self(I32F32::LOG10_E)
    }

    /// ln(2.0).
    #[inline]
    fn ln_2() -> Self {
        Self(I32F32::LN_2)
    }

    /// ln(10.0).
    #[inline]
    fn ln_10() -> Self {
        Self(I32F32::LN_10)
    }
}

//
// critical-point extension
//

impl Fx {
    #[inline]
    pub fn to_i8(&self) -> i8 {
        return self.0.to_num::<i8>();
    }

    #[inline]
    pub fn to_u8(&self) -> i8 {
        return self.0.to_num::<i8>();
    }

    #[inline]
    pub fn to_i16(&self) -> i16 {
        return self.0.to_num::<i16>();
    }

    #[inline]
    pub fn to_u16(&self) -> i16 {
        return self.0.to_num::<i16>();
    }

    #[inline]
    pub fn to_i32(&self) -> i32 {
        return self.0.to_num::<i32>();
    }

    #[inline]
    pub fn to_u32(&self) -> u32 {
        return self.0.to_num::<u32>();
    }

    #[inline]
    pub fn to_i64(&self) -> i64 {
        return self.0.to_num::<i64>();
    }

    #[inline]
    pub fn to_u64(&self) -> u64 {
        return self.0.to_num::<u64>();
    }

    #[inline]
    pub fn to_isize(&self) -> isize {
        return self.0.to_num::<isize>();
    }

    #[inline]
    pub fn to_usize(&self) -> usize {
        return self.0.to_num::<usize>();
    }

    #[inline]
    pub fn to_f32(&self) -> f32 {
        return self.0.to_num::<f32>();
    }

    #[inline]
    pub fn to_f64(&self) -> f64 {
        return self.0.to_num::<f64>();
    }
}
