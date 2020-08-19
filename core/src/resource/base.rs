use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum ResCoordinate {
    World,
    Source,
    Target,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum ResLerpFunction {
    Linear,
    QuadIn,
    QuadOut,
    QuadInOut,
    CubicIn,
    CubicOut,
    CubicInOut,
    QuartIn,
    QuartOut,
    QuartInOut,
    SinIn,
    SinOut,
    SinInOut,
    ExpoIn,
    ExpoOut,
    ExpoInOut,
}

impl Default for ResLerpFunction {
    fn default() -> ResLerpFunction {
        return ResLerpFunction::Linear;
    }
}

pub type ResLerpParameter = [Fx; 4];
