use crate::id::{FastResID, ResID};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct IDTable {
    res_table: HashMap<ResID, FastResID>,
}

impl IDTable {
    pub fn new() -> IDTable {
        return IDTable {
            res_table: HashMap::with_capacity(128),
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

    pub fn res_count(&self) -> usize {
        return self.res_table.len();
    }
}
