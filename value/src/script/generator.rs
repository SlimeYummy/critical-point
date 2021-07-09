use super::ast::{
    AstBlock, AstExpr, AstExprBranch, AstExprFunc, AstExprLogic, AstExprMethod, AstLogicType,
    AstStat, AstStatAssign, AstStatBranch, AstStatMethod,
};
use super::byte_code::ScriptByteCode;
use super::command::{
    ScriptAddr, ScriptCmd, ScriptCmdFunc, ScriptCmdJmp, ScriptCmdJmpCas, ScriptCmdJmpCmp,
    ScriptCmdJmpSet, ScriptCmdMethod, ScriptOpt,
};
use super::segment::{
    ScriptCtx, ScriptVar, MAX_CONSTANTS, MAX_REGISTERS, SEGMENT_CONSTANT, SEGMENT_REGISTER,
};
use anyhow::{anyhow, Result};
use math::Fx;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::mem;

pub struct ScriptGenerator {
    register_max: u16,
    register_heap: BinaryHeap<Reverse<u16>>,
    code_writer: CodeWriter,
    const_writer: ConstWriter,
}

impl ScriptGenerator {
    pub fn new() -> ScriptGenerator {
        return ScriptGenerator {
            register_max: 0,
            register_heap: BinaryHeap::new(),
            code_writer: CodeWriter::default(),
            const_writer: ConstWriter::default(),
        };
    }

    pub fn run<C: ScriptCtx>(&mut self, block: AstBlock) -> Result<ScriptByteCode> {
        return self.run_impl(block, C::ctx_id());
    }

    pub fn run_impl(&mut self, block: AstBlock, ctx_id: u8) -> Result<ScriptByteCode> {
        self.visit_block(&block)?;

        let byte_code = ScriptByteCode::new(
            ctx_id,
            &self.const_writer.inner(),
            &self.code_writer.inner(),
        );

        self.register_max = 0;
        self.register_heap.clear();
        self.code_writer.clear();
        self.const_writer.clear();

        return Ok(byte_code);
    }

    fn visit_block(&mut self, block: &AstBlock) -> Result<()> {
        self.register_max = 0;
        self.register_heap.clear();
        for stat in &block.stats {
            let jump_to_ends = self.visit_stat(stat)?;

            // jump to if statement end
            if !jump_to_ends.is_empty() {
                let pc_addr = self.const_writer.write_pc(self.code_writer.len())?;
                for to_end in &jump_to_ends {
                    self.code_writer.update_addr(*to_end, pc_addr);
                }
            }
        }
        return Ok(());
    }

    fn visit_stat(&mut self, stat: &AstStat) -> Result<Vec<usize>> {
        return match stat {
            AstStat::Assign(assign) => self.visit_stat_assign(assign),
            AstStat::Method(method) => self.visit_stat_method(method),
            AstStat::Branch(branch) => self.visit_stat_branch(branch),
        };
    }

    fn visit_stat_assign(&mut self, assign: &AstStatAssign) -> Result<Vec<usize>> {
        let expr_addr = self.visit_expr(&assign.expr)?;

        match assign.opt {
            None => {
                if assign.expr.is_normal() && expr_addr.segment() == SEGMENT_REGISTER {
                    let last = self.code_writer.len() - 1;
                    self.code_writer.update_addr(last, assign.var);
                } else {
                    self.code_writer.write(&ScriptCmdFunc {
                        opt: ScriptOpt::Mov,
                        src: [expr_addr],
                        dst: assign.var,
                    });
                }
            }
            Some(opt) => {
                self.code_writer.write(&ScriptCmdFunc {
                    opt,
                    src: [expr_addr, assign.var],
                    dst: assign.var,
                });
            }
        };

        self.free_register(expr_addr);
        return Ok(Vec::new());
    }

