use std::convert::TryFrom;

use byteorder::ReadBytesExt;

use crate::packing::error::{BoltReadMarkerError, UnpackError};
use crate::packing::ll::{BoltRead, BoltReadable, MarkerByte, TinySizeMarker, ValueMap};
use crate::packing::types::*;
use crate::packing::{MinusTinyInt, ValueList};

pub trait Unpackable
where
    Self: std::marker::Sized,
{
    fn unpack_from<T: BoltRead>(buf: &mut T) -> Result<Self, UnpackError>;
}

pub trait Unpack: BoltRead {
    fn unpack<T: Unpackable>(&mut self) -> Result<T, UnpackError> {
        <T>::unpack_from(self)
    }
}

// ------------------------
// GENERIC IMPLEMENTATIONS:
// ------------------------

impl Unpackable for i64 {
    fn unpack_from<T: BoltRead>(buf: &mut T) -> Result<Self, UnpackError> {
        // read marker:
        let u = buf.read_u8()?;
        let m: MarkerByte =
            MarkerByte::try_from(u).map_err(BoltReadMarkerError::MarkerParseError)?;

        match m {
            MarkerByte::MinusTinyInt => {
                let mty = unsafe { MinusTinyInt::new_unchecked(u) };
                Ok(i64::from(mty))
            }
            MarkerByte::PlusTinyInt => Ok(i64::from(u)),
            MarkerByte::Int8 => Ok(<i8>::fixed_body_from(buf)? as i64),
            MarkerByte::Int16 => Ok(<i16>::fixed_body_from(buf)? as i64),
            MarkerByte::Int32 => Ok(<i32>::fixed_body_from(buf)? as i64),
            MarkerByte::Int64 => Ok(<i64>::fixed_body_from(buf)? as i64),
            _ => Err(UnpackError::UnexpectedMarker(m, "i64")),
        }
    }
}

impl Unpackable for String {
    fn unpack_from<T: BoltRead>(buf: &mut T) -> Result<Self, UnpackError> {
        let m: TinySizeMarker = buf.bolt_read()?;

        match m.marker {
            MarkerByte::TinyString => <String>::tiny_sized_body_from(m.tiny_size, buf),
            MarkerByte::String8 => <String as SizedUnpackableAs<u8>>::sized_body_from(buf),
            MarkerByte::String16 => <String as SizedUnpackableAs<u16>>::sized_body_from(buf),
            MarkerByte::String32 => <String as SizedUnpackableAs<u32>>::sized_body_from(buf),
            _ => Err(UnpackError::UnexpectedMarker(m.marker, "String")),
        }
    }
}

impl Unpackable for f64 {
    fn unpack_from<T: BoltRead>(buf: &mut T) -> Result<Self, UnpackError> {
        Ok(Float64::unpack_from(buf)?.0)
    }
}

impl Unpackable for bool {
    fn unpack_from<T: BoltRead>(buf: &mut T) -> Result<Self, UnpackError> {
        let m = MarkerByte::bolt_read_from(buf)?;
        match m {
            MarkerByte::BoolFalse => Ok(false),
            MarkerByte::BoolTrue => Ok(true),
            _ => Err(UnpackError::UnexpectedMarker(m, "bool")),
        }
    }
}

impl<I: Unpackable> Unpackable for ValueList<I> {
    fn unpack_from<T: BoltRead>(buf: &mut T) -> Result<Self, UnpackError> {
        let m = TinySizeMarker::bolt_read_from(buf)?;

        match m.marker {
            MarkerByte::TinyList => <ValueList<I>>::tiny_sized_body_from(m.tiny_size, buf),
            MarkerByte::List8 => Ok(<ValueList<I> as SizedUnpackableAs<u8>>::sized_body_from(
                buf,
            )?),
            MarkerByte::List16 => Ok(<ValueList<I> as SizedUnpackableAs<u16>>::sized_body_from(
                buf,
            )?),
            MarkerByte::List32 => Ok(<ValueList<I> as SizedUnpackableAs<u32>>::sized_body_from(
                buf,
            )?),
            _ => Err(UnpackError::UnexpectedMarker(m.marker, "List")),
        }
    }
}

impl<I: Unpackable> Unpackable for ValueMap<I> {
    fn unpack_from<T: BoltRead>(buf: &mut T) -> Result<Self, UnpackError> {
        let m = TinySizeMarker::bolt_read_from(buf)?;

        match m.marker {
            MarkerByte::TinyMap => <ValueMap<I>>::tiny_sized_body_from(m.tiny_size, buf),
            MarkerByte::Map8 => Ok(<ValueMap<I> as SizedUnpackableAs<u8>>::sized_body_from(
                buf,
            )?),
            MarkerByte::Map16 => Ok(<ValueMap<I> as SizedUnpackableAs<u16>>::sized_body_from(
                buf,
            )?),
            MarkerByte::Map32 => Ok(<ValueMap<I> as SizedUnpackableAs<u32>>::sized_body_from(
                buf,
            )?),
            _ => Err(UnpackError::UnexpectedMarker(m.marker, "List")),
        }
    }
}
