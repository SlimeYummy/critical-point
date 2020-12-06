use super::base::{ResLerpFunction, ResLerpParameter};
use m::{fi, Fx};
use nalgebra::{UnitComplex, Vector3};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResAction {
    pub id: String,
    pub world_transforms: Vec<ResWorldTransform>,
}

impl ResAction {
    pub(super) fn restore(&mut self, frac_1_fps: Fx) {
        for transform in &mut self.world_transforms {
            transform.restore(frac_1_fps);
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ResWorldTransform {
    Velocity(ResWorldVelocity),
    Towards(ResWorldTowards),
    Switches(ResWorldSwitches),
}

impl ResWorldTransform {
    pub(super) fn restore(&mut self, frac_1_fps: Fx) {
        match self {
            ResWorldTransform::Velocity(velocity) => velocity.restore(frac_1_fps),
            ResWorldTransform::Towards(towards) => towards.restore(frac_1_fps),
            ResWorldTransform::Switches(switches) => switches.restore(frac_1_fps),
        };
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResWorldVelocity {
    pub start_frame: u32,
    pub finish_frame: u32,
    pub start_velocity: Vector3<Fx>,
    pub finish_velocity: Vector3<Fx>,
    pub start_velocity_frame: Vector3<Fx>,
    pub finish_velocity_frame: Vector3<Fx>,
    pub lerp_func: ResLerpFunction,
    pub lerp_param: ResLerpParameter,
}

impl ResWorldVelocity {
    pub(super) fn restore(&mut self, frac_1_fps: Fx) {
        self.start_velocity_frame = self.start_velocity * frac_1_fps;
        self.finish_velocity_frame = self.finish_velocity * frac_1_fps;
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResWorldTowards {
    pub start_frame: u32,
    pub finish_frame: u32,
    pub start_towards: UnitComplex<Fx>,
    pub finish_towards: UnitComplex<Fx>,
    pub start_towards_frame: UnitComplex<Fx>,
    pub finish_towards_frame: UnitComplex<Fx>,
    pub lerp_func: ResLerpFunction,
    pub lerp_param: ResLerpParameter,
}

impl ResWorldTowards {
    pub(super) fn restore(&mut self, _frac_1_fps: Fx) {
        self.start_towards_frame = self.start_towards;
        self.finish_towards_frame = self.finish_towards;
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResWorldSwitches {
    pub gravity: Vector3<Fx>,
    pub rebound_force: Vector3<Fx>,
    pub rebound_times: Fx,
}

impl ResWorldSwitches {
    pub(super) fn restore(&mut self, _frac_1_fps: Fx) {}
}

// #[derive(Debug, Deserialize, Serialize)]
// pub struct ResAction {
//     character: Vec<ResTransform>,
// }

// #[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
// #[serde(tag = "type")]
// pub enum ResTransform {
//     // Linear Velocity
//     LVConstant(ResLinearVelocityConstant),
//     LVLinear(ResLinearVelocityLerp<LerpLinear>),
//     LVQuadIn(ResLinearVelocityLerp<LerpQuadIn>),
//     LVQuadOut(ResLinearVelocityLerp<LerpQuadOut>),
//     LVQuadInOut(ResLinearVelocityLerp<LerpQuadInOut>),
//     LVCubicIn(ResLinearVelocityLerp<LerpCubicIn>),
//     LVCubicOut(ResLinearVelocityLerp<LerpCubicOut>),
//     LVCubicInOut(ResLinearVelocityLerp<LerpCubicInOut>),
//     LVQuartIn(ResLinearVelocityLerp<LerpQuartIn>),
//     LVQuartOut(ResLinearVelocityLerp<LerpQuartOut>),
//     LVQuartInOut(ResLinearVelocityLerp<LerpQuartInOut>),
//     LVSinIn(ResLinearVelocityLerp<LerpSinIn>),
//     LVSinOut(ResLinearVelocityLerp<LerpSinOut>),
//     LVSinInOut(ResLinearVelocityLerp<LerpSinInOut>),
//     LVExpoIn(ResLinearVelocityLerp<LerpExpoIn>),
//     LVExpoOut(ResLinearVelocityLerp<LerpExpoOut>),
//     LVExpoInOut(ResLinearVelocityLerp<LerpExpoInOut>),
//     // Linear Velocity
//     AVConstant(ResAnuglarVelocityConstant),
//     AVLinear(ResAnuglarVelocityLerp<LerpLinear>),
//     AVQuadIn(ResAnuglarVelocityLerp<LerpQuadIn>),
//     AVQuadOut(ResAnuglarVelocityLerp<LerpQuadOut>),
//     AVQuadInOut(ResAnuglarVelocityLerp<LerpQuadInOut>),
//     AVCubicIn(ResAnuglarVelocityLerp<LerpCubicIn>),
//     AVCubicOut(ResAnuglarVelocityLerp<LerpCubicOut>),
//     AVCubicInOut(ResAnuglarVelocityLerp<LerpCubicInOut>),
//     AVQuartIn(ResAnuglarVelocityLerp<LerpQuartIn>),
//     AVQuartOut(ResAnuglarVelocityLerp<LerpQuartOut>),
//     AVQuartInOut(ResAnuglarVelocityLerp<LerpQuartInOut>),
//     AVSinIn(ResAnuglarVelocityLerp<LerpSinIn>),
//     AVSinOut(ResAnuglarVelocityLerp<LerpSinOut>),
//     AVSinInOut(ResAnuglarVelocityLerp<LerpSinInOut>),
//     AVExpoIn(ResAnuglarVelocityLerp<LerpExpoIn>),
//     AVExpoOut(ResAnuglarVelocityLerp<LerpExpoOut>),
//     AVExpoInOut(ResAnuglarVelocityLerp<LerpExpoInOut>),
// }

// // Gravity

// pub struct ResGravitySwitch {
//     pub open: bool,
// }

// // Linear Velocity

// pub trait ResLinearVelocity {
//     fn init(&mut self, fps: u32);
//     fn value(&self, frames: u32) -> Vector3<Fx>;
// }

// #[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
// pub struct ResLinearVelocityConstant {
//     pub frames: u32,
//     pub value: Vector3<Fx>,
//     #[serde(default)]
//     pub frac_value: Vector3<Fx>,
// }

// impl ResLinearVelocity for ResLinearVelocityConstant {
//     fn init(&mut self, fps: u32) {
//         let frac_1_fps = fx(1) / fx(fps);
//         self.frac_value = self.value * frac_1_fps;
//     }

//     fn value(&self, _frames: u32) -> Vector3<Fx> {
//         return self.frac_value;
//     }
// }

// #[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
// pub struct ResLinearVelocityLerp<L: LerpFunction> {
//     pub frames: u32,
//     pub begin: Vector3<Fx>,
//     pub end: Vector3<Fx>,
//     #[serde(default)]
//     pub frac_begin: Vector3<Fx>,
//     #[serde(default)]
//     pub frac_end: Vector3<Fx>,
//     #[serde(flatten)]
//     pub lerp: L,
// }

// impl<L: LerpFunction> ResLinearVelocity for ResLinearVelocityLerp<L> {
//     fn init(&mut self, fps: u32) {
//         let frac_1_fps = fx(1) / fx(fps);
//         self.frac_begin = self.begin * frac_1_fps;
//         self.frac_end = self.end * frac_1_fps;
//     }

//     fn value(&self, frames: u32) -> Vector3<Fx> {
//         let process = self.lerp.lerp(fx(frames) / fx(self.frames));
//         return Vector3::lerp(&self.frac_begin, &self.frac_end, process);
//     }
// }

// // Anuglar Velocity

// pub trait ResAnuglarVelocity {
//     fn init(&mut self, fps: u32);
//     fn value(&self, frames: u32) -> UnitQuaternion<Fx>;
// }

// #[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
// pub struct ResAnuglarVelocityConstant {
//     pub frames: u32,
//     pub value: UnitQuaternion<Fx>,
//     #[serde(default)]
//     pub frac_value: UnitQuaternion<Fx>,
// }

// impl ResAnuglarVelocity for ResAnuglarVelocityConstant {
//     fn init(&mut self, fps: u32) {
//         let frac_1_fps = fx(1) / fx(fps);
//         self.frac_value = self.value * frac_1_fps;
//     }

//     fn value(&self, _frames: u32) -> UnitQuaternion<Fx> {
//         return self.frac_value;
//     }
// }

// #[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
// pub struct ResAnuglarVelocityLerp<L: LerpFunction> {
//     pub frames: u32,
//     pub begin: UnitQuaternion<Fx>,
//     pub end: UnitQuaternion<Fx>,
//     #[serde(default)]
//     pub frac_begin: UnitQuaternion<Fx>,
//     #[serde(default)]
//     pub frac_end: UnitQuaternion<Fx>,
//     #[serde(flatten)]
//     pub lerp: L,
// }

// impl<L: LerpFunction> ResAnuglarVelocity for ResAnuglarVelocityLerp<L> {
//     fn init(&mut self, fps: u32) {
//         let frac_1_fps = fx(1) / fx(fps);
//         self.frac_begin = self.begin.nlerp(&UnitQuaternion::identity(), frac_1_fps);
//         self.frac_end = self.end.nlerp(&UnitQuaternion::identity(), frac_1_fps);
//     }

//     fn value(&self, frames: u32) -> UnitQuaternion<Fx> {
//         let process = self.lerp.lerp(fx(frames) / fx(self.frames));
//         return UnitQuaternion::nlerp(&self.frac_begin, &self.frac_end, process);
//     }
// }

// pub struct ResActionWorld {

// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use approx::relative_eq;
//     use serde_json;

//     #[test]
//     fn test_linear_speed_constant() {
//         let ls =
//             serde_json::from_str::<ResLinearVelocityConstant>(r#"{"frames": 10, "value": [1,1,1]}"#)
//                 .unwrap();
//         assert_eq!(
//             ls,
//             ResLinearVelocityConstant {
//                 frames: 10,
//                 value: Vector3::new(fx(1), fx(1), fx(1)),
//             }
//         );
//         assert_eq!(ls.value(5), Vector3::new(fx(1), fx(1), fx(1)));
//     }

//     #[test]
//     fn test_linear_speed_linear() {
//         let mut ls = serde_json::from_str::<ResLinearVelocityLinear>(
//             r#"{"frames": 10, "begin": [1,1,1], "end": [5,5,5]}"#,
//         )
//         .unwrap();
//         assert_eq!(
//             ls,
//             ResLinearVelocityLinear {
//                 frames: 10,
//                 begin: Vector3::new(fx(1), fx(1), fx(1)),
//                 end: Vector3::new(fx(5), fx(5), fx(5)),
//                 frac_begin: na::zero(),
//                 frac_end: na::zero(),
//             }
//         );
//         ls.init(10);
//         relative_eq!(ls.value(5), Vector3::new(fx(0.3), fx(0.3), fx(0.3)));
//     }
// }
