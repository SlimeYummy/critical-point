#![feature(raw)]
#![feature(vec_remove_item)]

extern crate failure;
#[macro_use]
extern crate gdnative;
extern crate libc;
extern crate macros;

mod cpp;
// mod graphic;
mod id;
// mod logic;
mod state;
mod utils;

use gdnative as gd;

#[derive(gd::NativeClass)]
#[inherit(gd::Node)]
struct HelloWorld;

#[methods]
impl HelloWorld {
    fn _init(_owner: gd::Node) -> Self {
        HelloWorld
    }

    #[export]
    fn _ready(&self, _owner: gd::Node) {
        godot_print!("hello, world.")
    }
}

fn init(handle: gd::init::InitHandle) {
    handle.add_class::<HelloWorld>();
}

godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!();

// #[no_mangle]
// extern "C" fn critical_point() {
//     File::create("E:/a.txt").unwrap();
//
//     let mut lo_engine = LoEngine::new();
//     lo_engine.run_command(LoCmdNewStage::new()).unwrap();
//     lo_engine.run_command(LoCmdNewCharacter::new()).unwrap();
//
//     // let _gr_engine = GrEngine::new("Perfect Glue", 1280, 720, 60);
//     for _ in 0..50 {
//         lo_engine
//             .run_step(Duration::from_secs_f32(10.0 / 60.0))
//             .unwrap();
//         let state = lo_engine.fetch_state().unwrap();
//         println!("{:?}\n", state);
//     }
// }