    fn visit_stat_method(&mut self, method: &AstStatMethod) -> Result<Vec<usize>> {
        let cmd = ScriptCmdMethod {
            opt: method.opt,
            var_id: method.var_id,
            var_seg: method.var_seg,
            src: [self.visit_expr(&method.args[0])?],
            dst: ScriptAddr::default(),
        };
        self.free_registers(&cmd.src);
        self.code_writer.write(&cmd);
        return Ok(Vec::new());
    }

    fn visit_stat_branch(&mut self, branch: &AstStatBranch) -> Result<Vec<usize>> {
        let mut jump_to_ends = Vec::new();

        // if/elsif condition
        let mut jump_to_next = ScriptAddr::default();
        if let Some(cond) = &branch.cond {
            let cond_expr = self.visit_expr(&cond)?;
            jump_to_next = self.const_writer.write_pc(0)?; // jump to next branch
            self.code_writer.write(&ScriptCmdJmpCmp {
                opt: ScriptOpt::JmpCmp,
                cond: cond_expr,
                pc: jump_to_next,
            });
        }

        // if/elsif/else statements
        let mut stat_jump_ends = Vec::new();
        for stat in &branch.stats {
            // jump to if statement end
            if !stat_jump_ends.is_empty() {
                let pc_addr = self.const_writer.write_pc(self.code_writer.len())?;
                for to_end in &stat_jump_ends {
                    self.code_writer.update_addr(*to_end, pc_addr);
                }
            }

            stat_jump_ends = self.visit_stat(stat)?;
        }
        jump_to_ends.extend(stat_jump_ends.iter());

        // elsif/else next branch
        if branch.next.is_some() {
            let jump_to_end = self.code_writer.write(&ScriptCmdJmp {
                opt: ScriptOpt::Jmp,
                pc: ScriptAddr::default(),
            });
            jump_to_ends.push(jump_to_end);
        }

        // if/elsif condition
        if branch.cond.is_some() {
            // jump to next branch
            self.const_writer
                .update_pc(jump_to_next, self.code_writer.len())?;
        }

        // elsif/else next branch
        if let Some(next) = &branch.next {
            let branch_jump_ends = self.visit_stat_branch(&next)?;
            jump_to_ends.extend(branch_jump_ends.iter());
        }

        return Ok(jump_to_ends);
    }

    fn visit_expr(&mut self, expr: &AstExpr) -> Result<ScriptAddr> {
        return match expr {
            AstExpr::Num(num) => self.visit_expr_fx(*num),
            AstExpr::ID(id) => self.visit_expr_id(*id),
            AstExpr::Var(addr) => self.visit_expr_var(*addr),
            AstExpr::Func(func) => self.visit_expr_func(func),
            AstExpr::Method(ext) => self.visit_expr_method(ext),
            AstExpr::Branch(branch) => self.visit_expr_branch(branch),
            AstExpr::Logic(logic) => self.visit_expr_logic(logic),
        };
    }

    fn visit_expr_fx(&mut self, num: Fx) -> Result<ScriptAddr> {
        return self.const_writer.write_num(num);
    }

    fn visit_expr_id(&mut self, id: usize) -> Result<ScriptAddr> {
        return self.const_writer.write_id(id);
    }

    fn visit_expr_var(&mut self, addr: ScriptAddr) -> Result<ScriptAddr> {
        return Ok(addr);
    }

