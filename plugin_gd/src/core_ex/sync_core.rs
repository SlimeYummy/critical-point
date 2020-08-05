use core::agent::SyncLogicAgent;
use gdnative::prelude::*;
use m::fx;

#[derive(NativeClass)]
#[inherit(Node)]
#[user_data(LocalCellData<GdSyncCore>)]
pub struct GdSyncCore {
    agent: SyncLogicAgent,
}

#[methods]
impl GdSyncCore {
    fn new(_owner: &Node) -> GdSyncCore {
        godot_print!("GdSyncCore::new()");
        return GdSyncCore {
            agent: SyncLogicAgent::new(fx(1.0 / 60.0)),
        };
    }

    #[export]
    fn _ready(&mut self, owner: &Node) {
        godot_print!("GdSyncCore::_ready()");
        owner.set_physics_process(true);
    }

    #[export]
    fn _physics_process(&mut self, _owner: &Node, _delta: f64) {
        self.agent.tick().unwrap();
    }

    pub fn get_agent(&self) -> SyncLogicAgent {
        return self.agent.clone();
    }
}
