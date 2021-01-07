use crate::engine::Command;
use crate::engine::LogicEngine;
use crate::resource::ResCache;
use crate::state::{StateBinder, StateBus};
use anyhow::Result;
use m::Fx;
use std::mem;
use std::sync::Arc;

const DEFAULT_VEC_CAPACITY: usize = 128;

pub struct SyncLogicAgent {
    state_bus: StateBus,
    engine: LogicEngine,
    commands: Vec<Command>,
}

impl !Send for SyncLogicAgent {}
impl !Sync for SyncLogicAgent {}

impl SyncLogicAgent {
    pub fn new(res_cache: Arc<ResCache>, fps: Fx) -> Result<SyncLogicAgent> {
        return Ok(SyncLogicAgent {
            state_bus: StateBus::new(),
            engine: LogicEngine::new(res_cache, fps)?,
            commands: Vec::with_capacity(DEFAULT_VEC_CAPACITY),
        });
    }

    pub fn new_binder(&self) -> StateBinder {
        return self.state_bus.new_binder();
    }

    pub fn run_command(&mut self, cmd: Command) {
        self.commands.push(cmd);
    }

    pub fn run_tick(&mut self) -> Result<()> {
        let commands = mem::replace(&mut self.commands, Vec::with_capacity(DEFAULT_VEC_CAPACITY));
        for cmd in &commands {
            self.engine.run_command(cmd)?;
        }
        let pool = self.engine.run_tick()?;
        self.state_bus.dispatch_states(pool);
        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::CmdNewStageGeneral;
    use crate::id::{FastObjID, ResID};
    use crate::stage::StateStageGeneral;
    use crate::state::{StateLifecycle, StateRef};
    use m::ff;

    #[test]
    fn test_sync_logic_agent() {
        let mut agent = SyncLogicAgent::new(Arc::new(ResCache::new()), ff(1.0 / 20.0)).unwrap();

        let state_ref: StateRef<StateStageGeneral> =
            StateRef::new_and_start(FastObjID::from(123), agent.new_binder()).unwrap();

        agent.run_command(Command::NewStageGeneral(CmdNewStageGeneral {
            res_id: ResID::from("res-id"),
            fobj_id: FastObjID::from(123),
        }));
        agent.run_tick().unwrap();

        assert_eq!(state_ref.state().unwrap().fobj_id, FastObjID::from(100000));
        assert_eq!(
            state_ref.state().unwrap().lifecycle,
            StateLifecycle::Created
        );
    }
}
