use std::alloc::{self, Layout};
use std::cell::Cell;
use std::error::Error;
use std::fmt::{self, Debug, Display};
use std::marker::{PhantomData, Unsize};
use std::mem;
use std::ops::{CoerceUnsized, Deref, DerefMut, DispatchFromDyn};
use std::ptr::{self, NonNull};

#[repr(C)]
struct RcCellBox<T: ?Sized> {
    strong: Cell<usize>,
    borrow: Cell<isize>,
    value: T,
}

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
        self.inc_strong();
        return RcCell {
            ptr: self.ptr,
            phantom: PhantomData,
        };
    }
}

impl<T: ?Sized> Drop for RcCell<T> {
    fn drop(&mut self) {
        if self.dec_strong() == 0 {
            unsafe {
                ptr::drop_in_place(self.ptr.as_ptr());
                alloc::dealloc(
                    self.ptr.as_ptr() as *mut u8,
                    Layout::for_value(self.ptr.as_ref()),
                );
            }
        }
    }
}

impl<T: ?Sized> RcCell<T> {
    #[inline]
    fn box_ref(&self) -> &RcCellBox<T> {
        return unsafe { &*self.ptr.as_ref() };
    }

    #[inline]
    fn get_strong(&self) -> usize {
        return self.box_ref().strong.get();
    }

    #[inline]
    fn inc_strong(&self) -> usize {
        let refer = self.box_ref();
        refer.strong.set(refer.strong.get() + 1);
        return refer.strong.get();
    }

    #[inline]
    fn dec_strong(&self) -> usize {
        let refer = self.box_ref();
        refer.strong.set(refer.strong.get() - 1);
        return refer.strong.get();
    }

    #[inline]
    fn get_borrow(&self) -> isize {
        return self.box_ref().borrow.get();
    }

    #[inline]
    fn set_borrow(&self, borrow: isize) {
        self.box_ref().borrow.set(borrow);
    }

    #[inline]
    pub unsafe fn as_ptr_mut(this: &Self) -> *mut T {
        let ptr = this.ptr.as_ptr();
        let fake_ptr = ptr as *mut T;
        let offset = data_offset(this.box_ref());
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
                borrow: &self.box_ref().borrow,
                value: unsafe { RcCell::as_ref_mut(self) },
            });
        } else {
            return Err(RcCellError { _private: () });
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
                borrow: &self.box_ref().borrow,
                value: unsafe { RcCell::as_ref_mut(self) },
            });
        } else {
            return Err(RcCellError { _private: () });
        }
    }

    #[inline]
    pub fn borrow_mut(&self) -> RcCellRefMut<'_, T> {
        return self.try_borrow_mut().expect("already mutably borrowed");
    }
}

pub struct RcCellError {
    _private: (),
}

impl Error for RcCellError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        return None;
    }
}

impl Debug for RcCellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BorrowError").finish()
    }
}

impl Display for RcCellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt("already mutably borrowed", f)
    }
}

unsafe fn set_data_ptr<T: ?Sized, U>(mut ptr: *mut T, data: *mut U) -> *mut T {
    let pptr = (&mut ptr as *mut _) as *mut *mut u8;
    ptr::write(pptr, data as *mut u8);
    return ptr;
}

unsafe fn data_offset<T: ?Sized>(ptr: *const T) -> isize {
    let align = mem::align_of_val(&*ptr);
    let layout = Layout::new::<RcCellBox<()>>();
    return (layout.size() + layout.padding_needed_for(align)) as isize;
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
