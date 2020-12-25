use super::{StateData, StateDataStatic, StateLifecycle};
use crate::id::{ClassID, FastObjID};
use libc::c_void;
use std::mem;
use std::ptr;
use std::raw::TraitObject;

//
// State Pool
//

#[derive(Debug)]
pub struct StatePoolItem {
    pub(crate) state: *mut u8,
    pub(crate) vtable: *mut u8,
    pub(crate) fobj_id: FastObjID,
    pub(crate) class_id: ClassID,
    pub(crate) lifecycle: StateLifecycle,
}

#[derive(Debug)]
pub struct StatePool {
    chunk_size: usize,
    threshold_size: usize,
    chunks: Vec<MemoryChunk>,
    buffers: MemoryBuffers,
    states: Vec<StatePoolItem>,
}

unsafe impl Sync for StatePool {}
unsafe impl Send for StatePool {}

impl StatePool {
    pub fn new(chunk_size: usize) -> StatePool {
        let mut pool = StatePool {
            chunk_size,
            threshold_size: chunk_size / 8,
            chunks: Vec::with_capacity(64),
            buffers: MemoryBuffers::new(128),
            states: Vec::with_capacity(1024),
        };
        pool.chunks.push(MemoryChunk::new(chunk_size));
        return pool;
    }

    pub fn make<S>(&mut self, fobj_id: FastObjID, lifecycle: StateLifecycle) -> &mut S
    where
        S: StateData + StateDataStatic,
    {
        let size = (mem::size_of::<S>() + 15) & !15;
        let ptr = if size <= self.threshold_size {
            self.alloc_from_pool(size)
        } else {
            self.alloc_from_libc(size)
        };

        unsafe {
            let state = &mut *(ptr as *mut S);
            ptr::write(state, S::init(fobj_id, lifecycle));

            self.states.push(StatePoolItem {
                state: ptr,
                vtable: state_vtable::<S>(),
                fobj_id: state.fobj_id(),
                class_id: state.class_id(),
                lifecycle: state.lifecycle(),
            });

            return state; // must not null
        };
    }

    pub fn for_each<F>(&self, mut callback: F)
    where
        F: FnMut(usize, &StatePoolItem),
    {
        for (index, item) in self.states.iter().enumerate() {
            callback(index, item);
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
        for item in states {
            unsafe {
                let to: &mut dyn StateData = mem::transmute(TraitObject {
                    data: item.state as *mut (),
                    vtable: item.vtable as *mut (),
                });
                ptr::drop_in_place(to);
            };
        }
    }
}

//
// Small state allocator
//

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

//
// Large state allocator
//

#[derive(Debug)]
struct MemoryBuffers {
    buffers: Vec<*mut u8>,
}

impl MemoryBuffers {
    fn new(cap: usize) -> MemoryBuffers {
        return MemoryBuffers {
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

//
// vtable
//

pub unsafe fn state_vtable<S: StateData>() -> *mut u8 {
    let re: &S = TransmuterPtr::<S> { n: 0 }.re;
    TransmuterTO::<dyn StateData> { re }.to.vtable as *mut u8
}

union TransmuterPtr<'t, T: 't> {
    n: isize,
    re: &'t T,
}

union TransmuterTO<'t, TO: ?Sized + 't> {
    re: &'t TO,
    to: TraitObject,
}

//
// tests
//

#[cfg(test)]
mod tests {
    use super::*;
    use derive::StateDataX;

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

    #[derive(StateDataX, Default)]
    #[class_id(StageGeneral)]
    struct StateTest {
        fobj_id: FastObjID,
        lifecycle: StateLifecycle,
        num: u32,
        text: String,
    }

    impl Drop for StateTest {
        fn drop(&mut self) {
            println!(
                "drop() => self({:?}) num({}) text({})",
                self as *mut Self, self.num, self.text,
            );
        }
    }

    #[derive(StateDataX)]
    #[class_id(StageGeneral)]
    struct StateTest2 {
        fobj_id: FastObjID,
        lifecycle: StateLifecycle,
        data: [u128; 8],
    }

    impl Default for StateTest2 {
        fn default() -> StateTest2 {
            return StateTest2 {
                fobj_id: FastObjID::default(),
                lifecycle: StateLifecycle::default(),
                data: [0u128; 8],
            };
        }
    }

    #[test]
    fn test_state_pool() {
        let mut sp = StatePool::new(512);
        assert_eq!(sp.chunk_size, 512);
        assert_eq!(sp.threshold_size, 512 / 8);

        let state1 = sp.make::<StateTest>(FastObjID::from(1), StateLifecycle::Updated);
        assert_eq!(state1.num, 0);
        assert_eq!(state1.text, String::new());
        assert_eq!(
            sp.chunks[0].offset,
            (mem::size_of::<StateTest>() + 15) & !15
        );

        let _ = sp.make::<StateTest2>(FastObjID::from(2), StateLifecycle::Updated);
        assert_eq!(sp.buffers.buffers.len(), 1);
    }
}
