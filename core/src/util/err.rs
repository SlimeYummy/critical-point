use failure::{format_err, Error};

pub fn make_err<T>(msg: &str) -> Result<T, Error> {
    return Err(format_err!("{}", msg));
}

pub fn try_err<T, F>(f: F) -> Result<T, Error>
where
    F: FnOnce() -> Result<T, Error>,
{
    return f();
}
