use crate::real::RealExt;
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
    #[derivative(PartialEq = "ignore")]
    bottom_height: N,
    #[derivative(PartialEq = "ignore")]
    cone_height: N,
}

impl<N: RealField + RealExt> HumanBounding<N> {
    pub fn new(capsule_radius: N, capsule_height: N, bottom_radius: N) -> HumanBounding<N> {
        let cone_height = N::sqrt3() * (capsule_radius - bottom_radius);
        let bottom_height = N::frac2() * bottom_radius;
        return HumanBounding {
            capsule_radius,
            capsule_height,
            bottom_radius,
            bottom_height,
            cone_height,
        };
    }

    #[inline]
    pub fn capsule_radius(&self) -> N {
        return self.capsule_radius;
    }

    #[inline]
    pub fn capsule_height(&self) -> N {
        return self.capsule_height;
    }

    #[inline]
    pub fn bottom_radius(&self) -> N {
        return self.bottom_radius;
    }

    #[inline]
    pub fn bottom_height(&self) -> N {
        return self.bottom_height;
    }

    #[inline]
    fn cone_height(&self) -> N {
        return self.cone_height;
    }
}

impl<N: RealField + RealExt> SupportMap<N> for HumanBounding<N> {
    fn support_point(&self, transform: &Isometry3<N>, dir: &Vector3<N>) -> Point3<N> {
        return self.support_point_toward(transform, &Unit::new_normalize(*dir));
    }

