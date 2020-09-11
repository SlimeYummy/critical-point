use super::{fx, v3_to_p3, Fx, approx_zero, approx_lt};
use na::{self, Point3, RealField, Unit, Vector2, Vector3};
use num_traits::Zero;

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

#[derive(Clone, Debug)]
pub struct DirectionCone<F: RealField> {
    pub center: Point3<F>,
    pub radius2: F,
}

impl<F: RealField> Default for DirectionCone<F> {
    fn default() -> DirectionCone<F> {
        return DirectionCone {
            center: Point3::new(na::zero(), na::zero(), na::zero()),
            radius2: na::zero(),
        };
    }
}

// ±nv1 != ±nv2 != ±nv3
pub fn cone_from_unit_vec(
    nv1: Unit<Vector3<Fx>>,
    nv2: Unit<Vector3<Fx>>,
    nv3: Unit<Vector3<Fx>>,
) -> Option<DirectionCone<Fx>> {
    let v1 = nv2.xyz() - nv1.xyz();
    let v2 = nv3.xyz() - nv1.xyz();
    let n = Vector3::cross(&v1, &v2);
    if approx_zero(&n) {
        return None;
    }
    let t = (n[0] * nv1[0] + n[1] * nv1[1] + n[2] * nv1[2]) /
        (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]);
    let center = Vector3::new(n[0] * t, n[1] * t, n[2] * t);
    return Some(DirectionCone {
        center: v3_to_p3(center),
        radius2: (nv1.xyz() - center).norm_squared(),
    });
}

pub fn unit_vec_in_cone(c: &DirectionCone<Fx>, nv: Unit<Vector3<Fx>>) -> bool {
    let distance2 = (nv.xyz() - c.center.coords).norm_squared();
    return approx_lt(distance2, c.radius2);
}
