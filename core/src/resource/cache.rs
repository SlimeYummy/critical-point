// use super::action::ResAction;
use super::base::ResObj;
use super::id_table::IDTable;
use super::shape::{ShapeCacheKey, ShapeCacheValue};
use crate::id::{FastObjID, FastResID, ObjID, ResID, FastResIDGener, FastObjIDGener};
use anyhow::{anyhow, Context, Result};
use derivative::Derivative;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json;
use serde_yaml;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::PathBuf;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CacheStatus {
    Compiling,
    Compiled,
    Restoring,
    Restored,
}

pub struct ResCache {
    status: CacheStatus,
    file_pathes: HashSet<PathBuf>,
    id_table: IDTable,
    res_cache: HashMap<ResID, Arc<dyn ResObj>>,
    fres_cache: HashMap<FastResID, Arc<dyn ResObj>>,
    shape_cache: HashMap<ShapeCacheKey, ShapeCacheValue>,
}

impl ResCache {
    pub fn compile(res_file: &str) -> Result<Arc<ResCache>> {
        let mut cache = ResCache {
            status: CacheStatus::Compiling,
            file_pathes: HashSet::new(),
            id_table: IDTable::new(),
            res_cache: HashMap::new(),
            fres_cache: HashMap::new(),
            shape_cache: HashMap::new(),
        };

        let res_path = PathBuf::from(res_file).canonicalize()?;
        cache.load_res_objs(res_path)?;

        cache.compile_res_objs()?;
        cache.status = CacheStatus::Compiled;

        return Ok(Arc::new(cache));
    }

    pub fn restore(res_file: &str, id_file: &str) -> Result<Arc<ResCache>> {
        let mut cache = ResCache {
            status: CacheStatus::Restoring,
            file_pathes: HashSet::new(),
            id_table: Self::load_file(&PathBuf::from(id_file))?,
            res_cache: HashMap::new(),
            fres_cache: HashMap::new(),
            shape_cache: HashMap::new(),
        };

        let res_path = PathBuf::from(res_file).canonicalize()?;
        cache.load_res_objs(res_path)?;

        cache.restore_res_objs()?;
        cache.status = CacheStatus::Restored;

        return Ok(Arc::new(cache));
    }

    fn load_res_objs(&mut self, res_path: PathBuf) -> Result<()> {
        if self.file_pathes.contains(&res_path) {
            return Ok(());
        }
        let res_file: ResFile =
            Self::load_file(&res_path).with_context(|| format!("file {:?}", res_path))?;
        self.file_pathes.insert(res_path.clone());

        for res in &res_file.resource {
            let res_id = res.res_id().clone();
            if self.res_cache.contains_key(&res_id) {
                return Err(anyhow!("ResID conflict {:?}", res_id));
            }
            self.res_cache.insert(res_id, res.clone());
        }

        for inc_file in &res_file.include {
            let inc_path = Self::offset_path(&res_path, inc_file)?;
            self.load_res_objs(inc_path)?;
        }
        return Ok(());
    }

    fn compile_res_objs(&mut self) -> Result<()> {
        let mut res_cache = self.res_cache.clone();
        let mut ctx = CompileContext {
            cache: self,
            res_gener: FastResIDGener::new(1),
            obj_gener: FastObjIDGener::new(1),
        };
        for (_, res) in &mut res_cache {
            unsafe { Arc::get_mut_unchecked(res).compile(&mut ctx) }?;
        }
        return Ok(());
    }

    fn restore_res_objs(&mut self) -> Result<()> {
        let mut res_cache = self.res_cache.clone();
        let mut fres_cache = HashMap::new();
        let mut ctx = RestoreContext { cache: self };
        for (_, res) in &mut res_cache {
            unsafe { Arc::get_mut_unchecked(res).restore(&mut ctx) }?;
            let fres_id = res.fres_id();
            if fres_cache.insert(fres_id, res.clone()).is_some() {
                return Err(anyhow!("FastResID conflict {:?}", fres_id));
            }
        }
        self.fres_cache = fres_cache;
        return Ok(());
    }

    fn offset_path(path: &PathBuf, sub_path: &str) -> Result<PathBuf> {
        let mut res_path = path.clone();
        res_path.pop();
        res_path.push(sub_path);
        res_path = res_path.canonicalize()?;
        return Ok(res_path);
    }

