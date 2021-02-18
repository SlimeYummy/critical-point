#![feature(arbitrary_enum_discriminant)]
#![feature(try_blocks)]

extern crate anyhow;
extern crate core;
extern crate derivative;
extern crate euclid;
extern crate gdnative;
extern crate lazy_static;
extern crate maplit;
extern crate math as m;
extern crate nalgebra as na;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;

mod application;
mod character;
mod core_ex;
mod stage;
mod utils;

use crate::application::Application;
use crate::character::CharaHuman;
use crate::core_ex::{init_sync_agent, load_res_cache};
use crate::stage::StageGeneral;
use gdnative::prelude::*;

fn init(handle: InitHandle) {
    if let Err(err) = load_res_cache() {
        godot_error!("load_res_cache() => {:?}", err);
    } else {
        godot_print!("init ResCache success");
    }

    if let Err(err) = init_sync_agent() {
        godot_error!("init_sync_agent() => {:?}", err);
    } else {
        godot_print!("init SyncLogicAgent success");
    }

    handle.add_class::<Application>();
    handle.add_class::<StageGeneral>();
    handle.add_class::<CharaHuman>();
}

godot_init!(init);
