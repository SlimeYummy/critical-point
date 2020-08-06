mod input;

use crate::core_ex::GdSyncCore;
use crate::util::NodeExt;
use core::logic::{CmdNewCharacter, CmdNewStage, Command};
use failure::Error;
use gdnative::api::InputEvent;
use gdnative::prelude::*;
use input::AppInput;
use m::fx;
use na;

#[derive(NativeClass)]
#[inherit(Node)]
#[user_data(LocalCellData<GdApp>)]
pub struct GdApp {
    input: AppInput,
}

#[methods]
impl GdApp {
    fn new(_owner: &Node) -> GdApp {
        return GdApp {
            input: AppInput::new(),
        };
    }

    #[export]
    fn _ready(&mut self, owner: &Node) {
        godot_print!("GdApp::_ready()");

        self.init_sync_core(owner)
            .expect("GdCharacter::new() => SyncCore");

        self.input.register_events();
    }

    #[export]
    fn _process(&mut self, _owner: &Node, _duration: f32) {}

    #[export]
    fn _input(&mut self, owner: &Node, event: Ref<InputEvent>) {
        self.input.handle_events(owner, event).unwrap();
    }
}

impl GdApp {
    fn init_sync_core(&self, owner: &Node) -> Result<(), Error> {
        let core = unsafe { owner.root_instance_ref::<GdSyncCore, Node, _>("./Root/SyncCore")? };
        core.map_mut(|core, _| {
            let agent = core.get_agent();
            agent.command(Command::NewStage(CmdNewStage {}));
            agent.command(Command::NewCharacter(CmdNewCharacter {
                position: na::Point3::new(fx(0), fx(10), fx(0)),
                direction: na::Vector2::new(fx(0), fx(-1)),
                speed: fx(6), // 6 m/s
                is_main: true,
            }));
        })
        .unwrap();
        return Ok(());
    }
}
