mod camera;
mod input;
mod operation;

use crate::core_ex::SYNC_AGENT;
use crate::utils::NodeExt;
use anyhow::Result;
use camera::AppCamera;
use core::engine::{CmdRunResCommand, Command};
use core::id::ResID;
use gdnative::api::InputEvent;
use gdnative::prelude::*;
use input::AppInputKeyMouse;
use operation::{OpChangeMouseMode, OpMoveCamera, Operation};

#[derive(NativeClass)]
#[inherit(Node)]
#[register_with(register_properties)]
#[user_data(LocalCellData<Application>)]
pub struct Application {
    input: AppInputKeyMouse,
    camera: AppCamera,
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
    fn new(_owner: TRef<Node>) -> Application {
        return Application {
            input: AppInputKeyMouse::new("./user/input.yaml"),
            camera: AppCamera::new("Character/Camera"),
            command_id: ResID::invalid(),
        };
    }

    #[export]
    fn _ready(&mut self, owner: TRef<Node>) {
        godot_print!("Application::_ready() => call");
        owner.set_physics_process(true);

        godot_print!("{:?}", owner.get_path());

        self.input.init().unwrap();
        self.camera.init(owner).unwrap();

        if self.command_id.is_valid() {
            SYNC_AGENT().run_command(Command::RunResCommand(CmdRunResCommand {
                res_id: ResID::from(self.command_id.clone()),
            }));
        } else {
            godot_warn!("Application::_ready() => miss command_id");
        }
    }

    #[export]
    fn _process(&mut self, _owner: TRef<Node>, _duration: f32) {}

    #[export]
    fn _physics_process(&mut self, _owner: TRef<Node>, _delta: f64) {
        let ops = self.input.tick();
        if !ops.is_empty() {
            godot_print!("{:?}", ops);
        }

        if let Err(err) = SYNC_AGENT().run_tick() {
            godot_error!("Application::_physics_process() => {:?}", err);
        }
    }

    #[export]
    fn _input(&mut self, _owner: TRef<Node>, event: Ref<InputEvent>) {
        let result: Result<()> = try {
            let op = self.input.handle_events(event);
            if let Some(op) = op {
                match op {
                    Operation::Core(op) => godot_print!("{:?}", op),
                    Operation::ChangeMouseMode(op) => self.change_mouse_mode(op)?,
                    Operation::MoveCamera(op) => self.move_camera(op)?,
                }
            }
        };
        if let Err(err) = result {
            godot_error!("Application::_input() => {:?}", err);
        }
    }

    fn change_mouse_mode(&mut self, _op: OpChangeMouseMode) -> Result<()> {
        let input = Input::godot_singleton();
        if input.get_mouse_mode().0 != Input::MOUSE_MODE_CAPTURED {
            input.set_mouse_mode(Input::MOUSE_MODE_CAPTURED);
        } else {
            input.set_mouse_mode(Input::MOUSE_MODE_VISIBLE);
        }
        return Ok(());
    }

    fn move_camera(&mut self, op: OpMoveCamera) -> Result<()> {
        self.camera.rotate_camera(op)?;
        let rotation2 = self.camera.rotation2();
        let rotation3 = self.camera.rotation3()?;
        self.input.set_camera_rotation(rotation2, rotation3);
        return Ok(());
    }
}
