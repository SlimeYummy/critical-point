use crate::id::{FastObjID, FastResID, ObjID, ResID};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct IDTable {
    res_table: HashMap<ResID, FastResID>,
    obj_table: HashMap<ObjID, FastObjID>,
}

impl IDTable {
    pub fn new() -> IDTable {
        return IDTable {
            res_table: HashMap::with_capacity(128),
            obj_table: HashMap::with_capacity(128),
        };
    }

    pub(crate) fn insert_res_id(&mut self, res_id: &ResID, fres_id: FastResID) -> Result<()> {
        return match self.res_table.insert(res_id.clone(), fres_id) {
            None => Ok(()),
            Some(_) => Err(anyhow!("ResID conflict {:?}", res_id)),
        };
    }

    pub fn get_fres_id(&self, res_id: &ResID) -> Result<FastResID> {
        return match self.res_table.get(res_id) {
            Some(id) => Ok(id.clone()),
            None => Err(anyhow!("ResID not found {:?}", res_id)),
        };
    }

    pub(crate) fn insert_obj_id(&mut self, obj_id: &ObjID, fobj_id: FastObjID) -> Result<()> {
        return match self.obj_table.insert(obj_id.clone(), fobj_id) {
            None => Ok(()),
            Some(_) => Err(anyhow!("ObjID conflict {:?}", obj_id)),
        };
    }

    pub fn get_fobj_id(&self, obj_id: &ObjID) -> Result<FastObjID> {
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
