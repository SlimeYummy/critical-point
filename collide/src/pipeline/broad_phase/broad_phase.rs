use super::fast_dbvt::FastDBVT;
use crate::pipeline::object::{CollisionObjectClass, CollisionObjectSlabHandle};
use math::Fx;
use ncollide3d::bounding_volume::BoundingVolume;
use ncollide3d::math::Point;
use ncollide3d::partitioning::{DBVTLeaf, DBVTLeafId};
use ncollide3d::pipeline::broad_phase::{
    BroadPhase, BroadPhaseInterferenceHandler, BroadPhaseProxyHandle,
};
use ncollide3d::query::visitors::{
    BoundingVolumeInterferencesCollector, PointInterferencesCollector, RayInterferencesCollector,
    RayIntersectionCostFnVisitor,
};
use ncollide3d::query::{PointQuery, Ray, RayCast, RayIntersection};
use ncollide3d::utils::{DeterministicState, SortedPair};
use slab::Slab;
use std::any::Any;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, VecDeque};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum ProxyStatus {
    Created,
    Deleted,
    Attached(DBVTLeafId),
    Detached(usize),
}

struct BroadPhaseProxy {
    data: CollisionObjectSlabHandle,
    class: CollisionObjectClass,
    status: ProxyStatus,
    updated: bool,
}

pub struct DBVTBroadPhase<BV> {
    proxies: Slab<BroadPhaseProxy>,
    stage_tree: FastDBVT<BroadPhaseProxyHandle, BV>,
    chara_tree: FastDBVT<BroadPhaseProxyHandle, BV>,
    hit_tree: FastDBVT<BroadPhaseProxyHandle, BV>,
    pairs: HashMap<SortedPair<BroadPhaseProxyHandle>, bool, DeterministicState>,
    margin: Fx,
    purge_all: bool,

    // Just to avoid dynamic allocations
    proxies_to_update: VecDeque<(BroadPhaseProxyHandle, BV)>,
    move_leaves_to_update: Vec<DBVTLeaf<Fx, BroadPhaseProxyHandle, BV>>,
    other_leaves_to_update: Vec<DBVTLeaf<Fx, BroadPhaseProxyHandle, BV>>,
    collector: Vec<BroadPhaseProxyHandle>,
}

