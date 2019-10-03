use std::convert::TryFrom;
use std::io;
use std::marker::Sized;

use crate::packing::error::{PackError, UnpackError};
use crate::packing::ll::{
    BoltRead, BoltReadable, BoltWrite, BoltWriteable, MarkerByte, ValueMap, WResult,
};
use crate::packing::{Packable, Unpackable, ValueList};

pub trait SizedUnpackableAs<U>
where
    Self: Sized,
{
    fn sized_body_from<T: BoltRead>(buf: &mut T) -> Result<Self, UnpackError>;
}

impl<U> SizedUnpackableAs<U> for String
where
    U: BoltReadable<Error = io::Error>,
    usize: TryFrom<U>,
{
    fn sized_body_from<T: BoltRead>(buf: &mut T) -> Result<Self, UnpackError> {
        let size = <usize>::try_from(<U>::bolt_read_from(buf)?)
            .map_err(|_| UnpackError::SizeConversionError)?;

        let s: String = buf.bolt_read_exact(size as u64)?;
        Ok(s)
    }
}

impl<U, I> SizedUnpackableAs<U> for ValueList<I>
where
    U: BoltReadable<Error = io::Error>,
    usize: TryFrom<U>,
    I: Unpackable,
{
    fn sized_body_from<T: BoltRead>(buf: &mut T) -> Result<Self, UnpackError> {
        let size = <usize>::try_from(<U>::bolt_read_from(buf)?)
            .map_err(|_| UnpackError::SizeConversionError)?;
        ValueList::unpack_body(size, buf)
    }
}

impl<U, I> SizedUnpackableAs<U> for ValueMap<I>
where
    U: BoltReadable<Error = io::Error>,
    usize: TryFrom<U>,
    I: Unpackable,
{
    fn sized_body_from<T: BoltRead>(buf: &mut T) -> Result<Self, UnpackError> {
        let size = <usize>::try_from(<U>::bolt_read_from(buf)?)
            .map_err(|_| UnpackError::SizeConversionError)?;
        ValueMap::unpack_body(size, buf)
    }
}

pub trait SizedPackableAs<U> {
    fn as_sized_to<T: BoltWrite>(&self, marker_byte: MarkerByte, buf: &mut T)
        -> WResult<PackError>;
}

pub fn sized_header_to<U: TryFrom<usize> + BoltWriteable<Error = io::Error>, T: BoltWrite>(
    marker_byte: MarkerByte,
    size: usize,
    generic_name: &'static str,
    buf: &mut T,
) -> WResult<PackError> {
    let size: U =
        <U>::try_from(size).map_err(|_| PackError::GenericSizeConversionError(generic_name))?;
    Ok(marker_byte.bolt_write_to(buf)? + size.bolt_write_to(buf)?)
}

impl<U> SizedPackableAs<U> for String
where
    U: BoltWriteable<Error = io::Error> + TryFrom<usize>,
{
    fn as_sized_to<T: BoltWrite>(
        &self,
        marker_byte: MarkerByte,
        buf: &mut T,
    ) -> Result<usize, PackError> {
        Ok(
            sized_header_to::<U, T>(marker_byte, self.len(), "String", buf)?
                + buf.bolt_write(self)?,
        )
    }
}

impl<I: Packable, U> SizedPackableAs<U> for ValueList<I>
where
    U: BoltWriteable<Error = io::Error> + TryFrom<usize>,
{
    fn as_sized_to<T: BoltWrite>(
        &self,
        marker_byte: MarkerByte,
        buf: &mut T,
    ) -> Result<usize, PackError> {
        let mut written = sized_header_to::<U, T>(marker_byte, self.len(), "ValueList", buf)?;

        for v in &self.0 {
            written += v.pack_to(buf)?;
        }
        Ok(written)
    }
}

impl<I: Packable, U> SizedPackableAs<U> for ValueMap<I>
where
    U: BoltWriteable<Error = io::Error> + TryFrom<usize>,
{
    fn as_sized_to<T: BoltWrite>(
        &self,
        marker_byte: MarkerByte,
        buf: &mut T,
    ) -> Result<usize, PackError> {
        let mut written = sized_header_to::<U, T>(marker_byte, self.len(), "ValueMap", buf)?;

        for (k, v) in &self.0 {
            written += k.pack_to(buf)? + v.pack_to(buf)?;
        }
        Ok(written)
    }
}

#[cfg(test)]
mod test {}
