use crate::packing::error::{PackError, UnpackError};
use crate::packing::ll::{
    read_expected_marker, BoltRead, BoltReadable, BoltWrite, BoltWriteable, MarkerByte,
    UnboundPackable, UnboundUnpackable, WResult,
};
use std::io;
use std::io::Write;
use std::ops::{Deref, DerefMut};

pub struct Fixed<I>(I);

impl<I> Deref for Fixed<I> {
    type Target = I;
    fn deref(&self) -> &I {
        &self.0
    }
}

impl<I> DerefMut for Fixed<I> {
    fn deref_mut(&mut self) -> &mut I {
        &mut self.0
    }
}

impl<I> Fixed<I> {
    pub fn into_inner(self) -> I {
        self.0
    }

    pub fn write_header<T: Write>(buf: &mut T, marker: MarkerByte) -> WResult<io::Error> {
        marker.bolt_write_to(buf)
    }
}

impl<I: BoltReadable<Error = io::Error>> UnboundUnpackable for Fixed<I> {
    type Marker = MarkerByte;
    fn unpack_as_from<T: BoltRead>(marker: MarkerByte, buf: &mut T) -> Result<Self, UnpackError> {
        let _: MarkerByte = read_expected_marker(marker, buf)?;
        let inner = buf.bolt_read::<I>()?;
        Ok(Fixed(inner))
    }
}

impl<I: BoltWriteable<Error = io::Error>> UnboundPackable for Fixed<I> {
    type Marker = MarkerByte;
    fn pack_as_to<T: BoltWrite>(self, marker: MarkerByte, buf: &mut T) -> WResult<PackError> {
        Ok(buf.bolt_write(marker)? + buf.bolt_write(self.0)?)
    }
}

impl<I> From<I> for Fixed<I> {
    fn from(input: I) -> Fixed<I> {
        Fixed(input)
    }
}
