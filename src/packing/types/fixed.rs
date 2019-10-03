use std::io;

use crate::packing::error::{PackError, UnpackError};
use crate::packing::ll::{BoltRead, BoltReadable, BoltWrite, BoltWriteable, MarkerByte, WResult};

pub trait FixedPackableAs {
    fn as_fixed_to<T: BoltWrite>(&self, marker_byte: MarkerByte, buf: &mut T)
        -> WResult<PackError>;
}

impl<U: BoltWriteable<Error = io::Error> + Copy> FixedPackableAs for U {
    fn as_fixed_to<T: BoltWrite>(
        &self,
        marker_byte: MarkerByte,
        buf: &mut T,
    ) -> Result<usize, PackError> {
        Ok(marker_byte.bolt_write_to(buf)? + self.bolt_write_to(buf)?)
    }
}

pub trait FixedUnpackableAs
where
    Self: Sized,
{
    fn fixed_body_from<T: BoltRead>(buf: &mut T) -> Result<Self, UnpackError>;
}

impl<U: BoltReadable<Error = io::Error>> FixedUnpackableAs for U {
    fn fixed_body_from<T: BoltRead>(buf: &mut T) -> Result<Self, UnpackError> {
        Ok(<U>::bolt_read_from(buf)?)
    }
}
