use super::object::{ResCharacter, ResSkill};
use failure::{format_err, Error};
use serde::{Deserialize, Serialize};
use serde_json;
use serde_yaml;
use std::collections::{HashMap, HashSet};
use std::fs::File;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResFile {
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub resource: Vec<ResAny>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ResAny {
    Character(ResCharacter),
}

pub struct ResCache {
    files: HashSet<String>,
    characters: HashMap<String, ResCharacter>,
}

impl ResCache {
    fn default() -> ResCache {
        return ResCache {
            files: HashSet::with_capacity(32),
            characters: HashMap::with_capacity(32),
        };
    }

    pub fn from_yaml(path: &str) -> Result<Box<ResCache>, Error> {
        let mut cache = Box::new(ResCache::default());
        cache.parse(path, |file_name| {
            Ok(serde_yaml::from_reader(File::open(file_name)?)?)
        })?;
        return Ok(cache);
    }

    pub fn from_json(path: &str) -> Result<Box<ResCache>, Error> {
        let mut cache = Box::new(ResCache::default());
        cache.parse(path, |file_name| {
            Ok(serde_json::from_reader(File::open(file_name)?)?)
        })?;
        return Ok(cache);
    }

    pub fn parse(
        &mut self,
        file_name: &str,
        loader: fn(&str) -> Result<ResFile, Error>,
    ) -> Result<(), Error> {
        self.files.insert(String::from(file_name));
        let res_file: ResFile = loader(file_name)?;
        self.save_to_cache(&res_file)?;
        for inc_file in res_file.include.iter() {
            if !self.files.contains(inc_file) {
                self.parse(inc_file, loader)?;
            }
        }
        return Ok(());
    }

    fn save_to_cache(&mut self, file: &ResFile) -> Result<(), Error> {
        for any in file.resource.iter() {
            match any {
                ResAny::Character(chara) => {
                    self.characters
                        .insert(chara.id.clone(), chara.clone())
                        .ok_or(format_err!("ResCache::save_to_cache() => Character id"))?;
                }
            }
        }
        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_res_cache_from_yaml() {
        let cache = ResCache::from_yaml("../test_files/resource.yml").unwrap();
    }

    #[test]
    fn test_from_json() {}
}
