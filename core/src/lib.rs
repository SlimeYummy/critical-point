#![feature(allocator_api)]
#![feature(alloc_layout_extra)]
#![feature(box_into_raw_non_null)]
#![feature(coerce_unsized)]
#![feature(dispatch_from_dyn)]
#![feature(negative_impls)]
#![feature(raw)]
#![feature(unsize)]
#![feature(untagged_unions)]
#![feature(vec_remove_item)]

extern crate alga;
extern crate approx;
extern crate derivative;
extern crate derive;
extern crate euclid;
extern crate failure;
extern crate fixed;
extern crate libc;
extern crate math as m;
extern crate nalgebra as na;
extern crate ncollide3d;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;
extern crate simba;

pub mod agent;
pub mod id;
pub mod logic;
pub mod resource;
pub mod state;
pub mod util;