impl<BV> DBVTBroadPhase<BV>
where
    BV: 'static + BoundingVolume<Fx> + Clone,
{
    pub fn new(margin: Fx) -> DBVTBroadPhase<BV> {
        return DBVTBroadPhase {
            proxies: Slab::new(),
            stage_tree: FastDBVT::new(),
            chara_tree: FastDBVT::new(),
            hit_tree: FastDBVT::new(),
            pairs: HashMap::with_hasher(DeterministicState::new()),
            purge_all: false,
            proxies_to_update: VecDeque::new(),
            move_leaves_to_update: Vec::new(),
            other_leaves_to_update: Vec::new(),
            collector: Vec::new(),
            margin,
        };
    }

    pub fn update(
        &mut self,
        is_move: bool,
        handler: &mut dyn BroadPhaseInterferenceHandler<CollisionObjectSlabHandle>,
    ) {
        self.remove_updated_leaves();

        let mut updated = self.reinsert_updated_leaves(true, handler);
        if !is_move {
            updated |= self.reinsert_updated_leaves(false, handler);
        }

        if updated {
            self.purge_some_contact_pairs(handler);
        }
    }

    fn purge_some_contact_pairs(
        &mut self,
        handler: &mut dyn BroadPhaseInterferenceHandler<CollisionObjectSlabHandle>,
    ) {
        let purge_all = self.purge_all;
        let proxies = &self.proxies;
        let trees = [
            &mut self.stage_tree,
            &mut self.chara_tree,
            &mut self.hit_tree,
        ];
        let pairs = &mut self.pairs;

        pairs.retain(|pair, up_to_date| {
            let mut retain = true;

            if purge_all || !*up_to_date {
                *up_to_date = true;

                let proxy1 = proxies
                    .get(pair.0.uid())
                    .expect("BroadPhaseProxy::purge_some_contact_pairs()");
                let proxy2 = proxies
                    .get(pair.1.uid())
                    .expect("BroadPhaseProxy::purge_some_contact_pairs()");

                if purge_all || proxy1.updated || proxy2.updated {
                    if handler.is_interference_allowed(&proxy1.data, &proxy2.data) {
                        let l1 = match proxy1.status {
                            ProxyStatus::Attached(leaf) => &trees[proxy1.class as usize][leaf],
                            _ => unreachable!("BroadPhaseProxy::purge_some_contact_pairs()"),
                        };

                        let l2 = match proxy2.status {
                            ProxyStatus::Attached(leaf) => &trees[proxy2.class as usize][leaf],
                            _ => unreachable!("BroadPhaseProxy::purge_some_contact_pairs()"),
                        };

                        if BoundingVolume::intersects(&l1.bounding_volume, &l2.bounding_volume) {
                            handler.interference_stopped(&proxy1.data, &proxy2.data);
                            retain = false;
                        }
                    } else {
                        handler.interference_stopped(&proxy1.data, &proxy2.data);
                        retain = false;
                    }
                }
            }

            *up_to_date = false;
            return retain;
        });
    }

    fn remove_updated_leaves(&mut self) {
        let trees = [
            &mut self.stage_tree,
            &mut self.chara_tree,
            &mut self.hit_tree,
        ];

        let leaves_to_updates = [
            unsafe { &mut *(&mut self.other_leaves_to_update as *mut _) },
            &mut self.move_leaves_to_update,
            unsafe { &mut *(&mut self.other_leaves_to_update as *mut _) },
        ];

        for (handle, bv) in self.proxies_to_update.drain(..) {
            if let Some(proxy) = self.proxies.get_mut(handle.uid()) {
                match proxy.status {
                    ProxyStatus::Attached(leaf) => {
                        let mut leaf = trees[proxy.class as usize].remove(leaf);
                        leaf.bounding_volume = bv;
                        leaves_to_updates[proxy.class as usize].push(leaf);
                        proxy.status = ProxyStatus::Detached(leaves_to_updates.len() - 1);
                    }
                    ProxyStatus::Detached(id) => {
                        let leaf = DBVTLeaf::new(bv, handle);
                        leaves_to_updates[proxy.class as usize][id] = leaf;
                    }
                    ProxyStatus::Created => {
                        let leaf = DBVTLeaf::new(bv, handle);
                        leaves_to_updates[proxy.class as usize].push(leaf);
                        proxy.status = ProxyStatus::Detached(leaves_to_updates.len() - 1);
                    }
                    ProxyStatus::Deleted => {
                        unreachable!("BroadPhaseProxy::remove_updated_leaves()")
                    }
                }
            }
        }
    }

    fn reinsert_updated_leaves(
        &mut self,
        is_move: bool,
        handler: &mut dyn BroadPhaseInterferenceHandler<CollisionObjectSlabHandle>,
    ) -> bool {
        let leaves_to_update: &mut Vec<DBVTLeaf<Fx, BroadPhaseProxyHandle, BV>>;
        let dynamic_tree: &mut FastDBVT<BroadPhaseProxyHandle, BV>;
        if is_move {
            leaves_to_update = &mut self.move_leaves_to_update;
            dynamic_tree = &mut self.chara_tree;
        } else {
            leaves_to_update = &mut self.other_leaves_to_update;
            dynamic_tree = &mut self.hit_tree;
        }

        let some_leaves_updated = leaves_to_update.len() != 0;

        for leaf in self.move_leaves_to_update.drain(..) {
            {
                let proxy1 = &self.proxies[leaf.data.uid()];
                {
                    let mut visitor = BoundingVolumeInterferencesCollector::new(
                        &leaf.bounding_volume,
                        &mut self.collector,
                    );

                    self.stage_tree.visit(&mut visitor);
                    dynamic_tree.visit(&mut visitor);
                }

                // Event generation.
                for proxy_key2 in &self.collector {
                    let proxy2 = &self.proxies[proxy_key2.uid()];

                    if handler.is_interference_allowed(&proxy1.data, &proxy2.data) {
                        match self.pairs.entry(SortedPair::new(leaf.data, *proxy_key2)) {
                            Entry::Occupied(entry) => *entry.into_mut() = true,
                            Entry::Vacant(entry) => {
                                handler.interference_started(&proxy1.data, &proxy2.data);
                                let _ = entry.insert(true);
                            }
                        }
                    }
                }

                self.collector.clear();
            }

            let proxy1 = &mut self.proxies[leaf.data.uid()];
            if let ProxyStatus::Detached(_) = proxy1.status {
                let leaf = dynamic_tree.insert(leaf);
                proxy1.status = ProxyStatus::Attached(leaf);
            } else {
                assert!(false, "status != ProxyStatus::Detached");
            }
        }

        return some_leaves_updated;
    }

    pub fn proxy(
        &self,
        handle: BroadPhaseProxyHandle,
    ) -> Option<(&BV, &CollisionObjectSlabHandle)> {
        let proxy = self.proxies.get(handle.uid())?;
        if let ProxyStatus::Attached(leaf) = proxy.status {
            let lf = match proxy.class {
                CollisionObjectClass::Move => &self.chara_tree[leaf],
                CollisionObjectClass::Hit => &self.hit_tree[leaf],
                CollisionObjectClass::Stage => &self.stage_tree[leaf],
            };
            return Some((&lf.bounding_volume, &proxy.data));
        }
        return None;
    }

    pub fn create_proxy(
        &mut self,
        class: CollisionObjectClass,
        bv: BV,
        data: CollisionObjectSlabHandle,
    ) -> BroadPhaseProxyHandle {
        let proxy = BroadPhaseProxy {
            data,
            class,
            status: ProxyStatus::Created,
            updated: true,
        };
        let handle = BroadPhaseProxyHandle(self.proxies.insert(proxy));
        self.proxies_to_update.push_back((handle, bv));
        return handle;
    }

    pub fn remove(&mut self, handles: &[BroadPhaseProxyHandle]) {
        for handle in handles {
            if let Some(proxy) = self.proxies.get_mut(handle.uid()) {
                if let ProxyStatus::Attached(leaf) = proxy.status {
                    match proxy.class {
                        CollisionObjectClass::Move => self.chara_tree.remove(leaf),
                        CollisionObjectClass::Hit => self.hit_tree.remove(leaf),
                        CollisionObjectClass::Stage => self.stage_tree.remove(leaf),
                    };
                }
                proxy.status = ProxyStatus::Deleted;
            } else {
                unreachable!("BroadPhaseProxy::remove()");
            }
        }

        {
            let proxies = &self.proxies;
            self.pairs.retain(|pair, _| {
                let proxy1 = proxies
                    .get(pair.0.uid())
                    .expect("BroadPhaseProxy::remove()");
                let proxy2 = proxies
                    .get(pair.1.uid())
                    .expect("BroadPhaseProxy::remove()");

                return proxy1.status != ProxyStatus::Deleted
                    && proxy2.status != ProxyStatus::Deleted;
            });
        }

        for handle in handles {
            let _ = self.proxies.remove(handle.uid());
        }
    }

    pub fn deferred_set_bounding_volume(
        &mut self,
        handle: BroadPhaseProxyHandle,
        bounding_volume: BV,
    ) {
        if let Some(proxy) = self.proxies.get(handle.uid()) {
            assert!(proxy.status != ProxyStatus::Deleted);

            let mut needs_update = false;
            if let ProxyStatus::Attached(leaf) = proxy.status {
                let lf = match proxy.class {
                    CollisionObjectClass::Move => &self.chara_tree[leaf],
                    CollisionObjectClass::Hit => &self.hit_tree[leaf],
                    CollisionObjectClass::Stage => &self.stage_tree[leaf],
                };
                needs_update = lf.bounding_volume.contains(&bounding_volume);
            }

            if needs_update {
                let new_bv = bounding_volume.loosened(self.margin);
                self.proxies_to_update.push_back((handle, new_bv));
            }
        } else {
            panic!("BroadPhaseProxy::deferred_set_bounding_volume()");
        }
    }

    pub fn deferred_recompute_all_proximities_with(&mut self, handle: BroadPhaseProxyHandle) {
        if let Some(proxy) = self.proxies.get(handle.uid()) {
            assert!(proxy.status != ProxyStatus::Deleted);

            if let ProxyStatus::Attached(leaf) = proxy.status {
                let lf = match proxy.class {
                    CollisionObjectClass::Move => &self.chara_tree[leaf],
                    CollisionObjectClass::Hit => &self.hit_tree[leaf],
                    CollisionObjectClass::Stage => &self.stage_tree[leaf],
                };
                self.proxies_to_update
                    .push_front((handle, lf.bounding_volume.clone()));
            }
        }
    }

    pub fn deferred_recompute_all_proximities(&mut self) {
        let trees = [
            &mut self.stage_tree,
            &mut self.chara_tree,
            &mut self.hit_tree,
        ];

        for (handle, proxy) in self.proxies.iter() {
            assert!(proxy.status != ProxyStatus::Deleted);

            if let ProxyStatus::Attached(leaf) = proxy.status {
                let lf = &trees[proxy.class as usize][leaf];
                self.proxies_to_update
                    .push_front((BroadPhaseProxyHandle(handle), lf.bounding_volume.clone()));
            }
        }

        self.purge_all = true;
    }
}

