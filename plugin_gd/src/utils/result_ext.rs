use anyhow::{anyhow, Error};
use gdnative::prelude::user_data::LocalCellError;

pub trait ResultExt<T> {
    fn cast_err(self) -> Result<T, Error>;
}

impl<T> ResultExt<T> for Result<T, LocalCellError> {
    fn cast_err(self) -> Result<T, Error> {
        return match self {
            Ok(val) => Ok(val),
            Err(err) => match err {
                LocalCellError::DifferentThread { original, current } => Err(anyhow!(
                    "LocalCellError::BorrowFailed({:?}, {:?})",
                    original,
                    current
                )),
                LocalCellError::BorrowFailed => Err(anyhow!("LocalCellError::BorrowFailed")),
            },
        };
    }
}
