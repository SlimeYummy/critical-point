use std::alloc::{self, Layout};
use std::cell::Cell;
use std::error::Error;
use std::fmt::{self, Debug, Display};
use std::marker::{PhantomData, Unsize};
use std::mem;
use std::ops::{CoerceUnsized, Deref, DerefMut, DispatchFromDyn};
use std::ptr::{self, NonNull};

//
// RcCellBox
//

#[repr(C)]
struct RcCellBox<T: ?Sized> {
    strong: Cell<usize>,
    weak: Cell<usize>,
    borrow: Cell<isize>,
    value: T,
}

impl<T: ?Sized> RcCellBox<T> {
    #[inline]
    fn inc_strong(&self) -> usize {
        self.strong.set(self.strong.get() + 1);
        return self.strong.get();
    }

    #[inline]
    fn dec_strong(&self) -> usize {
        self.strong.set(self.strong.get() - 1);
        return self.strong.get();
    }

    #[inline]
    fn inc_weak(&self) -> usize {
        self.weak.set(self.weak.get() + 1);
        return self.weak.get();
    }

    #[inline]
    fn dec_weak(&self) -> usize {
        self.weak.set(self.weak.get() - 1);
        return self.weak.get();
    }
}

//
// RcCell: reference count
//

pub struct RcCell<T: ?Sized> {
    ptr: NonNull<RcCellBox<T>>,
    phantom: PhantomData<RcCellBox<T>>,
}

impl<T> !Sync for RcCell<T> {}
impl<T> !Send for RcCell<T> {}

impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<RcCell<U>> for RcCell<T> {}
impl<T: ?Sized + Unsize<U>, U: ?Sized> DispatchFromDyn<RcCell<U>> for RcCell<T> {}

impl<T> RcCell<T> {
    pub fn new(value: T) -> RcCell<T> {
        return RcCell {
            ptr: Box::leak(Box::new(RcCellBox {
                strong: Cell::new(1),
                weak: Cell::new(1), // acquire "strong weak" pointer
                borrow: Cell::new(0),
                value,
            }))
            .into(),
            phantom: PhantomData,
        };
    }
}

impl<T: ?Sized> Clone for RcCell<T> {
    #[inline]
    fn clone(&self) -> RcCell<T> {
        self.inner().inc_strong();
        return RcCell {
            ptr: self.ptr,
            phantom: PhantomData,
        };
    }
}

impl<T: ?Sized> Drop for RcCell<T> {
    fn drop(&mut self) {
        if self.inner().dec_strong() == 0 {
            unsafe { ptr::drop_in_place(self.ptr.as_ptr()) };
            if self.inner().dec_weak() == 0 {
                let ptr = self.ptr.as_ptr() as *mut u8;
                unsafe { alloc::dealloc(ptr, Layout::for_value(self.ptr.as_ref())) };
            }
        }
    }
}

impl<T: ?Sized> RcCell<T> {
    #[inline]
    pub fn strong_count(&self) -> usize {
        return self.inner().strong.get();
    }

    #[inline]
    pub fn weak_count(&self) -> usize {
        return self.inner().weak.get() - 1;
    }

    #[inline]
    pub fn downgrade(&self) -> WeakCell<T> {
        self.inner().inc_weak();
        debug_assert!(!is_dangling(self.ptr));
        return WeakCell { ptr: self.ptr };
    }

    #[inline]
    fn inner(&self) -> &RcCellBox<T> {
        return unsafe { &*self.ptr.as_ref() };
    }
}

//
// WeakCell
//

pub struct WeakCell<T: ?Sized> {
    ptr: NonNull<RcCellBox<T>>,
}

impl<T> !Sync for WeakCell<T> {}
impl<T> !Send for WeakCell<T> {}

impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<WeakCell<U>> for WeakCell<T> {}
impl<T: ?Sized + Unsize<U>, U: ?Sized> DispatchFromDyn<WeakCell<U>> for WeakCell<T> {}

impl<T> WeakCell<T> {
    pub fn new() -> WeakCell<T> {
        return WeakCell {
            ptr: NonNull::new(usize::MAX as *mut RcCellBox<T>).expect("MAX is not 0"),
        };
    }
}

