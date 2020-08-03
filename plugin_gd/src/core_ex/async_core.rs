use core::agent::AsyncLogicAgent;
use gdnative::prelude::*;
use m::fx;

#[derive(NativeClass)]
#[inherit(Node)]
#[user_data(LocalCellData<GdAsyncCore>)]
pub struct GdAsyncCore {
    agent: AsyncLogicAgent,
}

#[methods]
impl GdAsyncCore {
    fn new(_owner: &Node) -> GdAsyncCore {
        godot_print!("GdAsyncCore::new()");
        return GdAsyncCore {
            agent: AsyncLogicAgent::new(fx(1.0 / 60.0)),
        };
    }

    #[export]
    fn _ready(&mut self, owner: &Node) {
        godot_print!("GdAsyncCore::_ready()");
        owner.set_physics_process(true);
    }

    #[export]
    fn _physics_process(&mut self, _owner: &Node, _delta: f64) {
        self.agent.tick().unwrap();
    }

    pub fn get_agent(&self) -> AsyncLogicAgent {
        return self.agent.clone();
    }
}
