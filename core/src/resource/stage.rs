use super::base::ResObj;
use super::cache::{CompileContext, RestoreContext};
use super::shape::ResShape;
use crate::derive::def_res;
use crate::id::{ClassID, FastResID, ResID};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[def_res(ClassID::StageGeneral)]
#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[def_res(ClassID::StageScenery)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResStageScenery {
    pub res_id: ResID,
    #[serde(skip)]
    pub fres_id: FastResID,
    pub collision: ResShape,
}

#[typetag::serde(name = "StageScenery")]
impl ResObj for ResStageScenery {
    fn compile(&mut self, ctx: &mut CompileContext) -> Result<()> {
        ctx.insert_res_id(&self.res_id)?;
        return Ok(());
    }

    fn restore(&mut self, ctx: &mut RestoreContext) -> Result<()> {
        self.fres_id = ctx.get_fres_id(&self.res_id)?;
        self.collision.restore(ctx)?;
        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resource::shape::{invalid_shape_handle, ResShapeAny, ResShapeCuboid};
    use crate::resource::ResObjSuper;
    use crate::{m::fi, resource::ResShapeBall};
    use na::Isometry3;
    use std::sync::Arc;

    #[test]
    fn test_res_stage_general() {
        let r1 = ResStageGeneral {
            res_id: ResID::from("Stage.Test"),
            fres_id: FastResID::from(1),
            world: ResShape {
                handle: invalid_shape_handle(),
                shape: ResShapeAny::Cuboid(ResShapeCuboid {
                    x: fi(100),
                    y: fi(1),
                    z: fi(100),
                }),
                transform: Isometry3::identity(),
            },
        };
        let o1: Arc<dyn ResObj> = Arc::new(r1.clone());
        let json = serde_json::to_string(&o1).unwrap();
        let o2: Arc<dyn ResObj> = serde_json::from_str(&json).unwrap();
        let r2: Arc<ResStageGeneral> = o2.cast_to().unwrap();

        assert_eq!(r1.class_id(), o2.class_id());
        assert_eq!(r1.res_id(), o2.res_id());
        assert!(o2.fres_id().is_invalid());
        assert_eq!(r1.world.shape, r2.world.shape);
    }

    #[test]
    fn test_res_stage_scenery() {
        let r1 = ResStageScenery {
            res_id: ResID::from("Stage.Test"),
            fres_id: FastResID::from(1),
            collision: ResShape {
                handle: invalid_shape_handle(),
                shape: ResShapeAny::Ball(ResShapeBall { radius: fi(3) }),
                transform: Isometry3::identity(),
            },
        };
        let o1: Arc<dyn ResObj> = Arc::new(r1.clone());
        let json = serde_json::to_string(&o1).unwrap();
        let o2: Arc<dyn ResObj> = serde_json::from_str(&json).unwrap();
        let r2: Arc<ResStageScenery> = o2.cast_to().unwrap();

        assert_eq!(r1.class_id(), o2.class_id());
        assert_eq!(r1.res_id(), o2.res_id());
        assert!(o2.fres_id().is_invalid());
        assert_eq!(r1.collision.shape, r2.collision.shape);
    }
}
