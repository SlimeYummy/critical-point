mod ast;
mod byte_code;
mod command;
mod executor;
mod generator;
mod optimizer;
mod parser;
mod test;
mod traits;

pub use byte_code::ScriptByteCode;
pub use command::{ScriptAddr, ScriptOpt, ScriptType, ScriptVal};
pub use executor::{ScriptError, ScriptExecutor, SEGMENT_VARS_START};
pub use traits::{
    ScriptCtx, ScriptCtxField, ScriptCtxFields, ScriptCtxVar, ScriptCtxVars, ScriptVar,
    ScriptVarField, ScriptVarFields,
};

use anyhow::Result;
use generator::ScriptGenerator;
use optimizer::ScriptOptimizer;
use parser::ScriptParser;

pub struct ScriptCompiler {
    parser: ScriptParser,
    optimizer: ScriptOptimizer,
    generator: ScriptGenerator,
}

impl ScriptCompiler {
    pub fn new() -> ScriptCompiler {
        return ScriptCompiler {
            parser: ScriptParser::new(),
            optimizer: ScriptOptimizer::new(),
            generator: ScriptGenerator::new(),
        };
    }

    pub fn run<C: ScriptCtx>(&mut self, code: &str) -> Result<ScriptByteCode> {
        let block = self.parser.run::<C>(code)?;
        let block = self.optimizer.run(block);
        let byte_code = self.generator.run::<C>(block)?;
        return Ok(byte_code);
    }
}
