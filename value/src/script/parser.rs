use super::ast::{AstBlock, AstExpr, AstLogicType, AstStat, AstStatBranch};
use super::command::{ScriptAddr, ScriptOpt, ScriptType};
use super::segment::{
    ScriptCtx, ScriptCtxFields, ScriptCtxVar, ScriptCtxVars, ScriptVar, EMPTY_CTX_FIELDS,
    EMPTY_CTX_VARS,
};
use super::test::VarTestOut;
use anyhow::{self, Result};
use lazy_static::lazy_static;
use math::{ff, fi, Fx, RealExt};
use pest::error::{Error, ErrorVariant};
use pest::iterators::{Pair, Pairs};
use pest::prec_climber::{Assoc, Operator, PrecClimber};
use pest::Parser as ParserTrait;
use std::collections::HashMap;
use std::lazy::SyncLazy;

static CONSTS_MAP: SyncLazy<HashMap<&'static str, Fx>> = SyncLazy::new(|| {
    let mut map = HashMap::new();
    map.insert("PI", Fx::pi());
    map.insert("E", Fx::e());
    map.insert("TAU", Fx::tau());
    map.insert("MAX", Fx::max_value());
    map.insert("MIN", Fx::min_value());
    return map;
});

#[derive(Debug, Clone)]
struct FuncItem {
    opt: ScriptOpt,
    var_id: Option<u8>,
    args: Vec<ScriptType>,
    ret: Option<ScriptType>,
}

impl FuncItem {
    fn func(opt: ScriptOpt, args: Vec<ScriptType>) -> FuncItem {
        return FuncItem {
            opt,
            var_id: None,
            args,
            ret: Some(ScriptType::Num),
        };
    }

    fn ext_stat<V: ScriptVar>(opt: ScriptOpt, args: Vec<ScriptType>) -> FuncItem {
        return FuncItem {
            opt,
            var_id: Some(V::var_id()),
            args,
            ret: None,
        };
    }

    fn ext_expr<V: ScriptVar>(opt: ScriptOpt, args: Vec<ScriptType>) -> FuncItem {
        return FuncItem {
            opt,
            var_id: Some(V::var_id()),
            args,
            ret: Some(ScriptType::Num),
        };
    }
}

static FUNCS_MAP: SyncLazy<HashMap<&'static str, FuncItem>> = SyncLazy::new(|| {
    use ScriptOpt::*;
    use ScriptType::*;

    let mut map = HashMap::new();

    map.insert("abs", FuncItem::func(Abs, vec![Num]));
    map.insert("min", FuncItem::func(Min, vec![Num, Num]));
    map.insert("max", FuncItem::func(Max, vec![Num, Num]));
    map.insert("floor", FuncItem::func(Floor, vec![Num]));
    map.insert("ceil", FuncItem::func(Ceil, vec![Num]));
    map.insert("round", FuncItem::func(Round, vec![Num]));
    map.insert("clamp", FuncItem::func(Clamp, vec![Num, Num, Num]));
    map.insert("saturate", FuncItem::func(Saturate, vec![Num]));
    map.insert("lerp", FuncItem::func(Lerp, vec![Num, Num, Num]));
    map.insert("sqrt", FuncItem::func(Sqrt, vec![Num]));
    map.insert("exp", FuncItem::func(Exp, vec![Num]));
    map.insert("degrees", FuncItem::func(Degrees, vec![Num]));
    map.insert("radians", FuncItem::func(Radians, vec![Num]));
    map.insert("sin", FuncItem::func(Sin, vec![Num]));
    map.insert("cos", FuncItem::func(Cos, vec![Num]));
    map.insert("tan", FuncItem::func(Tan, vec![Num]));

    map.insert(
        "test_out.add_id",
        FuncItem::ext_stat::<VarTestOut>(TestAddID, vec![ID]),
    );
    map.insert(
        "test_out.has_id",
        FuncItem::ext_expr::<VarTestOut>(TestHasID, vec![ID]),
    );

    return map;
});

