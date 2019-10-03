use std::convert::TryFrom;

use crate::packing::error::{PackError, UnpackError};
use crate::packing::ll::{
    BoltRead, BoltWrite, BoltWriteable, MarkerByte, TinySizeMarker, ValueMap, WResult,
};
use crate::packing::{Packable, Unpackable, ValueList};

pub const TINY_SIZE_MAX: u8 = 0x0F;

pub trait TinySizedPackableAs {
    fn as_tiny_sized_to<T: BoltWrite>(
        &self,
        marker_byte: MarkerByte,
        buf: &mut T,
    ) -> WResult<PackError>;
}

impl TinySizedPackableAs for String {
    fn as_tiny_sized_to<T: BoltWrite>(
        &self,
        marker_byte: MarkerByte,
        buf: &mut T,
    ) -> Result<usize, PackError> {
        let size: u8 =
            <u8>::try_from(self.len()).map_err(|_| PackError::GenericTooLarge("String"))?;
        if size <= TINY_SIZE_MAX {
            let written = TinySizeMarker::new(marker_byte, size).bolt_write_to(buf)?
                + self.bolt_write_to(buf)?;
            Ok(written)
        } else {
            Err(PackError::GenericTooLarge("String"))
        }
    }
}

impl<V: Packable> TinySizedPackableAs for ValueList<V> {
    fn as_tiny_sized_to<T: BoltWrite>(
        &self,
        marker_byte: MarkerByte,
        buf: &mut T,
    ) -> Result<usize, PackError> {
        let size: u8 =
            <u8>::try_from(self.len()).map_err(|_| PackError::GenericTooLarge("ValueList"))?;
        if size <= TINY_SIZE_MAX {
            let mut written = TinySizeMarker::new(marker_byte, size).bolt_write_to(buf)?;

            for v in &self.0 {
                written += v.pack_to(buf)?;
            }

            Ok(written)
        } else {
            Err(PackError::GenericTooLarge("ValueList"))
        }
    }
}

impl<V: Packable> TinySizedPackableAs for ValueMap<V> {
    fn as_tiny_sized_to<T: BoltWrite>(
        &self,
        marker_byte: MarkerByte,
        buf: &mut T,
    ) -> Result<usize, PackError> {
        let size: u8 =
            <u8>::try_from(self.len()).map_err(|_| PackError::GenericTooLarge("ValueMap"))?;
        if size <= TINY_SIZE_MAX {
            let mut written = TinySizeMarker::new(marker_byte, size).bolt_write_to(buf)?;

            for (k, v) in &self.0 {
                written += k.pack_to(buf)?;
                written += v.pack_to(buf)?;
            }

            Ok(written)
        } else {
            Err(PackError::GenericTooLarge("ValueMap"))
        }
    }
}

pub trait TinySizedUnpackableAs
where
    Self: Sized,
{
    fn tiny_sized_body_from<T: BoltRead>(size: u8, buf: &mut T) -> Result<Self, UnpackError>;
}

impl TinySizedUnpackableAs for String {
    fn tiny_sized_body_from<T: BoltRead>(size: u8, buf: &mut T) -> Result<Self, UnpackError> {
        Ok(buf.bolt_read_exact(size as u64)?)
    }
}

impl<I: Unpackable> TinySizedUnpackableAs for ValueList<I> {
    fn tiny_sized_body_from<T: BoltRead>(size: u8, buf: &mut T) -> Result<Self, UnpackError> {
        ValueList::unpack_body(size as usize, buf)
    }
}

impl<I: Unpackable> TinySizedUnpackableAs for ValueMap<I> {
    fn tiny_sized_body_from<T: BoltRead>(size: u8, buf: &mut T) -> Result<Self, UnpackError> {
        ValueMap::unpack_body(size as usize, buf)
    }
}
