use crate::pipeline::broad_phase::BroadPhaseProxyHandle;
use math::Fx;
use ncollide3d::math::Isometry;
use ncollide3d::pipeline::narrow_phase::CollisionObjectGraphIndex;
use ncollide3d::pipeline::{
    CollisionGroups, CollisionObjectRef, CollisionObjectUpdateFlags, GeometricQueryType,
};
use ncollide3d::shape::{Shape, ShapeHandle};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionObjectClass {
    Stage = 0,
    Move = 1,
    Hit = 2,
}

pub struct CollisionObject<T> {
    obj_type: CollisionObjectClass,
    proxy_handle: Option<BroadPhaseProxyHandle>,
    graph_index: Option<CollisionObjectGraphIndex>,
    position: Isometry<Fx>,
    predicted_position: Option<Isometry<Fx>>,
    shape: ShapeHandle<Fx>,
    collision_groups: CollisionGroups,
    query_type: GeometricQueryType<Fx>,
    update_flags: CollisionObjectUpdateFlags,
    data: T,
}

impl<T> CollisionObject<T> {
    pub fn new(
        obj_type: CollisionObjectClass,
        proxy_handle: Option<BroadPhaseProxyHandle>,
        graph_index: Option<CollisionObjectGraphIndex>,
        position: Isometry<Fx>,
        shape: ShapeHandle<Fx>,
        groups: CollisionGroups,
        query_type: GeometricQueryType<Fx>,
        data: T,
    ) -> CollisionObject<T> {
        return CollisionObject {
            obj_type,
            proxy_handle,
            graph_index,
            position,
            predicted_position: None,
            shape,
            collision_groups: groups,
            data,
            query_type,
            update_flags: CollisionObjectUpdateFlags::all(),
        };
    }

    #[inline]
    pub fn obj_type(&self) -> CollisionObjectClass {
        return self.obj_type;
    }

    #[inline]
    pub fn graph_index(&self) -> Option<CollisionObjectGraphIndex> {
        return self.graph_index;
    }

    #[inline]
    pub fn set_graph_index(&mut self, index: Option<CollisionObjectGraphIndex>) {
        self.graph_index = index
    }

    #[inline]
    pub fn update_flags_mut(&mut self) -> &mut CollisionObjectUpdateFlags {
        return &mut self.update_flags;
    }

    #[inline]
    pub fn clear_update_flags(&mut self) {
        self.update_flags = CollisionObjectUpdateFlags::empty()
    }

    #[inline]
    pub fn proxy_handle(&self) -> Option<BroadPhaseProxyHandle> {
        return self.proxy_handle;
    }

    #[inline]
    pub fn set_proxy_handle(&mut self, handle: Option<BroadPhaseProxyHandle>) {
        self.proxy_handle = handle;
    }

    #[inline]
    pub fn position(&self) -> &Isometry<Fx> {
        return &self.position;
    }

    #[inline]
    pub fn predicted_position(&self) -> Option<&Isometry<Fx>> {
        return self.predicted_position.as_ref();
    }

    #[inline]
    pub fn set_position(&mut self, pos: Isometry<Fx>) {
        self.update_flags |= CollisionObjectUpdateFlags::POSITION_CHANGED;
        self.update_flags |= CollisionObjectUpdateFlags::PREDICTED_POSITION_CHANGED;
        self.position = pos;
        self.predicted_position = None;
    }

    #[inline]
    pub fn set_position_with_prediction(&mut self, pos: Isometry<Fx>, prediction: Isometry<Fx>) {
        self.update_flags |= CollisionObjectUpdateFlags::POSITION_CHANGED;
        self.update_flags |= CollisionObjectUpdateFlags::PREDICTED_POSITION_CHANGED;
        self.position = pos;
        self.predicted_position = Some(prediction);
    }

    #[inline]
    pub fn set_predicted_position(&mut self, pos: Option<Isometry<Fx>>) {
        self.update_flags |= CollisionObjectUpdateFlags::PREDICTED_POSITION_CHANGED;
        self.predicted_position = pos;
    }

    #[inline]
    pub fn shape(&self) -> &ShapeHandle<Fx> {
        return &self.shape;
    }

    #[inline]
    pub fn set_shape(&mut self, shape: ShapeHandle<Fx>) {
        self.update_flags |= CollisionObjectUpdateFlags::SHAPE_CHANGED;
        self.shape = shape;
    }

    #[inline]
    pub fn collision_groups(&self) -> &CollisionGroups {
        return &self.collision_groups;
    }

    #[inline]
    pub fn set_collision_groups(&mut self, groups: CollisionGroups) {
        self.update_flags |= CollisionObjectUpdateFlags::COLLISION_GROUPS_CHANGED;
        self.collision_groups = groups;
    }

    #[inline]
    pub fn query_type(&self) -> GeometricQueryType<Fx> {
        return self.query_type;
    }

    #[inline]
    pub fn set_query_type(&mut self, query_type: GeometricQueryType<Fx>) {
        self.update_flags |= CollisionObjectUpdateFlags::QUERY_TYPE_CHANGED;
        self.query_type = query_type;
    }

    #[inline]
    pub fn data(&self) -> &T {
        return &self.data;
    }

    #[inline]
    pub fn data_mut(&mut self) -> &mut T {
        return &mut self.data;
    }
}

impl<T> CollisionObjectRef<Fx> for CollisionObject<T> {
    fn graph_index(&self) -> Option<CollisionObjectGraphIndex> {
        return self.graph_index();
    }

    fn proxy_handle(&self) -> Option<BroadPhaseProxyHandle> {
        return self.proxy_handle();
    }

    fn position(&self) -> &Isometry<Fx> {
        return self.position();
    }

    fn predicted_position(&self) -> Option<&Isometry<Fx>> {
        return self.predicted_position();
    }

    fn shape(&self) -> &dyn Shape<Fx> {
        return self.shape().as_ref();
    }

    fn collision_groups(&self) -> &CollisionGroups {
        return self.collision_groups();
    }

    fn query_type(&self) -> GeometricQueryType<Fx> {
        return self.query_type();
    }

    fn update_flags(&self) -> CollisionObjectUpdateFlags {
        return self.update_flags;
    }
}
