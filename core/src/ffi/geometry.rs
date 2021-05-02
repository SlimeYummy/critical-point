use m::{fx_f32, Fx};
use na::{Complex, Quaternion, UnitComplex, UnitQuaternion};

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct FFIComplex {
    pub re: f32,
    pub im: f32,
}

impl From<Complex<Fx>> for FFIComplex {
    fn from(v: Complex<Fx>) -> FFIComplex {
        return FFIComplex {
            re: v.re.to_f32(),
            im: v.im.to_f32(),
        };
    }
}

impl From<UnitComplex<Fx>> for FFIComplex {
    fn from(v: UnitComplex<Fx>) -> FFIComplex {
        return FFIComplex {
            re: v.re.to_f32(),
            im: v.im.to_f32(),
        };
    }
}

impl From<FFIComplex> for Complex<Fx> {
    fn from(v: FFIComplex) -> Complex<Fx> {
        return Complex::new(fx_f32(v.re), fx_f32(v.im));
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct FFIQuaternion {
    pub i: f32,
    pub j: f32,
    pub k: f32,
    pub w: f32,
}

impl From<Quaternion<Fx>> for FFIQuaternion {
    fn from(v: Quaternion<Fx>) -> FFIQuaternion {
        return FFIQuaternion {
            i: v.i.to_f32(),
            j: v.j.to_f32(),
            k: v.k.to_f32(),
            w: v.w.to_f32(),
        };
    }
}

impl From<UnitQuaternion<Fx>> for FFIQuaternion {
    fn from(v: UnitQuaternion<Fx>) -> FFIQuaternion {
        return FFIQuaternion {
            i: v.i.to_f32(),
            j: v.j.to_f32(),
            k: v.k.to_f32(),
            w: v.w.to_f32(),
        };
    }
}

impl From<FFIQuaternion> for Quaternion<Fx> {
    fn from(v: FFIQuaternion) -> Quaternion<Fx> {
        return Quaternion::new(fx_f32(v.w), fx_f32(v.i), fx_f32(v.j), fx_f32(v.k));
    }
}