    fn load_file<D: DeserializeOwned>(file_path: &PathBuf) -> Result<D> {
        if let Some(extension) = file_path.extension() {
            if extension == "yml" || extension == "yaml" {
                let file = File::open(file_path)?;
                return Ok(serde_yaml::from_reader(file)?);
            } else if extension == "json" {
                let file = File::open(file_path)?;
                return Ok(serde_json::from_reader(file)?);
            }
        }
        return Err(anyhow!("Unknown file foramt"));
    }
}

impl ResCache {
    #[inline]
    pub fn id_table(&self) -> &IDTable {
        return &self.id_table;
    }

    #[inline]
    pub fn get_fres_id(&self, res_id: &ResID) -> Result<FastResID> {
        return self.id_table.get_fres_id(res_id);
    }

    #[inline]
    pub fn get_fobj_id(&self, obj_id: &ObjID) -> Result<FastObjID> {
        return self.id_table.get_fobj_id(obj_id);
    }

    #[inline]
    pub fn find_res_by_id(&self, res_id: &ResID) -> Result<Arc<dyn ResObj>> {
        return match self.res_cache.get(res_id) {
            Some(res) => Ok(res.clone()),
            None => Err(anyhow!("ResObj not found {:?}", res_id)),
        };
    }

    #[inline]
    pub fn find_res_by_fid(&self, fres_id: FastResID) -> Result<Arc<dyn ResObj>> {
        return match self.fres_cache.get(&fres_id) {
            Some(res) => Ok(res.clone()),
            None => Err(anyhow!("ResObj not found {:?}", fres_id)),
        };
    }
}

pub struct CompileContext<'t> {
    cache: &'t mut ResCache,
    res_gener: FastResIDGener,
    obj_gener: FastObjIDGener,
}

impl<'t> CompileContext<'t> {
    pub(crate) fn insert_res_id(&mut self, res_id: &ResID) -> Result<()> {
        if self.cache.status != CacheStatus::Compiling {
            return Err(anyhow!("Not in compiling status"));
        }
        return self.cache.id_table.insert_res_id(res_id, self.res_gener.gen());
    }

    pub(crate) fn insert_obj_id(&mut self, obj_id: &ObjID) -> Result<()> {
        if self.cache.status != CacheStatus::Compiling {
            return Err(anyhow!("Not in compiling status"));
        }
        return self.cache.id_table.insert_obj_id(obj_id, self.obj_gener.gen());
    }
}

pub struct RestoreContext<'t> {
    cache: &'t mut ResCache,
}

impl<'t> RestoreContext<'t> {
    pub(crate) fn get_fres_id(&self, res_id: &ResID) -> Result<FastResID> {
        if self.cache.status != CacheStatus::Restoring {
            return Err(anyhow!("Not in restoring status"));
        }
        return self.cache.id_table.get_fres_id(res_id);
    }

    pub(crate) fn get_fobj_id(&self, obj_id: &ObjID) -> Result<FastObjID> {
        if self.cache.status != CacheStatus::Restoring {
            return Err(anyhow!("Not in restoring status"));
        }
        return self.cache.id_table.get_fobj_id(obj_id);
    }

    pub(crate) fn find_res<R: ResObj>(&self, res_id: &ResID) -> Result<Arc<R>> {
        if self.cache.status != CacheStatus::Restoring {
            return Err(anyhow!("Not in restoring status"));
        }
        return match self.cache.res_cache.get(&res_id) {
            Some(res) => res.clone().cast_as::<R>(),
            None => return Err(anyhow!("Character not found {:?}", res_id)),
        };
    }

    pub(crate) fn find_stage(&self, stage_id: &ResID) -> Result<Arc<dyn ResObj>> {
        if self.cache.status != CacheStatus::Restoring {
            return Err(anyhow!("Not in restoring status"));
        }
        let stage = match self.cache.res_cache.get(stage_id) {
            Some(stage) => stage,
            None => return Err(anyhow!("Stage not found {:?}", stage_id)),
        };
        if !stage.class_id().is_stage() {
            return Err(anyhow!("Stage not found {:?}", stage_id));
        }
        return Ok(stage.clone());
    }

