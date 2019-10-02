use std::convert::{Into, TryFrom};
use std::io::{Read, Write};
use std::ops::Deref;

use crate::packing::error::ConversionError;
use crate::packing::ll::TinySizeMarker;
use crate::packing::{pack::*, unpack::*};

pub const MAX_PLUS_TINY_INT: u8 = 0x7F;
pub const MIN_MINUS_TINY_INT: i8 = -16;

// --------------------------
// PLUSTINYINT
// --------------------------
/// The `PlusTinyInt` type covers integers from `0` to `127` and consists
/// in byte representation just of the number. Use the `From` and `TryFrom`
/// implementations to convert to and from the type.
/// ```
/// use raio::packing::types::*;
/// use std::convert::TryFrom;
///
/// assert_eq!(<u8>::from(<PlusTinyInt>::try_from(127).unwrap()), 127);
///
/// // upper bound + 1:
/// assert!(<PlusTinyInt>::try_from(128).is_err());
/// // lower bound:
/// assert!(<PlusTinyInt>::try_from(0).is_ok());
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PlusTinyInt(u8);

impl TryFrom<u8> for PlusTinyInt {
    type Error = ConversionError;
    fn try_from(input: u8) -> Result<Self, Self::Error> {
        if input <= MAX_PLUS_TINY_INT {
            Ok(PlusTinyInt(input))
        } else {
            Err(ConversionError::SourceTooLarge)
        }
    }
}

impl TryFrom<i64> for PlusTinyInt {
    type Error = ConversionError;
    fn try_from(input: i64) -> Result<Self, Self::Error> {
        if input <= MAX_PLUS_TINY_INT.into() {
            if input >= 0 {
                Ok(PlusTinyInt(input as u8))
            } else {
                Err(ConversionError::SourceTooSmall)
            }
        } else {
            Err(ConversionError::SourceTooLarge)
        }
    }
}

impl Deref for PlusTinyInt {
    type Target = u8;
    fn deref(&self) -> &u8 {
        &self.0
    }
}

// -------------------------
// MINUSTINYINT
// -------------------------
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// The `MinusTinyInt` type handles integers from `-16`to `-1`. The
/// number itself is packed into the low nibble of one byte, where the
/// high nibble is the marker. Use the `TryFrom` and `From` implementations
/// to convert into and from the type.
/// ```
/// use raio::packing::types::*;
/// use std::convert::TryFrom;
///
/// assert_eq!(<i8>::from(<MinusTinyInt>::try_from(-16).unwrap()), -16);
///
/// // limit is -16:
/// assert!(<MinusTinyInt>::try_from(-17).is_err());
/// // and only negatives:
/// assert!(<MinusTinyInt>::try_from(0).is_err());
/// ```
///
/// The internal representation is a `u8` which stores the low nibble part
/// of the bolt encoded byte for `MinusTinyInt`, which is any byte from `0x00`
/// to `0x0F` standing for the integers `-1` to `-16`.
pub struct MinusTinyInt(u8);

impl From<MinusTinyInt> for i8 {
    fn from(input: MinusTinyInt) -> i8 {
        let MinusTinyInt(u) = input;
        -(<i8 as TryFrom<u8>>::try_from(u).unwrap() + 1)
    }
}

impl TryFrom<i8> for MinusTinyInt {
    type Error = ConversionError;
    fn try_from(input: i8) -> Result<Self, Self::Error> {
        if input >= MIN_MINUS_TINY_INT {
            if input < 0 {
                Ok(MinusTinyInt(((-1) * input - 1) as u8))
            } else {
                Err(ConversionError::SourceTooLarge)
            }
        } else {
            Err(ConversionError::SourceTooSmall)
        }
    }
}

impl TryFrom<i64> for MinusTinyInt {
    type Error = ConversionError;
    fn try_from(input: i64) -> Result<Self, Self::Error> {
        if input >= MIN_MINUS_TINY_INT.into() {
            if input < 0 {
                Ok(MinusTinyInt(((-1) * input - 1) as u8))
            } else {
                Err(ConversionError::SourceTooLarge)
            }
        } else {
            Err(ConversionError::SourceTooSmall)
        }
    }
}

impl From<MinusTinyInt> for i64 {
    fn from(input: MinusTinyInt) -> i64 {
        i64::from(i8::from(input))
    }
}