//
// Interference with shape
//

impl<BV> DBVTBroadPhase<BV>
where
    BV: BoundingVolume<Fx> + RayCast<Fx> + PointQuery<Fx> + Any + Send + Sync + Clone,
{
    pub fn interferences_with_point<'a>(
        &'a mut self,
        class: CollisionObjectClass,
        point: &Point<Fx>,
        out: &mut Vec<CollisionObjectSlabHandle>,
    ) {
        assert!(self.collector.is_empty());

        {
            let mut visitor = PointInterferencesCollector::new(point, &mut self.collector);
            match class {
                CollisionObjectClass::Stage => {
                    self.stage_tree.visit(&mut visitor);
                }
                CollisionObjectClass::Move => {
                    self.stage_tree.visit(&mut visitor);
                    self.chara_tree.visit(&mut visitor);
                }
                CollisionObjectClass::Hit => {
                    self.stage_tree.visit(&mut visitor);
                    self.hit_tree.visit(&mut visitor);
                }
            }
        }

        for l in &self.collector {
            out.push(self.proxies[l.uid()].data);
        }
        self.collector.clear();
    }

    pub fn interferences_with_aabb<'a>(
        &'a mut self,
        class: CollisionObjectClass,
        bv: &BV,
        out: &mut Vec<CollisionObjectSlabHandle>,
    ) {
        assert!(self.collector.is_empty());

        {
            let mut visitor = BoundingVolumeInterferencesCollector::new(bv, &mut self.collector);
            match class {
                CollisionObjectClass::Stage => {
                    self.stage_tree.visit(&mut visitor);
                }
                CollisionObjectClass::Move => {
                    self.stage_tree.visit(&mut visitor);
                    self.chara_tree.visit(&mut visitor);
                }
                CollisionObjectClass::Hit => {
                    self.stage_tree.visit(&mut visitor);
                    self.hit_tree.visit(&mut visitor);
                }
            }
        }

        for l in &self.collector {
            out.push(self.proxies[l.uid()].data);
        }
        self.collector.clear();
    }

    pub fn interferences_with_ray<'a>(
        &'a mut self,
        class: CollisionObjectClass,
        ray: &Ray<Fx>,
        max_toi: Fx,
        out: &mut Vec<CollisionObjectSlabHandle>,
    ) {
        assert!(self.collector.is_empty());

        {
            let mut visitor = RayInterferencesCollector::new(ray, max_toi, &mut self.collector);
            match class {
                CollisionObjectClass::Stage => {
                    self.stage_tree.visit(&mut visitor);
                }
                CollisionObjectClass::Move => {
                    self.stage_tree.visit(&mut visitor);
                    self.chara_tree.visit(&mut visitor);
                }
                CollisionObjectClass::Hit => {
                    self.stage_tree.visit(&mut visitor);
                    self.hit_tree.visit(&mut visitor);
                }
            }
        }

        for l in &self.collector {
            out.push(self.proxies[l.uid()].data);
        }
        self.collector.clear();
    }

    pub fn first_interference_with_ray<'a, 'b>(
        &'a mut self,
        class: CollisionObjectClass,
        ray: &'b Ray<Fx>,
        max_toi: Fx,
        cost_fn: &'a dyn Fn(
            CollisionObjectSlabHandle,
            &'b Ray<Fx>,
            Fx,
        ) -> Option<(CollisionObjectSlabHandle, RayIntersection<Fx>)>,
    ) -> Option<(CollisionObjectSlabHandle, RayIntersection<Fx>)> {
        let mut res;
        {
            let this = unsafe { &*(self as *const _) };
            let mut visitor =
                RayIntersectionCostFnVisitor::<'a, 'b, Fx, CollisionObjectSlabHandle, BV>::new(
                    ray, max_toi, this, cost_fn,
                );

            match class {
                CollisionObjectClass::Stage => {
                    res = self.stage_tree.best_first_search(&mut visitor);
                }
                CollisionObjectClass::Move => {
                    res = self.stage_tree.best_first_search(&mut visitor);
                    if res.is_none() {
                        res = self.chara_tree.best_first_search(&mut visitor);
                    }
                }
                CollisionObjectClass::Hit => {
                    res = self.stage_tree.best_first_search(&mut visitor);
                    if res.is_none() {
                        res = self.hit_tree.best_first_search(&mut visitor);
                    }
                }
            }
        };

        return res.map(|(_, res)| res);
    }
}