    pub(crate) fn find_chara(&self, chara_id: &ResID) -> Result<Arc<dyn ResObj>> {
        if self.cache.status != CacheStatus::Restoring {
            return Err(anyhow!("Not in restoring status"));
        }
        let chara = match self.cache.res_cache.get(chara_id) {
            Some(chara) => chara,
            None => return Err(anyhow!("Character not found {:?}", chara_id)),
        };
        if !chara.class_id().is_stage() {
            return Err(anyhow!("Character not found {:?}", chara_id));
        }
        return Ok(chara.clone());
    }

    pub(crate) fn find_shape(&mut self, key: &ShapeCacheKey) -> Option<ShapeCacheValue> {
        if self.cache.status != CacheStatus::Restoring {
            return None;
        }
        return self.cache.shape_cache.get(&key).map(|value| value.clone());
    }

    pub(crate) fn insert_shape(&mut self, key: ShapeCacheKey, value: ShapeCacheValue) {
        if self.cache.status != CacheStatus::Restoring {
            return;
        }
        self.cache.shape_cache.insert(key, value);
    }
}

#[cfg(test)]
mod tests {
    use super::super::chara::ResCharaGeneral;
    use super::super::command::ResCommand;
    use super::super::stage::ResStageGeneral;
    use super::*;

    #[test]
    fn test_res_cache_compile() {
        let cache = ResCache::compile("../test_files/resource.yaml").unwrap();
        assert_eq!(cache.res_cache.len(), 3);
        assert_eq!(cache.fres_cache.len(), 0);

        let stage_id = ResID::from("Stage.Test");
        let stage = cache
            .find_res_by_id(&stage_id)
            .unwrap()
            .cast_as::<ResStageGeneral>()
            .unwrap();
        assert_eq!(stage_id, stage.res_id);

        let chara_id = ResID::from("Chara.Test");
        let chara = cache
            .find_res_by_id(&chara_id)
            .unwrap()
            .cast_as::<ResCharaGeneral>()
            .unwrap();
        assert_eq!(chara_id, chara.res_id);

        let cmd_id = ResID::from("Command.Test");
        let cmd = cache
            .find_res_by_id(&cmd_id)
            .unwrap()
            .cast_as::<ResCommand>()
            .unwrap();
        assert_eq!(cmd_id, cmd.res_id);

        let id_table = cache.id_table();
        assert!(id_table.get_fres_id(&stage_id).is_ok());
        assert!(id_table.get_fres_id(&chara_id).is_ok());
        assert!(id_table.get_fres_id(&cmd_id).is_ok());
        assert_eq!(id_table.res_count(), 3);
        assert!(id_table.get_fobj_id(&ObjID::from("Cmd.X1")).is_ok());
        assert_eq!(id_table.obj_count(), 1);
    }

    #[test]
    fn test_res_cache_restore() {
        let cache =
            ResCache::restore("../test_files/resource.yaml", "../test_files/id.yml").unwrap();
        assert_eq!(cache.res_cache.len(), 3);
        assert_eq!(cache.fres_cache.len(), 3);

        let id_table = cache.id_table();
        assert_eq!(id_table.res_count(), 3);
        assert_eq!(id_table.obj_count(), 1);

        let stage_id = id_table.get_fres_id(&ResID::from("Stage.Test")).unwrap();
        let stage = cache
            .find_res_by_fid(stage_id)
            .unwrap()
            .cast_as::<ResStageGeneral>()
            .unwrap();
        assert_eq!(stage_id, stage.fres_id);

        let chara_id = id_table.get_fres_id(&ResID::from("Chara.Test")).unwrap();
        let chara = cache
            .find_res_by_fid(chara_id)
            .unwrap()
            .cast_as::<ResCharaGeneral>()
            .unwrap();
        assert_eq!(chara_id, chara.fres_id);

        let cmd_id = id_table.get_fres_id(&ResID::from("Command.Test")).unwrap();
        let cmd = cache
            .find_res_by_fid(cmd_id)
            .unwrap()
            .cast_as::<ResCommand>()
            .unwrap();
        assert_eq!(cmd_id, cmd.fres_id);
    }
}
