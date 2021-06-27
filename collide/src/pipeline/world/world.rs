use super::handler::CollisionWorldInterferenceHandler;
use crate::pipeline::broad_phase::DBVTBroadPhase;
use crate::pipeline::object::{
    CollisionGroups, CollisionObject, CollisionObjectSet, CollisionObjectSlab,
    CollisionObjectSlabHandle, CollisionObjectType, CollisionObjects, GeometricQueryType,
};
use anyhow::{anyhow, Result};
use math::{fi, Fx, RealExt};
use na::Unit;
use ncollide3d::bounding_volume::{BoundingVolume, AABB};
use ncollide3d::math::{Isometry, Point, Rotation, Translation, Vector};
use ncollide3d::pipeline::{
    CollisionObjectRef, ContactAlgorithm, ContactEvents, DefaultContactDispatcher,
    DefaultProximityDispatcher, Interaction, InteractionGraph, NarrowPhase, ProximityDetector,
    ProximityEvents, TemporaryInteractionIndex,
};
use ncollide3d::query::{
    ContactManifold, DefaultTOIDispatcher, PointQuery, Proximity, Ray, RayCast, RayIntersection,
    TOIDispatcher, TOI,
};
use ncollide3d::shape::{Shape, ShapeHandle};

const NOT_REGISTERED_ERROR: &'static str =
    "This collision object has not been registered into a world (proxy indexes are None).";

pub struct CollisionWorld<T: 'static> {
    pub objects: CollisionObjectSlab<T>,
    pub broad_phase: DBVTBroadPhase<AABB<Fx>>,
    pub narrow_phase: NarrowPhase<Fx, CollisionObjectSlabHandle>,
    pub toi_dispatcher: Box<dyn TOIDispatcher<Fx>>,
    pub interactions: InteractionGraph<Fx, CollisionObjectSlabHandle>,

    // Just to avoid dynamic allocations
    handles: Vec<CollisionObjectSlabHandle>,
}

