use std::convert::TryFrom;
use std::fmt;
use std::io::{Read, Write};

use crate::packing::error::BoltReadMarkerError;
use crate::packing::ll::{combine_nibble, high_nibble, low_nibble, BoltReadable, BoltWriteable};

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
/// Type to have all marker bytes in one place. This is represented
/// as `u8` and can be used via `MarkerByte::TinyString as u8`. Converting from
/// `u8` to `MarkerByte` might fail, since not every possible value of `u8` corresponds
/// to a `MarkerByte`, but a `TryFrom<u8>` implementation is given.
pub enum MarkerByte {
    // tiny:
    PlusTinyInt = 0x00,  // 1 to 127 (up to 0x7F)
    MinusTinyInt = 0xF0, // -1 to -16
    TinyString = 0x80,
    TinyList = 0x90,
    TinyMap = 0xA0,
    TinyStruct = 0xB0,

    // primitives:
    Null = 0xC0,
    BoolFalse = 0xC2,
    BoolTrue = 0xC3,

    // numbers:
    Float64 = 0xC1,
    Int8 = 0xC8,
    Int16 = 0xC9,
    Int32 = 0xCA,
    Int64 = 0xCB,

    // strings:
    String8 = 0xD0,
    String16 = 0xD1,
    String32 = 0xD2,

    // lists:
    List8 = 0xD4,
    List16 = 0xD5,
    List32 = 0xD6,

    // maps:
    Map8 = 0xD8,
    Map16 = 0xD9,
    Map32 = 0xDA,

    // structs:
    Struct8 = 0xDC,
    Struct16 = 0xDD,
}

#[derive(Debug)]
/// Error type in case of an unknown marker while
/// converting from a mere `u8`.
pub struct UnknownMarkerError {
    pub read_byte: u8,
}

impl fmt::Display for UnknownMarkerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unknown marker byte {}", self.read_byte)
    }
}

impl std::error::Error for UnknownMarkerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

/// Implementation to convert from a mere `u8` into a valid,
/// known bolt protocol marker. For Tiny Types, this is done
/// by looking at the high nibble of the byte. The special case of
/// `PlusTinyInt` excepts any number `0 <= n <= 0x7F`.
/// ```
/// use raio::packing::ll::*;
/// use std::convert::TryFrom;
///
/// // there is no 0xCE marker:
/// assert!(MarkerByte::try_from(0xCE).is_err()); // there is no 0xCE marker
///
/// // but any number `<= 0x7F` is read as `MarkerByte::PlusTinyInt`:
/// assert_eq!(MarkerByte::try_from(0x7F).unwrap(), MarkerByte::PlusTinyInt);
///
/// // and tiny types are only converted by looking at the high nibble:
/// assert_eq!(MarkerByte::try_from(0x83).unwrap(), MarkerByte::TinyString);
/// ```
impl TryFrom<u8> for MarkerByte {
    type Error = UnknownMarkerError;
    fn try_from(input: u8) -> Result<MarkerByte, Self::Error> {
        // ----- SPECIAL ------
        // Tiny Int Plus is just the number:
        if input <= 0x7F {
            return Ok(MarkerByte::PlusTinyInt);
        }

        // now, look for the exact matches:
        match input {
            0xC0 => Ok(MarkerByte::Null),
            0xC2 => Ok(MarkerByte::BoolFalse),
            0xC3 => Ok(MarkerByte::BoolTrue),

            0xC1 => Ok(MarkerByte::Float64),
            0xC8 => Ok(MarkerByte::Int8),
            0xC9 => Ok(MarkerByte::Int16),
            0xCA => Ok(MarkerByte::Int32),
            0xCB => Ok(MarkerByte::Int64),

            0xD0 => Ok(MarkerByte::String8),
            0xD1 => Ok(MarkerByte::String16),
            0xD2 => Ok(MarkerByte::String32),

            0xD4 => Ok(MarkerByte::List8),
            0xD5 => Ok(MarkerByte::List16),
            0xD6 => Ok(MarkerByte::List32),

            0xD8 => Ok(MarkerByte::Map8),
            0xD9 => Ok(MarkerByte::Map16),
            0xDA => Ok(MarkerByte::Map32),

            0xDC => Ok(MarkerByte::Struct8),
            0xDD => Ok(MarkerByte::Struct16),

            // no exact matches, this still leaves the chance
            // for a high_nibble match (i.e. tiny marker with size)
            _ => from_high_nibble(input),
        }
    }
}

/// Internal function to get the marker by high nibble in case
/// of the tiny types (expect `MarkerByte::PlusTinyInt`).
fn from_high_nibble(input: u8) -> Result<MarkerByte, UnknownMarkerError> {
    // Tiny Int Minus is with higher nibble 0xF
    let high = high_nibble(input);
    match high {
        0x90 => Ok(MarkerByte::TinyList),
        0xA0 => Ok(MarkerByte::TinyMap),
        0x80 => Ok(MarkerByte::TinyString),
        0xB0 => Ok(MarkerByte::TinyStruct),
        0xF0 => Ok(MarkerByte::MinusTinyInt),
        _ => Err(UnknownMarkerError { read_byte: input }),
    }
}

impl BoltWriteable for MarkerByte {
    type Error = std::io::Error;
    fn bolt_write_to<T: Write>(self, buf: &mut T) -> Result<usize, Self::Error> {
        buf.write(&[self as u8])?;
        Ok(1)
    }
}