lazy_static! {
    static ref CLIMBER: PrecClimber<Rule> = PrecClimber::new(vec![
        Operator::new(Rule::Or, Assoc::Left),
        Operator::new(Rule::And, Assoc::Left),
        Operator::new(Rule::Eq, Assoc::Left) | Operator::new(Rule::Ne, Assoc::Left),
        Operator::new(Rule::Le, Assoc::Left)
            | Operator::new(Rule::Lt, Assoc::Left)
            | Operator::new(Rule::Ge, Assoc::Left)
            | Operator::new(Rule::Gt, Assoc::Left),
        Operator::new(Rule::Add, Assoc::Left) | Operator::new(Rule::Sub, Assoc::Left),
        Operator::new(Rule::Mul, Assoc::Left)
            | Operator::new(Rule::Div, Assoc::Left)
            | Operator::new(Rule::Rem, Assoc::Left),
    ]);
}

#[derive(Parser)]
#[grammar = "./script/script.pest"]
pub struct PestParser;

pub struct ScriptParser {
    ctx_fields: &'static ScriptCtxFields,
    ctx_vars: &'static ScriptCtxVars,
}

impl ScriptParser {
    pub fn new() -> ScriptParser {
        return ScriptParser {
            ctx_fields: &EMPTY_CTX_FIELDS,
            ctx_vars: &EMPTY_CTX_VARS,
        };
    }

    pub fn run<C: ScriptCtx>(&mut self, code: &str) -> Result<AstBlock> {
        self.ctx_fields = C::fields();
        self.ctx_vars = C::vars();
        let result = self.run_impl(code);
        self.ctx_fields = &EMPTY_CTX_FIELDS;
        self.ctx_vars = &EMPTY_CTX_VARS;
        return result;
    }

    fn run_impl(&mut self, code: &str) -> Result<AstBlock> {
        let mut pairs = PestParser::parse(Rule::Script, code).unwrap();
        let script_pair = pairs.next().expect("Unexpected error");
        let script_pairs = script_pair.clone().into_inner();
        assert_eq!(pairs.next(), None);

        let mut block = AstBlock { stats: Vec::new() };
        for pair in script_pairs {
            if pair.as_rule() != Rule::EOI {
                block.stats.push(self.parse_stat(pair)?);
            }
        }

        return Ok(block);
    }

    fn parse_stat(&self, pair: Pair<Rule>) -> Result<AstStat> {
        return match pair.as_rule() {
            Rule::Assign => self.parse_assign(pair),
            Rule::CallStat => self.parse_call_stat(pair),
            Rule::IfStat => self.parse_if_stat(pair),
            _ => Err(Self::error(&pair)),
        };
    }

    fn parse_assign(&self, pair: Pair<Rule>) -> Result<AstStat> {
        let mut pairs = pair.clone().into_inner();

        let var_pair = Self::assert_next_pair(&pair, &mut pairs, Rule::Ident)?;
        let assign_pair = Self::next_pair(&pair, &mut pairs)?;
        let expr_pair = Self::assert_next_pair(&pair, &mut pairs, Rule::Expr)?;
        Self::assert_end(&pair, pairs)?;

        let opt = match assign_pair.as_rule() {
            Rule::RawAssign => None,
            Rule::AddAssign => Some(ScriptOpt::Add),
            Rule::SubAssign => Some(ScriptOpt::Sub),
            _ => return Err(Self::error(&assign_pair)),
        };

        let var = self.parse_left_ident(var_pair)?;
        let expr = self.parse_expr(expr_pair, ScriptType::Num)?;

        return Ok(AstStat::new_assign(opt, var, expr));
    }

    fn parse_left_ident(&self, pair: Pair<Rule>) -> Result<ScriptAddr> {
        let var = self
            .ctx_fields
            .get(pair.as_str())
            .ok_or(Self::error(&pair))?;
        if !var.writable {
            return Err(Self::error(&pair));
        }
        return Ok(var.addr);
    }

