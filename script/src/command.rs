use std::{fmt, lazy::SyncLazy};

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
    Pos,
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

    // exponential function
    Sqrt,
    Exp,

    // circular functions
    Degrees,
    Radians,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Atan2,

    Invalid,
}

impl ScriptOpt {
    #[inline]
    pub fn is_sign(&self) -> bool {
        return !SCRIPT_OPT_METAS[*self as usize].func;
    }

    #[inline]
    pub fn is_func(&self) -> bool {
        return SCRIPT_OPT_METAS[*self as usize].func;
    }

    #[inline]
    pub fn ident(&self) -> &'static str {
        return SCRIPT_OPT_METAS[*self as usize].ident;
    }

    #[inline]
    pub fn args(&self) -> &[bool] {
        return &SCRIPT_OPT_METAS[*self as usize].args;
    }
}

#[derive(Default, Debug, Clone)]
pub struct ScriptOptMeta {
    pub func: bool,
    pub ident: &'static str,
    pub args: Vec<bool>,
}

impl ScriptOptMeta {
    fn sign(args: &[u8]) -> ScriptOptMeta {
        return ScriptOptMeta {
            func: false,
            ident: "",
            args: args.iter().map(|x| *x != 0).collect(),
        };
    }

    fn func(ident: &'static str, args: &[u8]) -> ScriptOptMeta {
        return ScriptOptMeta {
            func: true,
            ident,
            args: args.iter().map(|x| *x != 0).collect(),
        };
    }
}

static SCRIPT_OPT_METAS: SyncLazy<Vec<ScriptOptMeta>> = SyncLazy::new(|| {
    use ScriptOpt::*;

    let mut metas = vec![ScriptOptMeta::default(); ScriptOpt::Invalid as usize + 1];
    metas[Jmp as usize] = ScriptOptMeta::sign(&[]);
    metas[JmpCmp as usize] = ScriptOptMeta::sign(&[]);
    metas[JmpSet as usize] = ScriptOptMeta::sign(&[]);
    metas[JmpCas0 as usize] = ScriptOptMeta::sign(&[]);
    metas[JmpCas1 as usize] = ScriptOptMeta::sign(&[]);
    metas[Pos as usize] = ScriptOptMeta::sign(&[1]);
    metas[Neg as usize] = ScriptOptMeta::sign(&[1]);
    metas[Not as usize] = ScriptOptMeta::sign(&[1]);
    metas[Mul as usize] = ScriptOptMeta::sign(&[1, 1]);
    metas[Div as usize] = ScriptOptMeta::sign(&[1, 1]);
    metas[Rem as usize] = ScriptOptMeta::sign(&[1, 1]);
    metas[Add as usize] = ScriptOptMeta::sign(&[1, 1]);
    metas[Sub as usize] = ScriptOptMeta::sign(&[1, 1]);
    metas[Lt as usize] = ScriptOptMeta::sign(&[1, 1]);
    metas[Le as usize] = ScriptOptMeta::sign(&[1, 1]);
    metas[Gt as usize] = ScriptOptMeta::sign(&[1, 1]);
    metas[Ge as usize] = ScriptOptMeta::sign(&[1, 1]);
    metas[Eq as usize] = ScriptOptMeta::sign(&[1, 1]);
    metas[Ne as usize] = ScriptOptMeta::sign(&[1, 1]);
    metas[IfElse0 as usize] = ScriptOptMeta::sign(&[1, 1, 1]);
    metas[IfElse1 as usize] = ScriptOptMeta::sign(&[1, 1, 1]);
    metas[Abs as usize] = ScriptOptMeta::func("abs", &[1]);
    metas[Min as usize] = ScriptOptMeta::func("min", &[1, 1]);
    metas[Max as usize] = ScriptOptMeta::func("max", &[1, 1]);
    metas[Floor as usize] = ScriptOptMeta::func("floor", &[1]);
    metas[Ceil as usize] = ScriptOptMeta::func("ceil", &[1]);
    metas[Round as usize] = ScriptOptMeta::func("round", &[1]);
    metas[Clamp as usize] = ScriptOptMeta::func("clamp", &[1, 1, 1]);
    metas[Saturate as usize] = ScriptOptMeta::func("saturate", &[1]);
    metas[Lerp as usize] = ScriptOptMeta::func("lerp", &[1, 1, 1]);
    metas[Sqrt as usize] = ScriptOptMeta::func("sqrt", &[1]);
    metas[Exp as usize] = ScriptOptMeta::func("exp", &[1]);
    metas[Degrees as usize] = ScriptOptMeta::func("degrees", &[1]);
    metas[Radians as usize] = ScriptOptMeta::func("radians", &[1]);
    metas[Sin as usize] = ScriptOptMeta::func("sin", &[1]);
    metas[Cos as usize] = ScriptOptMeta::func("cos", &[1]);
    metas[Tan as usize] = ScriptOptMeta::func("tan", &[1]);
    metas[Asin as usize] = ScriptOptMeta::func("asin", &[1]);
    metas[Acos as usize] = ScriptOptMeta::func("acos", &[1]);
    metas[Atan as usize] = ScriptOptMeta::func("atan", &[1]);
    metas[Atan2 as usize] = ScriptOptMeta::func("atan2", &[1, 1]);

    return metas;
});

#[repr(C)]
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct ScriptAddr(u16);

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
    pub fn new(segment: u8, offset: u16) -> ScriptAddr {
        let segment = segment as u16;
        let offset = offset.max(0).min(Self::max_offset()) as u16;
        return ScriptAddr((segment << 12) | offset);
    }

    pub fn segment(&self) -> u8 {
        return (self.0 >> 12) as u8;
    }

    pub fn offset(&self) -> u16 {
        return (self.0 & 0xFFF) as u16;
    }

    pub const fn max_offset() -> u16 {
        return 0xFFF;
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScriptCmd1 {
    pub opt: ScriptOpt,
    pub src: ScriptAddr,
    pub dst: ScriptAddr,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScriptCmd2 {
    pub opt: ScriptOpt,
    pub src1: ScriptAddr,
    pub src2: ScriptAddr,
    pub dst: ScriptAddr,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScriptCmd3 {
    pub opt: ScriptOpt,
    pub src1: ScriptAddr,
    pub src2: ScriptAddr,
    pub src3: ScriptAddr,
    pub dst: ScriptAddr,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScriptCmdJmp {
    pub opt: ScriptOpt,
    pub pc: ScriptAddr,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScriptCmdJmpCmp {
    pub opt: ScriptOpt,
    pub cond: ScriptAddr,
    pub pc: ScriptAddr,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScriptCmdJmpSet {
    pub opt: ScriptOpt,
    pub src: ScriptAddr,
    pub dst: ScriptAddr,
    pub pc: ScriptAddr,
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