    fn visit_expr_func(&mut self, normal: &AstExprFunc) -> Result<ScriptAddr> {
        match normal.args.len() {
            1 => {
                let mut cmd = ScriptCmdFunc {
                    opt: normal.opt,
                    src: [self.visit_expr(&normal.args[0])?],
                    dst: ScriptAddr::default(),
                };
                self.free_registers(&cmd.src);

                cmd.dst = self.alloc_register()?;
                self.code_writer.write(&cmd);
                return Ok(cmd.dst);
            }
            2 => {
                let mut cmd = ScriptCmdFunc {
                    opt: normal.opt,
                    src: [
                        self.visit_expr(&normal.args[0])?,
                        self.visit_expr(&normal.args[1])?,
                    ],
                    dst: ScriptAddr::default(),
                };
                self.free_registers(&cmd.src);

                cmd.dst = self.alloc_register()?;
                self.code_writer.write(&cmd);
                return Ok(cmd.dst);
            }
            3 => {
                let mut cmd = ScriptCmdFunc {
                    opt: normal.opt,
                    src: [
                        self.visit_expr(&normal.args[0])?,
                        self.visit_expr(&normal.args[1])?,
                        self.visit_expr(&normal.args[2])?,
                    ],
                    dst: ScriptAddr::default(),
                };
                self.free_registers(&cmd.src);

                cmd.dst = self.alloc_register()?;
                self.code_writer.write(&cmd);
                return Ok(cmd.dst);
            }
            _ => return Err(anyhow!("Arguments too much {:?}", normal.opt)),
        }
    }

    fn visit_expr_method(&mut self, method: &AstExprMethod) -> Result<ScriptAddr> {
        let mut cmd = ScriptCmdMethod {
            opt: method.opt,
            var_id: method.var_id,
            var_seg: method.var_seg,
            src: [self.visit_expr(&method.args[0])?],
            dst: ScriptAddr::default(),
        };
        self.free_registers(&cmd.src);

        cmd.dst = self.alloc_register()?;
        self.code_writer.write(&cmd);

        return Ok(cmd.dst);
    }

    fn visit_expr_branch(&mut self, branch: &AstExprBranch) -> Result<ScriptAddr> {
        unimplemented!();
    }

    fn visit_expr_logic(&mut self, logic: &AstExprLogic) -> Result<ScriptAddr> {
        let left_expr = self.visit_expr(&logic.left)?;

        // * || (fx|var)
        if logic.right.is_fx() || logic.right.is_var() {
            let right_expr = self.visit_expr(&logic.right)?;

            let opt = match logic.typ {
                AstLogicType::And => ScriptOpt::IfElse0,
                AstLogicType::Or => ScriptOpt::IfElse1,
            };

            let cmd = ScriptCmdFunc {
                opt,
                src: [left_expr, left_expr, right_expr],
                dst: self.alloc_register()?,
            };
            self.code_writer.write(&cmd);
            return Ok(cmd.dst);

        // * || expr
        } else {
            let opt = match logic.typ {
                AstLogicType::And => ScriptOpt::JmpCas0,
                AstLogicType::Or => ScriptOpt::JmpCas1,
            };

            let cmd = ScriptCmdJmpCas {
                opt,
                cond: left_expr,
                src: left_expr,
                dst: self.alloc_register()?,
                pc: self.const_writer.write_pc(0)?,
            };
            self.code_writer.write(&cmd);
            self.free_register(cmd.dst);

            let right_expr = self.visit_expr(&logic.right)?;
            self.const_writer
                .update_pc(cmd.pc, self.code_writer.len())?;

            assert_eq!(cmd.dst, right_expr);
            return Ok(cmd.dst);
        }
    }

    fn alloc_register(&mut self) -> Result<ScriptAddr> {
        if let Some(offset) = self.register_heap.pop() {
            return Ok(ScriptAddr::new(SEGMENT_REGISTER, offset.0));
        }
        if self.register_max >= MAX_REGISTERS as u16 {
            return Err(anyhow!("Register segment overflow"));
        }
        let addr = Ok(ScriptAddr::new(SEGMENT_REGISTER, self.register_max));
        self.register_max += 1;
        return addr;
    }

    fn free_register(&mut self, addr: ScriptAddr) {
        if addr.segment() == SEGMENT_REGISTER {
            self.register_heap.push(Reverse(addr.offset()));
        }
    }

    fn free_registers(&mut self, addrs: &[ScriptAddr]) {
        for addr in addrs {
            self.free_register(*addr);
        }
    }
}

#[derive(Debug, Default)]
struct CodeWriter(Vec<u16>);

impl CodeWriter {
    fn len(&self) -> usize {
        return self.0.len();
    }

