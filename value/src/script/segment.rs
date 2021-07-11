use super::command::{ScriptAddr, ScriptVal};
use std::collections::HashMap;
use std::lazy::SyncLazy;

pub const MAX_REGISTERS: usize = 64;
pub const MAX_CONSTANTS: usize = 0x1000;
pub const MAX_SEGMENTS: usize = 16;

pub const SEGMENT_CONSTANT: u8 = 0;
pub const SEGMENT_REGISTER: u8 = 1;
pub const SEGMENT_VARS_START: u8 = 2;

//
// script variable
//

pub struct ScriptVarField {
    pub ident: &'static str,
    pub offset: u16,
}

pub type ScriptVarFields = SyncLazy<HashMap<&'static str, ScriptVarField>>;

// pub static EMPTY_VAR_ITEMS: ScriptVarFields = SyncLazy::new(|| HashMap::new());

pub trait ScriptVar {
    fn var_id() -> u8;
    fn prefix() -> &'static str;
    fn fields() -> &'static ScriptVarFields;
    fn max_offset() -> u16;

    fn field(ident: &'static str) -> &'static ScriptVarField {
        return &Self::fields()[ident];
    }
}

//
// script context
//

pub struct ScriptCtxField {
    pub ident: &'static str,
    pub writable: bool,
    pub addr: ScriptAddr,
}

pub type ScriptCtxFields = SyncLazy<HashMap<&'static str, ScriptCtxField>>;

pub static EMPTY_CTX_FIELDS: ScriptCtxFields = SyncLazy::new(|| HashMap::new());

pub struct ScriptCtxVar {
    pub var_id: u8,
    pub segment: u8,
    pub writable: bool,
}

pub type ScriptCtxVars = SyncLazy<HashMap<u8, ScriptCtxVar>>;

pub static EMPTY_CTX_VARS: ScriptCtxVars = SyncLazy::new(|| HashMap::new());

pub trait ScriptCtx {
    fn ctx_id() -> u8;
    fn fields() -> &'static ScriptCtxFields;
    fn vars() -> &'static ScriptCtxVars;
    fn fill_segments(&self, segments: &mut [*mut ScriptVal]);

    fn field(ident: &'static str) -> &'static ScriptCtxField {
        return &Self::fields()[ident];
    }

    fn var(var_id: u8) -> &'static ScriptCtxVar {
        return &Self::vars()[&var_id];
    }
}

// #[script_var(prefix = "test_out")]
// #[derive(Default)]
// pub struct ScriptExprOut {
//     out: Fx,
// }

// #[script_var(prefix = "chara")]
// #[derive(Default)]
// pub struct ScriptChara {
//     pub max_health: Fx,
//     pub now_health: Fx,

//     pub max_energy: Fx,
//     pub now_energy: Fx,
//     pub energy_speed: Fx,

//     pub max_shield: Fx,
//     pub now_shield: Fx,
//     pub extra_shield: Fx,
//     pub shield_speed: Fx,

//     pub physical_atk: Fx,
//     pub physical_def: Fx,
//     pub elemental_atk: Fx,
//     pub elemental_def: Fx,
//     pub arcane_atk: Fx,
//     pub arcane_def: Fx,
// }

// #[script_var(prefix = "hit")]
// #[derive(Default)]
// pub struct ScriptHit {
//     total_frames: Fx,
//     elapsed_frames: Fx,
//     hit_times: Fx,
//     refresh_times: Fx,
// }

// #[script_var(prefix = "src_effect")]
// #[derive(Default)]
// pub struct ScriptSrcEffect {
//     pub health: Fx,
//     pub energy: Fx,
//     pub shield: Fx,
// }

// #[script_var(prefix = "dst_effect")]
// #[derive(Default)]
// pub struct ScriptDstEffect {
//     pub physical_dmg: Fx,
//     pub physical_bk: Fx,
//     pub elemental_dmg: Fx,
//     pub elemental_bk: Fx,
//     pub arcane_dmg: Fx,
//     pub arcane_bk: Fx,

//     pub health: Fx,
//     pub energy: Fx,
//     pub shield: Fx,
// }