/// The `BoltRead` implementation for `Marker` uses the `TryFrom<u8>`
/// implementation of it. This means, that for example `[0x8C]` is read
/// as `TinyString` but written again as `[0x80]` and hence `write_bolt_into`
/// is not an left-inverse for `read_bolt` in this case.
/// ```
/// use raio::packing::ll::*;
///
/// // define marker with size:
/// let tiny_string_marker = combine_nibble(MarkerByte::TinyString as u8, 12);
/// let mut c = std::io::Cursor::new(vec![tiny_string_marker]);
///
/// // read that marker:
/// let marker : Marker = MarkerByte::read_bolt(&mut c).unwrap();
/// assert_eq!(marker, MarkerByte::TinyString);
///
/// // now, this written is just `0x80`:
/// let mut v : Vec<u8> = Vec::with_capacity(1);
/// marker.write_bolt_into(&mut v).unwrap();
/// assert_eq!(v[0], 0x80);
/// ```
impl BoltReadable for MarkerByte {
    type Error = BoltReadMarkerError;
    fn bolt_read_from<T: Read>(buf: &mut T) -> Result<Self, Self::Error> {
        let mut b: [u8; 1] = [0; 1];
        buf.read(&mut b)?;
        MarkerByte::try_from(b[0]).map_err(|e| e.into())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// Some header bytes are marker as well as size information in one byte,
/// where the high nibble of the byte stands for the marker and the low nibble
/// is an encoded size information. This type represents these kind of
/// `marker`. Their implementations of `BoltRead` and `BoltWrite` provide a
/// useful alternative.
pub struct TinySizeMarker {
    pub marker: MarkerByte,
    pub tiny_size: u8,
}

impl From<TinySizeMarker> for u8 {
    fn from(input: TinySizeMarker) -> u8 {
        combine_nibble(input.marker as u8, input.tiny_size)
    }
}

impl From<TinySizeMarker> for MarkerByte {
    fn from(input: TinySizeMarker) -> MarkerByte {
        input.marker
    }
}

impl BoltReadable for TinySizeMarker {
    type Error = BoltReadMarkerError;
    /// Reads in exactly one byte, parses the marker and sets the lower nibble
    /// of the byte as its `tiny_size`.
    /// ```
    /// use raio::packing::ll::*;
    /// use std::io::Cursor;
    ///
    /// // write a simple tiny size marker into a vector:
    /// let mut ts_marker_data : Vec<u8> = Vec::with_capacity(1);
    /// (TinySizeMarker { marker: MarkerByte::TinyString , tiny_size : 14 })
    ///          .write_bolt_into(&mut ts_marker_data).unwrap();
    ///
    /// // and read its data as a normal `Marker`:
    /// let mut ts_marker_cursor = Cursor::new(ts_marker_data);
    /// let ts_marker = MarkerByte::read_bolt(&mut ts_marker_cursor).unwrap();
    ///
    /// assert_eq!(MarkerByte::TinyString, ts_marker);
    /// ```
    /// It is guaranteed that `tiny_size` is `<= 15`.
    /// Although it is not guaranteed that the read marker is any `TinyFoo` marker.
    /// ```
    /// use raio::packing::ll::*;
    /// use std::io::Cursor;
    ///
    /// // write `MarkerByte::String8` which is not a `Tiny` marker:
    /// let mut s8_marker_data : Vec<u8> = Vec::with_capacity(1);
    /// MarkerByte::String8.write_bolt_into(&mut s8_marker_data).unwrap();
    ///
    /// // now read this as a `TinySizeMarker`:
    /// let mut s8_marker_cursor = Cursor::new(s8_marker_data);
    /// let s8_tiny : TinySizeMarker = TinySizeMarker::read_bolt(&mut s8_marker_cursor).unwrap();
    ///
    /// // this is valid:
    /// assert_eq!(s8_tiny, TinySizeMarker { marker: MarkerByte::String8, tiny_size : 0 });
    /// ```
    fn bolt_read_from<T: Read>(buf: &mut T) -> Result<Self, Self::Error> {
        let mut b: [u8; 1] = [0; 1];
        buf.read(&mut b)?;
        let m = MarkerByte::try_from(b[0]).map_err(|e| -> BoltReadMarkerError { e.into() })?;
        Ok(TinySizeMarker {
            marker: m,
            tiny_size: low_nibble(b[0]),
        })
    }
}

impl TryFrom<u8> for TinySizeMarker {
    type Error = UnknownMarkerError;
    fn try_from(input: u8) -> Result<Self, Self::Error> {
        let m = MarkerByte::try_from(input)?;
        Ok(TinySizeMarker {
            marker: m,
            tiny_size: low_nibble(input),
        })
    }
}

impl BoltWriteable for TinySizeMarker {
    type Error = std::io::Error;
    fn bolt_write_to<T: Write>(self, buf: &mut T) -> Result<usize, Self::Error> {
        let u = combine_nibble(self.marker as u8, self.tiny_size);
        buf.write(&[u])?;
        Ok(1)
    }
}

/// The `MarkerType` trait unifies functionality across the different
/// marker types like `Marker` and `TinySizeMarker`.
pub trait MarkerType: Into<MarkerByte> {
    fn validates(&self, m: MarkerByte) -> bool;
}

impl MarkerType for MarkerByte {
    fn validates(&self, m: MarkerByte) -> bool {
        m == *self
    }
}

impl MarkerType for TinySizeMarker {
    fn validates(&self, m: MarkerByte) -> bool {
        m == self.marker
    }
}

/// Tries to read a `Marker` out of a `Read` and checks, if it validates the
/// expected.
pub fn read_expected_marker<E, M: MarkerType + BoltReadable<Error = E>, T: Read>(
    expected: MarkerByte,
    buf: &mut T,
) -> Result<M, BoltReadMarkerError>
where
    BoltReadMarkerError: From<E>,
{
    let m: M = <M>::bolt_read_from(buf)?;
    if m.validates(expected) {
        Ok(m)
    } else {
        Err(BoltReadMarkerError::UnexpectedMarker(expected, m.into()))
    }
}
