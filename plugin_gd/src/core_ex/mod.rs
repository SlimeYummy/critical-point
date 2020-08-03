#![allow(dead_code)]

use core::agent::{AsyncStateReg, SyncStateReg};
use core::state::StateRef;

mod async_core;
mod sync_core;

pub use async_core::*;
pub use sync_core::*;

pub type SyncStateRef<S> = StateRef<S, SyncStateReg<S>>;
pub type AsyncStateRef<S> = StateRef<S, AsyncStateReg<S>>;
