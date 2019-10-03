use std::convert::{Into, TryFrom};
use std::ops::Deref;

use byteorder::ReadBytesExt;

use crate::packing::error::{BoltReadMarkerError, ConversionError, PackError, UnpackError};
use crate::packing::ll::{
    read_expected_marker, BoltRead, BoltWrite, BoltWriteable, MarkerByte, TinySizeMarker,
    UnknownMarkerError,
};
use crate::packing::{Packable, Unpackable};

pub const MAX_PLUS_TINY_INT: u8 = 0x7F;
pub const MIN_MINUS_TINY_INT: i8 = -16;

// --------------------------
// PLUSTINYINT
// --------------------------

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// The `PlusTinyInt` type covers integers from `0` to `127` and consists
/// in byte representation just of the number. Use the `From` and `TryFrom`
/// implementations to convert from and to the type.
/// ```
/// use raio::packing::*;
/// use std::convert::TryFrom;
///
/// assert_eq!(<u8>::from(<PlusTinyInt>::try_from(127).unwrap()), 127);
///
/// // upper bound + 1:
/// assert!(<PlusTinyInt>::try_from(128).is_err());
/// // lower bound:
/// assert!(<PlusTinyInt>::try_from(0).is_ok());
/// ```
pub struct PlusTinyInt(u8);

impl PlusTinyInt {
    /// # Safety
    /// Creating a new `PlusTinyInt` is safe whenever using the `TryFrom` trait. Using `new_unchecked`
    /// is a shortcut which wraps a `u8` just into a `PlusTinyInt`. This is a safe operation for all
    /// `u: u8` with `u <= MAX_PLUS_TINY_INT`.
    pub unsafe fn new_unchecked(from: u8) -> PlusTinyInt {
        PlusTinyInt(from)
    }
}

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

impl TryFrom<i32> for PlusTinyInt {
    type Error = ConversionError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Self::try_from(i64::from(value))
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

impl From<PlusTinyInt> for u8 {
    fn from(input: PlusTinyInt) -> u8 {
        input.0
    }
}

impl Deref for PlusTinyInt {
    type Target = u8;
    fn deref(&self) -> &u8 {
        &self.0
    }
}

impl Packable for PlusTinyInt {
    fn pack_to<T: BoltWrite>(&self, buf: &mut T) -> Result<usize, PackError> {
        Ok(self.0.bolt_write_to(buf)?)
    }
}

impl Unpackable for PlusTinyInt {
    fn unpack_from<T: BoltRead>(buf: &mut T) -> Result<Self, UnpackError> {
        let u = buf.read_u8()?;
        PlusTinyInt::try_from(u).map_err(|_| {
            UnpackError::MarkerReadError(BoltReadMarkerError::MarkerParseError(
                UnknownMarkerError { read_byte: u },
            ))
        })
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
/// use raio::packing::*;
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

impl MinusTinyInt {
    /// # Safety
    /// Using the `TryFrom` trait to create a `MinusTinyInt` is safe. Using `new_unchecked` should
    /// be used carefully. What needs to happen is a mapping where `-1` is mapped to `0x00: u8`
    /// and from there up to `-16` is mapped to `0x0F: u8`. So, whenever the input `i` has `-17 < i < 0`
    /// then `((-i) - 1) as u8` is the correct value for `new_unchecked`.
    pub unsafe fn new_unchecked(from: u8) -> MinusTinyInt {
        MinusTinyInt(from)
    }
}

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
                Ok(MinusTinyInt((-input - 1) as u8))
            } else {
                Err(ConversionError::SourceTooLarge)
            }
        } else {
            Err(ConversionError::SourceTooSmall)
        }
    }
}

impl TryFrom<i32> for MinusTinyInt {
    type Error = ConversionError;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Self::try_from(i64::from(value))
    }
}

impl TryFrom<i64> for MinusTinyInt {
    type Error = ConversionError;
    fn try_from(input: i64) -> Result<Self, Self::Error> {
        if input >= MIN_MINUS_TINY_INT.into() {
            if input < 0 {
                Ok(MinusTinyInt((-input - 1) as u8))
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

impl Packable for MinusTinyInt {
    fn pack_to<T: BoltWrite>(&self, buf: &mut T) -> Result<usize, PackError> {
        Ok(TinySizeMarker {
            marker: MarkerByte::MinusTinyInt,
            tiny_size: self.0,
        }
        .bolt_write_to(buf)?)
    }
}

impl Unpackable for MinusTinyInt {
    fn unpack_from<T: BoltRead>(buf: &mut T) -> Result<Self, UnpackError> {
        let m: TinySizeMarker = read_expected_marker(MarkerByte::MinusTinyInt, buf)?;
        Ok(MinusTinyInt(m.tiny_size))
    }
}
