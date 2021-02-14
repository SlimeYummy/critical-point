use anyhow::{anyhow, Result};
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use serde_json;
use serde_yaml;
use std::fs::File;
use std::path::Path;

pub fn serialize<P, T>(path: P, value: &T) -> Result<()>
where
    P: AsRef<Path>,
    T: Serialize,
{
    let file = File::open(path.as_ref())?;
    if path.as_ref().ends_with(".yaml") || path.as_ref().ends_with(".yml") {
        return Ok(serde_yaml::to_writer(file, value)?);
    } else if path.as_ref().ends_with(".json") {
        return Ok(serde_json::to_writer(file, value)?);
    } else {
        return Err(anyhow!("Unknown format ({:?})", path.as_ref()));
    }
}

pub fn deserialize<P, T>(path: P) -> Result<T>
where
    P: AsRef<Path>,
    T: DeserializeOwned,
{
    let file = File::open(path.as_ref())?;
    if path.as_ref().ends_with(".yaml") || path.as_ref().ends_with(".yml") {
        return Ok(serde_yaml::from_reader(file)?);
    } else if path.as_ref().ends_with(".json") {
        return Ok(serde_json::from_reader(file)?);
    } else {
        return Err(anyhow!("Unknown format ({:?})", path.as_ref()));
    }
}
