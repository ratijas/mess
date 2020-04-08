use imports::*;

use std::sync::mpsc::{SendError, RecvError};


pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Channel,
    Client(ClientError),
    Io(io::Error),
    Reason(String),
    Data(::algos::types::data::Error),
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

impl From<ClientError> for Error {
    fn from(e: ClientError) -> Self {
        match e {
            ClientError::IoError(io) => io.into(),
            _ => Error::Client(e),
        }
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

impl From<::algos::types::data::Error> for Error {
    fn from(e: ::algos::types::data::Error) -> Self {
        Error::Data(e)
    }
}