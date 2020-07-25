use failure::{AsFail, Fail, Error};
use gdnative::prelude::user_data::LocalCellError;
use std::convert::From;

impl AsFail for LocalCellError {
}
