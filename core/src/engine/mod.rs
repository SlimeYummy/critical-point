pub mod action;
pub mod engine;
pub mod logic_data;
pub mod logic_obj;
pub mod logic_obj_ref;
pub mod operation;

pub(crate) use crate::derive::{def_obj, def_prop, def_state};
pub use action::*;
pub use engine::LogicEngine;
pub use logic_data::{
    DataPool, LogicLifecycle, LogicProp, LogicPropStatic, LogicState, LogicStateStatic,
};
pub use logic_obj::{LogicChara, LogicStage};
pub use logic_obj::{LogicObj, LogicObjStatic, LogicObjSuper};
pub use logic_obj_ref::{RefObj, RefObjError, RefObjRef, RefObjRefMut};
pub use operation::{OpAction, OpCommand, OpMode, Operation};
