use m::{fx_f32, fx_i32, Fx};
use na::{Translation3, Vector2, Vector3, Vector4};

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct FFIVec2f {
    pub x: f32,
    pub y: f32,
}

impl FFIVec2f {
    pub fn new(x: f32, y: f32) -> FFIVec2f {
        return FFIVec2f { x, y };
    }
}

impl From<Vector2<Fx>> for FFIVec2f {
    fn from(v: Vector2<Fx>) -> FFIVec2f {
        return FFIVec2f {
            x: v.x.to_f32(),
            y: v.y.to_f32(),
        };
    }
}

impl From<FFIVec2f> for Vector2<Fx> {
    fn from(v: FFIVec2f) -> Vector2<Fx> {
        return Vector2::new(fx_f32(v.x), fx_f32(v.y));
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct FFIVec2i {
    pub x: i32,
    pub y: i32,
}

impl FFIVec2i {
    pub fn new(x: i32, y: i32) -> FFIVec2i {
        return FFIVec2i { x, y };
    }
}

impl From<Vector2<Fx>> for FFIVec2i {
    fn from(v: Vector2<Fx>) -> FFIVec2i {
        return FFIVec2i {
            x: v.x.to_i32(),
            y: v.y.to_i32(),
        };
    }
}

impl From<Vector2<i32>> for FFIVec2i {
    fn from(v: Vector2<i32>) -> FFIVec2i {
        return FFIVec2i { x: v.x, y: v.y };
    }
}

impl From<FFIVec2i> for Vector2<Fx> {
    fn from(v: FFIVec2i) -> Vector2<Fx> {
        return Vector2::new(fx_i32(v.x), fx_i32(v.y));
    }
}

impl From<FFIVec2i> for Vector2<i32> {
    fn from(v: FFIVec2i) -> Vector2<i32> {
        return Vector2::new(v.x, v.y);
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct FFIVec3f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl FFIVec3f {
    pub fn new(x: f32, y: f32, z: f32) -> FFIVec3f {
        return FFIVec3f { x, y, z };
    }
}

impl From<Vector3<Fx>> for FFIVec3f {
    fn from(v: Vector3<Fx>) -> FFIVec3f {
        return FFIVec3f {
            x: v.x.to_f32(),
            y: v.y.to_f32(),
            z: v.z.to_f32(),
        };
    }
}

impl From<Translation3<Fx>> for FFIVec3f {
    fn from(v: Translation3<Fx>) -> FFIVec3f {
        return FFIVec3f {
            x: v.x.to_f32(),
            y: v.y.to_f32(),
            z: v.z.to_f32(),
        };
    }
}

impl From<FFIVec3f> for Vector3<Fx> {
    fn from(v: FFIVec3f) -> Vector3<Fx> {
        return Vector3::new(fx_f32(v.x), fx_f32(v.y), fx_f32(v.z));
    }
}

impl From<FFIVec3f> for Translation3<Fx> {
    fn from(v: FFIVec3f) -> Translation3<Fx> {
        return Translation3::new(fx_f32(v.x), fx_f32(v.y), fx_f32(v.z));
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct FFIVec3i {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl FFIVec3i {
    pub fn new(x: i32, y: i32, z: i32) -> FFIVec3i {
        return FFIVec3i { x, y, z };
    }
}

impl From<Vector3<Fx>> for FFIVec3i {
    fn from(v: Vector3<Fx>) -> FFIVec3i {
        return FFIVec3i {
            x: v.x.to_i32(),
            y: v.y.to_i32(),
            z: v.z.to_i32(),
        };
    }
}

impl From<Vector3<i32>> for FFIVec3i {
    fn from(v: Vector3<i32>) -> FFIVec3i {
        return FFIVec3i {
            x: v.x,
            y: v.y,
            z: v.z,
        };
    }
}

impl From<FFIVec3i> for Vector3<Fx> {
    fn from(v: FFIVec3i) -> Vector3<Fx> {
        return Vector3::new(fx_i32(v.x), fx_i32(v.y), fx_i32(v.z));
    }
}

impl From<FFIVec3i> for Vector3<i32> {
    fn from(v: FFIVec3i) -> Vector3<i32> {
        return Vector3::new(v.x, v.y, v.z);
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct FFIVec4f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl FFIVec4f {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> FFIVec4f {
        return FFIVec4f { x, y, z, w };
    }
}

impl From<Vector4<Fx>> for FFIVec4f {
    fn from(v: Vector4<Fx>) -> FFIVec4f {
        return FFIVec4f {
            x: v.x.to_f32(),
            y: v.y.to_f32(),
            z: v.z.to_f32(),
            w: v.w.to_f32(),
        };
    }
}

impl From<FFIVec4f> for Vector4<Fx> {
    fn from(v: FFIVec4f) -> Vector4<Fx> {
        return Vector4::new(fx_f32(v.x), fx_f32(v.y), fx_f32(v.z), fx_f32(v.w));
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct FFIVec4i {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub w: i32,
}

impl FFIVec4i {
    pub fn new(x: i32, y: i32, z: i32, w: i32) -> FFIVec4i {
        return FFIVec4i { x, y, z, w };
    }
}

impl From<Vector4<Fx>> for FFIVec4i {
    fn from(v: Vector4<Fx>) -> FFIVec4i {
        return FFIVec4i {
            x: v.x.to_i32(),
            y: v.y.to_i32(),
            z: v.z.to_i32(),
            w: v.w.to_i32(),
        };
    }
}

impl From<Vector4<i32>> for FFIVec4i {
    fn from(v: Vector4<i32>) -> FFIVec4i {
        return FFIVec4i {
            x: v.x,
            y: v.y,
            z: v.z,
            w: v.w,
        };
    }
}

impl From<FFIVec4i> for Vector4<Fx> {
    fn from(v: FFIVec4i) -> Vector4<Fx> {
        return Vector4::new(fx_i32(v.x), fx_i32(v.y), fx_i32(v.z), fx_i32(v.w));
    }
}

impl From<FFIVec4i> for Vector4<i32> {
    fn from(v: FFIVec4i) -> Vector4<i32> {
        return Vector4::new(v.x, v.y, v.z, v.w);
    }
}
