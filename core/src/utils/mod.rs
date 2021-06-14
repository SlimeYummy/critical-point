#![allow(dead_code)]

mod err;
mod ptr;
mod rc_cell;
pub mod serde_helper;
mod serialize;

pub use err::{try_option, try_result, CPError, CPResult, OptionEx};
pub use ptr::{any_vtable, const_ptr, mut_ptr};
pub use rc_cell::{RcCell, RcCellError, RcCellRef, RcCellRefMut};
pub use serialize::{deserialize, serialize};
