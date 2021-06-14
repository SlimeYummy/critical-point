use std::any::Any;
use std::raw::TraitObject;

pub fn const_ptr<T, U>(val: &T) -> *const U {
    return (val as *const T) as *const U;
}

pub fn mut_ptr<T, U>(val: &mut T) -> *mut U {
    return (val as *mut T) as *mut U;
}

pub unsafe fn any_vtable<A: Any>() -> *mut u8 {
    let re: &A = TransmuterPtr::<A> { n: 0 }.re;
    TransmuterTO::<dyn Any> { re }.to.vtable as *mut u8
}

union TransmuterPtr<'t, T: 't> {
    n: isize,
    re: &'t T,
}

union TransmuterTO<'t, TO: ?Sized + 't> {
    re: &'t TO,
    to: TraitObject,
}
