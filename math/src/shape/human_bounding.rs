use crate::auto_gen::RealExt;
use crate::vector::{p3_to_v3, v3_to_p3};
use derivative::Derivative;
use na::{Isometry3, Point3, RealField, Unit, Vector3};
use ncollide3d::bounding_volume::{self, BoundingSphere, HasBoundingVolume, AABB};
use ncollide3d::query::{
    self, algorithms::VoronoiSimplex, PointProjection, PointQuery, Ray, RayCast, RayIntersection,
};
use ncollide3d::shape::{FeatureId, Shape, SupportMap};

#[derive(Clone, Debug, Derivative)]
#[derivative(PartialEq)]
pub struct HumanBounding<N> {
    capsule_radius: N,
    capsule_height: N,
    bottom_radius: N,
}

impl<N: RealField + RealExt> HumanBounding<N> {
    pub fn new(capsule_radius: N, capsule_height: N, bottom_radius: N) -> HumanBounding<N> {
        return HumanBounding {
            capsule_radius,
            capsule_height,
            bottom_radius,
        };
    }

    #[inline(always)]
    pub fn capsule_radius(&self) -> N {
        return self.capsule_radius;
    }

    #[inline(always)]
    pub fn capsule_height(&self) -> N {
        return self.capsule_height;
    }

    #[inline(always)]
    pub fn bottom_radius(&self) -> N {
        return self.bottom_radius;
    }

    #[inline(always)]
    fn cone_height(&self) -> N {
        return N::sqrt3() * (self.capsule_radius - self.bottom_radius);
    }

    #[inline(always)]
    pub fn bottom_height(&self) -> N {
        return N::frac2() * self.bottom_radius;
    }

    #[inline(always)]
    fn bottom_center(&self) -> N {
        return self.cone_height() - N::frac2() * self.bottom_radius;
    }
}

impl<N: RealField + RealExt> SupportMap<N> for HumanBounding<N> {
    fn local_support_point(&self, dir: &Vector3<N>) -> Point3<N> {
        if dir[1].is_positive() {
            // half capsule
            let mut pt_local = dir * self.capsule_radius();
            pt_local[1] += self.capsule_height();
            return Point3::from(pt_local);
        } else {
            if dir[1] > -N::frac2() {
                // cone
                let dir_radius = Vector3::new(dir[0], N::c0(), dir[2]);
                let pt_local = v3_to_p3(dir_radius.normalize() * self.capsule_radius());
                return Point3::from(pt_local);
            } else {
                // bottom sphere
                let mut pt_local = dir * self.bottom_radius();
                pt_local[1] -= self.bottom_center();
                return Point3::from(pt_local);
            }
        }
    }
}

impl<N: RealField + RealExt> HasBoundingVolume<N, AABB<N>> for HumanBounding<N> {
    #[inline]
    fn bounding_volume(&self, transform: &Isometry3<N>) -> AABB<N> {
        let bounding: AABB<N> = self.local_bounding_volume();
        return bounding.transform_by(transform);
    }

    #[inline]
    fn local_bounding_volume(&self) -> AABB<N> {
        let min = Point3::new(
            -self.capsule_radius(),
            -(self.cone_height() + self.bottom_height()),
            -self.capsule_radius(),
        );
        let max = Point3::new(
            self.capsule_radius(),
            self.capsule_height() + self.capsule_radius(),
            self.capsule_radius(),
        );
        return AABB::new(min, max);
    }
}

impl<N: RealField + RealExt> HasBoundingVolume<N, BoundingSphere<N>> for HumanBounding<N> {
    #[inline]
    fn bounding_volume(&self, transform: &Isometry3<N>) -> BoundingSphere<N> {
        let bounding: BoundingSphere<N> = self.local_bounding_volume();
        return bounding.transform_by(transform);
    }

    #[inline]
    fn local_bounding_volume(&self) -> BoundingSphere<N> {
        let pos_height = self.capsule_height() + self.capsule_radius();
        let neg_height = self.cone_height() + self.bottom_height();
        let center = pos_height - neg_height;
        let radius = N::frac2() * (pos_height + neg_height);
        return BoundingSphere::new(Point3::new(N::c0(), center, N::c0()), radius);
    }
}

impl<N: RealField + RealExt> RayCast<N> for HumanBounding<N> {
    fn toi_and_normal_with_ray(
        &self,
        transform: &Isometry3<N>,
        ray: &Ray<N>,
        max_toi: N,
        solid: bool,
    ) -> Option<RayIntersection<N>> {
        let ray_local = ray.inverse_transform_by(transform);
        return query::ray_intersection_with_support_map_with_params(
            &Isometry3::identity(),
            self,
            &mut VoronoiSimplex::new(),
            &ray_local,
            max_toi,
            solid,
        )
        .map(|mut res| {
            res.normal = transform * res.normal;
            return res;
        });
    }
}

