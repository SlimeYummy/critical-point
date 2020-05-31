#![feature(vec_remove_item)]

extern crate failure;
extern crate libc;

mod cpp;
// mod graphic;
mod id;
mod logic;
mod state_pool;
mod utils;

use std::time::Duration;

// use graphic::*;
use logic::*;
use std::fs::File;

#[no_mangle]
extern "C" fn critical_point() {
    File::create("E:/a.txt").unwrap();

    let mut lo_engine = LoEngine::new();
    lo_engine.run_command(LoCmdNewStage::new()).unwrap();
    lo_engine.run_command(LoCmdNewCharacter::new()).unwrap();

    // let _gr_engine = GrEngine::new("Perfect Glue", 1280, 720, 60);
    for _ in 0..50 {
        lo_engine
            .run_step(Duration::from_secs_f32(10.0 / 60.0))
            .unwrap();
        let state = lo_engine.fetch_state().unwrap();
        println!("{:?}\n", state);
    }
}
