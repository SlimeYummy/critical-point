use super::logic_data::RefData;
use crate::id::{ClassID, FastObjID};
use std::alloc::{Allocator, Global, Layout};
use std::cell::Cell;
use std::error::Error;
use std::ffi::c_void;
use std::fmt::{self, Debug, Display};
use std::intrinsics;
use std::marker::{PhantomData, Unsize};
use std::mem;
use std::ops::{CoerceUnsized, Deref, DerefMut, DispatchFromDyn};
use std::ptr::{self, NonNull};
use std::raw::TraitObject;
use std::sync::atomic::{self, AtomicUsize, Ordering};

const MAX_REF_COUNT: usize = (isize::MAX) as usize;

//
// LogicObj
//

pub trait LogicObjStatic {
    fn id() -> ClassID;
}

pub trait LogicObj {
    fn class_id(&self) -> ClassID;
    fn fobj_id(&self) -> FastObjID;
    fn prop_ptr(&self) -> *const c_void;
    fn state_ptr(&self) -> *const c_void;
}

impl dyn LogicObj {
    #[inline]
    pub fn is<O>(&self) -> bool
    where
        O: LogicObj + LogicObjStatic,
    {
        return self.class_id() == O::id();
    }

    #[inline]
    pub fn cast<O>(&self) -> Option<&O>
    where
        O: LogicObj + LogicObjStatic,
    {
        if self.class_id() == O::id() {
            return Some(unsafe { mem::transmute_copy(&self) });
        } else {
            return None;
        }
    }

    #[inline]
    pub fn cast_mut<O>(&mut self) -> Option<&mut O>
    where
        O: LogicObj + LogicObjStatic,
    {
        if self.class_id() == O::id() {
            return Some(unsafe { mem::transmute_copy(&self) });
        } else {
            return None;
        }
    }
}

//
// RefObjBox
//

#[repr(C)]
struct RefObjBox<T: ?Sized> {
    ref_count: AtomicUsize,
    borrow: Cell<isize>,
    data: T,
}

pub struct RefObj<T: ?Sized> {
    box_data: NonNull<RefObjBox<T>>,
    phantom: PhantomData<RefObjBox<T>>,
}

unsafe impl<T> Sync for RefObj<T> {}
unsafe impl<T> Send for RefObj<T> {}

impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<RefObj<U>> for RefObj<T> {}
impl<T: ?Sized + Unsize<U>, U: ?Sized> DispatchFromDyn<RefObj<U>> for RefObj<T> {}

//
// RefObj: reference count
//

impl<O> RefObj<O>
where
    O: LogicObj + 'static,
{
    #[inline]
    pub fn new(obj: O) -> RefObj<O> {
        return RefObj {
            box_data: Box::leak(Box::new(RefObjBox {
                ref_count: AtomicUsize::new(1),
                borrow: Cell::new(0),
                data: obj,
            }))
            .into(),
            phantom: PhantomData,
        };
    }

    #[inline]
    pub fn state(&self) -> Result<RefData<(), ()>, RefObjError> {
        return Ok(RefData::new(self));
    }
}

impl<T: ?Sized> Clone for RefObj<T> {
    #[inline]
    fn clone(&self) -> RefObj<T> {
        let old_size = self.box_data().ref_count.fetch_add(1, Ordering::Relaxed);
        if old_size > MAX_REF_COUNT {
            intrinsics::abort();
        }
        return RefObj {
            box_data: self.box_data,
            phantom: PhantomData,
        };
    }
}

impl<T: ?Sized> Drop for RefObj<T> {
    #[inline]
    fn drop(&mut self) {
        if self.box_data().ref_count.fetch_sub(1, Ordering::Release) != 1 {
            return;
        }
        atomic::fence(Ordering::Acquire);
        unsafe { self.drop_slow() };
    }
}

impl<T: ?Sized> RefObj<T> {
    #[inline]
    fn box_data(&self) -> &RefObjBox<T> {
        return unsafe { self.box_data.as_ref() };
    }

    #[inline]
    fn data(&self) -> &T {
        return &self.box_data().data;
    }

    #[inline(never)]
    unsafe fn drop_slow(&mut self) {
        ptr::drop_in_place(&mut (*self.box_data.as_ptr()).data);
        Global.deallocate(
            self.box_data.cast(),
            Layout::for_value_raw(self.box_data.as_ptr()),
        );
    }
}

impl RefObj<dyn LogicObj> {
    #[inline]
    pub fn is<O>(&self) -> bool
    where
        O: LogicObj + LogicObjStatic,
    {
        let refer = unsafe { RefObj::as_ref_mut(self) };
        return refer.class_id() == O::id();
    }

    #[inline]
    pub fn cast<O>(self) -> Option<RefObj<O>>
    where
        O: LogicObj + LogicObjStatic,
    {
        if self.borrow().class_id() == O::id() {
            unsafe {
                let to: TraitObject = mem::transmute(self);
                return Some(mem::transmute(to.data));
            }
        } else {
            return None;
        }
    }
}

//
// RefObj: borrow
//

impl<T: ?Sized> RefObj<T> {
    #[inline]
    fn get_borrow(&self) -> isize {
        return self.box_data().borrow.get();
    }

    #[inline]
    fn set_borrow(&self, borrow: isize) {
        self.box_data().borrow.set(borrow);
    }

    #[inline]
    pub unsafe fn as_ptr_mut(this: &Self) -> *mut T {
        let ptr_data = this.box_data.as_ptr();
        let fake_ptr = ptr_data as *mut T;
        let offset = data_offset::<RefObjBox<T>, RefObjBox<()>>(this.box_data());
        return set_data_ptr(fake_ptr, (ptr_data as *mut u8).offset(offset));
    }

