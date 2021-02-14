use anyhow::{anyhow, Result};
use thiserror::Error;

pub fn try_option<T, F>(f: F) -> Option<T>
where
    F: FnOnce() -> Option<T>,
{
    return f();
}

pub fn try_result<T, F>(f: F) -> Result<T>
where
    F: FnOnce() -> Result<T>,
{
    return f();
}

pub trait OptionEx<T> {
    fn to_result(self) -> Result<T>;
}

impl<T> OptionEx<T> for Option<T> {
    fn to_result(self) -> Result<T> {
        return match self {
            Some(val) => Ok(val),
            None => Err(anyhow!("Option None")),
        };
    }
}

#[derive(Error, Debug)]
pub enum CPError {
    #[error("Unknown error")]
    Unknown,
    #[error("Physics object not found")]
    PhysicsObjectNotFound,
}

pub type CPResult<T> = anyhow::Result<T, CPError>;