    fn inner(&self) -> &[u16] {
        return &self.0;
    }

    fn clear(&mut self) {
        self.0.clear();
    }

    fn write<C: ScriptCmd>(&mut self, cmd: &C) -> usize {
        cmd.write(&mut self.0);
        return self.len() - 1;
    }

    fn update_addr(&mut self, pos: usize, addr: ScriptAddr) {
        self.0[pos] = unsafe { mem::transmute(addr) };
    }
}

#[derive(Debug, Default)]
struct ConstWriter(Vec<usize>);

impl ConstWriter {
    fn len(&self) -> usize {
        return self.0.len();
    }

    fn inner(&self) -> &[usize] {
        return &self.0;
    }

    fn clear(&mut self) {
        self.0.clear();
    }

    fn write_num(&mut self, num: Fx) -> Result<ScriptAddr> {
        if self.len() >= MAX_CONSTANTS {
            return Err(anyhow!("Constant segment overflow"));
        }
        self.0.push(unsafe { mem::transmute(num) });
        let offset = self.len() as u16 - 1;
        return Ok(ScriptAddr::new(SEGMENT_CONSTANT, offset));
    }

    fn write_id(&mut self, id: usize) -> Result<ScriptAddr> {
        if self.len() >= MAX_CONSTANTS {
            return Err(anyhow!("Constant segment overflow"));
        }
        self.0.push(id);
        let offset = self.len() as u16 - 1;
        return Ok(ScriptAddr::new(SEGMENT_CONSTANT, offset));
    }

    fn write_pc(&mut self, pc: usize) -> Result<ScriptAddr> {
        if self.len() >= MAX_CONSTANTS {
            return Err(anyhow!("Constant segment overflow"));
        }
        self.0.push(pc);
        let offset = self.len() as u16 - 1;
        return Ok(ScriptAddr::new(SEGMENT_CONSTANT, offset));
    }

