use std::{fmt, io};

#[derive(Debug)]
pub enum PackError {
    WriteIOError(io::Error),
    GenericTooLarge(&'static str),
    GenericSizeConversionError(&'static str),
}

impl fmt::Display for PackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PackError::WriteIOError(e) => write!(f, "IO Error while writing: {}", e),
            PackError::GenericTooLarge(s) => write!(f, "Generic '{}' too large", s),
            PackError::GenericSizeConversionError(s) => {
                write!(f, "Error while converting size of generic '{}'", s)
            }
        }
    }
}

impl std::error::Error for PackError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PackError::WriteIOError(e) => Some(e),
            PackError::GenericTooLarge(_) => None,
            PackError::GenericSizeConversionError(_) => None,
        }
    }
}

impl From<io::Error> for PackError {
    fn from(input: io::Error) -> PackError {
        PackError::WriteIOError(input)
    }
}