    fn parse_call_stat(&self, pair: Pair<Rule>) -> Result<AstStat> {
        let mut pairs = pair.clone().into_inner();

        let ident_pair = Self::assert_next_pair(&pair, &mut pairs, Rule::Ident)?;
        let func = FUNCS_MAP
            .get(ident_pair.as_str())
            .ok_or(Self::error(&pair))?;

        // check is extern function
        let var_id = match func.var_id {
            Some(var_id) => var_id,
            None => return Err(Self::error(&pair)),
        };

        // check var in ctx
        let var_seg = match self.ctx_vars.get(&var_id) {
            Some(var) => var.segment,
            None => return Err(Self::error(&pair)),
        };

        // check is extend statement
        if func.ret != None {
            return Err(Self::error(&pair));
        }

        // parse arguments
        let mut args = Vec::new();
        for typ in &func.args {
            let arg_pair = Self::assert_next_pair(&pair, &mut pairs, Rule::Expr)?;
            args.push(self.parse_expr(arg_pair, *typ)?);
        }
        if args.len() != func.args.len() {
            return Err(Self::error(&pair));
        }

        Self::assert_end(&pair, pairs)?;
        return Ok(AstStat::new_method(func.opt, var_id, var_seg, args));
    }

    fn parse_if_stat(&self, pair: Pair<Rule>) -> Result<AstStat> {
        return self.parse_if_stat_impl(pair).map(AstStat::Branch);
    }

    fn parse_if_stat_impl(&self, pair: Pair<Rule>) -> Result<AstStatBranch> {
        let mut pairs = pair.clone().into_inner();

        let mut cond = None;
        if pair.as_rule() != Rule::ElseStat {
            let cond_pair = Self::assert_next_pair(&pair, &mut pairs, Rule::Expr)?;
            cond = Some(Box::new(self.parse_expr(cond_pair, ScriptType::Num)?));
        }

        let mut stats = Vec::new();
        let mut next = None;
        while let Some(iter_pair) = pairs.next() {
            match iter_pair.as_rule() {
                Rule::Assign | Rule::IfStat => stats.push(self.parse_stat(iter_pair)?),
                Rule::ElsifStat | Rule::ElseStat => {
                    next = Some(Box::new(self.parse_if_stat_impl(iter_pair)?));
                    break;
                }
                _ => return Err(Self::error(&pair)),
            };
        }

        return Ok(AstStatBranch { cond, stats, next });
    }

    fn parse_expr(&self, pair: Pair<Rule>, typ: ScriptType) -> Result<AstExpr> {
        return CLIMBER.climb(
            pair.into_inner(),
            |pair: Pair<Rule>| {
                return match pair.as_rule() {
                    Rule::Expr => self.parse_expr(pair, typ),
                    Rule::Group => self.parse_expr(pair, typ),
                    Rule::Unary => self.parse_unary(pair, typ),
                    Rule::Ident => self.parse_right_ident(pair, typ),
                    Rule::CallExpr => self.parse_call_expr(pair, typ),
                    Rule::Number => self.parse_number(pair, typ),
                    Rule::ID => self.parse_id(pair, typ),
                    _ => Err(Self::error(&pair)),
                };
            },
            |lhs: Result<AstExpr>, opt: Pair<Rule>, rhs: Result<AstExpr>| {
                let lhs = lhs?;
                let rhs = rhs?;
                if lhs.is_id() || rhs.is_id() {
                    return Err(Self::error(&opt));
                }
                match opt.as_rule() {
                    Rule::And => return Ok(AstExpr::new_logic(AstLogicType::And, lhs, rhs)),
                    Rule::Or => return Ok(AstExpr::new_logic(AstLogicType::Or, lhs, rhs)),
                    _ => {}
                };
                let opt = match opt.as_rule() {
                    Rule::Mul => ScriptOpt::Mul,
                    Rule::Div => ScriptOpt::Div,
                    Rule::Rem => ScriptOpt::Rem,
                    Rule::Add => ScriptOpt::Add,
                    Rule::Sub => ScriptOpt::Sub,
                    Rule::Lt => ScriptOpt::Lt,
                    Rule::Le => ScriptOpt::Le,
                    Rule::Gt => ScriptOpt::Gt,
                    Rule::Ge => ScriptOpt::Ge,
                    Rule::Eq => ScriptOpt::Eq,
                    Rule::Ne => ScriptOpt::Ne,
                    _ => return Err(Self::error(&opt)),
                };
                return Ok(AstExpr::new_call(opt, vec![lhs, rhs]));
            },
        );
    }

