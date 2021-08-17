use anyhow::{anyhow, Result};
use std::any::Any;
use std::mem;
use std::ptr;
use std::rc::Rc;
use std::sync::Arc;

pub fn const_ptr<T, U>(val: &T) -> *const U {
    return (val as *const T) as *const U;
}

pub fn mut_ptr<T, U>(val: &mut T) -> *mut U {
    return (val as *mut T) as *mut U;
}

pub trait CastRc {
    fn cast_as<D: Any>(self) -> Result<Rc<D>>;
    fn cast_to<D: Any>(&self) -> Result<Rc<D>>;
}

impl<S: ?Sized> CastRc for Rc<S> {
    fn cast_as<D: Any>(self) -> Result<Rc<D>> {
        let src_meta = ptr::metadata(Rc::as_ptr(&self));
        let src_drop = unsafe { *mem::transmute_copy::<_, *mut *mut u8>(&src_meta) };

        let dst_ref: &dyn Any = unsafe { mem::transmute_copy::<usize, &D>(&0) };
        let dst_meta = ptr::metadata(dst_ref);
        let dst_drop = unsafe { *mem::transmute_copy::<_, *mut *mut u8>(&dst_meta) };

        if src_drop != dst_drop {
            return Err(anyhow!("Not this variant"));
        }

        let (src_data, _) = Rc::into_raw(self).to_raw_parts();
        let dst_arc = unsafe { Rc::from_raw(src_data as *const D) };
        return Ok(dst_arc);
    }

    fn cast_to<D: Any>(&self) -> Result<Rc<D>> {
        return self.clone().cast_as();
    }
}

pub trait CastArc {
    fn cast_as<D: Any>(self) -> Result<Arc<D>>;
    fn cast_to<D: Any>(&self) -> Result<Arc<D>>;
}

impl<S: ?Sized> CastArc for Arc<S> {
    fn cast_as<D: Any>(self) -> Result<Arc<D>> {
        let src_meta = ptr::metadata(Arc::as_ptr(&self));
        let src_drop = unsafe { *mem::transmute_copy::<_, *mut *mut u8>(&src_meta) };

        let dst_ref: &dyn Any = unsafe { mem::transmute_copy::<usize, &D>(&0) };
        let dst_meta = ptr::metadata(dst_ref);
        let dst_drop = unsafe { *mem::transmute_copy::<_, *mut *mut u8>(&dst_meta) };

        if src_drop != dst_drop {
            return Err(anyhow!("Not this variant"));
        }

        let (src_data, _) = Arc::into_raw(self).to_raw_parts();
        let dst_arc = unsafe { Arc::from_raw(src_data as *const D) };
        return Ok(dst_arc);
    }

    fn cast_to<D: Any>(&self) -> Result<Arc<D>> {
        return self.clone().cast_as();
    }
}
