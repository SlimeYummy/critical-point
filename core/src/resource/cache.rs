// use super::action::ResAction;
use super::base::ResObj;
use super::shape::{ShapeCacheKey, ShapeCacheValue};
use crate::id::{ClassID, FastResID, FastResIDGener, ResID};
use crate::util::make_err;
use anyhow::anyhow;
use anyhow::Result;
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use serde_json;
use serde_yaml;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::mem;
use std::raw::TraitObject;
use std::sync::Arc;

#[derive(Derivative, Clone, Serialize, Deserialize)]
#[derivative(Debug)]
struct ResFile {
    #[serde(default)]
    include: Vec<String>,
    #[derivative(Debug = "ignore")]
    #[serde(default)]
    resource: Vec<Arc<dyn ResObj>>,
}

pub struct ResCache {
    loading: bool,
    files: HashSet<String>,
    fres_gener: FastResIDGener,
    res_cache: HashMap<ResID, Arc<dyn ResObj>>,
    fres_cache: HashMap<FastResID, Arc<dyn ResObj>>,
    shape_cache: HashMap<ShapeCacheKey, ShapeCacheValue>,
}

impl ResCache {
    fn default() -> ResCache {
        return ResCache {
            loading: true,
            files: HashSet::with_capacity(64),
            fres_gener: FastResIDGener::new(),
            res_cache: HashMap::with_capacity(128),
            fres_cache: HashMap::with_capacity(128),
            shape_cache: HashMap::with_capacity(512),
        };
    }

    #[allow(dead_code)]
    pub fn from_yaml(path: &str) -> Result<Box<ResCache>> {
        let mut cache = Box::new(ResCache::default());
        cache.parse(path, |file_name| {
            Ok(serde_yaml::from_reader(File::open(file_name)?)?)
        })?;
        cache.restore()?;
        return Ok(cache);
    }

    #[allow(dead_code)]
    pub fn from_json(path: &str) -> Result<Box<ResCache>> {
        let mut cache = Box::new(ResCache::default());
        cache.parse(path, |file_name| {
            Ok(serde_json::from_reader(File::open(file_name)?)?)
        })?;
        cache.restore()?;
        return Ok(cache);
    }

    fn parse(&mut self, file_name: &str, loader: fn(&str) -> Result<ResFile>) -> Result<()> {
        self.files.insert(String::from(file_name));
        let res_file: ResFile = loader(file_name)?;

        let include = res_file.include.clone();
        for inc_file in &include {
            if !self.files.contains(inc_file) {
                self.parse(inc_file, loader)?;
            }
        }

        for res in &res_file.resource {
            let res_id = res.res_id().clone();
            if self.res_cache.contains_key(&res_id) {
                return Err(anyhow!("Resource id conflict {:?}", res_id));
            }
            self.res_cache.insert(res_id, res.clone());
        }
        return Ok(());
    }

    fn restore(&mut self) -> Result<()> {
        let mut res_cache = self.res_cache.clone();
        let mut ctx = RestoreContext { cache: self };
        for (_, res) in &mut res_cache {
            unsafe { Arc::get_mut_unchecked(res).restore(&mut ctx) }?;
        }
        self.loading = false;
        return Ok(());
    }
}

pub struct RestoreContext<'t> {
    cache: &'t mut ResCache,
}

impl<'t> RestoreContext<'t> {
    pub(crate) fn gene_fast_res_id(&mut self) -> FastResID {
        if !self.cache.loading {
            return FastResID::invalid();
        }
        return self.cache.fres_gener.gen();
    }

    pub(crate) fn find_stage(&self, stage_id: ResID) -> Result<Arc<dyn ResObj>> {
        if !self.cache.loading {
            return Err(anyhow!("Cache restore finish"));
        }
        let stage = match self.cache.res_cache.get(&stage_id) {
            Some(stage) => stage,
            None => return Err(anyhow!("{:?} not found", stage_id)),
        };
        if !stage.class_id().is_stage() {
            return Err(anyhow!("{:?} not found", stage_id));
        }
        return Ok(stage.clone());
    }

    pub(crate) fn find_character(&self, chara_id: ResID) -> Result<Arc<dyn ResObj>> {
        if !self.cache.loading {
            return Err(anyhow!("Cache restore finish"));
        }
        let chara = match self.cache.res_cache.get(&chara_id) {
            Some(chara) => chara,
            None => return Err(anyhow!("{:?} not found", chara_id)),
        };
        if !chara.class_id().is_stage() {
            return Err(anyhow!("{:?} not found", chara_id));
        }
        return Ok(chara.clone());
    }

    pub(crate) fn find_shape(&mut self, key: &ShapeCacheKey) -> Option<ShapeCacheValue> {
        if !self.cache.loading {
            return None;
        }
        return self.cache.shape_cache.get(&key).map(|value| value.clone());
    }

    pub(crate) fn insert_shape(&mut self, key: ShapeCacheKey, value: ShapeCacheValue) {
        if !self.cache.loading {
            return;
        }
        self.cache.shape_cache.insert(key, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use na::geometry::Isometry3;

    #[test]
    fn test_res_cache_from_yaml() {
        let cache = ResCache::from_yaml("../test_files/resource.yml").unwrap();
    }

    #[test]
    fn test_from_json() {}
}
