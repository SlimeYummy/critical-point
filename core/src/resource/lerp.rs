use m::{ff, fi, Fx};
use na::{ComplexField, RealField};
use serde::{Deserialize, Serialize};

pub trait LerpFunction {
    fn lerp(&self, progress: Fx) -> Fx;
}

// constant

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct LerpConstant {}

impl LerpFunction for LerpConstant {
    fn lerp(&self, _progress: Fx) -> Fx {
        return fi(1);
    }
}

// linear

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct LerpLinear {}

impl LerpFunction for LerpLinear {
    fn lerp(&self, progress: Fx) -> Fx {
        return progress;
    }
}

// x^2

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct LerpQuadIn {}

impl LerpFunction for LerpQuadIn {
    fn lerp(&self, progress: Fx) -> Fx {
        return progress * progress;
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct LerpQuadOut {}

impl LerpFunction for LerpQuadOut {
    fn lerp(&self, progress: Fx) -> Fx {
        let progress = fi(1) - progress;
        return fi(1) - progress * progress;
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct LerpQuadInOut {}

impl LerpFunction for LerpQuadInOut {
    fn lerp(&self, progress: Fx) -> Fx {
        if progress < ff(0.5) {
            return fi(2) * progress * progress;
        } else {
            let progress = fi(1) - progress;
            return fi(1) - fi(2) * progress * progress;
        }
    }
}

// x^3

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct LerpCubicIn {}

impl LerpFunction for LerpCubicIn {
    fn lerp(&self, progress: Fx) -> Fx {
        return progress * progress * progress;
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct LerpCubicOut {}

impl LerpFunction for LerpCubicOut {
    fn lerp(&self, progress: Fx) -> Fx {
        let progress = fi(1) - progress;
        return fi(1) - progress * progress * progress;
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct LerpCubicInOut {}

impl LerpFunction for LerpCubicInOut {
    fn lerp(&self, progress: Fx) -> Fx {
        if progress < ff(0.5) {
            return fi(4) * progress * progress * progress;
        } else {
            let progress = fi(1) - progress;
            return fi(1) - fi(4) * progress * progress * progress;
        }
    }
}

// x^4

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct LerpQuartIn {}

impl LerpFunction for LerpQuartIn {
    fn lerp(&self, progress: Fx) -> Fx {
        return progress * progress * progress * progress;
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct LerpQuartOut {}

impl LerpFunction for LerpQuartOut {
    fn lerp(&self, progress: Fx) -> Fx {
        let progress = fi(1) - progress;
        return fi(1) - progress * progress * progress * progress;
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct LerpQuartInOut {}

impl LerpFunction for LerpQuartInOut {
    fn lerp(&self, progress: Fx) -> Fx {
        if progress < ff(0.5) {
            return fi(8) * progress * progress * progress * progress;
        } else {
            let progress = fi(1) - progress;
            return fi(1) - fi(8) * progress * progress * progress * progress;
        }
    }
}

// sin(x)

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct LerpSinIn {}

impl LerpFunction for LerpSinIn {
    fn lerp(&self, progress: Fx) -> Fx {
        return fi(1) - Fx::cos(progress * Fx::frac_pi_2());
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct LerpSinOut {}

impl LerpFunction for LerpSinOut {
    fn lerp(&self, progress: Fx) -> Fx {
        return Fx::sin(progress * Fx::frac_pi_2());
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct LerpSinInOut {}

impl LerpFunction for LerpSinInOut {
    fn lerp(&self, progress: Fx) -> Fx {
        return ff(0.5) - ff(0.5) * Fx::cos(Fx::pi() * progress);
    }
}

// 2 ^ 10x

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct LerpExpoIn {}

impl LerpFunction for LerpExpoIn {
    fn lerp(&self, progress: Fx) -> Fx {
        if progress == fi(0) {
            return fi(0);
        } else {
            return Fx::powf(fi(2), fi(10) * progress - fi(10));
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct LerpExpoOut {}

impl LerpFunction for LerpExpoOut {
    fn lerp(&self, progress: Fx) -> Fx {
        if progress == fi(1) {
            return fi(1);
        } else {
            let progress = fi(1) - progress;
            return fi(1) - Fx::powf(fi(2), fi(10) * progress - fi(10));
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct LerpExpoInOut {}

impl LerpFunction for LerpExpoInOut {
    fn lerp(&self, progress: Fx) -> Fx {
        if progress < ff(0.5) {
            if progress == fi(0) {
                return fi(0);
            } else {
                let progress = fi(2) * progress;
                return ff(0.5) * Fx::powf(fi(2), fi(10) * progress - fi(10));
            }
        } else {
            if progress == fi(1) {
                return fi(1);
            } else {
                let progress = fi(2) - fi(2) * progress;
                return fi(1) - ff(0.5) * Fx::powf(fi(2), fi(10) * progress - fi(10));
            }
        }
    }
}