    fn update_pc(&mut self, addr: ScriptAddr, pc: usize) -> Result<()> {
        if addr.segment() != SEGMENT_CONSTANT {
            return Err(anyhow!("Invalid constant segment"));
        }
        self.0[addr.offset() as usize] = pc;
        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use crate::script::command::ScriptCmdMethod;

    use super::super::parser::ScriptParser;
    use super::super::segment::ScriptCtx;
    use super::super::test::*;
    use super::*;
    use math::{ff, fi, RealExt};

    #[test]
    fn test_generator_assign() {
        let mut parser = ScriptParser::new();
        let mut generator = ScriptGenerator::new();

        let code = "test_out.xx = 2 * test_in.cc";
        let ast = parser.run::<CtxTest>(code).unwrap();
        let byte_code = generator.run::<CtxTest>(ast).unwrap();

        let mut const_writer = ConstWriter::default();
        const_writer.write_num(fi(2)).unwrap();
        assert_eq!(byte_code.const_segment(), const_writer.inner());

        let mut code_writer = CodeWriter::default();
        code_writer.write(&ScriptCmdFunc {
            opt: ScriptOpt::Mul,
            src: [
                ScriptAddr::new(SEGMENT_CONSTANT, 0),
                CtxTest::field("test_in.cc").addr,
            ],
            dst: CtxTest::field("test_out.xx").addr,
        });
        assert_eq!(byte_code.code_segment(), code_writer.inner());
    }

    #[test]
    fn test_generator_assign_add() {
        let mut parser = ScriptParser::new();
        let mut generator = ScriptGenerator::new();

        let code = "test_out.xx += 2 / 3 * PI";
        let ast = parser.run::<CtxTest>(code).unwrap();
        let byte_code = generator.run::<CtxTest>(ast).unwrap();

        let mut const_writer = ConstWriter::default();
        const_writer.write_num(fi(2)).unwrap();
        const_writer.write_num(fi(3)).unwrap();
        const_writer.write_num(Fx::pi()).unwrap();
        assert_eq!(byte_code.const_segment(), const_writer.inner());

        let mut code_writer = CodeWriter::default();
        code_writer.write(&ScriptCmdFunc {
            opt: ScriptOpt::Div,
            src: [
                ScriptAddr::new(SEGMENT_CONSTANT, 0),
                ScriptAddr::new(SEGMENT_CONSTANT, 1),
            ],
            dst: ScriptAddr::new(SEGMENT_REGISTER, 0),
        });
        code_writer.write(&ScriptCmdFunc {
            opt: ScriptOpt::Mul,
            src: [
                ScriptAddr::new(SEGMENT_REGISTER, 0),
                ScriptAddr::new(SEGMENT_CONSTANT, 2),
            ],
            dst: ScriptAddr::new(SEGMENT_REGISTER, 0),
        });
        code_writer.write(&ScriptCmdFunc {
            opt: ScriptOpt::Add,
            src: [
                ScriptAddr::new(SEGMENT_REGISTER, 0),
                CtxTest::field("test_out.xx").addr,
            ],
            dst: CtxTest::field("test_out.xx").addr,
        });
        assert_eq!(byte_code.code_segment(), code_writer.inner());
    }

    #[test]
    fn test_generator_group() {
        let mut parser = ScriptParser::new();
        let mut generator = ScriptGenerator::new();

        let code = "test_out.zz = 3 * (2 + 5.55)";
        let ast = parser.run::<CtxTest>(code).unwrap();
        let byte_code = generator.run::<CtxTest>(ast).unwrap();

        let mut const_writer = ConstWriter::default();
        const_writer.write_num(fi(3)).unwrap();
        const_writer.write_num(fi(2)).unwrap();
        const_writer.write_num(ff(5.55)).unwrap();
        assert_eq!(byte_code.const_segment(), const_writer.inner());

        let mut code_writer = CodeWriter::default();
        code_writer.write(&ScriptCmdFunc {
            opt: ScriptOpt::Add,
            src: [
                ScriptAddr::new(SEGMENT_CONSTANT, 1),
                ScriptAddr::new(SEGMENT_CONSTANT, 2),
            ],
            dst: ScriptAddr::new(SEGMENT_REGISTER, 0),
        });
        code_writer.write(&ScriptCmdFunc {
            opt: ScriptOpt::Mul,
            src: [
                ScriptAddr::new(SEGMENT_CONSTANT, 0),
                ScriptAddr::new(SEGMENT_REGISTER, 0),
            ],
            dst: CtxTest::field("test_out.zz").addr,
        });
        assert_eq!(byte_code.code_segment(), code_writer.inner());
    }

    #[test]
    fn test_generator_if_stat() {
        let mut parser = ScriptParser::new();
        let mut generator = ScriptGenerator::new();

        let code = "
            if test_in.dd > 0 {
                test_out.xx = 3
            } elsif 7 {
                test_out.yy = 6
            } else {
                test_out.zz = 9
            }";
        let ast = parser.run::<CtxTest>(code).unwrap();
        let byte_code = generator.run::<CtxTest>(ast).unwrap();

        let mut const_writer = ConstWriter::default();
        const_writer.write_num(fi(0)).unwrap();
        const_writer.write_pc(12).unwrap();
        const_writer.write_num(fi(3)).unwrap();
        const_writer.write_num(fi(7)).unwrap();
        const_writer.write_pc(20).unwrap();
        const_writer.write_num(fi(6)).unwrap();
        const_writer.write_num(fi(9)).unwrap();
        const_writer.write_pc(23).unwrap();
        assert_eq!(byte_code.const_segment(), const_writer.inner());

        let mut code_writer = CodeWriter::default();
        code_writer.write(&ScriptCmdFunc {
            opt: ScriptOpt::Gt,
            src: [
                CtxTest::field("test_in.dd").addr,
                ScriptAddr::new(SEGMENT_CONSTANT, 0),
            ],
            dst: ScriptAddr::new(SEGMENT_REGISTER, 0),
        });
        code_writer.write(&ScriptCmdJmpCmp {
            opt: ScriptOpt::JmpCmp,
            cond: ScriptAddr::new(SEGMENT_REGISTER, 0),
            pc: ScriptAddr::new(SEGMENT_CONSTANT, 1),
        });
        code_writer.write(&ScriptCmdFunc {
            opt: ScriptOpt::Mov,
            src: [ScriptAddr::new(SEGMENT_CONSTANT, 2)],
            dst: CtxTest::field("test_out.xx").addr,
        });
        code_writer.write(&ScriptCmdJmp {
            opt: ScriptOpt::Jmp,
            pc: ScriptAddr::new(SEGMENT_CONSTANT, 7),
        });
        code_writer.write(&ScriptCmdJmpCmp {
            opt: ScriptOpt::JmpCmp,
            cond: ScriptAddr::new(SEGMENT_CONSTANT, 3),
            pc: ScriptAddr::new(SEGMENT_CONSTANT, 4),
        });
        code_writer.write(&ScriptCmdFunc {
            opt: ScriptOpt::Mov,
            src: [ScriptAddr::new(SEGMENT_CONSTANT, 5)],
            dst: CtxTest::field("test_out.yy").addr,
        });
        code_writer.write(&ScriptCmdJmp {
            opt: ScriptOpt::Jmp,
            pc: ScriptAddr::new(SEGMENT_CONSTANT, 7),
        });
        code_writer.write(&ScriptCmdFunc {
            opt: ScriptOpt::Mov,
            src: [ScriptAddr::new(SEGMENT_CONSTANT, 6)],
            dst: CtxTest::field("test_out.zz").addr,
        });
        assert_eq!(byte_code.code_segment(), code_writer.inner());
    }

    #[test]
    fn test_generator_method_stat() {
        let mut parser = ScriptParser::new();
        let mut generator = ScriptGenerator::new();

        let code = "test_out.xx = test_out.has_id($id)";
        let ast = parser.run::<CtxTest>(code).unwrap();
        let byte_code = generator.run::<CtxTest>(ast).unwrap();

        let mut const_writer = ConstWriter::default();
        const_writer.write_id(0).unwrap();
        assert_eq!(byte_code.const_segment(), const_writer.inner());

        let mut code_writer = CodeWriter::default();
        code_writer.write(&ScriptCmdMethod {
            opt: ScriptOpt::TestHasID,
            var_id: VarTestOut::var_id(),
            var_seg: CtxTest::var(VarTestOut::var_id()).segment,
            src: [ScriptAddr::new(SEGMENT_CONSTANT, 0)],
            dst: ScriptAddr::new(SEGMENT_REGISTER, 0),
        });
        code_writer.write(&ScriptCmdFunc {
            opt: ScriptOpt::Mov,
            src: [ScriptAddr::new(SEGMENT_REGISTER, 0)],
            dst: CtxTest::field("test_out.xx").addr,
        });
        assert_eq!(byte_code.code_segment(), code_writer.inner());
    }

    #[test]
    fn test_generator_method_expr() {
        let mut parser = ScriptParser::new();
        let mut generator = ScriptGenerator::new();

        let code = "test_out.add_id($id)";
        let ast = parser.run::<CtxTest>(code).unwrap();
        let byte_code = generator.run::<CtxTest>(ast).unwrap();

        let mut const_writer = ConstWriter::default();
        const_writer.write_id(0).unwrap();
        assert_eq!(byte_code.const_segment(), const_writer.inner());

        let mut code_writer = CodeWriter::default();
        code_writer.write(&ScriptCmdMethod {
            opt: ScriptOpt::TestAddID,
            var_id: VarTestOut::var_id(),
            var_seg: CtxTest::var(VarTestOut::var_id()).segment,
            src: [ScriptAddr::new(SEGMENT_CONSTANT, 0)],
            dst: ScriptAddr::default(),
        });
        assert_eq!(byte_code.code_segment(), code_writer.inner());
    }
}
