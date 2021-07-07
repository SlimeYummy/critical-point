use super::ast::AstBlock;

pub struct ScriptOptimizer {}

impl ScriptOptimizer {
    pub fn new() -> ScriptOptimizer {
        return ScriptOptimizer {};
    }

    pub fn run(&mut self, block: AstBlock) -> AstBlock {
        return block;
    }
}
