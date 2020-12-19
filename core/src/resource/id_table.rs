use crate::id::{FastObjID, FastObjIDGener, FastResID, FastResIDGener, ObjID, ResID};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct IDTable {
    res_table: HashMap<ResID, FastResID>,
    obj_table: HashMap<ObjID, FastObjID>,
    #[serde(skip, default = "IDTable::default_res_gener")]
    res_gener: FastResIDGener,
    #[serde(skip, default = "IDTable::default_obj_gener")]
    obj_gener: FastObjIDGener,
}

impl IDTable {
    pub fn new(res_start: u64, obj_start: u64) -> IDTable {
        return IDTable {
            res_table: HashMap::with_capacity(128),
            obj_table: HashMap::with_capacity(128),
            res_gener: FastResIDGener::new(res_start),
            obj_gener: FastObjIDGener::new(obj_start),
        };
    }

    fn default_res_gener() -> FastResIDGener {
        return FastResIDGener::new(u64::MAX);
    }

    fn default_obj_gener() -> FastObjIDGener {
        return FastObjIDGener::new(u64::MAX);
    }

    pub(crate) fn insert_res_id(&mut self, res_id: &ResID) -> Result<()> {
        return match self.res_table.insert(res_id.clone(), self.res_gener.gen()) {
            None => Ok(()),
            Some(_) => Err(anyhow!("ResID conflict {:?}", res_id)),
        };
    }

    pub fn find_fres_id(&self, res_id: &ResID) -> Result<FastResID> {
        return match self.res_table.get(res_id) {
            Some(id) => Ok(id.clone()),
            None => Err(anyhow!("ResID not found {:?}", res_id)),
        };
    }

    pub(crate) fn insert_obj_id(&mut self, obj_id: &ObjID) -> Result<()> {
        return match self.obj_table.insert(obj_id.clone(), self.obj_gener.gen()) {
            None => Ok(()),
            Some(_) => Err(anyhow!("ObjID conflict {:?}", obj_id)),
        };
    }

    pub fn find_fobj_id(&self, obj_id: &ObjID) -> Result<FastObjID> {
        return match self.obj_table.get(obj_id) {
            Some(id) => Ok(id.clone()),
            None => Err(anyhow!("ObjID not found {:?}", obj_id)),
        };
    }

    pub fn res_count(&self) -> usize {
        return self.res_table.len();
    }

    pub fn obj_count(&self) -> usize {
        return self.obj_table.len();
    }
}