impl<T: ?Sized> Clone for WeakCell<T> {
    #[inline]
    fn clone(&self) -> WeakCell<T> {
        if let Ok(inner) = self.inner() {
            inner.inc_strong();
        }
        return WeakCell { ptr: self.ptr };
    }
}

impl<T: ?Sized> Drop for WeakCell<T> {
    fn drop(&mut self) {
        if let Ok(inner) = self.inner() {
            if inner.dec_weak() == 0 {
                let ptr = self.ptr.as_ptr() as *mut u8;
                unsafe { alloc::dealloc(ptr, Layout::for_value(self.ptr.as_ref())) };
            }
        }
    }
}

impl<T: ?Sized> WeakCell<T> {
    #[inline]
    pub fn strong_count(&self) -> usize {
        if let Ok(inner) = self.inner() {
            return inner.strong.get();
        }
        return 0;
    }

    #[inline]
    pub fn weak_count(&self) -> usize {
        if let Ok(inner) = self.inner() {
            if inner.strong.get() > 0 {
                return inner.weak.get() - 1;
            }
        }
        return 0;
    }

    #[inline]
    pub fn upgrade(&self) -> Result<RcCell<T>, RcCellError> {
        if self.inner()?.strong.get() > 0 {
            self.inner()?.inc_strong();
            return Ok(RcCell {
                ptr: self.ptr,
                phantom: PhantomData,
            });
        }
        return Err(RcCellError::DanglingPtr);
    }

    #[inline]
    fn inner(&self) -> Result<&RcCellBox<T>, RcCellError> {
        if !is_dangling(self.ptr) {
            return Ok(unsafe { self.ptr.as_ref() });
        } else {
            return Err(RcCellError::DanglingPtr);
        }
    }
}

//
// RcCell: borrow
//

impl<T: ?Sized> RcCell<T> {
    #[inline]
    fn get_borrow(&self) -> isize {
        return self.inner().borrow.get();
    }

    #[inline]
    fn set_borrow(&self, borrow: isize) {
        self.inner().borrow.set(borrow);
    }

    #[inline]
    pub unsafe fn as_ptr_mut(this: &Self) -> *mut T {
        let ptr = this.ptr.as_ptr();
        let fake_ptr = ptr as *mut T;
        let offset = data_offset::<RcCellBox<T>, RcCellBox<()>>(this.inner());
        return set_data_ptr(fake_ptr, (ptr as *mut u8).offset(offset));
    }

    #[inline]
    pub unsafe fn as_ref_mut(this: &Self) -> &mut T {
        return &mut *RcCell::as_ptr_mut(this);
    }
}

pub struct RcCellRef<'b, T: ?Sized + 'b> {
    borrow: &'b Cell<isize>,
    value: &'b T,
}

impl<T: ?Sized> Drop for RcCellRef<'_, T> {
    fn drop(&mut self) {
        self.borrow.set(self.borrow.get() - 1);
    }
}

impl<T: ?Sized> Deref for RcCellRef<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        return self.value;
    }
}

impl<T: ?Sized> RcCell<T> {
    #[inline]
    pub fn try_borrow(&self) -> Result<RcCellRef<'_, T>, RcCellError> {
        let borrow = self.get_borrow() + 1;
        if borrow > 0 {
            self.set_borrow(borrow);
            return Ok(RcCellRef {
                borrow: &self.inner().borrow,
                value: unsafe { RcCell::as_ref_mut(self) },
            });
        } else {
            return Err(RcCellError::MutBorrowed);
        }
    }

    #[inline]
    pub fn borrow(&self) -> RcCellRef<'_, T> {
        return self.try_borrow().expect("already mutably borrowed");
    }
}

pub struct RcCellRefMut<'b, T: ?Sized + 'b> {
    borrow: &'b Cell<isize>,
    value: &'b mut T,
}

impl<T: ?Sized> Drop for RcCellRefMut<'_, T> {
    fn drop(&mut self) {
        self.borrow.set(0);
    }
}

impl<T: ?Sized> Deref for RcCellRefMut<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        return self.value;
    }
}

impl<T: ?Sized> DerefMut for RcCellRefMut<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        return self.value;
    }
}

