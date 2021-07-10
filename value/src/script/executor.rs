use super::byte_code::ScriptByteCode;
use super::command::{
    ScriptAddr, ScriptCmdFunc, ScriptCmdJmp, ScriptCmdJmpCas, ScriptCmdJmpCmp, ScriptCmdMethod,
    ScriptOpt, ScriptVal,
};
use super::segment::{
    ScriptCtx, ScriptVar, MAX_REGISTERS, MAX_SEGMENTS, SEGMENT_CONSTANT, SEGMENT_REGISTER,
};
use super::test::VarTestOut;
use math::{fi, fx_bool, Fx, RealExt};
use simba::scalar::{ComplexField, RealField};
use std::mem::{self, MaybeUninit};
use thiserror::Error;

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptError {
    #[error("Class miss match")]
    ClassMissMatch,
    #[error("Out of range")]
    OutOfRange,
    #[error("Bad command")]
    BadCommand,
    #[error("Stack overflow")]
    StackOverflow,
}

pub struct ScriptExecutor {
    pc: usize,
    segments: [*mut ScriptVal; MAX_SEGMENTS],
    stack: [Fx; MAX_REGISTERS],
}

impl ScriptExecutor {
    pub fn new() -> ScriptExecutor {
        return ScriptExecutor {
            pc: 0,
            segments: unsafe { MaybeUninit::uninit().assume_init() },
            stack: unsafe { MaybeUninit::uninit().assume_init() },
        };
    }

    pub fn run<C: ScriptCtx>(
        &mut self,
        byte_code: &ScriptByteCode,
        context: C,
    ) -> Result<(), ScriptError> {
        if byte_code.ctx_id() != C::ctx_id() {
            return Err(ScriptError::ClassMissMatch);
        }
        self.segments[SEGMENT_CONSTANT as usize] =
            byte_code.const_segment().as_ptr() as *mut ScriptVal;
        self.segments[SEGMENT_REGISTER as usize] =
            self.stack.as_ptr() as *const _ as *mut ScriptVal;
        context.fill_segments(&mut self.segments[2..]);
        return self.run_impl(byte_code.code_segment());
    }

    pub fn run_impl(&mut self, code: &[u16]) -> Result<(), ScriptError> {
        self.pc = 0;
        while self.pc < code.len() {
            let opt = self.peek_opt(code)?;
            match opt {
                ScriptOpt::Jmp => self.jmp(code),
                ScriptOpt::JmpCmp => self.jmp_cmp(code),
                ScriptOpt::JmpSet => todo!(),
                ScriptOpt::JmpCas0 | ScriptOpt::JmpCas1 => {
                    let cond = opt == ScriptOpt::JmpCas1;
                    self.jmp_cas(code, cond);
                }
                ScriptOpt::Mov => self.func1(code, |x| x),
                ScriptOpt::Neg => self.func1(code, |x| -x),
                ScriptOpt::Not => self.func1(code, |x| fx_bool(x == Fx::c0())),
                ScriptOpt::Mul => self.func2(code, |x, y| x * y),
                ScriptOpt::Div => self.func2(code, |x, y| x / y),
                ScriptOpt::Rem => self.func2(code, |x, y| x % y),
                ScriptOpt::Add => self.func2(code, |x, y| x + y),
                ScriptOpt::Sub => self.func2(code, |x, y| x - y),
                ScriptOpt::Lt => self.func2(code, |x, y| fx_bool(x < y)),
                ScriptOpt::Le => self.func2(code, |x, y| fx_bool(x <= y)),
                ScriptOpt::Gt => self.func2(code, |x, y| fx_bool(x > y)),
                ScriptOpt::Ge => self.func2(code, |x, y| fx_bool(x >= y)),
                ScriptOpt::Eq => self.func2(code, |x, y| fx_bool(x == y)),
                ScriptOpt::Ne => self.func2(code, |x, y| fx_bool(x != y)),
                ScriptOpt::IfElse0 | ScriptOpt::IfElse1 => self.func3(code, |c, x, y| {
                    let cond = opt == ScriptOpt::IfElse1;
                    return if (c != Fx::c0()) == cond { x } else { y };
                }),
                ScriptOpt::Abs => self.func1(code, |x| x.abs()),
                ScriptOpt::Min => self.func2(code, |x, y| RealField::min(x, y)),
                ScriptOpt::Max => self.func2(code, |x, y| RealField::max(x, y)),
                ScriptOpt::Floor => self.func1(code, |x| x.floor()),
                ScriptOpt::Ceil => self.func1(code, |x| x.ceil()),
                ScriptOpt::Round => self.func1(code, |x| x.round()),
                ScriptOpt::Clamp => self.func3(code, |x, min, max| RealField::clamp(x, min, max)),
                ScriptOpt::Saturate => self.func1(code, |x| RealField::clamp(x, fi(0), fi(1))),
                ScriptOpt::Lerp => self.func3(code, |src, dst, step| src + step * (dst - src)),
                ScriptOpt::Sqrt => self.func1(code, |x| x.sqrt()),
                ScriptOpt::Exp => self.func1(code, |x| x.exp()),
                ScriptOpt::Degrees => self.func1(code, |x| Fx::frac_180_pi() * x),
                ScriptOpt::Radians => self.func1(code, |x| Fx::frac_pi_180() * x),
                ScriptOpt::Sin => self.func1(code, |x| x.sin()),
                ScriptOpt::Cos => self.func1(code, |x| x.cos()),
                ScriptOpt::Tan => self.func1(code, |x| x.tan()),
                ScriptOpt::TestAddID => self.ext_stat(code, |t: &mut VarTestOut, [a]| {
                    t.add_id(a.id());
                }),
                ScriptOpt::TestHasID => self.ext_expr(code, |t: &VarTestOut, [a]| {
                    return t.has_id(a.id()).into();
                }),
                ScriptOpt::Invalid => unreachable!(),
            };
        }
        return Ok(());
    }

