use super::super::script;
use crate::derive::{script_ctx, script_var};
use math::{ff, fx_bool, Fx};

#[script_var(prefix = "test_in")]
#[derive(Default, Debug)]
pub struct VarTestIn {
    pub aa: Fx,
    pub bb: Fx,
    pub cc: Fx,
    pub dd: Fx,
}

#[script_var(prefix = "test_out")]
#[derive(Default, Debug)]
pub struct VarTestOut {
    pub xx: Fx,
    pub yy: Fx,
    pub zz: Fx,
    #[script_skip]
    pub ww: Fx,
    pub ids: Vec<usize>,
}

impl VarTestOut {
    pub fn add_id(&mut self, id: usize) {
        self.ids.push(id);
    }

    pub fn has_id(&self, id: usize) -> Fx {
        return fx_bool(self.ids.iter().find(|x| **x == id).is_some());
    }
}

#[script_ctx]
#[derive(Debug)]
pub struct CtxTest<'t> {
    pub test_in: &'t VarTestIn,
    pub test_out: &'t mut VarTestOut,
}

impl<'t> CtxTest<'t> {
    pub fn new(test_in: &'t VarTestIn, test_out: &'t mut VarTestOut) -> CtxTest<'t> {
        return CtxTest { test_in, test_out };
    }
}
