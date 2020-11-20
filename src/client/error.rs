use crate::connectivity::connection::ConnectionError;
use deadpool::managed::PoolError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("IO error: {0}")]
    IOError(#[from] async_std::io::Error),
    #[error("Connection error: {0}")]
    ConnectionError(#[from] ConnectionError),
    #[error("No field information in query response")]
    NoFieldInformation,
    #[error("No qid information in query response")]
    NoQidInformation,
    #[error("Connection pool timed out")]
    PoolTimeOut,
    #[error("The number of fields does not match the number of result columns.")]
    FieldsToRecordMismatch,
    #[error("Cannot extract bookmark from commit")]
    NoBookmarkInformationInCommit,
    #[error("Stream still open after PULL all from last.")]
    StreamStillOpen,
}

impl From<PoolError<ConnectionError>> for ClientError {
    fn from(e: PoolError<ConnectionError>) -> Self {
        match e {
            PoolError::Backend(err) => ClientError::ConnectionError(err),
            PoolError::Timeout(_) => ClientError::PoolTimeOut,
        }
    }
}
