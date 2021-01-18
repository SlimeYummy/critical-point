mod control;

use crate::core_ex::SYNC_AGENT;
use control::AppControl;
use core::engine::{CmdRunResCommand, Command};
use core::id::ResID;
use gdnative::api::InputEvent;
use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(Node)]
#[register_with(register_properties)]
#[user_data(LocalCellData<Application>)]
pub struct Application {
    control: AppControl,
    command_id: ResID,
}

fn register_properties(builder: &ClassBuilder<Application>) {
    builder
        .add_property::<String>("critical_point/command_id")
        .with_default(ResID::invalid().into())
        .with_getter(|app: &Application, _| app.command_id.clone().into())
        .with_setter(|app: &mut Application, _, val: String| app.command_id = ResID::from(val))
        .done();
}

#[methods]
impl Application {
    fn new(_owner: &Node) -> Application {
        return Application {
            control: AppControl::new(),
            command_id: ResID::invalid(),
        };
    }

    #[export]
    fn _ready(&mut self, owner: &Node) {
        godot_print!("Application::_ready() => call");
        owner.set_physics_process(true);

        if self.command_id.is_valid() {
            SYNC_AGENT().run_command(Command::RunResCommand(CmdRunResCommand {
                res_id: ResID::from(self.command_id.clone()),
            }));
        } else {
            godot_warn!("Application::_ready() => miss command_id");
        }

        self.control.register_events();
    }

    #[export]
    fn _process(&mut self, _owner: &Node, _duration: f32) {}

    #[export]
    fn _physics_process(&mut self, _owner: &Node, _delta: f64) {
        if let Err(err) = SYNC_AGENT().run_tick() {
            godot_error!("Application::_physics_process() => {:?}", err);
        }
    }

    #[export]
    fn _input(&mut self, owner: &Node, event: Ref<InputEvent>) {
        self.control.handle_events(owner, event).unwrap();
    }
}
