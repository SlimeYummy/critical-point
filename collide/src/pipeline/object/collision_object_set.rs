use super::collision_object::CollisionObject;
use math::Fx;
use ncollide3d::pipeline::{CollisionObjectSet, CollisionObjectSlabHandle};
use slab::{Iter, IterMut, Slab};
use std::ops::{Index, IndexMut};

pub struct CollisionObjectSlab<T> {
    pub(crate) objects: Slab<CollisionObject<T>>,
}

impl<T> CollisionObjectSet<Fx> for CollisionObjectSlab<T> {
    type CollisionObject = CollisionObject<T>;
    type CollisionObjectHandle = CollisionObjectSlabHandle;

    fn collision_object(
        &self,
        handle: Self::CollisionObjectHandle,
    ) -> Option<&Self::CollisionObject> {
        return self.get(handle);
    }

    fn foreach(&self, mut f: impl FnMut(Self::CollisionObjectHandle, &Self::CollisionObject)) {
        for co in self.objects.iter() {
            f(CollisionObjectSlabHandle(co.0), co.1)
        }
    }
}

impl<T> CollisionObjectSlab<T> {
    pub fn new() -> CollisionObjectSlab<T> {
        return CollisionObjectSlab {
            objects: Slab::new(),
        };
    }

    pub fn with_capacity(capacity: usize) -> CollisionObjectSlab<T> {
        return CollisionObjectSlab {
            objects: Slab::with_capacity(capacity),
        };
    }

    #[inline]
    pub fn insert(&mut self, co: CollisionObject<T>) -> CollisionObjectSlabHandle {
        return CollisionObjectSlabHandle(self.objects.insert(co));
    }

    #[inline]
    pub fn remove(&mut self, handle: CollisionObjectSlabHandle) -> CollisionObject<T> {
        return self.objects.remove(handle.0);
    }

    #[inline]
    pub fn get(&self, handle: CollisionObjectSlabHandle) -> Option<&CollisionObject<T>> {
        return self.objects.get(handle.0);
    }

    #[inline]
    pub fn get_mut(
        &mut self,
        handle: CollisionObjectSlabHandle,
    ) -> Option<&mut CollisionObject<T>> {
        return self.objects.get_mut(handle.0);
    }

    #[inline]
    pub fn get_pair_mut(
        &mut self,
        handle1: CollisionObjectSlabHandle,
        handle2: CollisionObjectSlabHandle,
    ) -> (
        Option<&mut CollisionObject<T>>,
        Option<&mut CollisionObject<T>>,
    ) {
        assert_ne!(handle1, handle2, "The two handles must not be the same.");
        let a = self.objects.get_mut(handle1.0).map(|o| o as *mut _);
        return (
            a.map(|a| unsafe { std::mem::transmute(a) }),
            self.objects.get_mut(handle2.0),
        );
    }

    #[inline]
    pub fn contains(&self, handle: CollisionObjectSlabHandle) -> bool {
        return self.objects.contains(handle.0);
    }

    #[inline]
    pub fn iter(&self) -> CollisionObjects<T> {
        return CollisionObjects {
            iter: self.objects.iter(),
        };
    }

    #[inline]
    pub fn iter_mut(&mut self) -> CollisionObjectsMut<T> {
        return CollisionObjectsMut {
            iter_mut: self.objects.iter_mut(),
        };
    }

    #[inline]
    pub fn len(&self) -> usize {
        return self.objects.len();
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        return self.objects.capacity();
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.objects.reserve(additional);
    }

    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.objects.reserve_exact(additional);
    }
}

impl<T> Index<CollisionObjectSlabHandle> for CollisionObjectSlab<T> {
    type Output = CollisionObject<T>;

    #[inline]
    fn index(&self, handle: CollisionObjectSlabHandle) -> &Self::Output {
        return &self.objects[handle.0];
    }
}

impl<T> IndexMut<CollisionObjectSlabHandle> for CollisionObjectSlab<T> {
    #[inline]
    fn index_mut(&mut self, handle: CollisionObjectSlabHandle) -> &mut Self::Output {
        return &mut self.objects[handle.0];
    }
}

pub struct CollisionObjects<'a, T: 'a> {
    iter: Iter<'a, CollisionObject<T>>,
}

impl<'a, T: 'a> Iterator for CollisionObjects<'a, T> {
    type Item = (CollisionObjectSlabHandle, &'a CollisionObject<T>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        return self
            .iter
            .next()
            .map(|obj| ((CollisionObjectSlabHandle(obj.0), obj.1)));
    }
}

pub struct CollisionObjectsMut<'a, T: 'a> {
    iter_mut: IterMut<'a, CollisionObject<T>>,
}

impl<'a, T: 'a> Iterator for CollisionObjectsMut<'a, T> {
    type Item = (CollisionObjectSlabHandle, &'a mut CollisionObject<T>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        return self
            .iter_mut
            .next()
            .map(|obj| ((CollisionObjectSlabHandle(obj.0), obj.1)));
    }
}