    #[inline(always)]
    fn peek_opt(&mut self, code: &[u16]) -> Result<ScriptOpt, ScriptError> {
        let opt: ScriptOpt = unsafe { mem::transmute(code[self.pc]) };
        if opt >= ScriptOpt::Invalid {
            return Err(ScriptError::BadCommand);
        }
        return Ok(opt);
    }

    #[inline(always)]
    fn jmp(&mut self, code: &[u16]) {
        let cmd = unsafe { &*(&code[self.pc] as *const _ as *const ScriptCmdJmp) };
        self.pc = self.read_pc(&cmd.pc);
        println!("{:?}: >> {}", cmd.opt, self.pc);
    }

    #[inline(always)]
    fn jmp_cmp(&mut self, code: &[u16]) {
        let cmd = unsafe { &*(&code[self.pc] as *const _ as *const ScriptCmdJmpCmp) };
        let cond = self.read(&cmd.cond).num();
        if cond == Fx::c0() {
            self.pc = self.read_pc(&cmd.pc);
            println!("{:?}: {} >> {}", cmd.opt, cond, self.pc);
        } else {
            self.pc += mem::size_of::<ScriptCmdJmpCmp>() / mem::size_of::<u16>();
            println!("{:?}: {}", cmd.opt, cond);
        }
    }

    #[inline(always)]
    fn jmp_cas(&mut self, code: &[u16], cmd_cond: bool) {
        let cmd = unsafe { &*(&code[self.pc] as *const _ as *const ScriptCmdJmpCas) };
        let cond = self.read(&cmd.cond).num();
        if (cond != Fx::c0()) == cmd_cond {
            self.pc = self.read_pc(&cmd.pc);
            let val = self.read(&cmd.src).num();
            self.write(&cmd.dst, val.into());
            println!("{:?}: {} => {} >> {}", cmd.opt, cond, val, self.pc);
        } else {
            self.pc += mem::size_of::<ScriptCmdJmpCas>() / mem::size_of::<u16>();
            println!("{:?}: {}", cmd.opt, cond);
        }
    }

    #[inline(always)]
    fn func1<F>(&mut self, code: &[u16], lambda: F)
    where
        F: FnOnce(Fx) -> Fx,
    {
        let cmd = unsafe { &*(&code[self.pc] as *const _ as *const ScriptCmdFunc<1>) };
        self.pc += mem::size_of::<ScriptCmdFunc<1>>() / mem::size_of::<u16>();

        let src = self.read(&cmd.src[0]).num();
        let dst = lambda(src);
        self.write(&cmd.dst, dst.into());

        println!("{:?}: {:?} => {}", cmd.opt, src, dst);
    }

    #[inline(always)]
    fn func2<F>(&mut self, code: &[u16], lambda: F)
    where
        F: FnOnce(Fx, Fx) -> Fx,
    {
        let cmd = unsafe { &*(&code[self.pc] as *const _ as *const ScriptCmdFunc<2>) };
        self.pc += mem::size_of::<ScriptCmdFunc<2>>() / mem::size_of::<u16>();

        let src1 = self.read(&cmd.src[0]).num();
        let src2 = self.read(&cmd.src[1]).num();
        let dst = lambda(src1, src2);
        self.write(&cmd.dst, dst.into());

        println!("{:?}: {} {} => {}", cmd.opt, src1, src2, dst);
    }

