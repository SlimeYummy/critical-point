use super::base::{ResCoordinate, ResLerpFunction, ResLerpParameter};
use super::serde_helper;
use super::shape::ResShape;
use derivative::Derivative;
use m::Fx;
use na::Isometry3;
use nalgebra::{Point3, Translation3, UnitQuaternion};
use ncollide3d::shape::ShapeHandle;
use serde::{Deserialize, Serialize};

#[derive(Derivative, Clone, Deserialize, Serialize)]
#[derivative(Debug)]
pub struct ResSkill {
    pub frames: u32,
    pub shape: ResShape,
    #[derivative(Debug = "ignore")]
    #[serde(skip)]
    #[serde(default = "ResShape::default_handle")]
    pub h_shape: ShapeHandle<Fx>,
    pub center: Point3<Fx>,
    #[serde(with = "serde_helper::isometry")]
    pub origin: Isometry3<Fx>,
    pub origin_coord: ResCoordinate,
    pub motion: Vec<ResMotion>,
    pub motion_coord: ResCoordinate,
    pub effect: ResEffect,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum ResMotion {
    Move(ResMotionMove),
    Rotate(ResMotionRotate),
    MoveSpeed(ResMotionMoveSpeed),
    RotateSpeed(ResMotionRotateSpeed),
    SearchTarget(ResMotionSearchTarget),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ResMotionMove {
    pub start_frame: u32,
    pub finish_frame: u32,
    pub start_value: Translation3<Fx>,
    pub finish_value: Translation3<Fx>,
    #[serde(default)]
    pub lerp_function: ResLerpFunction,
    #[serde(default)]
    pub lerp_parameter: ResLerpParameter,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ResMotionRotate {
    pub start_frame: u32,
    pub finish_frame: u32,
    pub start_value: UnitQuaternion<Fx>,
    pub finish_value: UnitQuaternion<Fx>,
    #[serde(default)]
    pub lerp_function: ResLerpFunction,
    #[serde(default)]
    pub lerp_parameter: ResLerpParameter,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ResMotionMoveSpeed {
    pub start_frame: u32,
    pub finish_frame: u32,
    pub start_value: Translation3<Fx>,
    pub finish_value: Translation3<Fx>,
    #[serde(default)]
    pub lerp_function: ResLerpFunction,
    #[serde(default)]
    pub lerp_parameter: ResLerpParameter,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ResMotionRotateSpeed {
    pub start_frame: u32,
    pub finish_frame: u32,
    pub start_value: UnitQuaternion<Fx>,
    pub finish_value: UnitQuaternion<Fx>,
    #[serde(default)]
    pub lerp_function: ResLerpFunction,
    #[serde(default)]
    pub lerp_parameter: ResLerpParameter,
}

#[derive(Derivative, Clone, Deserialize, Serialize)]
#[derivative(Debug, PartialEq)]
pub struct ResMotionSearchTarget {
    pub start_frame: u32,
    pub finish_frame: u32,
    pub shape: ResShape,
    #[derivative(Debug = "ignore", PartialEq = "ignore")]
    #[serde(skip)]
    #[serde(default = "ResShape::default_handle")]
    pub h_shape: ShapeHandle<Fx>,
    #[serde(with = "serde_helper::isometry")]
    pub transform: Isometry3<Fx>,
    pub searcher: ResMotionSearcher,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum ResMotionSearcher {
    Nearest,
    Random,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ResMotionMoveDirection {
    pub start_frame: u32,
    pub finish_frame: u32,
    pub start_value: Fx,
    pub finish_value: Fx,
    #[serde(default)]
    pub lerp_function: ResLerpFunction,
    #[serde(default)]
    pub lerp_parameter: ResLerpParameter,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ResMotionRotateDirection {
    pub start_frame: u32,
    pub finish_frame: u32,
    pub start_value: Fx,
    pub finish_value: Fx,
    #[serde(default)]
    pub lerp_function: ResLerpFunction,
    #[serde(default)]
    pub lerp_parameter: ResLerpParameter,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResEffect {
    pub source_damage: ResDamage,
    pub target_damage: ResDamage,
    pub new_skill: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ResDamage {
    pub health: i32,
    pub energy: i32,
    pub posture: i32,
    pub physical: i32,
    pub elemental: i32,
    pub arcane: i32,
}

// pub struct ResDetectorMelee {
//     pub duration: u32,
//     pub shape: ResShape,
//     #[serde(skip)]
//     #[serde(default = "ResShape::default_handle")]
//     pub h_shape: ShapeHandle<Fx>,
//     pub center: Point3<Fx>,
//     #[serde(with = "serde_helper::isometry")]
//     pub origin: Isometry3<Fx>,
//     pub start_translation: Translation3<Fx>,
//     pub finish_translation: Translation3<Fx>,
//     pub start_rotation: UnitQuaternion<Fx>,
//     pub finish_rotation: UnitQuaternion<Fx>,
//     pub circle_rotation: i32,
// }
