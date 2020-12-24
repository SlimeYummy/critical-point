use super::cache::{CompileContext, RestoreContext};
use crate::id::{ClassID, FastResID, ResID};
use anyhow::{anyhow, Result};
pub(crate) use derive::ResObjX;
use m::Fx;
use serde::{Deserialize, Serialize};
use std::mem;
use std::raw::TraitObject;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum ResCoordinate {
    World,
    Source,
    Target,
    Skill,
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

pub trait ResObjStatic {
    fn id() -> ClassID;
}

pub trait ResObjSuper {
    fn res_id(&self) -> &ResID;
    fn fres_id(&self) -> FastResID;
    fn class_id(&self) -> ClassID;
}

#[typetag::serde(tag = "type")] // TODO: ResObjX
pub trait ResObj
where
    Self: ResObjSuper + Send + Sync,
{
    fn compile(&mut self, ctx: &mut CompileContext) -> Result<()>;
    fn restore(&mut self, ctx: &mut RestoreContext) -> Result<()>;
}

impl dyn ResObj {
    pub fn cast_as<R: ResObj>(self: Arc<dyn ResObj>) -> Result<Arc<R>> {
        let src_to: TraitObject = unsafe { mem::transmute(self) };
        let src_drop = unsafe { *(src_to.vtable as *mut *mut u8) };
        let dst_ref: &dyn ResObj = unsafe { mem::transmute_copy::<usize, &R>(&0) };
        let dst_to: TraitObject = unsafe { mem::transmute(dst_ref) };
        let dst_drop = unsafe { *(dst_to.vtable as *mut *mut u8) };
        if src_drop != dst_drop {
            return Err(anyhow!("ResObj type not match"));
        }
        let dst_arc: Arc<R> = unsafe { mem::transmute(src_to.data) };
        return Ok(dst_arc);
    }

    pub fn cast_to<R: ResObj>(self: &Arc<dyn ResObj>) -> Result<Arc<R>> {
        return self.clone().cast_as::<R>();
    }
}
