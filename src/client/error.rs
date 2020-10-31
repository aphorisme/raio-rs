use crate::ll::connection::ConnectionError;
use deadpool::managed::PoolError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("IO error: {0}")]
    IOError(#[from] async_std::io::Error),
    #[error("Connection error: {0}")]
    ConnectionError(#[from] ConnectionError),
    #[error("Cannot resolve endpoint")]
    CannotResolveEndpoint,
    #[error("No field information in query response")]
    NoFieldInformation,
    #[error("Connection pool timed out")]
    PoolTimeOut,
    #[error("The number of fields does not match the number of result columns.")]
    FieldsToRecordMismatch,
    #[error("Received a failure '{0}': {1}")]
    FailureResponse(String, String),
    #[error("Commit Pull got ignored")]
    CommitPullIgnored,
    #[error("Cannot extract bookmark from auto commit")]
    CannotExtractBookmarkFromAutoCommit,
}

impl From<PoolError<ConnectionError>> for ClientError {
    fn from(e: PoolError<ConnectionError>) -> Self {
        match e {
            PoolError::Backend(err) => ClientError::ConnectionError(err),
            PoolError::Timeout(_) => ClientError::PoolTimeOut,
        }
    }
}