impl<T: 'static> CollisionWorld<T> {
    pub fn new(margin: Fx) -> CollisionWorld<T> {
        let objects = CollisionObjectSlab::new();
        let coll_dispatcher = Box::new(DefaultContactDispatcher::new());
        let prox_dispatcher = Box::new(DefaultProximityDispatcher::new());
        let toi_dispatcher = Box::new(DefaultTOIDispatcher);
        let broad_phase = DBVTBroadPhase::new(margin);
        let narrow_phase = NarrowPhase::new(coll_dispatcher, prox_dispatcher);

        return CollisionWorld {
            interactions: InteractionGraph::new(),
            objects,
            broad_phase,
            narrow_phase,
            toi_dispatcher,
            handles: Vec::new(),
        };
    }

    pub fn add(
        &mut self,
        obj_type: CollisionObjectType,
        position: Isometry<Fx>,
        shape: ShapeHandle<Fx>,
        collision_groups: CollisionGroups,
        query_type: GeometricQueryType<Fx>,
        data: T,
    ) -> (CollisionObjectSlabHandle, &mut CollisionObject<T>) {
        let entry = self.objects.objects.vacant_entry();
        let handle = CollisionObjectSlabHandle(entry.key());

        let mut aabb = shape.aabb(&position);
        aabb.loosen(query_type.query_limit());

        let proxy_handle = self.broad_phase.create_proxy(obj_type, aabb, handle);
        let graph_index = self.interactions.add_node(handle);

        let co = CollisionObject::new(
            obj_type,
            Some(proxy_handle),
            Some(graph_index),
            position,
            shape,
            collision_groups,
            query_type,
            data,
        );

        return (handle, entry.insert(co));
    }

    pub fn update(&mut self, types: &[CollisionObjectType]) {
        self.narrow_phase.clear_events();

        Self::perform_broad_phase(
            types,
            &self.objects,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.interactions,
        );

        Self::perform_narrow_phase(
            &self.objects,
            &mut self.narrow_phase,
            &mut self.interactions,
        );

        // Clear update flags.
        for (_, co) in self.objects.iter_mut() {
            co.clear_update_flags();
        }
    }

    fn perform_broad_phase(
        types: &[CollisionObjectType],
        objects: &CollisionObjectSlab<T>,
        broad_phase: &mut DBVTBroadPhase<AABB<Fx>>,
        narrow_phase: &mut NarrowPhase<Fx, CollisionObjectSlabHandle>,
        interactions: &mut InteractionGraph<Fx, CollisionObjectSlabHandle>,
    ) {
        // Take changes into account.
        objects.foreach(|_, co| {
            let flags = co.update_flags();
            let proxy_handle = co.proxy_handle().expect(NOT_REGISTERED_ERROR);

            if flags.needs_bounding_volume_update() {
                broad_phase.deferred_set_bounding_volume(proxy_handle, co.compute_swept_aabb());
            }

            if flags.needs_broad_phase_redispatch() {
                broad_phase.deferred_recompute_all_proximities_with(proxy_handle);
            }
        });

        // Update the broad-phase.
        let mut handler = CollisionWorldInterferenceHandler {
            interactions,
            narrow_phase,
            objects,
        };
        broad_phase.update(types, &mut handler);
    }

    fn perform_narrow_phase(
        objects: &CollisionObjectSlab<T>,
        narrow_phase: &mut NarrowPhase<Fx, CollisionObjectSlabHandle>,
        interactions: &mut InteractionGraph<Fx, CollisionObjectSlabHandle>,
    ) {
        narrow_phase.update(interactions, objects);
    }

    pub fn clear_events(&mut self) {
        self.narrow_phase.clear_events();
    }

    pub fn remove(&mut self, handles: &[CollisionObjectSlabHandle]) {
        for handle in handles {
            let co = self.objects.remove(*handle);
            let graph_index = co.graph_index().expect(NOT_REGISTERED_ERROR);
            let proxy_handle = co.proxy_handle().expect(NOT_REGISTERED_ERROR);

            self.broad_phase.remove(&[proxy_handle]);
            self.interactions
                .remove_node(graph_index)
                .map(|h| self.objects[h].set_graph_index(Some(graph_index)));
        }
    }

    pub fn broad_phase_aabb(&self, handle: CollisionObjectSlabHandle) -> Option<&AABB<Fx>> {
        let co = self.objects.collision_object(handle)?;
        let proxy_handle = co.proxy_handle().expect(NOT_REGISTERED_ERROR);
        return self.broad_phase.proxy(proxy_handle).map(|p| p.0);
    }

    #[inline]
    pub fn collision_objects(&self) -> CollisionObjects<T> {
        return self.objects.iter();
    }

    #[inline]
    pub fn collision_object(
        &self,
        handle: CollisionObjectSlabHandle,
    ) -> Option<&CollisionObject<T>> {
        return self.objects.collision_object(handle);
    }

    #[inline]
    pub fn get_mut(
        &mut self,
        handle: CollisionObjectSlabHandle,
    ) -> Option<&mut CollisionObject<T>> {
        return self.objects.get_mut(handle);
    }

    #[inline]
    pub fn collision_object_pair_mut(
        &mut self,
        handle1: CollisionObjectSlabHandle,
        handle2: CollisionObjectSlabHandle,
    ) -> (
        Option<&mut CollisionObject<T>>,
        Option<&mut CollisionObject<T>>,
    ) {
        return self.objects.get_pair_mut(handle1, handle2);
    }

    //
    // Interference with shape
    //

    pub fn interferences_with_point<'a, 'b, 'c>(
        &'a mut self,
        obj_type: CollisionObjectType,
        point: &'b Point<Fx>,
        groups: &'b CollisionGroups,
        out: &'c mut Vec<CollisionObjectSlabHandle>,
    ) {
        assert!(self.handles.is_empty());
        self.broad_phase
            .interferences_with_point(obj_type, point, &mut self.handles);
        for handle in &self.handles {
            if let Some(co) = self.objects.collision_object(*handle) {
                if co.collision_groups().can_interact_with(groups)
                    && co.shape().contains_point(&co.position(), point)
                {
                    out.push(*handle);
                }
            }
        }
        self.handles.clear();
    }

    pub fn interferences_with_aabb<'a, 'b, 'c>(
        &'a mut self,
        obj_type: CollisionObjectType,
        aabb: &'b AABB<Fx>,
        groups: &'b CollisionGroups,
        out: &'c mut Vec<CollisionObjectSlabHandle>,
    ) {
        assert!(self.handles.is_empty());
        self.broad_phase
            .interferences_with_aabb(obj_type, aabb, &mut self.handles);
        for handle in &self.handles {
            if let Some(co) = self.objects.collision_object(*handle) {
                if co.collision_groups().can_interact_with(groups) {
                    out.push(*handle);
                }
            }
        }
        self.handles.clear();
    }

    pub fn interferences_with_ray<'a, 'b, 'c>(
        &'a mut self,
        obj_type: CollisionObjectType,
        ray: &'b Ray<Fx>,
        max_toi: Fx,
        groups: &'b CollisionGroups,
        out: &'c mut Vec<(CollisionObjectSlabHandle, RayIntersection<Fx>)>,
    ) {
        assert!(self.handles.is_empty());
        self.broad_phase
            .interferences_with_ray(obj_type, ray, max_toi, &mut self.handles);
        for handle in &self.handles {
            if let Some(co) = self.objects.collision_object(*handle) {
                if co.collision_groups().can_interact_with(groups) {
                    let inter =
                        co.shape()
                            .toi_and_normal_with_ray(&co.position(), ray, max_toi, true);

                    if let Some(inter) = inter {
                        out.push((*handle, inter));
                    }
                }
            }
        }
        self.handles.clear();
    }

    pub fn first_interference_with_ray<'a, 'b>(
        &'a mut self,
        obj_type: CollisionObjectType,
        ray: &'b Ray<Fx>,
        max_toi: Fx,
        groups: &'b CollisionGroups,
    ) -> Option<(CollisionObjectSlabHandle, RayIntersection<Fx>)> {
        let objects = &mut self.objects;
        let broad_phase = &mut self.broad_phase;

        let narrow_phase = move |handle: CollisionObjectSlabHandle, ray: &Ray<Fx>, max_toi: Fx| {
            let co = objects.collision_object(handle)?;
            if co.collision_groups().can_interact_with(groups) {
                let inter = co
                    .shape()
                    .toi_and_normal_with_ray(&co.position(), ray, max_toi, true);

                return inter.map(|inter| (handle, inter));
            }
            return None;
        };

        let (handle, inter) =
            broad_phase.first_interference_with_ray(obj_type, ray, max_toi, &narrow_phase)?;
        return Some((handle, inter));
    }

    pub fn sweep_test<'a, 'b>(
        &'a mut self,
        obj_type: CollisionObjectType,
        shape: &'b dyn Shape<Fx>,
        isometry: &'b Isometry<Fx>,
        direction: &'b Unit<Vector<Fx>>,
        max_distance: Fx,
        groups: &'b CollisionGroups,
        out: &mut Vec<(CollisionObjectSlabHandle, TOI<Fx>)>,
    ) {
        let a = shape.aabb(&isometry);
        let b = shape.aabb(&Isometry::from_parts(
            Translation::from(isometry.translation.vector + direction.as_ref() * max_distance),
            Rotation::identity(),
        ));
        let aabb = a.merged(&b);

        assert!(self.handles.is_empty());
        self.broad_phase
            .interferences_with_aabb(obj_type, &aabb, &mut self.handles);
        for handle in &self.handles {
            if let Some(co) = self.objects.collision_object(*handle) {
                if co.collision_groups().can_interact_with(groups) {
                    let dispatcher = &*self.toi_dispatcher;
                    let res = dispatcher
                        .time_of_impact(
                            dispatcher,
                            &isometry,
                            &direction,
                            shape,
                            co.position(),
                            &Vector::zeros(),
                            co.shape().as_ref(),
                            Fx::max_value(),
                            Fx::c0(),
                        )
                        .unwrap_or(None);
                    if let Some(toi) = res {
                        out.push((*handle, toi));
                    }
                }
            }
        }
        self.handles.clear();
    }

    pub fn first_impact_with_obj(
        &mut self,
        h_obj: CollisionObjectSlabHandle,
        isometry: &Isometry<Fx>,
        direction: &Unit<Vector<Fx>>,
        max_distance: Fx,
    ) -> Result<Option<(CollisionObjectSlabHandle, TOI<Fx>)>> {
        let obj = self
            .objects
            .collision_object(h_obj)
            .ok_or(anyhow!("Collision object not found"))?;
        let obj_type = obj.obj_type();
        let group = obj.collision_groups().clone();

        let a = obj.shape().aabb(&isometry);
        let b = obj.shape().aabb(&Isometry::from_parts(
            Translation::from(isometry.translation.vector + direction.as_ref() * max_distance),
            Rotation::identity(),
        ));
        let aabb = a.merged(&b);

        let mut ret = None;
        let min_toi = Fx::max_value();

        assert!(self.handles.is_empty());
        self.broad_phase
            .interferences_with_aabb(obj_type, &aabb, &mut self.handles);
        for handle in &self.handles {
            if let Some(co) = self.objects.collision_object(*handle) {
                if co.collision_groups().can_interact_with(&group) && *handle != h_obj {
                    let dispatcher = &*self.toi_dispatcher;
                    let res = dispatcher
                        .time_of_impact(
                            dispatcher,
                            &isometry,
                            &direction,
                            &**obj.shape(),
                            co.position(),
                            &Vector::zeros(),
                            co.shape().as_ref(),
                            Fx::max_value(),
                            Fx::c0(),
                        )
                        .unwrap_or(None);
                    if let Some(toi) = res {
                        if toi.toi < min_toi {
                            ret = Some((*handle, toi));
                        }
                    }
                }
            }
        }
        self.handles.clear();
        return Ok(ret);
    }

    //
    // Operations on the interaction graph.
    //

    pub fn interaction_pairs(
        &self,
        effective_only: bool,
    ) -> impl Iterator<
        Item = (
            CollisionObjectSlabHandle,
            CollisionObjectSlabHandle,
            &Interaction<Fx>,
        ),
    > {
        return self.interactions.interaction_pairs(effective_only);
    }

    pub fn contact_pairs(
        &self,
        effective_only: bool,
    ) -> impl Iterator<
        Item = (
            CollisionObjectSlabHandle,
            CollisionObjectSlabHandle,
            &ContactAlgorithm<Fx>,
            &ContactManifold<Fx>,
        ),
    > {
        return self.interactions.contact_pairs(effective_only);
    }

    pub fn proximity_pairs(
        &self,
        effective_only: bool,
    ) -> impl Iterator<
        Item = (
            CollisionObjectSlabHandle,
            CollisionObjectSlabHandle,
            &dyn ProximityDetector<Fx>,
            Proximity,
        ),
    > {
        return self.interactions.proximity_pairs(effective_only);
    }

    pub fn interaction_pair(
        &self,
        handle1: CollisionObjectSlabHandle,
        handle2: CollisionObjectSlabHandle,
        effective_only: bool,
    ) -> Option<(
        CollisionObjectSlabHandle,
        CollisionObjectSlabHandle,
        &Interaction<Fx>,
    )> {
        let co1 = self.objects.collision_object(handle1)?;
        let co2 = self.objects.collision_object(handle2)?;
        let id1 = co1.graph_index().expect(NOT_REGISTERED_ERROR);
        let id2 = co2.graph_index().expect(NOT_REGISTERED_ERROR);
        return self.interactions.interaction_pair(id1, id2, effective_only);
    }

    pub fn contact_pair(
        &self,
        handle1: CollisionObjectSlabHandle,
        handle2: CollisionObjectSlabHandle,
        effective_only: bool,
    ) -> Option<(
        CollisionObjectSlabHandle,
        CollisionObjectSlabHandle,
        &ContactAlgorithm<Fx>,
        &ContactManifold<Fx>,
    )> {
        let co1 = self.objects.collision_object(handle1)?;
        let co2 = self.objects.collision_object(handle2)?;
        let id1 = co1.graph_index().expect(NOT_REGISTERED_ERROR);
        let id2 = co2.graph_index().expect(NOT_REGISTERED_ERROR);
        return self.interactions.contact_pair(id1, id2, effective_only);
    }

    pub fn proximity_pair(
        &self,
        handle1: CollisionObjectSlabHandle,
        handle2: CollisionObjectSlabHandle,
        effective_only: bool,
    ) -> Option<(
        CollisionObjectSlabHandle,
        CollisionObjectSlabHandle,
        &dyn ProximityDetector<Fx>,
        Proximity,
    )> {
        let co1 = self.objects.collision_object(handle1)?;
        let co2 = self.objects.collision_object(handle2)?;
        let id1 = co1.graph_index().expect(NOT_REGISTERED_ERROR);
        let id2 = co2.graph_index().expect(NOT_REGISTERED_ERROR);
        return self.interactions.proximity_pair(id1, id2, effective_only);
    }

    pub fn interactions_with(
        &self,
        handle: CollisionObjectSlabHandle,
        effective_only: bool,
    ) -> Option<
        impl Iterator<
            Item = (
                CollisionObjectSlabHandle,
                CollisionObjectSlabHandle,
                &Interaction<Fx>,
            ),
        >,
    > {
        let co = self.objects.collision_object(handle)?;
        let id = co.graph_index().expect(NOT_REGISTERED_ERROR);
        return Some(self.interactions.interactions_with(id, effective_only));
    }

    pub fn interactions_with_mut(
        &mut self,
        handle: CollisionObjectSlabHandle,
    ) -> Option<(
        &mut NarrowPhase<Fx, CollisionObjectSlabHandle>,
        impl Iterator<
            Item = (
                CollisionObjectSlabHandle,
                CollisionObjectSlabHandle,
                TemporaryInteractionIndex,
                &mut Interaction<Fx>,
            ),
        >,
    )> {
        let co = self.objects.collision_object(handle)?;
        let id = co.graph_index().expect(NOT_REGISTERED_ERROR);
        return Some((
            &mut self.narrow_phase,
            self.interactions.interactions_with_mut(id),
        ));
    }

    pub fn proximities_with(
        &self,
        handle: CollisionObjectSlabHandle,
        effective_only: bool,
    ) -> Option<
        impl Iterator<
            Item = (
                CollisionObjectSlabHandle,
                CollisionObjectSlabHandle,
                &dyn ProximityDetector<Fx>,
                Proximity,
            ),
        >,
    > {
        let co = self.objects.collision_object(handle)?;
        let id = co.graph_index().expect(NOT_REGISTERED_ERROR);
        return Some(self.interactions.proximities_with(id, effective_only));
    }

    pub fn contacts_with(
        &self,
        handle: CollisionObjectSlabHandle,
        effective_only: bool,
    ) -> Option<
        impl Iterator<
            Item = (
                CollisionObjectSlabHandle,
                CollisionObjectSlabHandle,
                &ContactAlgorithm<Fx>,
                &ContactManifold<Fx>,
            ),
        >,
    > {
        let co = self.objects.collision_object(handle)?;
        let id = co.graph_index().expect(NOT_REGISTERED_ERROR);
        return Some(self.interactions.contacts_with(id, effective_only));
    }

    pub fn collision_objects_interacting_with<'a>(
        &'a self,
        handle: CollisionObjectSlabHandle,
    ) -> Option<impl Iterator<Item = CollisionObjectSlabHandle> + 'a> {
        let co = self.objects.collision_object(handle)?;
        let id = co.graph_index().expect(NOT_REGISTERED_ERROR);
        return Some(self.interactions.collision_objects_interacting_with(id));
    }

    pub fn collision_objects_in_contact_with<'a>(
        &'a self,
        handle: CollisionObjectSlabHandle,
    ) -> Option<impl Iterator<Item = CollisionObjectSlabHandle> + 'a> {
        let co = self.objects.collision_object(handle)?;
        let id = co.graph_index().expect(NOT_REGISTERED_ERROR);
        return Some(self.interactions.collision_objects_in_contact_with(id));
    }

    pub fn collision_objects_in_proximity_of<'a>(
        &'a self,
        handle: CollisionObjectSlabHandle,
    ) -> Option<impl Iterator<Item = CollisionObjectSlabHandle> + 'a> {
        let co = self.objects.collision_object(handle)?;
        let id = co.graph_index().expect(NOT_REGISTERED_ERROR);
        return Some(self.interactions.collision_objects_in_proximity_of(id));
    }

    pub fn contact_events(&self) -> &ContactEvents<CollisionObjectSlabHandle> {
        return self.narrow_phase.contact_events();
    }

    pub fn proximity_events(&self) -> &ProximityEvents<CollisionObjectSlabHandle> {
        return self.narrow_phase.proximity_events();
    }
}