impl<N: RealField + RealExt> PointQuery<N> for HumanBounding<N> {
    #[inline]
    fn project_point(
        &self,
        transform: &Isometry3<N>,
        pt: &Point3<N>,
        solid: bool,
    ) -> PointProjection<N> {
        let pt_local = transform.inverse_transform_point(pt);

        if pt_local[1] <= N::c0()
            || is_point_under_cone(N::tan15() * self.capsule_radius(), N::tan15(), pt_local)
        {
            // cone
            if is_point_under_cone(-self.bottom_center(), N::tan30(), pt_local) {
                // bottom sphere
                let vec_pc =
                    Vector3::new(pt_local[0], pt_local[1] + self.bottom_center(), pt_local[2]);
                let dist_squared = vec_pc.norm_squared();
                let inside = dist_squared < self.bottom_radius() * self.bottom_radius();
                if solid && inside {
                    return PointProjection::new(true, *pt);
                } else {
                    let vec_radius = vec_pc * (self.bottom_radius() / dist_squared.sqrt());
                    let pt_proj = transform
                        * Point3::new(
                            vec_radius[0],
                            vec_radius[1] - self.bottom_center(),
                            vec_radius[2],
                        );
                    return PointProjection::new(inside, pt_proj);
                }
            } else {
                // cone
                let vec_xz = Vector3::new(pt_local[0], N::c0(), pt_local[2]);
                if let Some((vec_xz, _)) = Unit::try_new_and_get(vec_xz, N::default_epsilon()) {
                    let pt_a = v3_to_p3(vec_xz.as_ref() * self.capsule_radius());
                    let pt_b = Point3::new(
                        vec_xz[0] * self.bottom_radius(),
                        -self.cone_height(),
                        vec_xz[2] * self.bottom_radius(),
                    );
                    let (pt_local_proj, proj_type) = segment_project_point(pt_a, pt_b, pt_local);

                    if proj_type == SegmentProjectType::A {
                        return PointProjection::new(false, transform * pt_a);
                    } else {
                        let pt_proj = transform * pt_local_proj;
                        let inside =
                            p3_to_v3(pt_proj).norm_squared() > (pt - pt_proj).norm_squared();
                        if solid && inside {
                            return PointProjection::new(true, *pt);
                        } else {
                            return PointProjection::new(inside, pt_proj);
                        }
                    }
                } else {
                    let height = self.cone_height() + self.bottom_height();
                    let inside = p3_to_v3(pt_local).norm_squared() <= height * height;
                    if solid && inside {
                        return PointProjection::new(true, *pt);
                    } else {
                        let pt_proj = transform * Point3::new(na::zero(), -height, na::zero());
                        return PointProjection::new(inside, pt_proj);
                    }
                }
            }
        } else {
            // capsule
            let (pt_local_proj, _) = segment_project_point(
                Point3::new(N::c0(), self.capsule_height(), N::c0()),
                Point3::origin(),
                pt_local,
            );
            let pt_proj = transform * pt_local_proj;
            let vec_proj = pt - pt_proj;

            if let Some((dir, dist)) = Unit::try_new_and_get(vec_proj, N::default_epsilon()) {
                let inside = dist <= self.capsule_radius();
                if solid && inside {
                    return PointProjection::new(true, *pt);
                } else {
                    let pt_local = pt_proj + dir.as_ref() * self.capsule_radius();
                    return PointProjection::new(inside, pt_local);
                }
            } else {
                if solid {
                    return PointProjection::new(true, *pt);
                } else {
                    let dir = transform * Vector3::new(N::c0(), N::c1(), N::c0());
                    return PointProjection::new(true, pt_proj + dir * self.capsule_radius());
                }
            }
        }
    }

    #[inline]
    fn project_point_with_feature(
        &self,
        transform: &Isometry3<N>,
        pt: &Point3<N>,
    ) -> (PointProjection<N>, FeatureId) {
        return (self.project_point(transform, pt, false), FeatureId::Face(0));
    }
}

// (y+yy)^2 = tan^2 * (x^2 + z^2)
fn is_point_under_cone<N: RealField + RealExt>(yy: N, tan: N, pt: Point3<N>) -> bool {
    if pt[1] > yy {
        return false;
    }
    let left = (pt[1] + yy).modulus_squared();
    let right = tan.modulus_squared() * (pt[0].modulus_squared() + pt[2].modulus_squared());
    return left >= right;
}

