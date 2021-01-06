#![feature(allocator_api)]
#![feature(alloc_layout_extra)]
#![feature(coerce_unsized)]
#![feature(dispatch_from_dyn)]
#![feature(get_mut_unchecked)]
#![feature(negative_impls)]
#![feature(raw)]
#![feature(unsize)]
#![feature(untagged_unions)]

extern crate anyhow;
extern crate approx;
extern crate derivative;
extern crate derive;
#[macro_use]
extern crate gdnative;
extern crate lazy_static;
extern crate libc;
extern crate math as m;
extern crate nalgebra as na;
extern crate ncollide3d;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;
extern crate strum;
extern crate thiserror;
extern crate typetag;
extern crate wavefront_obj;

pub mod action;
pub mod agent;
pub mod character;
pub mod engine;
pub mod id;
pub mod physic;
pub mod resource;
pub mod stage;
pub mod state;
pub mod util;