    #[inline]
    pub unsafe fn as_ref_mut(this: &Self) -> &mut T {
        return &mut *RefObj::as_ptr_mut(this);
    }
}

pub struct RefObjRef<'b, T: ?Sized + 'b> {
    borrow: &'b Cell<isize>,
    data: &'b T,
}

impl<T: ?Sized> Drop for RefObjRef<'_, T> {
    #[inline]
    fn drop(&mut self) {
        self.borrow.set(self.borrow.get() - 1);
    }
}

impl<T: ?Sized> Deref for RefObjRef<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        return self.data;
    }
}

impl<T: ?Sized> RefObj<T> {
    #[inline]
    pub fn try_borrow(&self) -> Result<RefObjRef<'_, T>, RefObjError> {
        let borrow = self.get_borrow() + 1;
        if borrow > 0 {
            self.set_borrow(borrow);
            return Ok(RefObjRef {
                borrow: &self.box_data().borrow,
                data: unsafe { RefObj::as_ref_mut(self) },
            });
        } else {
            return Err(RefObjError::MutBorrowed);
        }
    }

    #[inline]
    pub fn borrow(&self) -> RefObjRef<'_, T> {
        return self.try_borrow().expect("already mutably borrowed");
    }
}

pub struct RefObjRefMut<'b, T: ?Sized + 'b> {
    borrow: &'b Cell<isize>,
    data: &'b mut T,
}

impl<T: ?Sized> Drop for RefObjRefMut<'_, T> {
    #[inline]
    fn drop(&mut self) {
        self.borrow.set(0);
    }
}

impl<T: ?Sized> Deref for RefObjRefMut<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        return self.data;
    }
}

impl<T: ?Sized> DerefMut for RefObjRefMut<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        return self.data;
    }
}

impl<T: ?Sized> RefObj<T> {
    #[inline]
    pub fn try_borrow_mut(&self) -> Result<RefObjRefMut<'_, T>, RefObjError> {
        if self.get_borrow() == 0 {
            self.set_borrow(-1);
            return Ok(RefObjRefMut {
                borrow: &self.box_data().borrow,
                data: unsafe { RefObj::as_ref_mut(self) },
            });
        } else {
            return Err(RefObjError::MutBorrowed);
        }
    }

    #[inline]
    pub fn borrow_mut(&self) -> RefObjRefMut<'_, T> {
        return self.try_borrow_mut().expect("already mutably borrowed");
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

//
// RefObjError
//

#[derive(Debug)]
pub enum RefObjError {
    DanglingPtr,
    MutBorrowed,
    MismatchClassID,
}

impl Error for RefObjError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        return None;
    }
}

impl Display for RefObjError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return match self {
            RefObjError::DanglingPtr => Display::fmt("dangling pointer", f),
            RefObjError::MutBorrowed => Display::fmt("already mutably borrowed", f),
            RefObjError::MismatchClassID => Display::fmt("mismatch class id", f),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::super::logic_data::{LogicData, LogicLifecycle, LogicState, RefData};
    use super::super::{def_obj, def_prop, def_state};
    use super::*;
    use crate::id::FastResID;

    #[def_prop(StageGeneral)]
    #[derive(Default)]
    struct TestProp {
        fres_id: FastResID,
    }

    #[def_state(StageGeneral)]
    #[derive(Default)]
    struct TestState {
        lifecycle: LogicLifecycle,
    }

    #[def_obj(StageGeneral)]
    struct TestObj {
        fobj_id: FastObjID,
        data: LogicData<TestProp, TestState>,
    }

    #[test]
    fn test_logic_obj() {
        let ro1 = RefObj::new(TestObj {
            fobj_id: FastObjID::from(1),
            data: LogicData::new(Default::default()),
        });
        // let ref_state: RefData<TestObj, TestState> = ro1.state().unwrap();
        // let ro2: RefObj<dyn LogicObj> = ro1;
    }

    // impl TestObj {
    //     fn new(fobj_id: FastObjID) -> TestObj {
    //         return TestObj { fobj_id };
    //     }
    // }

    // impl LogicObj for TestObj {
    //     fn collide(&mut self, _ctx: &mut CollideContext) -> Result<()> {
    //         return Ok(());
    //     }

    //     fn update(&mut self, _ctx: &mut UpdateContext) -> Result<()> {
    //         return Ok(());
    //     }

    //     fn state(&mut self) -> Result<()> {
    //         return Ok(());
    //     }
    // }

    // #[test]
    // fn test_logic_obj() {
    //     let lo1 = TestObj::new(FastObjID::from(123));
    //     let to1: &dyn LogicObj = &lo1;
    //     let ct1 = to1.cast_ref::<TestObj>().unwrap();
    //     assert_eq!(FastObjID::from(123), ct1.fobj_id());

    //     let mut lo2 = TestObj::new(FastObjID::from(123));
    //     let to2: &mut dyn LogicObj = &mut lo2;
    //     let ct2 = to2.cast_ref::<TestObj>().unwrap();
    //     assert_eq!(FastObjID::from(123), ct2.fobj_id());

    //     let lo3 = RefObj::new(TestObj::new(FastObjID::from(123)));
    //     let to3: RefObj<dyn LogicObj> = lo3.clone();
    //     let ct3 = to3.cast::<TestObj>().unwrap();
    //     assert_eq!(FastObjID::from(123), ct3.borrow().fobj_id());
    // }
}