    fn parse_unary(&self, pair: Pair<Rule>, typ: ScriptType) -> Result<AstExpr> {
        if typ != ScriptType::Num {
            return Err(Self::error(&pair));
        }

        let mut pairs = pair.clone().into_inner();

        let unary_pair = Self::next_pair(&pair, &mut pairs)?;
        let expr_pair = Self::assert_next_pair(&pair, &mut pairs, Rule::Expr)?;
        Self::assert_end(&pair, pairs)?;

        if unary_pair.as_rule() == Rule::Pos {
            return self.parse_expr(expr_pair, ScriptType::Num);
        }

        let unary = match unary_pair.as_rule() {
            Rule::Neg => ScriptOpt::Neg,
            Rule::Not => ScriptOpt::Not,
            _ => return Err(Self::error(&unary_pair)),
        };
        let expr = self.parse_expr(pair, ScriptType::Num)?;
        return Ok(AstExpr::new_call(unary, vec![expr]));
    }

    fn parse_call_expr(&self, pair: Pair<Rule>, typ: ScriptType) -> Result<AstExpr> {
        let mut pairs = pair.clone().into_inner();

        let ident_pair = Self::assert_next_pair(&pair, &mut pairs, Rule::Ident)?;
        let func = FUNCS_MAP
            .get(ident_pair.as_str())
            .ok_or(Self::error(&pair))?;

        match func.ret {
            // is extend statement
            None => return Err(Self::error(&pair)),
            // is type match
            Some(ret_typ) => {
                if ret_typ != typ {
                    return Err(Self::error(&pair));
                }
            }
        };

        // parse arguments
        let mut args = Vec::new();
        for typ in &func.args {
            let arg_pair = Self::assert_next_pair(&pair, &mut pairs, Rule::Expr)?;
            args.push(self.parse_expr(arg_pair, *typ)?);
        }
        if args.len() != func.args.len() {
            return Err(Self::error(&pair));
        }

        Self::assert_end(&pair, pairs)?;

        match func.var_id {
            // inner function
            None => {
                return Ok(AstExpr::new_call(func.opt, args));
            }
            // extern function
            Some(var_id) => {
                // check var in ctx
                let var_seg = match self.ctx_vars.get(&var_id) {
                    Some(var) => var.segment,
                    None => return Err(Self::error(&pair)),
                };
                return Ok(AstExpr::new_method(func.opt, var_id, var_seg, args));
            }
        };
    }

    fn parse_number(&self, pair: Pair<Rule>, typ: ScriptType) -> Result<AstExpr> {
        if typ != ScriptType::Num {
            return Err(Self::error(&pair));
        }

        let mut pairs = pair.clone().into_inner();

        let num_pair = Self::next_pair(&pair, &mut pairs)?;
        let num_str = num_pair.as_str();
        let num = match num_pair.as_rule() {
            Rule::Float => ff(num_str.parse::<f64>().map_err(|_| Self::error(&pair))?),
            Rule::Hex => {
                fi(i64::from_str_radix(&num_str[2..], 16).map_err(|_| Self::error(&pair))?)
            }
            _ => return Err(Self::error(&pair)),
        };

        Self::assert_end(&pair, pairs)?;
        return Ok(AstExpr::new_num(num));
    }

    fn parse_id(&self, pair: Pair<Rule>, typ: ScriptType) -> Result<AstExpr> {
        if typ != ScriptType::ID {
            return Err(Self::error(&pair));
        }

        return Ok(AstExpr::new_id(0));
    }

    fn parse_right_ident(&self, pair: Pair<Rule>, typ: ScriptType) -> Result<AstExpr> {
        if typ != ScriptType::Num {
            return Err(Self::error(&pair));
        }

        if let Some(num) = CONSTS_MAP.get(pair.as_str()) {
            return Ok(AstExpr::new_num(*num));
        }

        if let Some(var) = self.ctx_fields.get(pair.as_str()) {
            if var.writable {
                return Err(Self::error(&pair));
            }
            return Ok(AstExpr::new_var(var.addr));
        }

        return Err(Self::error(&pair));
    }

    //
    // utils
    //

    fn error(pair: &Pair<Rule>) -> anyhow::Error {
        let rule_err = Error::<Rule>::new_from_span(
            ErrorVariant::CustomError {
                message: String::new(),
            },
            pair.as_span(),
        );
        return anyhow::Error::from(rule_err);
    }