#[derive(Debug, Eq, PartialEq)]
enum SegmentProjectType {
    A,
    B,
    AB,
}

fn segment_project_point<N: RealField + RealExt>(
    pt_a: Point3<N>,
    pt_b: Point3<N>,
    pt_local: Point3<N>,
) -> (Point3<N>, SegmentProjectType) {
    let vec_ab = pt_b - pt_a;
    let vec_ap = pt_local - pt_a;
    let dot_ab_ap = vec_ab.dot(&vec_ap);
    let norm_sq_ab = vec_ab.norm_squared();
    if dot_ab_ap <= N::c0() {
        return (pt_a, SegmentProjectType::A);
    } else if dot_ab_ap >= norm_sq_ab {
        return (pt_b, SegmentProjectType::B);
    } else {
        return (
            pt_a + vec_ab * (dot_ab_ap / norm_sq_ab),
            SegmentProjectType::AB,
        );
    }
}

impl<N: RealField + RealExt> Shape<N> for HumanBounding<N> {
    #[inline]
    fn aabb(&self, transform: &Isometry3<N>) -> AABB<N> {
        return bounding_volume::aabb(self, transform);
    }

    #[inline]
    fn local_aabb(&self) -> AABB<N> {
        return bounding_volume::local_aabb(self);
    }

    #[inline]
    fn bounding_sphere(&self, transform: &Isometry3<N>) -> BoundingSphere<N> {
        return bounding_volume::bounding_sphere(self, transform);
    }

    #[inline]
    fn as_ray_cast(&self) -> Option<&dyn RayCast<N>> {
        return Some(self);
    }

    #[inline]
    fn as_point_query(&self) -> Option<&dyn PointQuery<N>> {
        return Some(self);
    }

    #[inline]
    fn as_support_map(&self) -> Option<&dyn SupportMap<N>> {
        return Some(self);
    }

    #[inline]
    fn is_support_map(&self) -> bool {
        return true;
    }

