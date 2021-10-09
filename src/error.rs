use serde::ser;
use std::{error, fmt, io};

#[derive(Debug)]
enum ErrorKind {
    Io(io::Error),
    Custom(String),
    KeyMustBeAString,
}

#[derive(Debug)]
pub struct Error(Box<ErrorKind>);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &*self.0 {
            ErrorKind::Io(_) => f.write_str("IO error"),
            ErrorKind::Custom(e) => f.write_str(e),
            ErrorKind::KeyMustBeAString => f.write_str("key must be a string"),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match &*self.0 {
            ErrorKind::Io(e) => Some(e),
            ErrorKind::Custom(_) => None,
            ErrorKind::KeyMustBeAString => None,
        }
    }
}

impl ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Error(Box::new(ErrorKind::Custom(msg.to_string())))
    }
}

impl Error {
    pub(crate) fn io(e: io::Error) -> Self {
        Error(Box::new(ErrorKind::Io(e)))
    }

    pub(crate) fn key_must_be_a_string() -> Self {
        Error(Box::new(ErrorKind::KeyMustBeAString))
    }
}