    fn next_pair<'t>(pair: &Pair<Rule>, pairs: &mut Pairs<'t, Rule>) -> Result<Pair<'t, Rule>> {
        let pair = pairs.next().ok_or(Self::error(pair))?;
        return Ok(pair);
    }

    fn assert_next_pair<'t>(
        pair: &Pair<Rule>,
        pairs: &mut Pairs<'t, Rule>,
        rule: Rule,
    ) -> Result<Pair<'t, Rule>> {
        let pair = pairs.next().ok_or(Self::error(pair))?;
        return match pair.as_rule() == rule {
            true => Ok(pair),
            false => Err(Self::error(&pair)),
        };
    }

    fn assert_end(pair: &Pair<Rule>, mut pairs: Pairs<Rule>) -> Result<()> {
        return match pairs.next() {
            None => Ok(()),
            Some(_) => Err(Self::error(pair)),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::super::test::*;
    use super::super::ScriptCtx;
    use super::*;
    use math::{Fx, RealExt};

    #[test]
    fn test_parser_assign() {
        let mut parser = ScriptParser::new();
        let code = "test_out.xx = test_in.aa";
        let ast = parser.run::<CtxTest>(code).unwrap();
        assert_eq!(
            ast,
            AstBlock::new(vec![AstStat::new_assign(
                None,
                CtxTest::field("test_out.xx").addr,
                AstExpr::new_var(CtxTest::field("test_in.aa").addr),
            )]),
        );
    }

    #[test]
    fn test_parser_assign_add() {
        let mut parser = ScriptParser::new();
        let code = "test_out.xx += test_in.aa";
        let ast = parser.run::<CtxTest>(code).unwrap();
        assert_eq!(
            ast,
            AstBlock::new(vec![AstStat::new_assign(
                Some(ScriptOpt::Add),
                CtxTest::field("test_out.xx").addr,
                AstExpr::new_var(CtxTest::field("test_in.aa").addr),
            )]),
        );
    }

    #[test]
    fn test_parser_mul() {
        let mut parser = ScriptParser::new();
        let code = "test_out.xx = PI * test_in.cc";
        let ast = parser.run::<CtxTest>(code).unwrap();
        assert_eq!(
            ast,
            AstBlock::new(vec![AstStat::new_assign(
                None,
                CtxTest::field("test_out.xx").addr,
                AstExpr::new_call(
                    ScriptOpt::Mul,
                    vec![
                        AstExpr::new_num(Fx::pi()),
                        AstExpr::new_var(CtxTest::field("test_in.cc").addr),
                    ]
                ),
            )]),
        );
    }

    #[test]
    fn test_parser_func() {
        let mut parser = ScriptParser::new();
        let code = "test_out.yy = clamp(test_in.dd, 0.5, 1.5)";
        let ast = parser.run::<CtxTest>(code).unwrap();
        assert_eq!(
            ast,
            AstBlock::new(vec![AstStat::new_assign(
                None,
                CtxTest::field("test_out.yy").addr,
                AstExpr::new_call(
                    ScriptOpt::Clamp,
                    vec![
                        AstExpr::new_var(CtxTest::field("test_in.dd").addr),
                        AstExpr::new_num(ff(0.5)),
                        AstExpr::new_num(ff(1.5)),
                    ]
                ),
            )]),
        );
    }

    #[test]
    fn test_parser_group() {
        let mut parser = ScriptParser::new();
        let code = "test_out.xx = PI * (1 + 3.5)";
        let ast = parser.run::<CtxTest>(code).unwrap();
        assert_eq!(
            ast,
            AstBlock::new(vec![AstStat::new_assign(
                None,
                CtxTest::field("test_out.xx").addr,
                AstExpr::new_call(
                    ScriptOpt::Mul,
                    vec![
                        AstExpr::new_num(Fx::pi()),
                        AstExpr::new_call(
                            ScriptOpt::Add,
                            vec![AstExpr::new_num(fi(1)), AstExpr::new_num(ff(3.5)),]
                        ),
                    ]
                ),
            )]),
        );
    }

    #[test]
    fn test_parser_and() {
        let mut parser = ScriptParser::new();
        let code = "test_out.xx = PI && 10";
        let ast = parser.run::<CtxTest>(code).unwrap();
        assert_eq!(
            ast,
            AstBlock::new(vec![AstStat::new_assign(
                None,
                CtxTest::field("test_out.xx").addr,
                AstExpr::new_logic(
                    AstLogicType::And,
                    AstExpr::new_num(Fx::pi()),
                    AstExpr::new_num(fi(10)),
                ),
            )]),
        );
    }

    #[test]
    fn test_parser_if_stat() {
        let mut parser = ScriptParser::new();
        let code = "if test_in.aa < 2 { test_out.xx += 5.5 }";
        let ast = parser.run::<CtxTest>(code).unwrap();
        assert_eq!(
            ast,
            AstBlock::new(vec![AstStat::new_branch(
                Some(AstExpr::new_call(
                    ScriptOpt::Lt,
                    vec![
                        AstExpr::new_var(CtxTest::field("test_in.aa").addr),
                        AstExpr::new_num(fi(2)),
                    ]
                )),
                vec![AstStat::new_assign(
                    Some(ScriptOpt::Add),
                    CtxTest::field("test_out.xx").addr,
                    AstExpr::Num(ff(5.5)),
                )],
                None,
            )]),
        );
    }

    #[test]
    fn test_parser_elsif_stat() {
        let mut parser = ScriptParser::new();
        let code = "
            if 2 {
                test_out.xx += 3
            } elsif 0 {
                test_out.zz -= 6
            }";
        let ast = parser.run::<CtxTest>(code).unwrap();
        assert_eq!(
            ast,
            AstBlock::new(vec![AstStat::new_branch(
                Some(AstExpr::new_num(fi(2))),
                vec![AstStat::new_assign(
                    Some(ScriptOpt::Add),
                    CtxTest::field("test_out.xx").addr,
                    AstExpr::Num(fi(3)),
                )],
                Some(AstStatBranch::new(
                    Some(AstExpr::new_num(fi(0))),
                    vec![AstStat::new_assign(
                        Some(ScriptOpt::Sub),
                        CtxTest::field("test_out.zz").addr,
                        AstExpr::Num(fi(6)),
                    )],
                    None,
                )),
            )]),
        );
    }

    #[test]
    fn test_parser_else_stat() {
        let mut parser = ScriptParser::new();
        let code = "
            if 2 {
                test_out.xx += 3
            } else {
                test_out.zz -= 6
            }";
        let ast = parser.run::<CtxTest>(code).unwrap();
        assert_eq!(
            ast,
            AstBlock::new(vec![AstStat::new_branch(
                Some(AstExpr::new_num(fi(2))),
                vec![AstStat::new_assign(
                    Some(ScriptOpt::Add),
                    CtxTest::field("test_out.xx").addr,
                    AstExpr::Num(fi(3)),
                )],
                Some(AstStatBranch::new(
                    None,
                    vec![AstStat::new_assign(
                        Some(ScriptOpt::Sub),
                        CtxTest::field("test_out.zz").addr,
                        AstExpr::Num(fi(6)),
                    )],
                    None,
                )),
            )]),
        );
    }

    #[test]
    fn test_parser_method_expr() {
        let mut parser = ScriptParser::new();
        let code = "test_out.yy = test_out.has_id($fake.id)";
        let ast = parser.run::<CtxTest>(code).unwrap();
        assert_eq!(
            ast,
            AstBlock::new(vec![AstStat::new_assign(
                None,
                CtxTest::field("test_out.yy").addr,
                AstExpr::new_method(
                    ScriptOpt::TestHasID,
                    VarTestOut::var_id(),
                    3,
                    vec![AstExpr::new_id(0)]
                ),
            )]),
        );
    }

    #[test]
    fn test_parser_method_stat() {
        let mut parser = ScriptParser::new();
        let code = "test_out.add_id($fake.id)";
        let ast = parser.run::<CtxTest>(code).unwrap();
        assert_eq!(
            ast,
            AstBlock::new(vec![AstStat::new_method(
                ScriptOpt::TestAddID,
                VarTestOut::var_id(),
                3,
                vec![AstExpr::new_id(0)]
            )]),
        );
    }
}
