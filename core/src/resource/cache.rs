// use super::action::ResAction;
use super::base::ResObj;
use super::id_table::IDTable;
use super::shape::{ShapeCacheKey, ShapeCacheValue};
use crate::id::{FastResID, FastResIDGener, ResID};
use crate::utils::deserialize;
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
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
    Unknown,
    Compiling,
    Compiled,
    Restoring,
    Restored,
}

pub struct ResCache {
    status: CacheStatus,
    root_path: PathBuf,
    file_pathes: HashSet<PathBuf>,
    id_table: IDTable,
    res_cache: HashMap<ResID, Arc<dyn ResObj>>,
    fres_cache: HashMap<FastResID, Arc<dyn ResObj>>,
    shape_cache: HashMap<ShapeCacheKey, ShapeCacheValue>,
}

impl ResCache {
    pub(crate) fn new() -> ResCache {
        return ResCache {
            root_path: PathBuf::new(),
            status: CacheStatus::Unknown,
            file_pathes: HashSet::new(),
            id_table: IDTable::new(),
            res_cache: HashMap::new(),
            fres_cache: HashMap::new(),
            shape_cache: HashMap::new(),
        };
    }

    pub fn compile(root_path: &str, res_file: &str) -> Result<Arc<ResCache>> {
        let mut cache = ResCache::new();
        cache.status = CacheStatus::Compiling;
        cache.root_path = PathBuf::from(root_path);

        cache.load_res_objs(res_file)?;

        cache.compile_res_objs()?;
        cache.status = CacheStatus::Compiled;

        return Ok(Arc::new(cache));
    }

    pub fn restore(root_path: &str, res_file: &str, id_file: &str) -> Result<Arc<ResCache>> {
        let mut cache = ResCache::new();
        cache.status = CacheStatus::Restoring;
        cache.root_path = PathBuf::from(root_path);

        let id_path = cache.get_res_path(id_file)?;
        cache.id_table = deserialize(&id_path)?;
        cache.load_res_objs(res_file)?;

        let res = cache.restore_res_objs();
        res?;
        cache.status = CacheStatus::Restored;

        return Ok(Arc::new(cache));
    }

    fn load_res_objs(&mut self, res_file: &str) -> Result<()> {
        let get_res_path = self.get_res_path(res_file)?;
        if self.file_pathes.contains(&get_res_path) {
            return Ok(());
        }
        let res_file: ResFile =
            deserialize(&get_res_path).context(format!("file {:?}", get_res_path))?;
        self.file_pathes.insert(get_res_path.clone());

        for res in &res_file.resource {
            let res_id = res.res_id().clone();
            if self.res_cache.contains_key(&res_id) {
                return Err(anyhow!("ResID conflict {:?}", res_id));
            }
            self.res_cache.insert(res_id, res.clone());
        }

        for inc_file in &res_file.include {
            self.load_res_objs(inc_file)?;
        }
        return Ok(());
    }

    fn compile_res_objs(&mut self) -> Result<()> {
        let mut res_cache = self.res_cache.clone();
        let mut ctx = CompileContext {
            cache: self,
            res_gener: FastResIDGener::new(1),
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

    fn get_res_path(&self, file: &str) -> Result<PathBuf> {
        let mut path = self.root_path.clone();
        path.push(file);
        return Ok(path.canonicalize()?);
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
}

impl<'t> CompileContext<'t> {
    pub(crate) fn insert_res_id(&mut self, res_id: &ResID) -> Result<()> {
        if self.cache.status != CacheStatus::Compiling {
            return Err(anyhow!("Not in compiling status"));
        }
        return self
            .cache
            .id_table
            .insert_res_id(res_id, self.res_gener.gen());
    }
}

pub struct RestoreContext<'t> {
    cache: &'t mut ResCache,
}

#[allow(dead_code)]
impl<'t> RestoreContext<'t> {
    pub(crate) fn root_path(&self) -> &PathBuf {
        return &self.cache.root_path;
    }

    pub(crate) fn get_res_path(&self, file: &str) -> Result<PathBuf> {
        return self.cache.get_res_path(file);
    }

    pub(crate) fn get_fres_id(&self, res_id: &ResID) -> Result<FastResID> {
        if self.cache.status != CacheStatus::Restoring {
            return Err(anyhow!("Not in restoring status"));
        }
        return self.cache.id_table.get_fres_id(res_id);
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
    use super::*;
    use crate::resource::character::ResCharaHuman;
    use crate::resource::command::ResCommand;
    use crate::resource::stage::ResStageGeneral;

    #[test]
    fn test_res_cache_compile() {
        let cache = ResCache::compile("../test_files/resource", "resource.yaml").unwrap();
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
            .cast_as::<ResCharaHuman>()
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
            ResCache::restore("../test_files/resource/", "resource.yaml", "id.yml").unwrap();
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
            .cast_as::<ResCharaHuman>()
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
