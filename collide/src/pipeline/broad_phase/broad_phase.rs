use super::fast_dbvt::FastDBVT;
use crate::pipeline::object::{CollisionObjectSlabHandle, CollisionObjectType as CoType};
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
    typ: CoType,
    status: ProxyStatus,
    updated: bool,
}

pub struct DBVTBroadPhase<BV> {
    proxies: Slab<BroadPhaseProxy>,
    trees: [FastDBVT<BroadPhaseProxyHandle, BV>; 3],
    pairs: HashMap<SortedPair<BroadPhaseProxyHandle>, bool, DeterministicState>,
    margin: Fx,
    purge_all: bool,

    // Just to avoid dynamic allocations
    collector: Vec<BroadPhaseProxyHandle>,
    leaves_to_update: Vec<DBVTLeaf<Fx, BroadPhaseProxyHandle, BV>>,
    proxies_to_updates: [VecDeque<(BroadPhaseProxyHandle, BV)>; 3],
}

impl<BV> DBVTBroadPhase<BV>
where
    BV: 'static + BoundingVolume<Fx> + Clone,
{
    pub fn new(margin: Fx) -> DBVTBroadPhase<BV> {
        return DBVTBroadPhase {
            proxies: Slab::new(),
            trees: [FastDBVT::new(), FastDBVT::new(), FastDBVT::new()],
            pairs: HashMap::with_hasher(DeterministicState::new()),
            purge_all: false,
            collector: Vec::new(),
            leaves_to_update: Vec::new(),
            proxies_to_updates: [VecDeque::new(), VecDeque::new(), VecDeque::new()],
            margin,
        };
    }

    pub fn update(
        &mut self,
        types: &[CoType],
        handler: &mut dyn BroadPhaseInterferenceHandler<CollisionObjectSlabHandle>,
    ) {
        for typ in types {
            self.remove_updated_leaves(*typ);
        }

        let mut updated = false;
        for typ in types {
            self.reinsert_updated_leaves(*typ, handler);
        }

        self.purge_some_contact_pairs(handler);
    }

    fn purge_some_contact_pairs(
        &mut self,
        handler: &mut dyn BroadPhaseInterferenceHandler<CollisionObjectSlabHandle>,
    ) {
        let purge_all = self.purge_all;
        let proxies = &self.proxies;
        let trees = &mut self.trees;
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
                            ProxyStatus::Attached(leaf) => &trees[proxy1.typ as usize][leaf],
                            _ => unreachable!("BroadPhaseProxy::purge_some_contact_pairs()"),
                        };

                        let l2 = match proxy2.status {
                            ProxyStatus::Attached(leaf) => &trees[proxy2.typ as usize][leaf],
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

    fn remove_updated_leaves(&mut self, typ: CoType) {
        let tree = &mut self.trees[typ as usize];
        let proxies_to_update = &mut self.proxies_to_updates[typ as usize];

        for (handle, bv) in proxies_to_update.drain(..) {
            if let Some(proxy) = self.proxies.get_mut(handle.uid()) {
                match proxy.status {
                    ProxyStatus::Attached(leaf) => {
                        let mut leaf = tree.remove(leaf);
                        leaf.bounding_volume = bv;
                        self.leaves_to_update.push(leaf);
                        proxy.status = ProxyStatus::Detached(self.leaves_to_update.len() - 1);
                    }
                    ProxyStatus::Detached(id) => {
                        let leaf = DBVTLeaf::new(bv, handle);
                        self.leaves_to_update[id] = leaf;
                    }
                    ProxyStatus::Created => {
                        let leaf = DBVTLeaf::new(bv, handle);
                        self.leaves_to_update.push(leaf);
                        proxy.status = ProxyStatus::Detached(self.leaves_to_update.len() - 1);
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
        typ: CoType,
        handler: &mut dyn BroadPhaseInterferenceHandler<CollisionObjectSlabHandle>,
    ) -> bool {
        let some_leaves_updated = self.leaves_to_update.len() != 0;

        for leaf in self.leaves_to_update.drain(..) {
            {
                let proxy1 = &self.proxies[leaf.data.uid()];
                {
                    let mut visitor = BoundingVolumeInterferencesCollector::new(
                        &leaf.bounding_volume,
                        &mut self.collector,
                    );

                    match typ {
                        CoType::Static => {
                            self.trees[CoType::Move as usize].visit(&mut visitor);
                            self.trees[CoType::Hit as usize].visit(&mut visitor);
                        }
                        CoType::Move => {
                            self.trees[CoType::Static as usize].visit(&mut visitor);
                            self.trees[CoType::Move as usize].visit(&mut visitor);
                        }
                        CoType::Hit => {
                            self.trees[CoType::Static as usize].visit(&mut visitor);
                            self.trees[CoType::Hit as usize].visit(&mut visitor);
                        }
                    }
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
                let leaf = self.trees[typ as usize].insert(leaf);
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
            let lf = self.trees[proxy.typ as usize].get(leaf)?;
            return Some((&lf.bounding_volume, &proxy.data));
        }
        return None;
    }

    pub fn create_proxy(
        &mut self,
        typ: CoType,
        bv: BV,
        data: CollisionObjectSlabHandle,
    ) -> BroadPhaseProxyHandle {
        let proxy = BroadPhaseProxy {
            data,
            typ,
            status: ProxyStatus::Created,
            updated: true,
        };
        let handle = BroadPhaseProxyHandle(self.proxies.insert(proxy));
        self.proxies_to_updates[typ as usize].push_back((handle, bv));
        return handle;
    }

    pub fn remove(&mut self, handles: &[BroadPhaseProxyHandle]) {
        for handle in handles {
            if let Some(proxy) = self.proxies.get_mut(handle.uid()) {
                if let ProxyStatus::Attached(leaf) = proxy.status {
                    self.trees[proxy.typ as usize].remove(leaf);
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
            let needs_update = match proxy.status {
                ProxyStatus::Attached(leaf) => self.trees[proxy.typ as usize][leaf]
                    .bounding_volume
                    .contains(&bounding_volume),
                ProxyStatus::Detached(_) | ProxyStatus::Created => true,
                ProxyStatus::Deleted => {
                    panic!("DBVT broad phase: internal error, proxy not found.")
                }
            };

            if needs_update {
                let new_bv = bounding_volume.loosened(self.margin);
                self.proxies_to_updates[proxy.typ as usize].push_back((handle, new_bv));
            }
        } else {
            panic!("BroadPhaseProxy::deferred_set_bounding_volume()");
        }
    }

    pub fn deferred_recompute_all_proximities_with(&mut self, handle: BroadPhaseProxyHandle) {
        if let Some(proxy) = self.proxies.get(handle.uid()) {
            let bv = match proxy.status {
                ProxyStatus::Attached(leaf) => {
                    self.trees[proxy.typ as usize][leaf].bounding_volume.clone()
                }
                ProxyStatus::Detached(_) | ProxyStatus::Created => return,
                ProxyStatus::Deleted => {
                    panic!("DBVT broad phase: internal error, proxy not found.")
                }
            };

            self.proxies_to_updates[proxy.typ as usize].push_front((handle, bv));
        }
    }

    pub fn deferred_recompute_all_proximities(&mut self) {
        for (handle, proxy) in self.proxies.iter() {
            let bv = match proxy.status {
                ProxyStatus::Attached(leaf) => {
                    self.trees[proxy.typ as usize][leaf].bounding_volume.clone()
                }
                ProxyStatus::Detached(_) | ProxyStatus::Created => continue,
                ProxyStatus::Deleted => {
                    panic!("DBVT broad phase: internal error, proxy not found.")
                }
            };

            self.proxies_to_updates[proxy.typ as usize]
                .push_front((BroadPhaseProxyHandle(handle), bv));
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
        typ: CoType,
        point: &Point<Fx>,
        out: &mut Vec<CollisionObjectSlabHandle>,
    ) {
        assert!(self.collector.is_empty());

        {
            let mut visitor = PointInterferencesCollector::new(point, &mut self.collector);
            let trees = &mut self.trees;
            match typ {
                CoType::Static => {
                    trees[CoType::Static as usize].visit(&mut visitor);
                }
                CoType::Move => {
                    trees[CoType::Static as usize].visit(&mut visitor);
                    trees[CoType::Move as usize].visit(&mut visitor);
                }
                CoType::Hit => {
                    trees[CoType::Static as usize].visit(&mut visitor);
                    trees[CoType::Hit as usize].visit(&mut visitor);
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
        typ: CoType,
        bv: &BV,
        out: &mut Vec<CollisionObjectSlabHandle>,
    ) {
        assert!(self.collector.is_empty());

        {
            let mut visitor = BoundingVolumeInterferencesCollector::new(bv, &mut self.collector);
            let trees = &mut self.trees;
            match typ {
                CoType::Static => {
                    trees[CoType::Static as usize].visit(&mut visitor);
                }
                CoType::Move => {
                    trees[CoType::Static as usize].visit(&mut visitor);
                    trees[CoType::Move as usize].visit(&mut visitor);
                }
                CoType::Hit => {
                    trees[CoType::Static as usize].visit(&mut visitor);
                    trees[CoType::Hit as usize].visit(&mut visitor);
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
        typ: CoType,
        ray: &Ray<Fx>,
        max_toi: Fx,
        out: &mut Vec<CollisionObjectSlabHandle>,
    ) {
        assert!(self.collector.is_empty());

        {
            let mut visitor = RayInterferencesCollector::new(ray, max_toi, &mut self.collector);
            let trees = &mut self.trees;
            match typ {
                CoType::Static => {
                    trees[CoType::Static as usize].visit(&mut visitor);
                }
                CoType::Move => {
                    trees[CoType::Static as usize].visit(&mut visitor);
                    trees[CoType::Move as usize].visit(&mut visitor);
                }
                CoType::Hit => {
                    trees[CoType::Static as usize].visit(&mut visitor);
                    trees[CoType::Hit as usize].visit(&mut visitor);
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
        typ: CoType,
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

            match typ {
                CoType::Static => {
                    res = self.trees[CoType::Static as usize].best_first_search(&mut visitor);
                }
                CoType::Move => {
                    res = self.trees[CoType::Static as usize].best_first_search(&mut visitor);
                    if res.is_none() {
                        res = self.trees[CoType::Move as usize].best_first_search(&mut visitor);
                    }
                }
                CoType::Hit => {
                    res = self.trees[CoType::Static as usize].best_first_search(&mut visitor);
                    if res.is_none() {
                        res = self.trees[CoType::Hit as usize].best_first_search(&mut visitor);
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
