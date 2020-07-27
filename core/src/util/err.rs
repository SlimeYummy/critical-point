use failure::{format_err, Error};

pub fn make_err<T>(msg: &str) -> Result<T, Error> {
    return Err(format_err!("{}", msg));
}

pub fn try_option<T, F>(f: F) -> Option<T>
where
    F: FnOnce() -> Option<T>,
{
    return f();
}

pub fn try_result<T, F>(f: F) -> Result<T, Error>
where
    F: FnOnce() -> Result<T, Error>,
{
    return f();
}

pub trait OptionEx<T> {
    fn to_result(self) -> Result<T, Error>;
}

impl<T> OptionEx<T> for Option<T> {
    fn to_result(self) -> Result<T, Error> {
        return match self {
            Some(val) => Ok(val),
            None => Err(format_err!("Option None")),
        };
    }
}