impl<T: ?Sized> RcCell<T> {
    #[inline]
    pub fn try_borrow_mut(&self) -> Result<RcCellRefMut<'_, T>, RcCellError> {
        if self.get_borrow() == 0 {
            self.set_borrow(-1);
            return Ok(RcCellRefMut {
                borrow: &self.inner().borrow,
                value: unsafe { RcCell::as_ref_mut(self) },
            });
        } else {
            return Err(RcCellError::MutBorrowed);
        }
    }

    #[inline]
    pub fn borrow_mut(&self) -> RcCellRefMut<'_, T> {
        return self.try_borrow_mut().expect("already mutably borrowed");
    }
}

//
// RcCellError
//

#[derive(Debug)]
pub enum RcCellError {
    DanglingPtr,
    MutBorrowed,
}

impl Error for RcCellError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        return None;
    }
}

impl Display for RcCellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return match self {
            RcCellError::DanglingPtr => Display::fmt("dangling pointer", f),
            RcCellError::MutBorrowed => Display::fmt("already mutably borrowed", f),
        };
    }
}

//
// utils
//

unsafe fn set_data_ptr<T: ?Sized, U>(mut ptr: *mut T, data: *mut U) -> *mut T {
    let pptr = (&mut ptr as *mut _) as *mut *mut u8;
    ptr::write(pptr, data as *mut u8);
    return ptr;
}

unsafe fn data_offset<T: ?Sized, B>(ptr: *const T) -> isize {
    let align = mem::align_of_val(&*ptr);
    let layout = Layout::new::<B>();
    return (layout.size() + layout.padding_needed_for(align)) as isize;
}

fn is_dangling<T: ?Sized>(ptr: NonNull<T>) -> bool {
    let address = ptr.as_ptr() as *mut () as usize;
    return address == usize::MAX;
}

#[cfg(test)]
mod tests {
    use super::*;

    trait T {
        fn f(&self) -> i32;
    }

    #[derive(Debug)]
    struct S {}

    impl T for S {
        fn f(&self) -> i32 {
            10
        }
    }

    #[test]
    fn test_rc_cell_refcount() {
        let mut weak: WeakCell<S> = WeakCell::<S>::new();
        assert_eq!(weak.strong_count(), 0);
        assert_eq!(weak.weak_count(), 0);

        {
            let rc1: RcCell<S> = RcCell::<S>::new(S {});
            assert_eq!(rc1.strong_count(), 1);
            assert_eq!(rc1.weak_count(), 0);

            weak = rc1.downgrade();
            assert_eq!(weak.strong_count(), 1);
            assert_eq!(weak.weak_count(), 1);

            let rc2: RcCell<S> = weak.upgrade().unwrap();
            assert_eq!(rc2.strong_count(), 2);
            assert_eq!(rc2.weak_count(), 1);
        }

        assert_eq!(weak.strong_count(), 0);
        assert_eq!(weak.weak_count(), 0);
        assert!(weak.upgrade().is_err());
    }

    #[test]
    fn test_rc_cell_borrow_const() {
        let rc: RcCell<S> = RcCell::<S>::new(S {});
        let b1 = rc.try_borrow();
        assert!(b1.is_ok());
        let b2 = rc.try_borrow();
        assert!(b2.is_ok());
    }

    #[test]
    fn test_rc_cell_borrow_mut() {
        let rc: RcCell<S> = RcCell::<S>::new(S {});

        {
            let b1 = rc.try_borrow_mut();
            assert!(b1.is_ok());
            let b2 = rc.try_borrow_mut();
            assert!(b2.is_err());
        }

        {
            let b1 = rc.try_borrow();
            assert!(b1.is_ok());
            let b2 = rc.try_borrow_mut();
            assert!(b2.is_err());
        }
    }

    #[test]
    fn test_rc_cell_mix() {
        let rc1: RcCell<S> = RcCell::<S>::new(S {});
        let rc2: RcCell<dyn T> = rc1.clone();

        assert_eq!(rc1.borrow().f(), 10);
        assert_eq!(rc1.borrow_mut().f(), 10);
        assert_eq!(rc2.borrow().f(), 10);
        assert_eq!(rc2.borrow_mut().f(), 10);

        let b1 = rc1.try_borrow();
        assert!(b1.is_ok());
        let b2 = rc2.try_borrow_mut();
        assert!(b2.is_err());
    }
}
