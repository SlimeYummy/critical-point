use std::raw::TraitObject;
use crate::id::ObjectID;

pub trait StateData {}

#[derive(Debug)]
pub struct StateInner {
    pub obj_id: ObjectID,
    pub state: *mut u8,
    pub vtable: *mut u8,
}

pub unsafe fn state_vtable<S: StateData>() -> *mut u8 {
    let re: &S = TransmuterPtr::<S> { n: 0 }.re;
    TransmuterTO::<dyn StateData>{ re }.to.vtable as *mut u8
}

union TransmuterPtr<'t, T: 't> {
    n: isize,
    re: &'t T,
}

union TransmuterTO<'t, TO: ?Sized + 't> {
    re: &'t TO,
    to: TraitObject,
}