use CollisionObjectSlabHandle as H;

impl<BV> BroadPhase<Fx, BV, CollisionObjectSlabHandle> for DBVTBroadPhase<BV>
where
    BV: BoundingVolume<Fx> + RayCast<Fx> + PointQuery<Fx> + Any + Send + Sync + Clone,
{
    fn proxy(&self, handle: BroadPhaseProxyHandle) -> Option<(&BV, &H)> {
        return DBVTBroadPhase::proxy(&self, handle);
    }

    fn create_proxy(&mut self, _bv: BV, _data: H) -> BroadPhaseProxyHandle {
        unimplemented!()
    }

    fn remove(&mut self, _1: &[BroadPhaseProxyHandle], _2: &mut dyn FnMut(&H, &H)) {
        unimplemented!()
    }

    fn deferred_set_bounding_volume(&mut self, _1: BroadPhaseProxyHandle, _2: BV) {
        unimplemented!()
    }

    fn deferred_recompute_all_proximities_with(&mut self, _1: BroadPhaseProxyHandle) {
        unimplemented!()
    }

    fn deferred_recompute_all_proximities(&mut self) {
        unimplemented!()
    }

    fn update(&mut self, _1: &mut dyn BroadPhaseInterferenceHandler<H>) {
        unimplemented!()
    }

    fn interferences_with_bounding_volume<'a>(&'a self, _1: &BV, _2: &mut Vec<&'a H>) {
        unimplemented!()
    }

    fn interferences_with_ray<'a>(&'a self, _1: &Ray<Fx>, _2: Fx, _3: &mut Vec<&'a H>) {
        unimplemented!()
    }

    fn interferences_with_point<'a>(&'a self, _1: &Point<Fx>, _2: &mut Vec<&'a H>) {
        unimplemented!()
    }

    fn first_interference_with_ray<'a, 'b>(
        &'a self,
        _1: &'b Ray<Fx>,
        _2: Fx,
        _3: &'a dyn Fn(H, &'b Ray<Fx>, Fx) -> Option<(H, RayIntersection<Fx>)>,
    ) -> Option<(H, RayIntersection<Fx>)> {
        unimplemented!()
    }
}
