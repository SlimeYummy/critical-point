use m::{fx_f32, Fx};
use na::{Matrix2, Matrix3, Matrix4, Rotation2, Rotation3};

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct FFIMat2f {
    pub m11: f32,
    pub m21: f32,
    pub m12: f32,
    pub m22: f32,
}

impl FFIMat2f {
    pub fn new(m11: f32, m21: f32, m12: f32, m22: f32) -> FFIMat2f {
        return FFIMat2f { m11, m21, m12, m22 };
    }
}

impl From<Matrix2<Fx>> for FFIMat2f {
    fn from(v: Matrix2<Fx>) -> FFIMat2f {
        return FFIMat2f {
            m11: v.m11.to_f32(),
            m21: v.m21.to_f32(),
            m12: v.m12.to_f32(),
            m22: v.m22.to_f32(),
        };
    }
}

impl From<Rotation2<Fx>> for FFIMat2f {
    fn from(v: Rotation2<Fx>) -> FFIMat2f {
        return FFIMat2f::from(Matrix2::from(v));
    }
}

impl From<FFIMat2f> for Matrix2<Fx> {
    fn from(v: FFIMat2f) -> Matrix2<Fx> {
        return Matrix2::new(fx_f32(v.m11), fx_f32(v.m12), fx_f32(v.m21), fx_f32(v.m22));
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct FFIMat3f {
    pub m11: f32,
    pub m21: f32,
    pub m31: f32,
    pub m12: f32,
    pub m22: f32,
    pub m32: f32,
    pub m13: f32,
    pub m23: f32,
    pub m33: f32,
}

impl FFIMat3f {
    pub fn new(
        m11: f32,
        m21: f32,
        m31: f32,
        m12: f32,
        m22: f32,
        m32: f32,
        m13: f32,
        m23: f32,
        m33: f32,
    ) -> FFIMat3f {
        return FFIMat3f {
            m11,
            m21,
            m31,
            m12,
            m22,
            m32,
            m13,
            m23,
            m33,
        };
    }
}

impl From<Matrix3<Fx>> for FFIMat3f {
    fn from(v: Matrix3<Fx>) -> FFIMat3f {
        return FFIMat3f {
            m11: v.m11.to_f32(),
            m21: v.m21.to_f32(),
            m31: v.m31.to_f32(),
            m12: v.m12.to_f32(),
            m22: v.m22.to_f32(),
            m32: v.m32.to_f32(),
            m13: v.m13.to_f32(),
            m23: v.m23.to_f32(),
            m33: v.m33.to_f32(),
        };
    }
}

impl From<Rotation3<Fx>> for FFIMat3f {
    fn from(v: Rotation3<Fx>) -> FFIMat3f {
        return FFIMat3f::from(Matrix3::from(v));
    }
}

impl From<FFIMat3f> for Matrix3<Fx> {
    fn from(v: FFIMat3f) -> Matrix3<Fx> {
        return Matrix3::new(
            fx_f32(v.m11),
            fx_f32(v.m12),
            fx_f32(v.m13),
            fx_f32(v.m21),
            fx_f32(v.m22),
            fx_f32(v.m23),
            fx_f32(v.m31),
            fx_f32(v.m32),
            fx_f32(v.m33),
        );
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct FFIMat4f {
    pub m11: f32,
    pub m21: f32,
    pub m31: f32,
    pub m41: f32,
    pub m12: f32,
    pub m22: f32,
    pub m32: f32,
    pub m42: f32,
    pub m13: f32,
    pub m23: f32,
    pub m33: f32,
    pub m43: f32,
    pub m14: f32,
    pub m24: f32,
    pub m34: f32,
    pub m44: f32,
}

impl FFIMat4f {
    pub fn new(
        m11: f32,
        m21: f32,
        m31: f32,
        m41: f32,
        m12: f32,
        m22: f32,
        m32: f32,
        m42: f32,
        m13: f32,
        m23: f32,
        m33: f32,
        m43: f32,
        m14: f32,
        m24: f32,
        m34: f32,
        m44: f32,
    ) -> FFIMat4f {
        return FFIMat4f {
            m11,
            m21,
            m31,
            m41,
            m12,
            m22,
            m32,
            m42,
            m13,
            m23,
            m33,
            m43,
            m14,
            m24,
            m34,
            m44,
        };
    }
}

impl From<Matrix4<Fx>> for FFIMat4f {
    fn from(v: Matrix4<Fx>) -> FFIMat4f {
        return FFIMat4f {
            m11: v.m11.to_f32(),
            m21: v.m21.to_f32(),
            m31: v.m31.to_f32(),
            m41: v.m41.to_f32(),
            m12: v.m12.to_f32(),
            m22: v.m22.to_f32(),
            m32: v.m32.to_f32(),
            m42: v.m42.to_f32(),
            m13: v.m13.to_f32(),
            m23: v.m23.to_f32(),
            m33: v.m33.to_f32(),
            m43: v.m43.to_f32(),
            m14: v.m14.to_f32(),
            m24: v.m24.to_f32(),
            m34: v.m34.to_f32(),
            m44: v.m44.to_f32(),
        };
    }
}

impl From<FFIMat4f> for Matrix4<Fx> {
    fn from(v: FFIMat4f) -> Matrix4<Fx> {
        return Matrix4::new(
            fx_f32(v.m11),
            fx_f32(v.m12),
            fx_f32(v.m13),
            fx_f32(v.m14),
            fx_f32(v.m21),
            fx_f32(v.m22),
            fx_f32(v.m23),
            fx_f32(v.m24),
            fx_f32(v.m31),
            fx_f32(v.m32),
            fx_f32(v.m33),
            fx_f32(v.m34),
            fx_f32(v.m41),
            fx_f32(v.m42),
            fx_f32(v.m43),
            fx_f32(v.m44),
        );
    }
}
