use crate::packing::ll::{UnknownSignatureError, Signature};
use std::fmt;

#[derive(Debug)]
pub enum BoltReadSignatureError {
    SignatureParseError(UnknownSignatureError),
    UnexpectedSignatureError(Signature),
    ReadIOError(std::io::Error),
}

impl fmt::Display for BoltReadSignatureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BoltReadSignatureError::SignatureParseError(e) => {
                write!(f, "Signature parsing error: {}", e)
            }
            BoltReadSignatureError::ReadIOError(e) => write!(f, "IO reading error: {}", e),
            BoltReadSignatureError::UnexpectedSignatureError(sig) => {
                write!(f, "Unexpected signature found: {:?}", sig)
            }
        }
    }
}

impl std::error::Error for BoltReadSignatureError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            BoltReadSignatureError::SignatureParseError(e) => Some(e),
            BoltReadSignatureError::ReadIOError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for BoltReadSignatureError {
    fn from(input: std::io::Error) -> BoltReadSignatureError {
        BoltReadSignatureError::ReadIOError(input)
    }
}

impl From<UnknownSignatureError> for BoltReadSignatureError {
    fn from(input: UnknownSignatureError) -> BoltReadSignatureError {
        BoltReadSignatureError::SignatureParseError(input)
    }
}
