#![allow(dead_code)]

mod err;
mod ptr;
mod rc_cell;

pub use err::{try_option, try_result, OptionEx};
pub use ptr::{const_ptr, mut_ptr};
pub use rc_cell::{RcCell, RcCellError, RcCellRef, RcCellRefMut};
