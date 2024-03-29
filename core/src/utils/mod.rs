#![allow(dead_code)]

mod ptr;
mod rc_cell;
pub mod serde_helper;
mod serialize;

pub use ptr::{const_ptr, mut_ptr, size_of_array, size_of_type, CastArc, CastRc};
pub use rc_cell::{RcCell, RcCellError, RcCellRef, RcCellRefMut};
pub use serialize::{deserialize, serialize};
