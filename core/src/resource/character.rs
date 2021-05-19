// use super::action::ResAction;
use super::base::ResObj;
use super::cache::{CompileContext, RestoreContext};
use super::shape::ResShape;
use crate::derive::def_res;
use crate::id::{ClassID, FastResID, ResID};
use anyhow::Result;
use m::Fx;
use serde::{Deserialize, Serialize};

#[def_res(ClassID::CharaHuman)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResCharaHuman {
    pub res_id: ResID,
    #[serde(skip)]
    pub fres_id: FastResID,
    pub collision: ResShape,
    pub max_health: i32,
    pub max_energy: i32,
    pub max_posture: i32,
    pub move_speed: Fx,
    pub physical_attack: i32,
    pub physical_defense: i32,
    pub elemental_attack: i32,
    pub elemental_defense: i32,
    pub arcane_attack: i32,
    pub arcane_defense: i32,
}

#[typetag::serde(name = "CharaHuman")]
impl ResObj for ResCharaHuman {
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
    // use super::*;
    // use super::resource::ResBall;
    // use approx::relative_eq;
    // use m::fi;

    // #[test]
    // fn test_res_character() {
    //     let c1 = ResCharaHuman {
    //         id: String::from("Character"),
    //         collision: ResShape::Ball(ResBall { radius: fi(1) }),
    //         h_bounding: ResShapeCache::default_handle(),
    //         origin: Isometry3::new(na::zero(), na::zero()),
    //         max_health: 10000,
    //         max_energy: 1000,
    //         max_posture: 1000,
    //         move_speed: fi(10),
    //         physical_attack: 1000,
    //         physical_defense: 500,
    //         elemental_attack: 300,
    //         elemental_defense: 200,
    //         arcane_attack: 750,
    //         arcane_defense: 400,
    //     };
    //     let json = serde_json::to_string(&c1).unwrap();
    //     let c2 = serde_json::from_str::<ResCharaHuman>(&json).unwrap();
    //     assert_eq!(c1.id, c2.id);
    //     assert_eq!(c1.collision, c2.collision);
    //     relative_eq!(c1.origin, c2.origin);
    // }
}
