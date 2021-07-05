mod ast;
mod byte_code;
mod command;
mod executor;
mod generator;
mod optimizer;
mod parser;
mod segment;
#[cfg(test)]
mod test_helper;

pub use byte_code::ScriptByteCode;
pub use command::{ScriptAddr, ScriptOpt, ScriptType, ScriptVal};
pub use executor::{ScriptError, ScriptExecutor, ScriptExecutorAgent};
pub use generator::ScriptGenerator;
pub use optimizer::ScriptOptimizer;
pub use parser::ScriptParser;
pub use segment::{
    ScriptCtx, ScriptCtxItem, ScriptCtxItems, ScriptExt, ScriptExtItem, ScriptExtItems, ScriptVar,
    ScriptVarItem, ScriptVarItems, SEGMENT_CONSTANT, SEGMENT_REGISTER, SEGMENT_VARS_START,
};

pub struct ScriptCompiler {}

#[cfg(test)]
mod tests {

    #[test]
    fn test_stat() {
        // let mut parser = ScriptParser::new();
        // let mut executor = ScriptExecutor::new();

        // let mut test_in = ScriptTestIn::default();
        // test_in.d = fi(12);
        // let mut test_out = ScriptTestOut::default();

        // test_out.z = fi(0);
        // let ast = parser.parse("test_out.z = 1 + test_in.d").unwrap();
        // executor.execute(&code, &test_in, &mut test_out).unwrap();
        // assert_eq!(fi(13), test_out.z);

        // test_out.z = fi(0);
        // let code = parser
        //     .parse(ScriptEnv::Test, "test_out.z += 2 * test_in.d")
        //     .unwrap();
        // executor.execute(&code, &test_in, &mut test_out).unwrap();
        // assert_eq!(fi(24), test_out.z);

        // test_out.x = fi(0);
        // let code = parser
        //     .parse(ScriptEnv::Test, "test_out.x -= (2 * -test_in.d) + 4")
        //     .unwrap();
        // executor.execute(&code, &test_in, &mut test_out).unwrap();
        // assert_eq!(fi(20), test_out.x);

        // test_out.x = fi(0);
        // test_out.y = fi(0);
        // let code = parser
        //     .parse(ScriptEnv::Test, "test_out.x = 1\ntest_out.y = 2")
        //     .unwrap();
        // executor.execute(&code, &test_in, &mut test_out).unwrap();
        // assert_eq!(fi(1), test_out.x);
        // assert_eq!(fi(2), test_out.y);
    }

    // #[test]
    // fn test_executor_operation() {
    //     let parser = ScriptParser::new();
    //     let mut executor = ScriptExecutor::new();

    //     let byte_code = parser.parse("+ +TAU").unwrap();
    //     let res = executor.execute(&byte_code).unwrap();
    //     assert_eq!(res, Fx::tau());

    //     let byte_code = parser.parse("- + -0.01").unwrap();
    //     let res = executor.execute(&byte_code).unwrap();
    //     assert_eq!(res, ff(0.01));

    //     let byte_code = parser.parse("(1.5 + 0x1)").unwrap();
    //     let res = executor.execute(&byte_code).unwrap();
    //     assert_eq!(res, ff(2.5));

    //     let byte_code = parser.parse("1.5 - (-1 - 0)").unwrap();
    //     let res = executor.execute(&byte_code).unwrap();
    //     assert_eq!(res, ff(2.5));

    //     let byte_code = parser.parse("1.5 * -1").unwrap();
    //     let res = executor.execute(&byte_code).unwrap();
    //     assert_eq!(res, ff(-1.5));

    //     let byte_code = parser.parse("1.5 / -1").unwrap();
    //     let res = executor.execute(&byte_code).unwrap();
    //     assert_eq!(res, ff(-1.5));

    //     let byte_code = parser.parse("5 % 3").unwrap();
    //     let res = executor.execute(&byte_code).unwrap();
    //     assert_eq!(res, fi(2));

    //     let byte_code = parser.parse("PI > TAU").unwrap();
    //     let res = executor.execute(&byte_code).unwrap();
    //     assert_eq!(res, fi(0));

    //     let byte_code = parser.parse("PI >= TAU").unwrap();
    //     let res = executor.execute(&byte_code).unwrap();
    //     assert_eq!(res, fi(0));

    //     let byte_code = parser.parse("111 < 222").unwrap();
    //     let res = executor.execute(&byte_code).unwrap();
    //     assert_eq!(res, fi(1));

    //     let byte_code = parser.parse("222 <= 222").unwrap();
    //     let res = executor.execute(&byte_code).unwrap();
    //     assert_eq!(res, fi(1));
    // }