    #[inline(always)]
    fn func3<F>(&mut self, code: &[u16], lambda: F)
    where
        F: FnOnce(Fx, Fx, Fx) -> Fx,
    {
        let cmd = unsafe { &*(&code[self.pc] as *const _ as *const ScriptCmdFunc<3>) };
        self.pc += mem::size_of::<ScriptCmdFunc<3>>() / mem::size_of::<u16>();

        let src1 = self.read(&cmd.src[0]).num();
        let src2 = self.read(&cmd.src[1]).num();
        let src3 = self.read(&cmd.src[2]).num();
        let dst = lambda(src1, src2, src3);
        self.write(&cmd.dst, dst.into());

        println!("{:?}: {} {} {} => {}", cmd.opt, src1, src2, src3, dst);
    }

    #[inline(always)]
    fn func<F, const N: usize>(&mut self, code: &[u16], lambda: F)
    where
        F: FnOnce([ScriptVal; N]) -> ScriptVal,
    {
        let cmd = unsafe { &*(&code[self.pc] as *const _ as *const ScriptCmdFunc<3>) };
        self.pc += mem::size_of::<ScriptCmdFunc<3>>() / mem::size_of::<u16>();

        let mut src = [ScriptVal::default(); N];
        for idx in 0..N {
            src[idx] = self.read(&cmd.src[0]);
        }
        let dst = lambda(src);
        self.write(&cmd.dst, dst);
    }

    #[inline(always)]
    fn ext_stat<V, F, const N: usize>(&mut self, code: &[u16], lambda: F)
    where
        V: ScriptVar,
        F: FnOnce(&mut V, [ScriptVal; N]),
    {
        let cmd = unsafe { &*(&code[self.pc] as *const _ as *const ScriptCmdMethod<N>) };
        self.pc += mem::size_of::<ScriptCmdMethod<N>>() / mem::size_of::<u16>();

        let mut src: [ScriptVal; N] = [0.into(); N];
        for idx in 0..N {
            src[idx] = self.read(&cmd.src[idx]);
        }

        let var = unsafe { self.var_at(cmd.var_id, cmd.var_seg) };
        lambda(var, src);
    }

    #[inline(always)]
    fn ext_expr<V, F, const N: usize>(&mut self, code: &[u16], lambda: F)
    where
        V: ScriptVar,
        F: FnOnce(&V, [ScriptVal; N]) -> ScriptVal,
    {
        let cmd = unsafe { &*(&code[self.pc] as *const _ as *const ScriptCmdMethod<N>) };
        self.pc += mem::size_of::<ScriptCmdMethod<N>>() / mem::size_of::<u16>();

        let mut src: [ScriptVal; N] = [0.into(); N];
        for idx in 0..N {
            src[idx] = self.read(&cmd.src[idx]);
        }

        let var = unsafe { self.var_at(cmd.var_id, cmd.var_seg) };
        let dst = lambda(var, src);
        self.write(&cmd.dst, dst);
    }

    #[inline(always)]
    fn read_pc(&self, addr: &ScriptAddr) -> usize {
        let seg = addr.segment() as isize;
        let off = addr.offset() as isize;
        unsafe {
            let segments = self.segments.as_ptr();
            let segment = segments.offset(seg).read();
            let data = segment.offset(off).read().id();
            return data;
        }
    }

    #[inline(always)]
    fn read(&self, addr: &ScriptAddr) -> ScriptVal {
        let seg = addr.segment() as isize;
        let off = addr.offset() as isize;
        unsafe {
            let segments = self.segments.as_ptr();
            let segment = segments.offset(seg).read();
            return segment.offset(off).read();
        }
    }

    #[inline(always)]
    fn write(&self, addr: &ScriptAddr, val: ScriptVal) {
        let seg = addr.segment() as isize;
        let off = addr.offset() as isize;
        unsafe {
            let segments = self.segments.as_ptr();
            let segment = segments.offset(seg).read();
            segment.offset(off).write(val);
        }
    }

    #[cfg(not(unsafe_fast))]
    #[inline(always)]
    unsafe fn var_at<V: ScriptVar>(&self, var_id: u8, var_seg: u8) -> &mut V {
        let segments = self.segments.as_ptr();
        let segment = segments.offset(var_seg as isize).read();
        assert_eq!(V::var_id(), var_id);
        return &mut *(segment as *mut V);
    }

    #[cfg(unsafe_fast)]
    #[inline(always)]
    unsafe fn var_at<V: ScriptVar>(&self, var_id: u8, var_seg: u8) -> &mut V {
        let segments = self.segments.as_ptr();
        let segment = segments.offset(var_seg as isize).read();
        return &mut *(segment as *mut V);
    }
}

#[cfg(test)]
mod tests {
    use super::super::generator::ScriptGenerator;
    use super::super::parser::ScriptParser;
    use super::super::test::{CtxTest, VarTestIn, VarTestOut};
    use super::*;
    use math::{ff, fi};

