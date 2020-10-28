#![feature(const_fn)]
#![allow(dead_code)]

extern crate approx;
extern crate cordic;
extern crate derivative;
extern crate fixed;
extern crate nalgebra as na;
extern crate ncollide3d;
extern crate num_traits;
extern crate rand;
extern crate serde;
extern crate simba;

mod algorithm;
mod auto_gen;
mod fx;
mod near;
mod shape;
mod vector;

pub use algorithm::*;
pub use auto_gen::*;
pub use fx::*;
pub use near::*;
pub use shape::*;
pub use vector::*;
