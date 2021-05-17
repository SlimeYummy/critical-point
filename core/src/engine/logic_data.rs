use crate::derive::def_enum;
use crate::id::{ClassID, ObjID};
use crate::utils;
use anyhow::{anyhow, Result};
use std::any::Any;
use std::ffi::c_void;
use std::mem;
use std::ptr;
use std::raw::TraitObject;

//
// Lifecycle
//

#[def_enum]
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
// LogicProp & LogicState
//

pub trait LogicPropStatic {
    fn id() -> ClassID;
}

pub trait LogicStateStatic {
    fn id() -> ClassID;
}

#[repr(C)]
#[derive(Debug)]
pub struct LogicProp<P> {
    obj_id: ObjID,
    class_id: ClassID,
    vtable: *mut u8,
    _padding_: [u8; 8], // for 16bit align
    prop: P,
}

#[repr(C)]
#[derive(Debug)]
pub struct LogicState<S> {
    obj_id: ObjID,
    class_id: ClassID,
    lifecycle: LogicLifecycle,
    vtable: *mut u8,
    _padding_: [u8; 8], // for 16bit align
    state: S,
}

//
// Data Pool
//

#[derive(Debug)]
pub struct DataPool {
    props: Vec<*mut LogicProp<()>>,
    states: Vec<*mut LogicState<()>>,
    chunk_size: usize,
    threshold_size: usize,
    chunks: Vec<MemoryChunk>,
}

unsafe impl Sync for DataPool {}
unsafe impl Send for DataPool {}

impl DataPool {
    pub fn new(chunk_size: usize) -> DataPool {
        let mut pool = DataPool {
            props: Vec::with_capacity(256),
            states: Vec::with_capacity(1024),
            chunk_size,
            threshold_size: chunk_size / 8,
            chunks: Vec::with_capacity(64),
        };
        pool.chunks.push(MemoryChunk::new(chunk_size));
        return pool;
    }

    pub fn prop<P>(&mut self, obj_id: ObjID, prop: P) -> Result<&mut LogicProp<P>>
    where
        P: LogicPropStatic + 'static,
    {
        let size = (mem::size_of::<LogicProp<P>>() + 15) & !15;
        let ptr = self.alloc(size)?;
        unsafe {
            ptr::write(
                ptr as *mut LogicProp<P>,
                LogicProp {
                    obj_id,
                    class_id: P::id(),
                    vtable: utils::any_vtable::<P>(),
                    _padding_: [0u8; 8],
                    prop,
                },
            );
        };
        self.props.push(ptr as *mut LogicProp<()>);
        return Ok(unsafe { &mut *(ptr as *mut LogicProp<P>) });
    }

    pub fn state<S>(
        &mut self,
        obj_id: ObjID,
        lifecycle: LogicLifecycle,
        state: S,
    ) -> Result<&mut LogicState<S>>
    where
        S: LogicStateStatic + 'static,
    {
        let size = (mem::size_of::<LogicProp<S>>() + 15) & !15;
        let ptr = self.alloc(size)?;
        unsafe {
            ptr::write(
                ptr as *mut LogicState<S>,
                LogicState {
                    obj_id,
                    class_id: S::id(),
                    lifecycle: lifecycle,
                    vtable: utils::any_vtable::<S>(),
                    _padding_: [0u8; 8],
                    state,
                },
            );
        };
        self.states.push(ptr as *mut LogicState<()>);
        return Ok(unsafe { &mut *(ptr as *mut LogicState<S>) });
    }

    fn alloc(&mut self, size: usize) -> Result<*mut u8> {
        if size > self.threshold_size {
            return Err(anyhow!("DataPool::alloc() => memory too large"));
        }

        let ptr = self.chunks.last_mut().unwrap().alloc(size);
        if !ptr.is_null() {
            return Ok(ptr);
        }

        self.chunks.push(MemoryChunk::new(self.chunk_size));
        let ptr = self.chunks.last_mut().unwrap().alloc(size);
        if !ptr.is_null() {
            return Ok(ptr);
        }

        return Err(anyhow!("DataPool::alloc() => unexcepted error"));
    }
}

impl Drop for DataPool {
    fn drop(&mut self) {
        let props = mem::replace(&mut self.props, Vec::new());
        for prop in props {
            unsafe {
                let prop = &mut *prop;
                let to: &mut dyn Any = mem::transmute(TraitObject {
                    data: &mut prop.prop as *mut (),
                    vtable: prop.vtable as *mut (),
                });
                ptr::drop_in_place(to);
            };
        }

        let states = mem::replace(&mut self.states, Vec::new());
        for state in states {
            unsafe {
                let state = &mut *state;
                let to: &mut dyn Any = mem::transmute(TraitObject {
                    data: &mut state.state as *mut (),
                    vtable: state.vtable as *mut (),
                });
                ptr::drop_in_place(to);
            };
        }
    }
}

#[derive(Debug)]
struct MemoryChunk {
    size: usize,
    offset: usize,
    buffer: *mut u8,
}

impl MemoryChunk {
    fn new(size: usize) -> MemoryChunk {
        return MemoryChunk {
            size: size,
            offset: 0,
            buffer: unsafe { libc::malloc(size) as *mut u8 },
        };
    }

    fn alloc(&mut self, size: usize) -> *mut u8 {
        if size > self.size - self.offset {
            return ptr::null_mut();
        }
        let ptr = unsafe { self.buffer.offset(self.offset as isize) };
        self.offset += size;
        return ptr as *mut u8;
    }
}

impl Drop for MemoryChunk {
    fn drop(&mut self) {
        if !self.buffer.is_null() {
            unsafe { libc::free(self.buffer as *mut c_void) }
            self.buffer = ptr::null_mut();
        }
        self.size = 0;
        self.offset = 0;
    }
}