    #[test]
    fn test_executor_func() {
        let mut test_out = VarTestOut::default();
        let mut test_in = VarTestIn::default();
        test_in.aa = fi(2);

        let mut parser = ScriptParser::new();
        let mut generator = ScriptGenerator::new();
        let mut executor = ScriptExecutor::new();

        let code = "test_out.xx = (1 + test_in.aa) * (4 - 3 * 0.5)";
        let ast = parser.run::<CtxTest>(code).unwrap();
        let byte_code = generator.run::<CtxTest>(ast).unwrap();
        executor
            .run(&byte_code, CtxTest::new(&test_in, &mut test_out))
            .unwrap();
        assert_eq!(test_out.xx, ff(7.5));
    }

    #[test]
    fn test_executor_if_stat() {
        let mut test_out = VarTestOut::default();
        let mut test_in = VarTestIn::default();
        test_in.aa = fi(2);
        test_in.bb = ff(2.5);
        test_in.cc = fi(-3);

        let mut parser = ScriptParser::new();
        let mut generator = ScriptGenerator::new();
        let mut executor = ScriptExecutor::new();

        let code = "
            if test_in.aa < PI {
                test_out.yy = -5
            } else {
                test_out.yy = 10
            }
        ";
        let ast = parser.run::<CtxTest>(code).unwrap();
        let byte_code = generator.run::<CtxTest>(ast).unwrap();
        executor
            .run(&byte_code, CtxTest::new(&test_in, &mut test_out))
            .unwrap();
        assert_eq!(test_out.yy, fi(-5));

        let code = "
            if 0.0 {
                test_out.yy = -5
            } elsif 2 < 1 {
            } else {
                test_out.yy = 10
                test_out.zz = test_in.bb + test_in.cc
            }
        ";
        let ast = parser.run::<CtxTest>(code).unwrap();
        let byte_code = generator.run::<CtxTest>(ast).unwrap();
        executor
            .run(&byte_code, CtxTest::new(&test_in, &mut test_out))
            .unwrap();
        assert_eq!(test_out.yy, fi(10));
        assert_eq!(test_out.zz, ff(-0.5));
    }

    #[test]
    fn test_executor_logic() {
        let mut test_out = VarTestOut::default();
        let mut test_in = VarTestIn::default();
        test_in.dd = ff(-1.5);

        let mut parser = ScriptParser::new();
        let mut generator = ScriptGenerator::new();
        let mut executor = ScriptExecutor::new();

        let code = "test_out.zz = test_in.dd + 1.5 && 3";
        let ast = parser.run::<CtxTest>(code).unwrap();
        let byte_code = generator.run::<CtxTest>(ast).unwrap();
        executor
            .run(&byte_code, CtxTest::new(&test_in, &mut test_out))
            .unwrap();
        assert_eq!(test_out.zz, fi(0));

        let code = "test_out.zz = test_in.dd || 3";
        let ast = parser.run::<CtxTest>(code).unwrap();
        let byte_code = generator.run::<CtxTest>(ast).unwrap();
        executor
            .run(&byte_code, CtxTest::new(&test_in, &mut test_out))
            .unwrap();
        assert_eq!(test_out.zz, ff(-1.5));

        let code = "test_out.zz = 9 && test_in.dd + 5";
        let ast = parser.run::<CtxTest>(code).unwrap();
        let byte_code = generator.run::<CtxTest>(ast).unwrap();
        executor
            .run(&byte_code, CtxTest::new(&test_in, &mut test_out))
            .unwrap();
        assert_eq!(test_out.zz, ff(3.5));

        let code = "test_out.zz = 9 || test_in.dd + 6";
        let ast = parser.run::<CtxTest>(code).unwrap();
        let byte_code = generator.run::<CtxTest>(ast).unwrap();
        executor
            .run(&byte_code, CtxTest::new(&test_in, &mut test_out))
            .unwrap();
        assert_eq!(test_out.zz, fi(9));
    }

    #[test]
    fn test_executor_method() {
        let mut test_out = VarTestOut::default();
        let test_in = VarTestIn::default();
        test_out.ids.push(1);
        test_out.ids.push(2);

        let mut parser = ScriptParser::new();
        let mut generator = ScriptGenerator::new();
        let mut executor = ScriptExecutor::new();

        let code = "
            test_out.add_id($id.id)
            test_out.xx = test_out.has_id($id.id)
        ";
        let ast = parser.run::<CtxTest>(code).unwrap();
        let byte_code = generator.run::<CtxTest>(ast).unwrap();
        executor
            .run(&byte_code, CtxTest::new(&test_in, &mut test_out))
            .unwrap();
        assert_eq!(test_out.ids, vec![1, 2, 0]);
        assert_eq!(test_out.xx, ff(1.0));
    }
}
