use super::command::{ScriptAddr, ScriptOpt};
use math::Fx;

#[derive(Debug, Clone, PartialEq)]
pub struct AstBlock {
    pub stats: Vec<AstStat>,
}

impl AstBlock {
    pub fn new(stats: Vec<AstStat>) -> AstBlock {
        return AstBlock { stats };
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AstStat {
    Assign(AstStatAssign),
    Method(AstStatMethod),
    Branch(AstStatBranch),
}

impl AstStat {
    pub fn new_assign(opt: Option<ScriptOpt>, var: ScriptAddr, expr: AstExpr) -> AstStat {
        return AstStat::Assign(AstStatAssign::new(opt, var, expr));
    }
    pub fn new_method(opt: ScriptOpt, var_id: u8, var_seg: u8, args: Vec<AstExpr>) -> AstStat {
        return AstStat::Method(AstStatMethod::new(opt, var_id, var_seg, args));
    }

    pub fn new_branch(
        cond: Option<AstExpr>,
        stats: Vec<AstStat>,
        next: Option<AstStatBranch>,
    ) -> AstStat {
        return AstStat::Branch(AstStatBranch::new(cond, stats, next));
    }

    pub fn is_assign(&self) -> bool {
        return match self {
            &AstStat::Assign(_) => true,
            _ => false,
        };
    }

    pub fn is_call_ext(&self) -> bool {
        return match self {
            &AstStat::Method(_) => true,
            _ => false,
        };
    }

    pub fn is_branch(&self) -> bool {
        return match self {
            &AstStat::Branch(_) => true,
            _ => false,
        };
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AstStatAssign {
    pub opt: Option<ScriptOpt>,
    pub var: ScriptAddr,
    pub expr: Box<AstExpr>,
}

impl AstStatAssign {
    pub fn new(opt: Option<ScriptOpt>, var: ScriptAddr, expr: AstExpr) -> AstStatAssign {
        return AstStatAssign {
            opt,
            var,
            expr: Box::new(expr),
        };
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AstStatMethod {
    pub opt: ScriptOpt,
    pub var_id: u8,
    pub var_seg: u8,
    pub args: Vec<AstExpr>,
}

impl AstStatMethod {
    pub fn new(opt: ScriptOpt, var_id: u8, var_seg: u8, args: Vec<AstExpr>) -> AstStatMethod {
        return AstStatMethod {
            opt,
            var_id,
            var_seg,
            args,
        };
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AstStatBranch {
    pub cond: Option<Box<AstExpr>>,
    pub stats: Vec<AstStat>,
    pub next: Option<Box<AstStatBranch>>,
}

impl AstStatBranch {
    pub fn new(
        cond: Option<AstExpr>,
        stats: Vec<AstStat>,
        next: Option<AstStatBranch>,
    ) -> AstStatBranch {
        return AstStatBranch {
            cond: cond.map(|c| Box::new(c)),
            stats,
            next: next.map(|n| Box::new(n)),
        };
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AstExpr {
    Num(Fx),
    ID(usize),
    Var(ScriptAddr),
    Func(AstExprFunc),
    Method(AstExprMethod),
    Branch(AstExprBranch),
    Logic(AstExprLogic),
}

impl AstExpr {
    pub fn new_num(num: Fx) -> AstExpr {
        return AstExpr::Num(num);
    }

    pub fn new_id(num: usize) -> AstExpr {
        return AstExpr::ID(num);
    }

    pub fn new_var(addr: ScriptAddr) -> AstExpr {
        return AstExpr::Var(addr);
    }

    pub fn new_call(opt: ScriptOpt, args: Vec<AstExpr>) -> AstExpr {
        return AstExpr::Func(AstExprFunc::new(opt, args));
    }

    pub fn new_method(opt: ScriptOpt, var_id: u8, var_seg: u8, args: Vec<AstExpr>) -> AstExpr {
        return AstExpr::Method(AstExprMethod::new(opt, var_id, var_seg, args));
    }

    pub fn new_branch(cond: AstExpr, left: AstExpr, right: Option<AstExpr>) -> AstExpr {
        return AstExpr::Branch(AstExprBranch::new(cond, left, right));
    }

    pub fn new_logic(typ: AstLogicType, left: AstExpr, right: AstExpr) -> AstExpr {
        return AstExpr::Logic(AstExprLogic::new(typ, left, right));
    }

    pub fn is_fx(&self) -> bool {
        return match self {
            &AstExpr::Num(_) => true,
            _ => false,
        };
    }

    pub fn is_id(&self) -> bool {
        return match self {
            &AstExpr::ID(_) => true,
            _ => false,
        };
    }

    pub fn is_var(&self) -> bool {
        return match self {
            &AstExpr::Var(_) => true,
            _ => false,
        };
    }

    pub fn is_normal(&self) -> bool {
        return match self {
            &AstExpr::Func(_) => true,
            _ => false,
        };
    }

    pub fn is_branch(&self) -> bool {
        return match self {
            &AstExpr::Branch(_) => true,
            _ => false,
        };
    }

    pub fn is_logic(&self) -> bool {
        return match self {
            &AstExpr::Logic(_) => true,
            _ => false,
        };
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AstExprFunc {
    pub opt: ScriptOpt,
    pub args: Vec<AstExpr>,
}

impl AstExprFunc {
    pub fn new(opt: ScriptOpt, args: Vec<AstExpr>) -> AstExprFunc {
        return AstExprFunc { opt, args };
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AstExprMethod {
    pub opt: ScriptOpt,
    pub var_id: u8,
    pub var_seg: u8,
    pub args: Vec<AstExpr>,
}

impl AstExprMethod {
    pub fn new(opt: ScriptOpt, var_id: u8, var_seg: u8, args: Vec<AstExpr>) -> AstExprMethod {
        return AstExprMethod {
            opt,
            var_id,
            var_seg,
            args,
        };
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AstExprBranch {
    pub cond: Box<AstExpr>,
    pub left: Box<AstExpr>,
    pub right: Box<Option<AstExpr>>,
}

impl AstExprBranch {
    pub fn new(cond: AstExpr, left: AstExpr, right: Option<AstExpr>) -> AstExprBranch {
        return AstExprBranch {
            cond: Box::new(cond),
            left: Box::new(left),
            right: Box::new(right),
        };
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AstLogicType {
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AstExprLogic {
    pub typ: AstLogicType,
    pub left: Box<AstExpr>,
    pub right: Box<AstExpr>,
}

impl AstExprLogic {
    pub fn new(typ: AstLogicType, left: AstExpr, right: AstExpr) -> AstExprLogic {
        return AstExprLogic {
            typ,
            left: Box::new(left),
            right: Box::new(right),
        };
    }
}
