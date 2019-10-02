use std::fmt;
use std::num::TryFromIntError;

#[derive(Debug)]
pub enum ConversionError {
    IntConversionError(TryFromIntError),
    SourceTooLarge,
    SourceTooSmall,
}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConversionError::IntConversionError(e) => write!(f, "Cannot convert from usize: {}", e),
            ConversionError::SourceTooLarge => {
                write!(f, "Cannot convert, because source is too large.")
            }
            ConversionError::SourceTooSmall => write!(f, "Cannot convert, source is too small."),
        }
    }
}

impl std::error::Error for ConversionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConversionError::IntConversionError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<TryFromIntError> for ConversionError {
    fn from(input: TryFromIntError) -> Self {
        ConversionError::IntConversionError(input)
    }
}
