use libc::c_void;
use std::mem;
use std::raw::TraitObject;
use std::ptr;
use crate::id::ObjectID;
use super::StateData;
use super::base::{StateInner, state_vtable};

#[derive(Debug)]
pub struct StatePool {
    chunk_size: usize,
    threshold_size: usize,
    chunks: Vec<MemoryChunk>,
    buffers: MemoryBuffers,
    states: Vec<StateInner>,
}

impl StatePool {
    pub fn new(chunk_size: usize) -> StatePool {
        let mut pool = StatePool{
            chunk_size,
            threshold_size: chunk_size / 8,
            chunks: Vec::with_capacity(64),
            buffers: MemoryBuffers::new(128),
            states: Vec::with_capacity(1024),
        };
        pool.chunks.push(MemoryChunk::new(chunk_size));
        return pool;
    }

    pub fn make<S>(&mut self, obj_id: ObjectID) -> &mut S
        where S: StateData + Default + Sized
    {
        let size = (mem::size_of::<S>() + 15) & !15;
        let state =
            if size <= self.threshold_size {
                self.alloc_from_pool(size)
            } else {
                self.alloc_from_libc(size)
            };
        
        unsafe {
            ptr::write(state as *mut S, S::default());
            self.states.push(StateInner{
                obj_id,
                state: state,
                vtable: state_vtable::<S>(),
            });
            return (state as *mut S).as_mut().unwrap(); // must not null
        };
    }

    pub fn for_each<F>(&self, mut callback: F)
        where F: FnMut(ObjectID, *mut u8, *mut u8)
    {
        for inner in &self.states {
            callback(inner.obj_id, inner.state, inner.vtable);
        }
    }

    fn alloc_from_pool(&mut self, size: usize) -> *mut u8 {
        let last = self.chunks.last_mut().unwrap(); // must have one chunk
        let ptr = last.alloc(size);
        if !ptr.is_null() {
            return ptr;
        }

        self.chunks.push(MemoryChunk::new(self.chunk_size));
        let last = self.chunks.last_mut().unwrap(); // must have one chunk
        let ptr = last.alloc(size);
        if !ptr.is_null() {
            return ptr;
        }

        panic!("TickPool unexcepted error!");
    }

    fn alloc_from_libc(&mut self, size: usize) -> *mut u8 {
        return self.buffers.alloc(size);
    }
}

impl Drop for StatePool {
    fn drop(&mut self) {
        let states = mem::replace(&mut self.states, Vec::new());
        for inner in states {
            unsafe {
                let to: &mut dyn StateData = mem::transmute(TraitObject {
                    data: inner.state as *mut (),
                    vtable: inner.vtable as *mut (),
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
        return MemoryChunk{
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

#[derive(Debug)]
struct MemoryBuffers {
    buffers: Vec<*mut u8>,
}

impl MemoryBuffers {
    fn new(cap: usize) -> MemoryBuffers {
        return MemoryBuffers{
            buffers: Vec::with_capacity(cap),
        };
    }

    fn alloc(&mut self, size: usize) -> *mut u8 {
        let ptr = unsafe { libc::malloc(size) as *mut u8 };
        self.buffers.push(ptr);
        return ptr;
    }
}

impl Drop for MemoryBuffers {
    fn drop(&mut self) {
        let buffers = mem::replace(&mut self.buffers, Vec::new());
        for buffer in buffers {
            unsafe { libc::free(buffer as *mut c_void) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_chunk() {
        let mut mc = MemoryChunk::new(1024);
        assert_eq!(mc.size, 1024);

        let ptr1 = mc.alloc(64);
        assert_eq!(mc.buffer, ptr1);
        assert_eq!(mc.offset, 64);

        let ptr2 = mc.alloc(64);
        assert_eq!(unsafe { mc.buffer.offset(64) }, ptr2);
        assert_eq!(mc.offset, 128);

        let ptr3 = mc.alloc(1024);
        assert!(ptr3.is_null());
        assert_eq!(mc.offset, 128);
    }

    #[test]
    fn test_memory_buffers() {
        let mut mb = MemoryBuffers::new(32);

        let ptr1 = mb.alloc(64);
        assert_eq!(mb.buffers[0], ptr1);
        assert_eq!(mb.buffers.len(), 1);

        let ptr2 = mb.alloc(64);
        assert_eq!(mb.buffers[1], ptr2);
        assert_eq!(mb.buffers.len(), 2);
    }

    #[derive(Default)]
    struct StateTest {
        num: u32,
        text: String,
    }

    impl StateData for StateTest {}

    impl Drop for StateTest {
        fn drop(&mut self) {
            println!(
                "drop() => self({:?}) num({}) text({})",
                self as *mut Self,
                self.num,
                self.text,
            );
        }
    }

    #[derive(Default)]
    struct StateTest2 {
        u1: u128,
        u2: u128,
        u3: u128,
        u4: u128,
        u5: u128,
        u6: u128,
        u7: u128,
        u8: u128,
        u9: u128,
    }

    impl StateData for StateTest2 {}

    #[test]
    fn test_state_pool() {
        let mut sp = StatePool::new(1024);
        assert_eq!(sp.chunk_size, 1024);
        assert_eq!(sp.threshold_size, 1024 / 8);

        let state1 = sp.make::<StateTest>(ObjectID::from(1));
        assert_eq!(state1.num, 0);
        assert_eq!(state1.text, String::new());
        assert_eq!(sp.chunks[0].offset, mem::size_of::<StateTest>());

        let state2 = sp.make::<StateTest2>(ObjectID::from(2));
        assert_eq!(sp.buffers.buffers.len(), 1);
    }
}