    // #[test]
    // fn test_executor_numeric() {
    //     let parser = ScriptParser::new();
    //     let mut executor = ScriptExecutor::new();

    //     let byte_code = parser.parse("abs(-3)").unwrap();
    //     let res = executor.execute(&byte_code).unwrap();
    //     assert_eq!(res, fi(3));

    //     let byte_code = parser.parse("min(-3, -100)").unwrap();
    //     let res = executor.execute(&byte_code).unwrap();
    //     assert_eq!(res, fi(-100));

    //     let byte_code = parser.parse("max(20, 3 + (-5))").unwrap();
    //     let res = executor.execute(&byte_code).unwrap();
    //     assert_eq!(res, fi(20));

    //     let byte_code = parser.parse("floor(--3.1) + 2").unwrap();
    //     let res = executor.execute(&byte_code).unwrap();
    //     assert_eq!(res, fi(5));

    //     let byte_code = parser.parse("2 * ceil(3.1)").unwrap();
    //     let res = executor.execute(&byte_code).unwrap();
    //     assert_eq!(res, fi(8));

    //     let byte_code = parser.parse("--round(-3.1)").unwrap();
    //     let res = executor.execute(&byte_code).unwrap();
    //     assert_eq!(res, fi(-3));

    //     let byte_code = parser.parse("clamp(1.1, 2, 4)").unwrap();
    //     let res = executor.execute(&byte_code).unwrap();
    //     assert_eq!(res, fi(2));

    //     let byte_code = parser.parse("saturate(0.5 + 0.7) + (1 - 0.5)").unwrap();
    //     let res = executor.execute(&byte_code).unwrap();
    //     assert_eq!(res, ff(1.5));

    //     let byte_code = parser.parse("lerp(0, 2, 0.25)").unwrap();
    //     let res = executor.execute(&byte_code).unwrap();
    //     assert_eq!(res, ff(0.5));
    // }

    // #[test]
    // fn test_executor_exponential() {
    //     let parser = ScriptParser::new();
    //     let mut executor = ScriptExecutor::new();

    //     let byte_code = parser.parse("sqrt(4)").unwrap();
    //     let res = executor.execute(&byte_code).unwrap();
    //     assert_eq!(res, fi(2));

    //     let byte_code = parser.parse("exp(1)").unwrap();
    //     let res = executor.execute(&byte_code).unwrap();
    //     assert_eq!(res, Fx::e());
    // }

    // #[test]
    // fn test_executor_circular() {
    //     let parser = ScriptParser::new();
    //     let mut executor = ScriptExecutor::new();

    //     let byte_code = parser.parse("degrees(PI)").unwrap();
    //     let res = executor.execute(&byte_code).unwrap();
    //     assert_eq!(res, Fx::frac_180_pi() * Fx::pi());

    //     let byte_code = parser.parse("radians(180)").unwrap();
    //     let res = executor.execute(&byte_code).unwrap();
    //     assert_eq!(res, Fx::frac_pi_180() * fi(180));

    //     let byte_code = parser.parse("sin(PI / 6)").unwrap();
    //     let res = executor.execute(&byte_code).unwrap();
    //     assert_eq!(res, Fx::sin(Fx::pi() / fi(6)));

    //     let byte_code = parser.parse("cos(PI / 6)").unwrap();
    //     let res = executor.execute(&byte_code).unwrap();
    //     assert_eq!(res, Fx::cos(Fx::pi() / fi(6)));

    //     let byte_code = parser.parse("tan(PI / 8)").unwrap();
    //     let res = executor.execute(&byte_code).unwrap();
    //     assert_eq!(res, Fx::tan(Fx::pi() / fi(8)));

    //     let byte_code = parser.parse("asin(0.5)").unwrap();
    //     let res = executor.execute(&byte_code).unwrap();
    //     assert_eq!(res, Fx::asin(ff(0.5)));

    //     let byte_code = parser.parse("acos(0.5)").unwrap();
    //     let res = executor.execute(&byte_code).unwrap();
    //     assert_eq!(res, Fx::acos(ff(0.5)));

    //     let byte_code = parser.parse("atan(1)").unwrap();
    //     let res = executor.execute(&byte_code).unwrap();
    //     assert_eq!(res, Fx::atan(fi(1)));

    //     let byte_code = parser.parse("atan2(1, 1)").unwrap();
    //     let res = executor.execute(&byte_code).unwrap();
    //     assert!(res - Fx::pi() / fi(4) < ff(0.00000001));
    // }
}
