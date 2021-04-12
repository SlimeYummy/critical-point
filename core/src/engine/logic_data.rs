use super::logic_obj::{LogicObj, RefObj};
use crate::id::{ClassID, FastObjID};
use std::ffi::c_void;
use std::marker::PhantomData;
use std::mem;
use std::slice::Iter;
use derivative::Derivative;
use std::raw::TraitObject;

//
// LogicState & LogicProp
//

pub trait LogicPropStatic {
    fn id() -> ClassID;
}

pub trait LogicProp {
    fn class_id(&self) -> ClassID;
}

pub trait LogicStateStatic {
    fn id() -> ClassID;
}

pub trait LogicState {
    fn class_id(&self) -> ClassID;
    fn lifecycle(&self) -> LogicLifecycle;
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogicLifecycle {
    Created,
    Running,
    Destroyed,
}

impl Default for LogicLifecycle {
    fn default() -> LogicLifecycle {
        return LogicLifecycle::Created;
    }
}

//
// LogicData
//

static mut LOGIC_THREAD_INDEX: usize = 0;
static mut RENDER_THREAD_INDEX: usize = 1;

// call by logic thread
pub(crate) unsafe fn update_logic_thread_index() -> usize {
    LOGIC_THREAD_INDEX = (LOGIC_THREAD_INDEX + 1) % 2;
    return LOGIC_THREAD_INDEX;
}

// call by render thread
// logic thread send write_idx to render thread through channel
pub(crate) unsafe fn update_render_thread_index(logic_idx: usize) -> usize {
    RENDER_THREAD_INDEX = (logic_idx + 1) % 2;
    return RENDER_THREAD_INDEX;
}

#[derive(Debug)]
pub struct LogicData<P, S>
where
    P: LogicProp + LogicPropStatic,
    S: LogicState + LogicStateStatic,
{
    prop: P,
    state: [S; 2],
}

impl<P, S> !Sync for LogicData<P, S> {}
impl<P, S> !Send for LogicData<P, S> {}

impl<P, S> LogicData<P, S>
where
    P: LogicProp + LogicPropStatic,
    S: LogicState + LogicStateStatic + Default,
{
    pub fn new(prop: P) -> LogicData<P, S> {
        return LogicData {
            prop,
            state: [S::default(), S::default()],
        };
    }

    pub fn prop_ptr(&self) -> *const c_void {
        return &self.prop as *const _ as *const c_void;
    }

    pub fn state_ptr(&self) -> *const c_void {
        return &self.state[0] as *const _ as *const c_void;
    }

    pub fn prop(&mut self) -> &P {
        return &self.prop;
    }

    pub fn state(&mut self) -> &S {
        let idx = unsafe { LOGIC_THREAD_INDEX };
        return &self.state[idx];
    }

    pub fn state_mut(&mut self) -> &mut S {
        let idx = unsafe { LOGIC_THREAD_INDEX };
        return &mut self.state[idx];
    }
}

//
// RefData
//

#[repr(C)]
#[derive(Clone, Derivative)]
#[derivative(Debug)]
pub struct RefData<P, S> {
    #[derivative(Debug = "ignore")]
    obj: RefObj<dyn LogicObj>,
    class_id: ClassID,
    fobj_id: FastObjID,
    prop: *const c_void,
    state: *const c_void,
    phantom: PhantomData<(P, S)>,
}

impl<P, S> !Sync for RefData<P, S> {}
impl<P, S> !Send for RefData<P, S> {}

impl<P, S> RefData<P, S> {
    pub fn class_id(&self) -> ClassID {
        return self.class_id;
    }

    pub fn fobj_id(&self) -> FastObjID {
        return self.fobj_id;
    }

    pub fn is_valid(&self) -> bool {
        let to: TraitObject = unsafe { mem::transmute_copy(&self.obj) };
        return !to.vtable.is_null() && !to.data.is_null();
    }

    pub fn is_invalid(&self) -> bool {
        return !self.is_valid();
    }
}

impl RefData<(), ()> {
    #[inline]
    pub(crate) fn new<O>(obj: &RefObj<O>) -> RefData<(), ()>
    where
        O: LogicObj + 'static,
    {
        return RefData {
            obj: obj.clone(),
            class_id: obj.borrow().class_id(),
            fobj_id: obj.borrow().fobj_id(),
            prop: obj.borrow().prop_ptr(),
            state: obj.borrow().state_ptr(),
            phantom: PhantomData,
        };
    }

    #[inline]
    pub fn is<P, S>(&self) -> bool
    where
        P: LogicProp + LogicPropStatic,
        S: LogicState + LogicStateStatic,
    {
        return self.class_id == P::id() && self.class_id == S::id();
    }

    #[inline]
    pub fn cast<P, S>(self) -> Option<RefData<P, S>>
    where
        P: LogicProp + LogicPropStatic,
        S: LogicState + LogicStateStatic,
    {
        if self.class_id == P::id() && self.class_id == S::id() {
            return Some(RefData {
                obj: self.obj,
                class_id: self.class_id,
                fobj_id: self.fobj_id,
                prop: self.prop,
                state: self.state,
                phantom: PhantomData,
            });
        } else {
            return None;
        }
    }
}

impl<P, S> RefData<P, S>
where
    P: LogicProp + LogicPropStatic,
    S: LogicState + LogicStateStatic,
{
    #[inline]
    fn prop(&self) -> &P {
        return unsafe { &*(self.prop as *const P) };
    }

    #[inline]
    fn state(&self) -> &S {
        unsafe {
            let offset = RENDER_THREAD_INDEX as isize;
            return &*(self.state.offset(offset) as *const S);
        };
    }
}

//
// RefDataPool
//

pub struct RefDataPool(Vec<RefData<(), ()>>);

unsafe impl Sync for RefDataPool {}
unsafe impl Send for RefDataPool {}

impl RefDataPool {
    pub fn new() -> RefDataPool {
        return RefDataPool(Vec::new());
    }

    pub fn with_capacity(size: usize) -> RefDataPool {
        return RefDataPool(Vec::with_capacity(size));
    }

    pub fn push(&mut self, state: RefData<(), ()>) {
        return self.0.push(state);
    }

    pub fn len(&self) -> usize {
        return self.0.len();
    }

    pub fn iter(&self) -> Iter<'_, RefData<(), ()>> {
        return self.0.iter();
    }

    pub fn to_vec(self) -> Vec<RefData<(), ()>> {
        return self.0;
    }
}