    fn support_point_toward(&self, transform: &Isometry3<N>, dir: &Unit<Vector3<N>>) -> Point3<N> {
        let local_dir = transform.inverse_transform_vector(dir);

        if local_dir[1].is_positive() {
            // half capsule
            let mut pt_local = local_dir * self.capsule_radius();
            pt_local[1] += self.capsule_height();
            return transform * Point3::from(pt_local);
        } else {
            if local_dir[1] > -N::frac2() {
                // cone
                let radius_dir = Vector3::new(local_dir[0], na::zero(), local_dir[2]);
                let pt_local = v3_to_p3(radius_dir.normalize() * self.capsule_radius());
                return transform * Point3::from(pt_local);
            } else {
                // small ball
                let mut pt_local = local_dir * self.bottom_radius();
                pt_local[1] -= self.cone_height() - N::frac2() * self.bottom_radius();
                return transform * Point3::from(pt_local);
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
            self.capsule_radius() + self.capsule_height(),
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
        let center = self.capsule_radius() + self.capsule_height()
            - self.cone_height()
            - self.bottom_height();
        let radius = N::frac2()
            * (self.capsule_radius()
                + self.capsule_height()
                + self.cone_height()
                + self.bottom_height());
        return BoundingSphere::new(Point3::new(na::zero(), center, na::zero()), radius);
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
        let ls_ray = ray.inverse_transform_by(transform);

        return query::ray_intersection_with_support_map_with_params(
            &Isometry3::identity(),
            self,
            &mut VoronoiSimplex::new(),
            &ls_ray,
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

        if pt_local[1] >= na::zero() {
            // capsule
            let (pt_local_proj, _) = segment_project_point(
                Point3::new(na::zero(), self.capsule_height(), na::zero()),
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
                    let dir = transform * Vector3::new(na::zero(), na::one(), na::zero());
                    return PointProjection::new(true, pt_proj + dir * self.capsule_radius());
                }
            }
        } else {
            // cone + ball
            let vec_xz = Vector3::new(pt_local[0], na::zero(), pt_local[2]);
            if let Some((vec_xz, _)) = Unit::try_new_and_get(vec_xz, N::default_epsilon()) {
                let pt_a = v3_to_p3(vec_xz.as_ref() * self.capsule_radius());
                let mut pt_b = v3_to_p3(vec_xz.as_ref() * self.bottom_radius());
                pt_b[1] = -self.cone_height();
                let (pt_local_proj, proj_type) = segment_project_point(pt_a, pt_b, pt_local);

                if proj_type == ProjectType::A {
                    return PointProjection::new(false, pt_a);
                } else if proj_type == ProjectType::AB {
                    println!("=========={:?} {:?} {:?}", pt_a, pt_b, pt_local);
                    println!("=========={:?} {:?}", proj_type, pt_local_proj);
                    let pt_proj = transform * pt_local_proj;
                    let inside = p3_to_v3(pt_proj).norm_squared() > (pt - pt_proj).norm_squared();
                    if solid && inside {
                        return PointProjection::new(true, *pt);
                    } else {
                        return PointProjection::new(inside, pt_proj);
                    }
                } else {
                    let center = self.cone_height() - self.bottom_height();
                    let vec_pc = Vector3::new(pt_local[0], pt_local[1] + center, pt_local[2]);
                    let dist_squared = vec_pc.norm_squared();
                    let inside = dist_squared < self.bottom_radius() * self.bottom_radius();
                    if solid && inside {
                        return PointProjection::new(true, *pt);
                    } else {
                        let vec_radius = vec_pc * (self.bottom_radius() / dist_squared.sqrt());
                        let pt_proj = transform
                            * Point3::new(vec_radius[0], vec_radius[1] - center, vec_radius[2]);
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

#[derive(Clone, Copy, Debug, PartialEq)]
enum ProjectType {
    A,
    AB,
    B,
}

fn segment_project_point<N: RealField + RealExt>(
    pt_a: Point3<N>,
    pt_b: Point3<N>,
    pt_local: Point3<N>,
) -> (Point3<N>, ProjectType) {
    let vec_ab = pt_b - pt_a;
    let vec_ap = pt_local - pt_a;
    let dot_ab_ap = vec_ab.dot(&vec_ap);
    let norm_sq_ab = vec_ab.norm_squared();
    if dot_ab_ap <= na::zero() {
        return (pt_a, ProjectType::A);
    } else if dot_ab_ap >= norm_sq_ab {
        return (pt_b, ProjectType::B);
    } else {
        return (pt_a + vec_ab * (dot_ab_ap / norm_sq_ab), ProjectType::AB);
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
    use crate::fx::{fx, Fx};
    use approx::assert_relative_eq;
    use na::{Translation3, UnitQuaternion};

    #[test]
    fn test_human_bounding_general() {
        let hb = HumanBounding::new(fx(0.4), fx(0.7), fx(0.05));
        assert_relative_eq!(hb.capsule_radius(), fx(0.4));
        assert_relative_eq!(hb.capsule_height(), fx(0.7));
        assert_relative_eq!(hb.bottom_radius(), fx(0.05));
        assert_relative_eq!(hb.cone_height(), fx(0.35) * Fx::sqrt3());
        assert_relative_eq!(hb.bottom_height(), fx(0.025));
    }

    #[test]
    fn test_human_bounding_support_map() {
        let hb = HumanBounding::new(fx(0.4), fx(0.7), fx(0.05));
        let transform = Isometry3::from_parts(
            Translation3::from(Vector3::new(fx(1), fx(2), fx(3))),
            UnitQuaternion::identity(),
        );

        // -90°
        let pt_real = hb.support_point(&transform, &Vector3::new(fx(0), fx(-10), fx(0)));
        let pt_local = Point3::new(fx(0), -(hb.cone_height() + hb.bottom_height()), fx(0));
        assert_relative_eq!(pt_real, transform * pt_local);

        // -45°
        let pt_real = hb.support_point(&transform, &Vector3::new(fx(10), fx(-10), fx(0)));
        let pt_local = Point3::new(
            Fx::sqrt2() * hb.bottom_height(),
            -(hb.cone_height() + (Fx::sqrt2() - fx(1)) * hb.bottom_height()),
            fx(0),
        );
        assert_relative_eq!(pt_real, transform * pt_local);

        // -30°
        let pt_real = hb.support_point(&transform, &Vector3::new(fx(0), -fx(1), Fx::sqrt3()));
        let pt_local = Point3::new(fx(0), -hb.cone_height(), Fx::sqrt3() * hb.bottom_height());
        assert_relative_eq!(pt_real, transform * pt_local);

        // -30° ~ -0°
        let pt_real = hb.support_point(&transform, &Vector3::new(fx(0), -fx(1), -fx(3)));
        let pt_local = Point3::new(fx(0), fx(0), -hb.capsule_radius());
        assert_relative_eq!(pt_real, transform * pt_local);

        // 45°
        let pt_real = hb.support_point(&transform, &Vector3::new(-fx(4), fx(4), fx(0)));
        let pt_local = Point3::new(
            -Fx::sqrt2() / fx(2) * hb.capsule_radius(),
            Fx::sqrt2() / fx(2) * hb.capsule_radius() + hb.capsule_height(),
            fx(0),
        );
        assert_relative_eq!(pt_real, transform * pt_local);

        // 90°
        let pt_real = hb.support_point(&transform, &Vector3::new(fx(0), fx(4), fx(0)));
        let pt_local = Point3::new(fx(0), hb.capsule_radius() + hb.capsule_height(), fx(0));
        assert_relative_eq!(pt_real, transform * pt_local);
    }

    #[test]
    fn test_human_bounding_aabb() {
        let hb = HumanBounding::new(fx(0.4), fx(0.7), fx(0.05));
        let transform = Isometry3::from_parts(
            Translation3::from(Vector3::new(fx(1), fx(2), fx(3))),
            UnitQuaternion::identity(),
        );
        let aabb: AABB<Fx> = hb.bounding_volume(&transform);
        let excepted: AABB<Fx> = AABB::new(
            Point3::new(-fx(0.4), -(hb.cone_height() + hb.bottom_height()), -fx(0.4)),
            Point3::new(fx(0.4), hb.capsule_height() + hb.capsule_radius(), fx(0.4)),
        )
        .transform_by(&transform);
        assert_relative_eq!(aabb.mins(), excepted.mins());
        assert_relative_eq!(aabb.maxs(), excepted.maxs());
    }

    #[test]
    fn test_human_bounding_sphere() {
        let hb = HumanBounding::new(fx(0.4), fx(0.7), fx(0.05));
        let transform = Isometry3::from_parts(Translation3::identity(), UnitQuaternion::identity());
        let sphere: BoundingSphere<Fx> = hb.bounding_sphere(&transform);
        let excepted: BoundingSphere<Fx> = BoundingSphere::new(
            Point3::new(fx(0), fx(0.4 + 0.7 - f64::sqrt3() * 0.35 - 0.025), fx(0)),
            fx((0.4 + 0.7 + f64::sqrt3() * 0.35 + 0.025) / 2.0),
        );
        assert_relative_eq!(sphere.center(), excepted.center());
        assert_relative_eq!(sphere.radius(), excepted.radius());
    }

    #[test]
    fn test_human_bounding_project_point() {
        let hb = HumanBounding::new(fx(1), fx(0.5), fx(0.1));
        let transform = Isometry3::from_parts(
            Translation3::from(Vector3::new(fx(-2), fx(-2), fx(-2))),
            UnitQuaternion::identity(),
        );
        let center_height = hb.cone_height() - hb.bottom_height();

        // bottom & y axis
        let pt_in = Point3::new(fx(0), fx(-1.3), fx(0));
        let proj = hb.project_point(&transform, &(transform * pt_in), true);
        assert_eq!(proj.is_inside, true);
        assert_relative_eq!(proj.point, transform * Point3::new(fx(0), fx(-1.3), fx(0)));

        let proj = hb.project_point(&transform, &(transform * pt_in), false);
        assert_eq!(proj.is_inside, true);
        assert_relative_eq!(
            proj.point,
            transform * Point3::new(fx(0), -hb.cone_height() - hb.bottom_height(), fx(0))
        );

        let pt_in = Point3::new(fx(0), fx(-10), fx(0));
        let proj = hb.project_point(&transform, &(transform * pt_in), true);
        assert_eq!(proj.is_inside, false);
        assert_relative_eq!(
            proj.point,
            transform * Point3::new(fx(0), -hb.cone_height() - hb.bottom_height(), fx(0))
        );

        // bottom ball
        let pt_in = Point3::new(fx(0), -center_height - fx(0.05), fx(0.05));
        let proj = hb.project_point(&transform, &(transform * pt_in), true);
        assert_eq!(proj.is_inside, true);
        assert_relative_eq!(
            proj.point,
            transform * Point3::new(fx(0), -center_height - fx(0.05), fx(0.05),)
        );

        let proj = hb.project_point(&transform, &(transform * pt_in), false);
        assert_eq!(proj.is_inside, true);
        let pt = Point3::new(
            fx(0),
            -center_height - fx(0.05) * Fx::sqrt2(),
            fx(0.05) * Fx::sqrt2(),
        );
        assert_relative_eq!(proj.point, transform * pt);
        
        let pt_in = Point3::new(fx(0), -center_height - fx(1), fx(1));
        let proj = hb.project_point(&transform, &(transform * pt_in), false);
        assert_eq!(proj.is_inside, false);
        let pt = Point3::new(
            fx(0),
            -center_height - fx(0.05) * Fx::sqrt2(),
            fx(0.05) * Fx::sqrt2(),
        );
        assert_relative_eq!(proj.point, transform * pt);
        
        // bottom cone
        let pt_in = Point3::new(fx(0), -fx(0.3) * Fx::sqrt3(), -fx(0.3));
        let proj = hb.project_point(&transform, &(transform * pt_in), true);
        assert_eq!(proj.is_inside, true);
        assert_relative_eq!(
            proj.point,
            transform * Point3::new(fx(0), -fx(0.3) * Fx::sqrt3(), -fx(0.3))
        );
        
        let proj = hb.project_point(&transform, &(transform * pt_in), false);
        assert_eq!(proj.is_inside, true);
        assert_relative_eq!(
            proj.point,
            transform * Point3::new(fx(0), -fx(0.4) * Fx::sqrt3(), -fx(0.6))
        );
        
        let pt_in = Point3::new(fx(0), -fx(0.5) * Fx::sqrt3(), -fx(0.9));
        let proj = hb.project_point(&transform, &(transform * pt_in), false);
        assert_eq!(proj.is_inside, true);
        assert_relative_eq!(
            proj.point,
            transform * Point3::new(fx(0), -fx(0.4) * Fx::sqrt3(), -fx(0.6))
        );

        // xz plane
        let pt_in = Point3::new(Fx::sqrt2(), fx(0), Fx::sqrt2());
        let proj = hb.project_point(&transform, &(transform * pt_in), true);
        assert_eq!(proj.is_inside, true);
        assert_relative_eq!(
            proj.point,
            transform * Point3::new(fx(0), -fx(0.3) * Fx::sqrt3(), -fx(0.3))
        );
    }
}
