use super::base::{ResObj, ResObjX};
use super::cache::{CompileContext, RestoreContext};
use super::shape::ResShape;
use crate::id::{FastResID, ResID};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(ResObjX, Debug, Clone, Serialize, Deserialize)]
#[class_id(StageGeneral)]
pub struct ResStageGeneral {
    pub res_id: ResID,
    #[serde(skip)]
    pub fres_id: FastResID,
    pub world: ResShape,
}

#[typetag::serde(name = "StageGeneral")]
impl ResObj for ResStageGeneral {
    fn compile(&mut self, ctx: &mut CompileContext) -> Result<()> {
        ctx.insert_res_id(&self.res_id)?;
        return Ok(());
    }

    fn restore(&mut self, ctx: &mut RestoreContext) -> Result<()> {
        self.fres_id = ctx.get_fres_id(&self.res_id)?;
        self.world.restore(ctx)?;
        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use super::super::shape::{ResShapeAny, ResShapeCuboid};
    use super::*;
    use crate::id::ClassID;
    use crate::m::fi;
    use serde_yaml;
    use std::sync::Arc;

    #[test]
    fn test_res_stage_general() {
        const YAML: &'static str = r#"
            type: StageGeneral
            res_id: Stage.Test
            world:
                type: Cuboid
                x: 100
                y: 1
                z: 100
        "#;

        let stage: Arc<dyn ResObj> = serde_yaml::from_str(YAML).unwrap();
        assert_eq!(stage.class_id(), ClassID::StageGeneral);
        assert_eq!(stage.res_id(), &ResID::from("Stage.Test"));
        assert_eq!(stage.fres_id(), FastResID::invalid());

        let general: Arc<ResStageGeneral> = stage.cast_as().unwrap();
        assert_eq!(
            general.world.shape,
            ResShapeAny::Cuboid(ResShapeCuboid {
                x: fi(100),
                y: fi(1),
                z: fi(100)
            })
        );
    }
}
