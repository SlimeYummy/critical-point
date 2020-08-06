#![feature(clamp)]

extern crate core;
extern crate euclid;
extern crate failure;
extern crate gdnative;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate maplit;
extern crate math as m;
extern crate nalgebra as na;
extern crate serde;
extern crate serde_json;

mod app;
mod character;
mod core_ex;
mod stage;
mod util;

use crate::app::GdApp;
use crate::character::GdCharacter;
use crate::core_ex::{GdAsyncCore, GdSyncCore};
use crate::stage::GdStage;
use gdnative::prelude::*;

fn init(handle: InitHandle) {
    godot_print!("init()");
    handle.add_class::<GdSyncCore>();
    handle.add_class::<GdAsyncCore>();
    handle.add_class::<GdStage>();
    handle.add_class::<GdCharacter>();
    handle.add_class::<GdApp>();
}

godot_init!(init);
