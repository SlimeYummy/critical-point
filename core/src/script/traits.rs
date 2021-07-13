use super::command::{ScriptAddr, ScriptVal};
use std::collections::HashMap;
use std::lazy::SyncLazy;

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
