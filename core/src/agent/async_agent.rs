use crate::engine::Command;
use crate::engine::LogicEngine;
use crate::resource::ResCache;
use crate::state::{StateBinder, StateBus, StatePool};
use anyhow::Result;
use m::Fx;
use std::mem;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;

const DEFAULT_VEC_CAPACITY: usize = 128;

struct AsyncInput {
    commands: Vec<Command>,
}

pub struct AsyncLogicAgent {
    state_bus: StateBus,
    commands: Vec<Command>,
    input_sx: Sender<Option<AsyncInput>>,
    output_rx: Receiver<Box<StatePool>>,
}

impl !Send for AsyncLogicAgent {}
impl !Sync for AsyncLogicAgent {}

impl AsyncLogicAgent {
    pub fn new(res_cache: Arc<ResCache>, fps: Fx) -> AsyncLogicAgent {
        let (input_sx, input_rx) = channel::<Option<AsyncInput>>();
        let (output_sx, output_rx) = channel::<Box<StatePool>>();
        let _ = thread::spawn(move || Self::engine_thread(res_cache, fps, input_rx, output_sx));

        return AsyncLogicAgent {
            state_bus: StateBus::new(),
            commands: Vec::with_capacity(DEFAULT_VEC_CAPACITY),
            input_sx,
            output_rx,
        };
    }

    fn engine_thread(
        res_cache: Arc<ResCache>,
        fps: Fx,
        input_rx: Receiver<Option<AsyncInput>>,
        output_sx: Sender<Box<StatePool>>,
    ) -> Result<()> {
        let mut engine = LogicEngine::new(res_cache, fps)?;
        while let Some(input) = input_rx.recv()? {
            for cmd in &input.commands {
                engine.run_command(cmd)?;
            }
            let pool = engine.run_tick()?;
            output_sx.send(pool)?;
        }
        return Ok(());
    }

    pub fn new_binder(&self) -> StateBinder {
        return self.state_bus.new_binder();
    }

    pub fn run_command(&mut self, cmd: Command) {
        self.commands.push(cmd);
    }

    pub fn run_tick(&mut self) -> Result<()> {
        let input = AsyncInput {
            commands: mem::replace(&mut self.commands, Vec::with_capacity(DEFAULT_VEC_CAPACITY)),
        };
        self.input_sx.send(Some(input))?;
        let pool = self.output_rx.recv()?;
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
    fn test_async_logic_agent() {
        let mut agent = AsyncLogicAgent::new(Arc::new(ResCache::new()), ff(1.0 / 20.0));

        let state_ref: StateRef<StateStageGeneral> =
            StateRef::new_and_start(FastObjID::from(123), agent.new_binder()).unwrap();

        agent.run_command(Command::NewStageGeneral(CmdNewStageGeneral {
            res_id: ResID::from("res-id"),
            fobj_id: FastObjID::from(123),
        }));
        agent.run_tick().unwrap();

        assert_eq!(state_ref.state().unwrap().fobj_id, FastObjID::from(123));
        assert_eq!(
            state_ref.state().unwrap().lifecycle,
            StateLifecycle::Created
        );
    }
}
