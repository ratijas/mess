use imports::*;

use std::sync::mpsc::{SendError, RecvError};


pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Channel,
    Io(io::Error),
    Reason(String),
}

impl<T> From<SendError<T>> for Error {
    fn from(_: SendError<T>) -> Self {
        Error::Channel
    }
}

impl From<RecvError> for Error {
    fn from(_: RecvError) -> Self {
        Error::Channel
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<String> for Error {
    fn from(e: String) -> Self {
        Error::Reason(e)
    }
}

impl<'a> From<&'a str> for Error {
    fn from(e: &'a str) -> Self {
        Error::Reason(e.into())
    }
}