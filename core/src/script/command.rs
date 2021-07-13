use math::Fx;
use std::fmt;
use std::mem;

//
// script operation
//

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScriptOpt {
    // jump
    Jmp,
    JmpCmp,  // if !expr { pc = addr; }
    JmpSet,  // stack.push(val); pc = addr;
    JmpCas0, // if !expr { stack.push(val); pc = addr; }
    JmpCas1, // if expr { stack.push(val); pc = addr; }

    // unary
    Mov,
    Neg,
    Not,

    // binary
    Mul,
    Div,
    Rem,
    Add,
    Sub,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,

    // ternary
    IfElse0, // if !expr { stack.push(x) } else { stack.push(y) }
    IfElse1, // if expr { stack.push(x) } else { stack.push(y) }

    // numeric functions
    Abs, // x => |x|
    Min, // a, b => a < b ? b : b
    Max, // a, b => a > b ? a : b
    Floor,
    Ceil,
    Round,
    Clamp,    // x, min, max => x in [min, max]
    Saturate, // x => x in [0, 1]
    Lerp,     // x, y, s => x + s(y - x)

    // exponential functions
    Sqrt,
    Exp,

    // circular functions
    Degrees,
    Radians,
    Sin,
    Cos,
    Tan,

    // extern function
    TestAddID,
    TestHasID,

    Invalid,
}

//
// script type & value
//

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptType {
    Num,
    ID,
}

#[derive(Clone, Copy)]
pub union ScriptVal {
    num: Fx,
    id: usize,
    pc: usize,
}

impl Default for ScriptVal {
    fn default() -> ScriptVal {
        return ScriptVal { id: 0 };
    }
}

impl ScriptVal {
    #[inline(always)]
    pub fn num(&self) -> Fx {
        return unsafe { self.num };
    }

    #[inline(always)]
    pub fn id(&self) -> usize {
        return unsafe { self.id };
    }

    #[inline(always)]
    pub fn pc(&self) -> usize {
        return unsafe { self.pc };
    }
}

impl From<Fx> for ScriptVal {
    #[inline(always)]
    fn from(num: Fx) -> ScriptVal {
        return ScriptVal { num };
    }
}

impl From<usize> for ScriptVal {
    #[inline(always)]
    fn from(id: usize) -> ScriptVal {
        return ScriptVal { id };
    }
}

impl PartialEq for ScriptVal {
    fn eq(&self, other: &ScriptVal) -> bool {
        return unsafe { self.id == other.id };
    }
}

//
// script address
//

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ScriptAddr(u16);

impl Default for ScriptAddr {
    fn default() -> ScriptAddr {
        return ScriptAddr(0xFFF);
    }
}

impl fmt::Debug for ScriptAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return f
            .debug_struct("ScriptAddr")
            .field("segment", &self.segment())
            .field("offset", &self.offset())
            .finish();
    }
}

impl ScriptAddr {
    #[inline(always)]
    pub fn new(segment: u8, offset: u16) -> ScriptAddr {
        let segment = segment as u16;
        let offset = offset.max(0).min(Self::max_offset()) as u16;
        return ScriptAddr((segment << 12) | offset);
    }

    #[inline(always)]
    pub fn segment(&self) -> u8 {
        return (self.0 >> 12) as u8;
    }

    #[inline(always)]
    pub fn offset(&self) -> u16 {
        return (self.0 & 0xFFF) as u16;
    }

    #[inline(always)]
    pub const fn max_offset() -> u16 {
        return 0xFFF;
    }
}

//
// script command
//

pub trait ScriptCmd {
    fn write(&self, code: &mut Vec<u16>);
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScriptCmdFunc<const N: usize> {
    pub opt: ScriptOpt,
    pub src: [ScriptAddr; N],
    pub dst: ScriptAddr,
}

impl<const N: usize> ScriptCmd for ScriptCmdFunc<N> {
    #[inline(always)]
    fn write(&self, code: &mut Vec<u16>) {
        unsafe {
            code.push(mem::transmute::<_, u16>(self.opt));
            for idx in 0..N {
                code.push(mem::transmute::<_, u16>(self.src[idx]));
            }
            code.push(mem::transmute::<_, u16>(self.dst));
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScriptCmdJmp {
    pub opt: ScriptOpt,
    pub pc: ScriptAddr,
}

impl ScriptCmd for ScriptCmdJmp {
    #[inline(always)]
    fn write(&self, code: &mut Vec<u16>) {
        unsafe { code.extend_from_slice(&mem::transmute::<_, [u16; 2]>(*self)) };
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScriptCmdJmpCmp {
    pub opt: ScriptOpt,
    pub cond: ScriptAddr,
    pub pc: ScriptAddr,
}

impl ScriptCmd for ScriptCmdJmpCmp {
    #[inline(always)]
    fn write(&self, code: &mut Vec<u16>) {
        unsafe { code.extend_from_slice(&mem::transmute::<_, [u16; 3]>(*self)) };
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScriptCmdJmpSet {
    pub opt: ScriptOpt,
    pub src: ScriptAddr,
    pub dst: ScriptAddr,
    pub pc: ScriptAddr,
}

impl ScriptCmd for ScriptCmdJmpSet {
    #[inline(always)]
    fn write(&self, code: &mut Vec<u16>) {
        unsafe { code.extend_from_slice(&mem::transmute::<_, [u16; 4]>(*self)) };
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScriptCmdJmpCas {
    pub opt: ScriptOpt,
    pub cond: ScriptAddr,
    pub src: ScriptAddr,
    pub dst: ScriptAddr,
    pub pc: ScriptAddr,
}

impl ScriptCmd for ScriptCmdJmpCas {
    #[inline(always)]
    fn write(&self, code: &mut Vec<u16>) {
        unsafe { code.extend_from_slice(&mem::transmute::<_, [u16; 5]>(*self)) };
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScriptCmdMethod<const N: usize> {
    pub opt: ScriptOpt,
    pub var_id: u8,
    pub var_seg: u8,
    pub src: [ScriptAddr; N],
    pub dst: ScriptAddr,
}

impl<const N: usize> ScriptCmd for ScriptCmdMethod<N> {
    #[inline(always)]
    fn write(&self, code: &mut Vec<u16>) {
        unsafe {
            code.push(mem::transmute::<_, u16>(self.opt));
            code.push(mem::transmute::<_, u16>([self.var_id, self.var_seg]));
            for idx in 0..N {
                code.push(mem::transmute::<_, u16>(self.src[idx]));
            }
            code.push(mem::transmute::<_, u16>(self.dst));
        }
    }
}
