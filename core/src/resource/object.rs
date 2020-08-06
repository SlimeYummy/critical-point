use super::ResShape;
use fixed::traits::ToFixed;
use math::Fx;
use na::geometry::Isometry3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ncollide3d::shape::ShapeHandle;
use derivative::Derivative;

#[derive(Clone, Derivative, Deserialize, Serialize)]
#[derivative(Debug)]
pub struct ResCharacter {
    pub id: String,
    pub collision: ResShape,
    pub transform: Isometry3<Fx>,
    pub max_health: u32,
    pub max_energy: u32,
    pub max_balance: u32,
    pub default_patience: u32,
    pub move_speed: Fx,
    pub physical_attack: u32,
    pub physical_defense: u32,
    pub elemental_attack: u32,
    pub elemental_defense: u32,
    pub arcane_attack: u32,
    #[derivative(Debug="ignore")]
    pub arcane_defense: u32,
    pub skill_ids: Vec<String>,
    #[serde(default)]
    pub skills: HashMap<String, ResSkill>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResSkill {
    pub id: String,
}
