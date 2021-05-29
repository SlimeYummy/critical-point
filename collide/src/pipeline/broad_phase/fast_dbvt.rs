use math::{Fx, RealExt};
use ncollide3d::bounding_volume::BoundingVolume;
use ncollide3d::partitioning::{
    BestFirstVisitStatus, BestFirstVisitor, DBVTLeaf, DBVTLeafId, DBVTNodeId, VisitStatus, Visitor,
    BVH, DBVT,
};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::ops::Index;

pub struct FastDBVT<T, BV> {
    pub dbvt: DBVT<Fx, T, BV>,
    stack: Vec<DBVTNodeId>,
    queue: BinaryHeap<WeightedValue<Fx, DBVTNodeId>>,
}

impl<T, BV> FastDBVT<T, BV>
where
    BV: 'static + BoundingVolume<Fx> + Clone,
{
    pub fn new() -> FastDBVT<T, BV> {
        return FastDBVT {
            dbvt: DBVT::new(),
            stack: Vec::new(),
            queue: BinaryHeap::new(),
        };
    }

    pub fn insert(&mut self, leaf: DBVTLeaf<Fx, T, BV>) -> DBVTLeafId {
        return self.dbvt.insert(leaf);
    }

    pub fn remove(&mut self, leaf_id: DBVTLeafId) -> DBVTLeaf<Fx, T, BV> {
        return self.dbvt.remove(leaf_id);
    }

    pub fn visit(&mut self, visitor: &mut impl Visitor<T, BV>) {
        assert!(self.stack.is_empty());

        if let Some(root) = self.dbvt.root() {
            self.stack.push(root);

            while let Some(node) = self.stack.pop() {
                let content = self.dbvt.content(node);

                match visitor.visit(content.0, content.1) {
                    VisitStatus::Continue => {
                        for i in 0..self.dbvt.num_children(node) {
                            self.stack.push(self.dbvt.child(i, node))
                        }
                    }
                    VisitStatus::ExitEarly => return,
                    VisitStatus::Stop => {}
                }
            }
        }
        self.stack.clear();
    }

    pub fn best_first_search<BFS>(&mut self, visitor: &mut BFS) -> Option<(DBVTNodeId, BFS::Result)>
    where
        BFS: BestFirstVisitor<Fx, T, BV>,
    {
        assert!(self.queue.is_empty());

        // The lowest cost collision with actual scene geometry.
        let mut best_cost = Fx::max_value();
        let mut best_result = None;

        if let Some(root) = self.dbvt.root() {
            let (root_bv, root_data) = self.dbvt.content(root);

            match visitor.visit(best_cost, root_bv, root_data) {
                BestFirstVisitStatus::Continue { cost, result } => {
                    // Root may be a leaf node
                    if let Some(res) = result {
                        best_cost = cost;
                        best_result = Some((root, res));
                    }

                    self.queue.push(WeightedValue::new(root, -cost))
                }
                BestFirstVisitStatus::Stop => return None,
                BestFirstVisitStatus::ExitEarly(result) => return result.map(|res| (root, res)),
            }

            while let Some(entry) = self.queue.pop() {
                if -entry.cost >= best_cost {
                    // No BV left in the tree that has a lower cost than best_result
                    break; // Solution found.
                }

                for i in 0..self.dbvt.num_children(entry.value) {
                    let child = self.dbvt.child(i, entry.value);
                    let (child_bv, child_data) = self.dbvt.content(child);

                    match visitor.visit(best_cost, child_bv, child_data) {
                        BestFirstVisitStatus::Continue { cost, result } => {
                            if cost < best_cost {
                                if result.is_some() {
                                    // This is the nearest collision so far
                                    best_cost = cost;
                                    best_result = result.map(|res| (child, res));
                                }
                                // BV may have a child with lower cost, evaluate it next.
                                self.queue.push(WeightedValue::new(child, -cost))
                            }
                        }
                        BestFirstVisitStatus::ExitEarly(result) => {
                            return result.map(|res| (child, res)).or(best_result)
                        }
                        BestFirstVisitStatus::Stop => {}
                    }
                }
            }
        }

        self.queue.clear();
        return best_result;
    }
}

impl<T, BV> Index<DBVTLeafId> for FastDBVT<T, BV> {
    type Output = DBVTLeaf<Fx, T, BV>;

    #[inline]
    fn index(&self, id: DBVTLeafId) -> &Self::Output {
        return &self.dbvt[id];
    }
}

struct WeightedValue<N, T> {
    value: T,
    cost: N,
}

impl<N, T> WeightedValue<N, T> {
    /// Creates a new reference packed with a cost value.
    #[inline]
    fn new(value: T, cost: N) -> WeightedValue<N, T> {
        WeightedValue {
            value: value,
            cost: cost,
        }
    }
}

impl<N: PartialEq, T> PartialEq for WeightedValue<N, T> {
    #[inline]
    fn eq(&self, other: &WeightedValue<N, T>) -> bool {
        self.cost.eq(&other.cost)
    }
}

impl<N: PartialEq, T> Eq for WeightedValue<N, T> {}

impl<N: PartialOrd, T> PartialOrd for WeightedValue<N, T> {
    #[inline]
    fn partial_cmp(&self, other: &WeightedValue<N, T>) -> Option<Ordering> {
        self.cost.partial_cmp(&other.cost)
    }
}

impl<N: PartialOrd, T> Ord for WeightedValue<N, T> {
    #[inline]
    fn cmp(&self, other: &WeightedValue<N, T>) -> Ordering {
        if self.cost < other.cost {
            Ordering::Less
        } else if self.cost > other.cost {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}
