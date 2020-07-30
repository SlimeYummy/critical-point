use crate::logic::{Command, LogicEngine, Operation};
use crate::state::{StateBus, StateData, StateDataStatic, StatePool, StateRef, StateReg};
use crate::util::make_err;
use failure::Error;
use failure::_core::marker::PhantomData;
use m::Fx;
use std::cell::RefCell;
use std::mem;
use std::ptr;
use std::rc::{Rc, Weak};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

const DEFAULT_VEC_CAPACITY: usize = 128;

struct AsyncInput {
    operations: Vec<Operation>,
    commands: Vec<Command>,
}

struct AsyncInner {
    state_bus: StateBus,
    operations: Vec<Operation>,
    commands: Vec<Command>,
    input_sx: Sender<Option<AsyncInput>>,
    output_rx: Receiver<Box<StatePool>>,
}

#[derive(Clone)]
pub struct AsyncLogicAgent {
    inner: Rc<RefCell<AsyncInner>>,
}

impl !Send for AsyncLogicAgent {}
impl !Sync for AsyncLogicAgent {}

impl AsyncLogicAgent {
    pub fn new(duration: Fx) -> AsyncLogicAgent {
        let (input_sx, input_rx) = channel::<Option<AsyncInput>>();
        let (output_sx, output_rx) = channel::<Box<StatePool>>();
        let _ = thread::spawn(move || Self::engine_thread(duration, input_rx, output_sx));

        return AsyncLogicAgent {
            inner: Rc::new(RefCell::new(AsyncInner {
                state_bus: StateBus::new(),
                operations: Vec::with_capacity(DEFAULT_VEC_CAPACITY),
                commands: Vec::with_capacity(DEFAULT_VEC_CAPACITY),
                input_sx,
                output_rx,
            })),
        };
    }

    fn engine_thread(
        duration: Fx,
        input_rx: Receiver<Option<AsyncInput>>,
        output_sx: Sender<Box<StatePool>>,
    ) -> Result<(), Error> {
        let mut engine = LogicEngine::new(duration);
        while let Some(input) = input_rx.recv()? {
            for op in &input.operations {
                let cmd = engine.operate(op)?;
                engine.command(&cmd)?;
            }
            for cmd in &input.commands {
                engine.command(cmd)?;
            }
            let pool = engine.tick()?;
            output_sx.send(pool)?;
        }
        return Ok(());
    }

    pub fn operate(&self, op: Operation) {
        self.inner.borrow_mut().operations.push(op);
    }

    pub fn command(&self, cmd: Command) {
        self.inner.borrow_mut().commands.push(cmd);
    }

    pub fn tick(&self) -> Result<(), Error> {
        let inner = &mut self.inner.borrow_mut();

        let input = AsyncInput {
            operations: mem::replace(
                &mut inner.operations,
                Vec::with_capacity(DEFAULT_VEC_CAPACITY),
            ),
            commands: mem::replace(
                &mut inner.commands,
                Vec::with_capacity(DEFAULT_VEC_CAPACITY),
            ),
        };
        inner.input_sx.send(Some(input))?;
        let pool = inner.output_rx.recv()?;
        inner.state_bus.dispatch(pool);
        return Ok(());
    }
}

pub struct AsyncStateReg<S>
where
    S: StateData + StateDataStatic,
{
    rc_ptr: *const RefCell<AsyncInner>,
    phantom: PhantomData<S>,
}

impl<S> !Send for AsyncStateReg<S> {}
impl<S> !Sync for AsyncStateReg<S> {}

impl<S> Drop for AsyncStateReg<S>
where
    S: StateData + StateDataStatic,
{
    fn drop(&mut self) {
        if !self.rc_ptr.is_null() {
            let _ = unsafe { Weak::from_raw(self.rc_ptr) };
        }
    }
}

impl<S> Default for AsyncStateReg<S>
where
    S: StateData + StateDataStatic,
{
    fn default() -> AsyncStateReg<S> {
        return AsyncStateReg {
            rc_ptr: ptr::null(),
            phantom: PhantomData,
        };
    }
}

impl<S> AsyncStateReg<S>
where
    S: StateData + StateDataStatic,
{
    pub fn new(agent: &AsyncLogicAgent) -> AsyncStateReg<S> {
        return AsyncStateReg {
            rc_ptr: Weak::into_raw(Rc::downgrade(&agent.inner)),
            phantom: PhantomData,
        };
    }
}

impl<S> StateReg<S> for AsyncStateReg<S>
where
    S: StateData + StateDataStatic,
{
    fn register<B: StateReg<S>>(&self, refer: &StateRef<S, B>) -> Result<(), Error> {
        if self.rc_ptr.is_null() {
            return make_err("AsyncStateReg::register() => uninited");
        }
        let weak = unsafe { Weak::from_raw(self.rc_ptr) };
        let inner_ptr = weak.as_ptr();
        mem::forget(weak);
        if !inner_ptr.is_null() {
            return unsafe { &*inner_ptr }
                .borrow_mut()
                .state_bus
                .register(refer);
        }
        return Ok(());
    }

    fn unregister<B: StateReg<S>>(&self, refer: &StateRef<S, B>) -> Result<(), Error> {
        if self.rc_ptr.is_null() {
            return make_err("AsyncStateReg::unregister() => uninited");
        }
        let weak = unsafe { Weak::from_raw(self.rc_ptr) };
        let inner_ptr = weak.as_ptr();
        mem::forget(weak);
        if !inner_ptr.is_null() {
            return unsafe { &*inner_ptr }
                .borrow_mut()
                .state_bus
                .unregister(refer);
        }
        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::ObjID;
    use crate::logic::{CmdNewStage, StateStage};
    use crate::state::StateLifecycle;
    use m::fx;

    #[test]
    fn test_async_logic_agent() {
        let mut agent = AsyncLogicAgent::new(fx(1.0 / 20.0));

        let state_ref: StateRef<StateStage, AsyncStateReg<StateStage>> =
            StateRef::new(ObjID::from(100000), AsyncStateReg::new(&agent));
        state_ref.register().unwrap();

        agent.command(Command::NewStage(CmdNewStage {}));
        agent.tick().unwrap();

        assert_eq!(state_ref.state().unwrap().obj_id, ObjID::from(100000));
        assert_eq!(
            state_ref.state().unwrap().lifecycle,
            StateLifecycle::Created
        );
    }
}
