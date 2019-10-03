use crate::net::{QueryResultError, Response};
use crate::packing::{MessageReadError, MessageWriteError};
use std::fmt::Formatter;
use std::{fmt, io};

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    UnsupportedServerVersion(u32),
    MessageWriteError(MessageWriteError),
    MessageReadError(MessageReadError),
    UnexpectedResponse(Response, &'static str),
    MissingMetaData(&'static str),
    QueryResultError(QueryResultError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Error::IOError(e) => write!(f, "IO error: {}", e),
            Error::UnsupportedServerVersion(v) => write!(
                f,
                "Unsupported server version. Server answered with version: {}",
                v
            ),
            Error::MessageWriteError(e) => write!(f, "Message write error: {}", e),
            Error::MessageReadError(e) => write!(f, "Message read error: {}", e),
            Error::UnexpectedResponse(r, expected) => {
                write!(f, "Expected {} as response but received {:?}", expected, r)
            }
            Error::MissingMetaData(field) => write!(f, "Missing meta data field '{}", field),
            Error::QueryResultError(err) => write!(f, "Error while building query result: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::IOError(e) => Some(e),
            Error::MessageWriteError(e) => Some(e),
            Error::MessageReadError(e) => Some(e),
            Error::QueryResultError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(input: io::Error) -> Self {
        Error::IOError(input)
    }
}

impl From<MessageWriteError> for Error {
    fn from(input: MessageWriteError) -> Self {
        Error::MessageWriteError(input)
    }
}

impl From<MessageReadError> for Error {
    fn from(input: MessageReadError) -> Self {
        Error::MessageReadError(input)
    }
}

impl From<QueryResultError> for Error {
    fn from(e: QueryResultError) -> Self {
        Error::QueryResultError(e)
    }
}
