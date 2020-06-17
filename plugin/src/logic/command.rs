use super::*;

#[derive(Clone, Debug)]
pub struct CmdNewStage {}

impl CmdNewStage {
    pub fn new() -> CmdNewStage {
        CmdNewStage {}
    }
}

impl Command for CmdNewStage {
    fn get_name(&self) -> &'static str {
        "NewStage"
    }

    fn execute(&self, engine: &mut LogicEngine) -> Result<(), Error> {
        let stage = LogicStage::new(engine.gene_obj_id());
        return engine.register_stage(stage);
    }
}

#[derive(Clone, Debug)]
pub struct CmdNewCharacter {}

impl CmdNewCharacter {
    pub fn new() -> CmdNewCharacter {
        CmdNewCharacter {}
    }
}

impl Command for CmdNewCharacter {
    fn get_name(&self) -> &'static str {
        "NewCharacter"
    }

    fn execute(&self, engine: &mut LogicEngine) -> Result<(), Error> {
        let chara = LogicCharacter::new(engine.gene_obj_id());
        return engine.register_character(chara);
    }
}