    fn tangent_cone_contains_dir(
        &self,
        _: FeatureId,
        _: &Isometry3<N>,
        _: Option<&[N]>,
        _: &Unit<Vector3<N>>,
    ) -> bool {
        return false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fx::{ff, fi, Fx};
    use approx::assert_relative_eq;
    use na::{Translation3, UnitQuaternion};

    #[test]
    fn test_human_bounding_general() {
        let hb = HumanBounding::new(ff(0.4), ff(0.7), ff(0.05));
        assert_relative_eq!(hb.capsule_radius(), ff(0.4));
        assert_relative_eq!(hb.capsule_height(), ff(0.7));
        assert_relative_eq!(hb.bottom_radius(), ff(0.05));
        assert_relative_eq!(hb.cone_height(), ff(0.35) * Fx::sqrt3());
        assert_relative_eq!(hb.bottom_height(), ff(0.025));
    }

    #[test]
    fn test_human_bounding_support_map() {
        let hb = HumanBounding::new(ff(0.4), ff(0.7), ff(0.05));
        let transform = Isometry3::from_parts(
            Translation3::from(Vector3::new(fi(1), fi(2), fi(3))),
            UnitQuaternion::identity(),
        );

        // -90°
        let pt_real = hb.support_point(&transform, &Vector3::new(fi(0), fi(-10), fi(0)));
        let pt_local = Point3::new(fi(0), -(hb.cone_height() + hb.bottom_height()), fi(0));
        assert_relative_eq!(pt_real, transform * pt_local);

        // -45°
        let pt_real = hb.support_point(&transform, &Vector3::new(fi(10), fi(-10), fi(0)));
        let pt_local = Point3::new(
            Fx::sqrt2() * hb.bottom_height(),
            -(hb.cone_height() + (Fx::sqrt2() - fi(1)) * hb.bottom_height()),
            fi(0),
        );
        assert_relative_eq!(pt_real, transform * pt_local);

        // -30°
        let pt_real = hb.support_point(&transform, &Vector3::new(fi(0), -fi(1), Fx::sqrt3()));
        let pt_local = Point3::new(fi(0), -hb.cone_height(), Fx::sqrt3() * hb.bottom_height());
        assert_relative_eq!(pt_real, transform * pt_local);

        // -30° ~ -0°
        let pt_real = hb.support_point(&transform, &Vector3::new(fi(0), -fi(1), -fi(3)));
        let pt_local = Point3::new(fi(0), fi(0), -hb.capsule_radius());
        assert_relative_eq!(pt_real, transform * pt_local);

        // 45°
        let pt_real = hb.support_point(&transform, &Vector3::new(-fi(4), fi(4), fi(0)));
        let pt_local = Point3::new(
            -Fx::sqrt2() / fi(2) * hb.capsule_radius(),
            Fx::sqrt2() / fi(2) * hb.capsule_radius() + hb.capsule_height(),
            fi(0),
        );
        assert_relative_eq!(pt_real, transform * pt_local);

        // 90°
        let pt_real = hb.support_point(&transform, &Vector3::new(fi(0), fi(4), fi(0)));
        let pt_local = Point3::new(fi(0), hb.capsule_radius() + hb.capsule_height(), fi(0));
        assert_relative_eq!(pt_real, transform * pt_local);
    }

    #[test]
    fn test_human_bounding_aabb() {
        let hb = HumanBounding::new(ff(0.4), ff(0.7), ff(0.05));
        let transform = Isometry3::from_parts(
            Translation3::from(Vector3::new(fi(1), fi(2), fi(3))),
            UnitQuaternion::identity(),
        );
        let aabb: AABB<Fx> = hb.bounding_volume(&transform);
        let excepted: AABB<Fx> = AABB::new(
            Point3::new(-ff(0.4), -(hb.cone_height() + hb.bottom_height()), -ff(0.4)),
            Point3::new(ff(0.4), hb.capsule_height() + hb.capsule_radius(), ff(0.4)),
        )
        .transform_by(&transform);
        assert_relative_eq!(aabb.mins, excepted.mins);
        assert_relative_eq!(aabb.maxs, excepted.maxs);
    }

    #[test]
    fn test_human_bounding_sphere() {
        let hb = HumanBounding::new(ff(0.4), ff(0.7), ff(0.05));
        let transform = Isometry3::from_parts(Translation3::identity(), UnitQuaternion::identity());
        let sphere: BoundingSphere<Fx> = hb.bounding_sphere(&transform);
        let excepted: BoundingSphere<Fx> = BoundingSphere::new(
            Point3::new(fi(0), ff(0.4 + 0.7 - f64::sqrt3() * 0.35 - 0.025), fi(0)),
            ff((0.4 + 0.7 + f64::sqrt3() * 0.35 + 0.025) / 2.0),
        );
        assert_relative_eq!(sphere.center(), excepted.center());
        assert_relative_eq!(sphere.radius(), excepted.radius());
    }

    #[test]
    fn test_human_bounding_project_point() {
        let hb = HumanBounding::new(fi(1), ff(0.5), ff(0.1));
        let transform = Isometry3::from_parts(
            Translation3::from(Vector3::new(fi(-2), fi(-2), fi(-2))),
            UnitQuaternion::identity(),
        );

        // bottom & y axis
        let pt_in = Point3::new(fi(0), ff(-1.3), fi(0));
        let proj = hb.project_point(&transform, &(transform * pt_in), true);
        assert_eq!(proj.is_inside, true);
        assert_relative_eq!(proj.point, transform * Point3::new(fi(0), ff(-1.3), fi(0)));

        let proj = hb.project_point(&transform, &(transform * pt_in), false);
        assert_eq!(proj.is_inside, true);
        assert_relative_eq!(
            proj.point,
            transform * Point3::new(fi(0), -hb.cone_height() - hb.bottom_height(), fi(0))
        );

        let pt_in = Point3::new(fi(0), fi(-10), fi(0));
        let proj = hb.project_point(&transform, &(transform * pt_in), true);
        assert_eq!(proj.is_inside, false);
        assert_relative_eq!(
            proj.point,
            transform * Point3::new(fi(0), -hb.cone_height() - hb.bottom_height(), fi(0))
        );

        // bottom sphere
        let pt_in = Point3::new(fi(0), -hb.bottom_center() - ff(0.05), ff(0.05));
        let proj = hb.project_point(&transform, &(transform * pt_in), true);
        assert_eq!(proj.is_inside, true);
        assert_relative_eq!(
            proj.point,
            transform * Point3::new(fi(0), -hb.bottom_center() - ff(0.05), ff(0.05),)
        );

        let proj = hb.project_point(&transform, &(transform * pt_in), false);
        assert_eq!(proj.is_inside, true);
        let pt = Point3::new(
            fi(0),
            -hb.bottom_center() - ff(0.05) * Fx::sqrt2(),
            ff(0.05) * Fx::sqrt2(),
        );
        assert_relative_eq!(proj.point, transform * pt);

        let pt_in = Point3::new(fi(0), -hb.bottom_center() - fi(1), fi(1));
        let proj = hb.project_point(&transform, &(transform * pt_in), false);
        assert_eq!(proj.is_inside, false);
        let pt = Point3::new(
            fi(0),
            -hb.bottom_center() - ff(0.05) * Fx::sqrt2(),
            ff(0.05) * Fx::sqrt2(),
        );
        assert_relative_eq!(proj.point, transform * pt);

        // bottom cone
        let pt_in = Point3::new(fi(0), -ff(0.3) * Fx::sqrt3(), -ff(0.3));
        let proj = hb.project_point(&transform, &(transform * pt_in), true);
        assert_eq!(proj.is_inside, true);
        assert_relative_eq!(
            proj.point,
            transform * Point3::new(fi(0), -ff(0.3) * Fx::sqrt3(), -ff(0.3))
        );

        let proj = hb.project_point(&transform, &(transform * pt_in), false);
        assert_eq!(proj.is_inside, true);
        assert_relative_eq!(
            proj.point,
            transform * Point3::new(fi(0), -ff(0.4) * Fx::sqrt3(), -ff(0.6))
        );

        let pt_in = Point3::new(fi(0), -ff(0.5) * Fx::sqrt3(), -ff(0.9));
        let proj = hb.project_point(&transform, &(transform * pt_in), false);
        assert_eq!(proj.is_inside, true);
        assert_relative_eq!(
            proj.point,
            transform * Point3::new(fi(0), -ff(0.4) * Fx::sqrt3(), -ff(0.6))
        );

        // -30° - 0°
        let pt_in = Point3::new(
            Fx::sqrt3() + hb.capsule_radius(),
            -Fx::sqrt2() + ff(0.1),
            Fx::sqrt3() + hb.capsule_radius(),
        );
        let proj = hb.project_point(&transform, &(transform * pt_in), true);
        assert_eq!(proj.is_inside, false);
        assert_relative_eq!(
            proj.point,
            transform * Point3::new(Fx::sqrt2() * Fx::frac2(), fi(0), Fx::sqrt2() * Fx::frac2()),
        );

        // xz plane
        let pt_in = Point3::new(Fx::frac2(), fi(0), Fx::frac2());
        let proj = hb.project_point(&transform, &(transform * pt_in), true);
        assert_eq!(proj.is_inside, true);
        assert_relative_eq!(
            proj.point,
            transform * Point3::new(Fx::frac2(), fi(0), Fx::frac2()),
        );

        let pt_in = Point3::new(ff(2.0), fi(0), ff(2.0));
        let proj = hb.project_point(&transform, &(transform * pt_in), false);
        assert_eq!(proj.is_inside, false);
        assert_relative_eq!(
            proj.point,
            transform * Point3::new(Fx::sqrt2() * Fx::frac2(), fi(0), Fx::sqrt2() * Fx::frac2()),
        );

        // cone in capsule
        let pt_in = Point3::new(ff(0.1), ff(0.1), ff(0.1));
        let proj = hb.project_point(&transform, &(transform * pt_in), false);
        assert_eq!(proj.is_inside, true);
        assert!(proj.point[1] < -fi(2));

        // capsule body
        let pt_in = Point3::new(ff(0.5), ff(0.3), ff(0.5));
        let proj = hb.project_point(&transform, &(transform * pt_in), true);
        assert_eq!(proj.is_inside, true);
        assert_relative_eq!(
            proj.point,
            transform * Point3::new(ff(0.5), ff(0.3), ff(0.5))
        );

        let proj = hb.project_point(&transform, &(transform * pt_in), false);
        assert_eq!(proj.is_inside, true);
        assert_relative_eq!(
            proj.point,
            transform
                * Point3::new(
                    Fx::sqrt2() * Fx::frac2(),
                    ff(0.3),
                    Fx::sqrt2() * Fx::frac2()
                ),
        );

        // capsule top
        let pt_in = Point3::new(ff(0.0), ff(1.2), ff(0.0));
        let proj = hb.project_point(&transform, &(transform * pt_in), true);
        assert_eq!(proj.is_inside, true);
        assert_relative_eq!(
            proj.point,
            transform * Point3::new(ff(0.0), ff(1.2), ff(0.0))
        );

        let proj = hb.project_point(&transform, &(transform * pt_in), false);
        assert_eq!(proj.is_inside, true);
        assert_relative_eq!(
            proj.point,
            transform * Point3::new(ff(0.0), ff(1.5), ff(0.0))
        );
    }
}
