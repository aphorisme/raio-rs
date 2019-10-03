use std::fmt;

use crate::packing::ll::{MarkerByte, UnknownMarkerError};

#[derive(Debug)]
pub enum BoltReadMarkerError {
    MarkerParseError(UnknownMarkerError),
    ReadIOError(std::io::Error),
    UnexpectedMarker(MarkerByte, MarkerByte),
}

impl fmt::Display for BoltReadMarkerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BoltReadMarkerError::MarkerParseError(e) => write!(f, "Marker parsing error: {}", e),
            BoltReadMarkerError::ReadIOError(e) => write!(f, "IO reading error: {}", e),
            BoltReadMarkerError::UnexpectedMarker(exp, act) => write!(
                f,
                "Unexpected marker read. Expected: {:?}, read: {:?}",
                exp, act
            ),
        }
    }
}

impl std::error::Error for BoltReadMarkerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            BoltReadMarkerError::MarkerParseError(e) => Some(e),
            BoltReadMarkerError::ReadIOError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for BoltReadMarkerError {
    fn from(input: std::io::Error) -> BoltReadMarkerError {
        BoltReadMarkerError::ReadIOError(input)
    }
}

impl From<UnknownMarkerError> for BoltReadMarkerError {
    fn from(input: UnknownMarkerError) -> BoltReadMarkerError {
        BoltReadMarkerError::MarkerParseError(input)
    }
}
