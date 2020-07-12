use super::num::*;
use euclid::num::Zero;
use na::{self, Point2, Point3, RealField, Vector2, Vector3};

pub fn p2_to_v2<F: RealField>(p: Point2<F>) -> Vector2<F> {
    return Vector2::new(p.x, p.y);
}

pub fn v2_to_p2<F: RealField>(p: Vector2<F>) -> Point2<F> {
    return Point2::new(p.x, p.y);
}

pub fn p3_to_v3<F: RealField>(p: Point3<F>) -> Vector3<F> {
    return Vector3::new(p.x, p.y, p.z);
}

pub fn v3_to_p3<F: RealField>(p: Vector3<F>) -> Point3<F> {
    return Point3::new(p.x, p.y, p.z);
}

// normal => (a, b, c)
// plane => ax + by + cz = 0
// y = - (ax + cz) / b
pub fn direction_on_plane(normal: &Vector3<Fx>, direction: &Vector2<Fx>) -> Vector3<Fx> {
    let a: Fx = normal.x;
    let b: Fx = normal.y;
    let c: Fx = normal.z;
    if b == Fx::zero() {
        return Vector3::new(Fx::zero(), fx(-1), Fx::zero());
    }
    let x = direction.x;
    let z = direction.y;
    let y = fx(-1) * (a * x + c * z) / b;
    return Vector3::new(x, y, z).normalize();
}
