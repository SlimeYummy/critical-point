#![feature(const_fn_union)]
#![allow(dead_code)]

extern crate approx;
#[macro_use]
extern crate derivative;
extern crate fixed;
extern crate nalgebra as na;
extern crate ncollide3d;
extern crate num_traits;
extern crate rand;
extern crate serde;

mod algorithm;
mod auto_gen;
mod fx;
mod near;
mod velocity;

pub use algorithm::*;
pub use auto_gen::*;
pub use fx::*;
pub use near::*;
pub use velocity::*;
