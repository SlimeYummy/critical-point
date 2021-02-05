use super::{approx_zero, fi, Fx, RealExt};
use na::{ComplexField, Unit, Vector2, Vector3};
use std::convert::From;

#[derive(Debug, Clone, Copy)]
pub struct Velocity2 {
    pub speed: Fx,
    pub direction: Unit<Vector2<Fx>>,
}

impl From<Vector2<Fx>> for Velocity2 {
    #[inline]
    fn from(vec: Vector2<Fx>) -> Velocity2 {
        let square = vec.norm_squared();
        if approx_zero(square) {
            return Velocity2::new(Fx::c0(), -Vector2::y_axis());
        }
        let speed = square.sqrt();
        let normalize = vec.scale(Fx::c1() / speed);
        let direction = *Unit::from_ref_unchecked(&normalize);
        return Velocity2::new(speed, direction);
    }
}

impl Velocity2 {
    #[inline(always)]
    pub fn new(speed: Fx, direction: Unit<Vector2<Fx>>) -> Velocity2 {
        return Velocity2 { speed, direction };
    }

    #[inline(always)]
    pub fn to_vec(&self) -> Vector2<Fx> {
        return self.direction.scale(self.speed);
    }

    #[inline(always)]
    pub fn combine_vec(vels: &[Velocity2]) -> Vector2<Fx> {
        let vec: Vector2<Fx> = vels.iter().map(|v| v.direction.scale(v.speed)).sum();
        return vec / fi(vels.len() as i64);
    }

    #[inline(always)]
    pub fn combine(vels: &[Velocity2]) -> Velocity2 {
        let vec = Velocity2::combine_vec(vels);
        return Velocity2::from(vec);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Velocity3 {
    pub speed: Fx,
    pub direction: Unit<Vector3<Fx>>,
}

impl From<Vector3<Fx>> for Velocity3 {
    #[inline]
    fn from(vec: Vector3<Fx>) -> Velocity3 {
        let square = vec.norm_squared();
        if approx_zero(square) {
            return Velocity3::new(Fx::c0(), -Vector3::z_axis());
        }
        let speed = square.sqrt();
        let normalize = vec.scale(Fx::c1() / speed);
        let direction = *Unit::from_ref_unchecked(&normalize);
        return Velocity3::new(speed, direction);
    }
}

impl Velocity3 {
    #[inline(always)]
    pub fn new(speed: Fx, direction: Unit<Vector3<Fx>>) -> Velocity3 {
        return Velocity3 { speed, direction };
    }

    #[inline(always)]
    pub fn to_vec(&self) -> Vector3<Fx> {
        return self.direction.scale(self.speed);
    }

    #[inline(always)]
    pub fn combine_vec(vels: &[Velocity3]) -> Vector3<Fx> {
        let vec: Vector3<Fx> = vels.iter().map(|v| v.direction.scale(v.speed)).sum();
        return vec / fi(vels.len() as i64);
    }

    #[inline(always)]
    pub fn combine(vels: &[Velocity3]) -> Velocity3 {
        let vec = Velocity3::combine_vec(vels);
        return Velocity3::from(vec);
    }
}
