use std::{fmt, io};

#[derive(Debug)]
pub enum PackError {
    WriteIOError(io::Error),
}

impl fmt::Display for PackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PackError::WriteIOError(e) => write!(f, "IO Error while writing: {}", e),
        }
    }
}

impl std::error::Error for PackError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PackError::WriteIOError(e) => Some(e),
        }
    }
}

impl From<io::Error> for PackError {
    fn from(input: io::Error) -> PackError {
        PackError::WriteIOError(input)
    }
}
