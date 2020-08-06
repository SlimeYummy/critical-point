use m::{fx, Fx};
use nalgebra::{Vector2, Vector3};

pub enum FieldType {
    Gravity,
    LinearVelocity,
    LinearAcceleration,
    AnuglarVelocity,
    AnuglarAcceleration,
}

pub trait Lerper1D {
    fn lerp(&self, frames: u32) -> Fx;
}

pub trait Lerper2D {
    fn lerp(&self, frames: u32) -> Vector2<Fx>;
}

pub trait Lerper3D {
    fn lerp(&self, frames: u32) -> Vector3<Fx>;
}

// constant

pub struct ConstantLerper1D {
    field: FieldType,
    value: Fx,
    frames: u32,
}

impl Lerper1D for ConstantLerper1D {
    fn lerp(&self, _: u32) -> Fx {
        return self.value;
    }
}

pub struct ConstantLerper2D {
    field: FieldType,
    value: Vector2<Fx>,
    frames: u32,
}

impl Lerper2D for ConstantLerper2D {
    fn lerp(&self, _: u32) -> Vector2<Fx> {
        return self.value;
    }
}

pub struct ConstantLerper3D {
    field: FieldType,
    value: Vector3<Fx>,
    frames: u32,
}

impl Lerper3D for ConstantLerper3D {
    fn lerp(&self, _: u32) -> Vector3<Fx> {
        return self.value;
    }
}

// linear

pub struct LinearLerper1D {
    field: FieldType,
    begin: Fx,
    end: Fx,
    frames: u32,
}

impl Lerper1D for LinearLerper1D {
    fn lerp(&self, frames: u32) -> Fx {
        let progress = fx(frames) / fx(self.frames);
        return self.begin * (fx(1) - progress) + self.end * progress;
    }
}

pub struct LinearLerper2D {
    field: FieldType,
    begin: Vector2<Fx>,
    end: Vector2<Fx>,
    frames: u32,
}

impl Lerper2D for LinearLerper2D {
    fn lerp(&self, frames: u32) -> Vector2<Fx> {
        let progress = fx(frames) / fx(self.frames);
        return self.begin * (fx(1) - progress) + self.end * progress;
    }
}

pub struct LinearLerper3D {
    field: FieldType,
    begin: Vector3<Fx>,
    end: Vector3<Fx>,
    frames: u32,
}

impl Lerper3D for LinearLerper3D {
    fn lerp(&self, frames: u32) -> Vector3<Fx> {
        let progress = fx(frames) / fx(self.frames);
        return self.begin * (fx(1) - progress) + self.end * progress;
    }
}
