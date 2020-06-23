#![feature(negative_impls)]
#![feature(raw)]
#![feature(vec_remove_item)]

extern crate euclid;
extern crate failure;
extern crate fixed;
#[macro_use]
extern crate gdnative;
extern crate libc;
extern crate macros;
extern crate nalgebra as na;
extern crate ncollide3d;
extern crate simba;

// mod graphic;
mod id;
mod logic;
// mod model;
mod state;
mod sup;
mod utils;

use gdnative as gd;
use logic::*;
use na::Vector3;

#[derive(gd::NativeClass)]
#[inherit(gd::Spatial)]
struct HelloWorld{
    start: gd::Vector3,
    counter: f32,
}

#[methods]
impl HelloWorld {
    fn _init(_owner: gd::Spatial) -> Self {
        return HelloWorld{
            start: gd::Vector3::zero(),
            counter: 0.0,
        };
    }

    #[export]
    fn _ready(&mut self, mut owner: gd::Spatial) {
        unsafe {
            self.start = owner.get_translation();
            owner.set_physics_process(true);
        };
        godot_print!("start {:?}", self.start);
        // let mut engine = LogicEngine::new();
        // engine.command(Command::NewStage(CmdNewStage{}));
        // engine.command(Command::NewCharacter(CmdNewCharacter{
        //     position: Vector3::new(fixed64(0), fixed64(0), fixed64(5)),
        // }));
    }

    #[export]
    unsafe fn _physics_process(&mut self, mut owner: gd::Spatial, _delta: f64) {
        let offset = gd::Vector3::new(0.0, self.counter, 0.0);
        self.counter += 1.0 / 60.0;
        unsafe { owner.set_translation(self.start + offset); }
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
