use crate::packing::ll::{BoltReadable, BoltWriteable};
use byteorder::ReadBytesExt;
use std::convert::TryFrom;
use std::fmt;
use std::io::{Read, Write};
use crate::packing::error::BoltReadSignatureError;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signature {
    // value structs:
    Node = 0x4E,
    Relationship = 0x52,
    Path = 0x50,
    UnboundRelationship = 0x72,

    // message structs:
    Init = 0x01,
    Run = 0x10,
    DiscardAll = 0x2F,
    PullAll = 0x3F,
    AckFailure = 0x0E,
    Reset = 0x0F,
    Record = 0x71,
    Success = 0x70,
    Failure = 0x7F,
    Ignored = 0x7E,
}

impl Signature {
    pub fn validates(&self, sig: Signature) -> bool {
        *self == sig
    }
    pub fn read_expected<T: Read>(self, buf: &mut T) -> Result<Signature, BoltReadSignatureError> {
        let sig: Signature = Signature::bolt_read_from(buf)?;
        if sig.validates(self) {
            Ok(sig)
        } else {
            Err(BoltReadSignatureError::UnexpectedSignatureError(sig))
        }
    }
}

#[derive(Debug)]
/// Error type in case of an unknown marker while
/// converting from a mere `u8`.
pub struct UnknownSignatureError {
    pub read_byte: u8,
}

impl fmt::Display for UnknownSignatureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unknown signature byte {}", self.read_byte)
    }
}

impl std::error::Error for UnknownSignatureError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl TryFrom<u8> for Signature {
    type Error = UnknownSignatureError;
    fn try_from(input: u8) -> Result<Signature, Self::Error> {
        match input {
            0x4E => Ok(Signature::Node),
            0x52 => Ok(Signature::Relationship),
            0x50 => Ok(Signature::Path),
            0x72 => Ok(Signature::UnboundRelationship),

            // messages:
            0x01 => Ok(Signature::Init),
            0x10 => Ok(Signature::Run),
            0x2F => Ok(Signature::DiscardAll),
            0x3F => Ok(Signature::PullAll),
            0x0E => Ok(Signature::AckFailure),
            0x0F => Ok(Signature::Reset),
            0x71 => Ok(Signature::Record),
            0x70 => Ok(Signature::Success),
            0x7F => Ok(Signature::Failure),
            0x7E => Ok(Signature::Ignored),

            _ => Err(UnknownSignatureError { read_byte: input }),
        }
    }
}

impl BoltWriteable for Signature {
    type Error = std::io::Error;
    fn bolt_write_to<T: Write>(self, buf: &mut T) -> Result<usize, Self::Error> {
        (self as u8).bolt_write_to(buf)
    }
}

impl BoltReadable for Signature {
    type Error = BoltReadSignatureError;
    fn bolt_read_from<T: Read>(buf: &mut T) -> Result<Self, Self::Error> {
        let b = buf.read_u8()?;
        Signature::try_from(b).map_err(|e| e.into())
    }
}
