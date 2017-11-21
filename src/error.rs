use imports::*;

use std::sync::mpsc::{SendError, RecvError};


pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Channel,
    Io(io::Error),
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
