use core::slice;
use std::mem;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptEnv {
    Action,
    Skill,
    Buff,
    Expr,
    Test,
}

#[derive(Debug, Clone)]
pub struct ScriptEnvSet(u32);

impl ScriptEnvSet {
    pub fn new(envs: &[ScriptEnv]) -> ScriptEnvSet {
        let mut bits = 0;
        for env in envs {
            bits |= 1 << (*env as u32);
        }
        return ScriptEnvSet(bits);
    }

    pub fn conatin(&self, env: ScriptEnv) -> bool {
        return self.0 & (1 << (env as u32)) != 0;
    }
}

#[derive(Debug)]
pub struct ScriptByteCode {
    env: ScriptEnv,
    const_len: usize,
    code_len: usize,
    buffer: Vec<u8>,
}

impl ScriptByteCode {
    pub(super) fn new(
        env: ScriptEnv,
        const_segment: &[usize],
        code_segment: &[u16],
    ) -> ScriptByteCode {
        let const_bytes = const_segment.len() * mem::size_of::<usize>();
        let code_bytes = code_segment.len() * mem::size_of::<u16>();
        let mut buffer: Vec<u8> = Vec::with_capacity(code_bytes + const_bytes);
        buffer.extend_from_slice(unsafe {
            slice::from_raw_parts(const_segment.as_ptr() as *const _, const_bytes)
        });
        buffer.extend_from_slice(unsafe {
            slice::from_raw_parts(code_segment.as_ptr() as *const _, code_bytes)
        });

        return ScriptByteCode {
            env,
            const_len: const_segment.len(),
            code_len: code_segment.len(),
            buffer,
        };
    }

    pub fn env(&self) -> ScriptEnv {
        return self.env;
    }

    pub fn const_len(&self) -> usize {
        return self.const_len;
    }

    pub fn code_len(&self) -> usize {
        return self.const_len;
    }

    pub fn const_segment(&self) -> &[usize] {
        return unsafe { slice::from_raw_parts(self.buffer.as_ptr() as *const _, self.const_len) };
    }

    pub fn code_segment(&self) -> &[u16] {
        let offset = (self.const_len * mem::size_of::<usize>()) as isize;
        return unsafe {
            slice::from_raw_parts(
                self.buffer.as_ptr().offset(offset) as *const _,
                self.code_len,
            )
        };
    }
}
