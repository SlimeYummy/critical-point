use na::{Isometry3, Point3, RealField, Unit, Vector3};
use ncollide3d::bounding_volume::{self, BoundingSphere, HasBoundingVolume, AABB};
use ncollide3d::query::{PointProjection, PointQuery, Ray, RayCast, RayIntersection};
use ncollide3d::shape::{Cone, FeatureId, Shape, SupportMap};

#[derive(Clone, Debug, PartialEq)]
pub struct ConeExt<N> {
    cone: Cone<N>,
}

impl<N: RealField> ConeExt<N> {
    pub fn new(half_height: N, radius: N) -> ConeExt<N> {
        return ConeExt {
            cone: Cone::new(half_height, radius),
        };
    }

    #[inline]
    pub fn half_height(&self) -> N {
        return self.cone.half_height();
    }

    #[inline]
    pub fn radius(&self) -> N {
        return self.cone.radius();
    }
}

impl<N: RealField> SupportMap<N> for ConeExt<N> {
    fn support_point(&self, transform: &Isometry3<N>, dir: &Vector3<N>) -> Point3<N> {
        return self.cone.support_point(transform, dir);
    }

    fn support_point_toward(&self, transform: &Isometry3<N>, dir: &Unit<Vector3<N>>) -> Point3<N> {
        return self.cone.support_point_toward(transform, dir);
    }
}

impl<N: RealField> HasBoundingVolume<N, AABB<N>> for ConeExt<N> {
    #[inline]
    fn bounding_volume(&self, transform: &Isometry3<N>) -> AABB<N> {
        return self.cone.bounding_volume(transform);
    }

    #[inline]
    fn local_bounding_volume(&self) -> AABB<N> {
        return self.cone.local_bounding_volume();
    }
}

impl<N: RealField> HasBoundingVolume<N, BoundingSphere<N>> for ConeExt<N> {
    #[inline]
    fn bounding_volume(&self, transform: &Isometry3<N>) -> BoundingSphere<N> {
        return self.cone.bounding_volume(transform);
    }

    #[inline]
    fn local_bounding_volume(&self) -> BoundingSphere<N> {
        return self.cone.local_bounding_volume();
    }
}

impl<N: RealField> RayCast<N> for ConeExt<N> {
    fn toi_and_normal_with_ray(
        &self,
        transform: &Isometry3<N>,
        ray: &Ray<N>,
        max_toi: N,
        solid: bool,
    ) -> Option<RayIntersection<N>> {
        return self
            .cone
            .toi_and_normal_with_ray(transform, ray, max_toi, solid);
    }
}

impl<N: RealField> PointQuery<N> for ConeExt<N> {
    #[inline]
    fn project_point(
        &self,
        transform: &Isometry3<N>,
        pt: &Point3<N>,
        solid: bool,
    ) -> PointProjection<N> {
        return self.cone.project_point(transform, pt, solid);
    }

    #[inline]
    fn project_point_with_feature(
        &self,
        transform: &Isometry3<N>,
        pt: &Point3<N>,
    ) -> (PointProjection<N>, FeatureId) {
        return self.cone.project_point_with_feature(transform, pt);
    }
}

impl<N: RealField> Shape<N> for ConeExt<N> {
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
