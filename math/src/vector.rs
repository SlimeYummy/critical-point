use na::{self, Isometry2, Isometry3, Point2, Point3, RealField, Vector2, Vector3};

#[inline]
pub fn p2_to_v2<F: RealField>(p: Point2<F>) -> Vector2<F> {
    return Vector2::new(p.x, p.y);
}

#[inline]
pub fn v2_to_p2<F: RealField>(p: Vector2<F>) -> Point2<F> {
    return Point2::new(p.x, p.y);
}

#[inline]
pub fn p3_to_v3<F: RealField>(p: Point3<F>) -> Vector3<F> {
    return Vector3::new(p.x, p.y, p.z);
}

#[inline]
pub fn v3_to_p3<F: RealField>(p: Vector3<F>) -> Point3<F> {
    return Point3::new(p.x, p.y, p.z);
}

#[inline]
pub fn is2_to_p2<F: RealField>(is: Isometry2<F>) -> Point2<F> {
    let v = is.translation.vector;
    return Point2::new(v.x, v.y);
}

#[inline]
pub fn is3_to_p3<F: RealField>(is: Isometry3<F>) -> Point3<F> {
    let v = is.translation.vector;
    return Point3::new(v.x, v.y, v.z);
}
