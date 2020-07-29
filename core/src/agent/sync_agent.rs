use crate::logic::{Command, LogicEngine, Operation};
use crate::state::{StateBus, StateData, StateDataStatic, StateRef, StateReg};
use crate::util::make_err;
use failure::Error;
use failure::_core::marker::PhantomData;
use m::Fx;
use std::cell::RefCell;
use std::mem;
use std::ptr;
use std::rc::{Rc, Weak};

const DEFAULT_VEC_CAPACITY: usize = 128;

struct SyncInner {
    state_bus: StateBus,
    logic_engine: LogicEngine,
    operations: Vec<Operation>,
    commands: Vec<Command>,
}

#[derive(Clone)]
pub struct SyncLogicAgent {
    inner: Rc<RefCell<SyncInner>>,
}

impl !Send for SyncLogicAgent {}
impl !Sync for SyncLogicAgent {}

impl SyncLogicAgent {
    pub fn new(duration: Fx) -> SyncLogicAgent {
        return SyncLogicAgent {
            inner: Rc::new(RefCell::new(SyncInner {
                state_bus: StateBus::new(),
                logic_engine: LogicEngine::new(duration),
                operations: Vec::with_capacity(DEFAULT_VEC_CAPACITY),
                commands: Vec::with_capacity(DEFAULT_VEC_CAPACITY),
            })),
        };
    }

    pub fn operate(&self, op: Operation) {
        self.inner.borrow_mut().operations.push(op);
    }

    pub fn command(&self, cmd: Command) {
        self.inner.borrow_mut().commands.push(cmd);
    }

    pub fn tick(&self) -> Result<(), Error> {
        let inner = &mut self.inner.borrow_mut();

        let operations = mem::replace(
            &mut inner.operations,
            Vec::with_capacity(DEFAULT_VEC_CAPACITY),
        );
        for op in &operations {
            let cmd = inner.logic_engine.operate(op)?;
            inner.logic_engine.command(&cmd)?;
        }

        let commands = mem::replace(
            &mut inner.commands,
            Vec::with_capacity(DEFAULT_VEC_CAPACITY),
        );
        for cmd in &commands {
            inner.logic_engine.command(cmd)?;
        }

        let pool = inner.logic_engine.tick()?;
        inner.state_bus.dispatch(pool);
        return Ok(());
    }
}

pub struct SyncStateReg<S>
where
    S: StateData + StateDataStatic,
{
    rc_ptr: *const RefCell<SyncInner>,
    phantom: PhantomData<S>,
}

impl<S> !Send for SyncStateReg<S> {}
impl<S> !Sync for SyncStateReg<S> {}

impl<S> Drop for SyncStateReg<S>
where
    S: StateData + StateDataStatic,
{
    fn drop(&mut self) {
        if !self.rc_ptr.is_null() {
            let _ = unsafe { Weak::from_raw(self.rc_ptr) };
        }
    }
}

impl<S> Default for SyncStateReg<S>
where
    S: StateData + StateDataStatic,
{
    fn default() -> SyncStateReg<S> {
        return SyncStateReg {
            rc_ptr: ptr::null(),
            phantom: PhantomData,
        };
    }
}

impl<S> SyncStateReg<S>
where
    S: StateData + StateDataStatic,
{
    pub fn new(agent: &SyncLogicAgent) -> SyncStateReg<S> {
        return SyncStateReg {
            rc_ptr: Weak::into_raw(Rc::downgrade(&agent.inner)),
            phantom: PhantomData,
        };
    }
}

impl<S> StateReg<S> for SyncStateReg<S>
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
    fn test_sync_logic_agent() {
        let mut agent = SyncLogicAgent::new(fx(1.0 / 20.0));

        let state_ref: StateRef<StateStage, SyncStateReg<StateStage>> =
            StateRef::new(ObjID::from(100000), SyncStateReg::new(&agent));
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
