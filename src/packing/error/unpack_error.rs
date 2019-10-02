use crate::packing::error::{BoltReadMarkerError, BoltReadSignatureError};
use std::{fmt, io};

#[derive(Debug)]
pub enum UnpackError {
    ReadIOError(io::Error),
    MarkerReadError(BoltReadMarkerError),
    SignatureReadError(BoltReadSignatureError),
    UnexpectedSignatureSize(u8, u8),
}

impl fmt::Display for UnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnpackError::ReadIOError(e) => write!(f, "IO error while reading: {}", e),
            UnpackError::MarkerReadError(e) => write!(f, "Marker read error: {}", e),
            UnpackError::SignatureReadError(e) => write!(f, "Signature read error: {}", e),
            UnpackError::UnexpectedSignatureSize(exp, act) => write!(
                f,
                "Unexpected signature size. Expected: {}, actual: {}",
                exp, act
            ),
        }
    }
}

impl std::error::Error for UnpackError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            UnpackError::ReadIOError(e) => Some(e),
            UnpackError::MarkerReadError(e) => Some(e),
            UnpackError::SignatureReadError(e) => Some(e),
            UnpackError::UnexpectedSignatureSize(_, _) => None,
        }
    }
}

impl From<io::Error> for UnpackError {
    fn from(input: io::Error) -> UnpackError {
        UnpackError::ReadIOError(input)
    }
}

impl From<BoltReadMarkerError> for UnpackError {
    fn from(input: BoltReadMarkerError) -> UnpackError {
        UnpackError::MarkerReadError(input)
    }
}

impl From<BoltReadSignatureError> for UnpackError {
    fn from(input: BoltReadSignatureError) -> UnpackError {
        UnpackError::SignatureReadError(input)
    }
}
