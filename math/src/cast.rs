#![allow(dead_code)]

use super::{fx, Fx};
use fixed::traits::ToFixed;
use na::geometry::{
    Isometry2, Isometry3, IsometryMatrix2, IsometryMatrix3, Point2, Point3, Quaternion, Rotation2,
    Rotation3, Similarity2, Similarity3, Translation2, Translation3, UnitComplex, UnitQuaternion,
};
use na::{Complex, Matrix2, Matrix3, Matrix4, RealField, Unit, Vector2, Vector3, Vector4};

#[inline]
pub fn fx_v2<N: ToFixed + RealField>(v: &Vector2<N>) -> Vector2<Fx> {
    return Vector2::new(fx(v.x), fx(v.y));
}

#[inline]
pub fn fx_v3<N: ToFixed + RealField>(v: &Vector3<N>) -> Vector3<Fx> {
    return Vector3::new(fx(v.x), fx(v.y), fx(v.z));
}

#[inline]
pub fn fx_v4<N: ToFixed + RealField>(v: &Vector4<N>) -> Vector4<Fx> {
    return Vector4::new(fx(v.x), fx(v.y), fx(v.z), fx(v.w));
}

#[inline]
pub fn fx_p2<N: ToFixed + RealField>(p: &Point2<N>) -> Point2<Fx> {
    return Point2::new(fx(p.x), fx(p.y));
}

#[inline]
pub fn fx_p3<N: ToFixed + RealField>(p: &Point3<N>) -> Point3<Fx> {
    return Point3::new(fx(p.x), fx(p.y), fx(p.z));
}

#[inline]
pub fn fx_m2<N: ToFixed + RealField>(m: &Matrix2<N>) -> Matrix2<Fx> {
    return Matrix2::from_iterator(m.iter().map(|n| fx(*n)));
}

#[inline]
pub fn fx_m3<N: ToFixed + RealField>(m: &Matrix3<N>) -> Matrix3<Fx> {
    return Matrix3::from_iterator(m.iter().map(|n| fx(*n)));
}

#[inline]
pub fn fx_m4<N: ToFixed + RealField>(m: &Matrix4<N>) -> Matrix4<Fx> {
    return Matrix4::from_iterator(m.iter().map(|n| fx(*n)));
}

#[inline]
pub fn fx_t2<N: ToFixed + RealField>(t: &Translation2<N>) -> Translation2<Fx> {
    return Translation2::new(fx(t.vector.x), fx(t.vector.y));
}

#[inline]
pub fn fx_t3<N: ToFixed + RealField>(t: &Translation3<N>) -> Translation3<Fx> {
    return Translation3::new(fx(t.vector.x), fx(t.vector.y), fx(t.vector.z));
}

#[inline]
pub fn fx_r2<N: ToFixed + RealField>(r: &Rotation2<N>) -> Rotation2<Fx> {
    return Rotation2::from_matrix(&fx_m2(r.matrix()));
}

#[inline]
pub fn fx_r3<N: ToFixed + RealField>(r: &Rotation3<N>) -> Rotation3<Fx> {
    return Rotation3::from_matrix(&fx_m3(r.matrix()));
}

#[inline]
pub fn fx_c<N: ToFixed + RealField>(c: &Complex<N>) -> Complex<Fx> {
    return Complex::new(fx(c.re), fx(c.re));
}

#[inline]
pub fn fx_uc<N: ToFixed + RealField>(uc: &UnitComplex<N>) -> UnitComplex<Fx> {
    return Unit::new_unchecked(Complex::new(fx(uc.re), fx(uc.re)));
}

#[inline]
pub fn fx_q<N: ToFixed + RealField>(q: &Quaternion<N>) -> Quaternion<Fx> {
    return Quaternion::from(fx_v4(&q.coords));
}

#[inline]
pub fn fx_uq<N: ToFixed + RealField>(uq: &UnitQuaternion<N>) -> UnitQuaternion<Fx> {
    return Unit::new_unchecked(Quaternion::from(fx_v4(&uq.coords)));
}

#[inline]
pub fn fx_is2<N: ToFixed + RealField>(is: &Isometry2<N>) -> Isometry2<Fx> {
    return Isometry2::from_parts(fx_t2(&is.translation), fx_uc(&is.rotation));
}

#[inline]
pub fn fx_is3<N: ToFixed + RealField>(is: &Isometry3<N>) -> Isometry3<Fx> {
    return Isometry3::from_parts(fx_t3(&is.translation), fx_uq(&is.rotation));
}

#[inline]
pub fn fx_ism2<N: ToFixed + RealField>(is: &IsometryMatrix2<N>) -> IsometryMatrix2<Fx> {
    return IsometryMatrix2::from_parts(fx_t2(&is.translation), fx_r2(&is.rotation));
}

#[inline]
pub fn fx_ism3<N: ToFixed + RealField>(is: &IsometryMatrix3<N>) -> IsometryMatrix3<Fx> {
    return IsometryMatrix3::from_parts(fx_t3(&is.translation), fx_r3(&is.rotation));
}

#[inline]
pub fn fx_sm2<N: ToFixed + RealField>(sm: &Similarity2<N>) -> Similarity2<Fx> {
    return Similarity2::from_isometry(fx_is2(&sm.isometry), fx(sm.scaling()));
}

#[inline]
pub fn fx_sm3<N: ToFixed + RealField>(sm: Similarity3<N>) -> Similarity3<Fx> {
    return Similarity3::from_isometry(fx_is3(&sm.isometry), fx(sm.scaling()));
}
