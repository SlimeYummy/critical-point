#[repr(C)]
pub struct FFIArray<T>(Vec<T>);

#[repr(C)]
pub struct FFISlice<'t, T>(&'t [T]);
