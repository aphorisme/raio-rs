use crate::packing::error::ConversionError;
use crate::packing::error::{PackError, UnpackError};
use crate::packing::ll::{
    read_expected_marker, BoltRead, BoltReadable, BoltWrite, BoltWriteable, MarkerByte,
    UnboundPackable, UnboundUnpackable, WResult,
};
use std::convert::TryFrom;
use std::io::{Read, Write};
use std::num::TryFromIntError;
use std::ops::Deref;
use std::{fmt, io};

pub struct Sized<S, I> {
    size: S,
    inner: I,
}

impl<S, I> Deref for Sized<S, I> {
    type Target = I;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<S: Copy + Clone, I> Sized<S, I> {
    pub fn size(&self) -> S {
        self.size.clone()
    }

    pub fn into_inner(self) -> I {
        self.inner
    }

    pub fn inner_ref(&self) -> &I {
        &self.inner
    }
}

impl<E: From<io::Error>, S: BoltWriteable<Error = E>> BoltWriteable for Sized<S, String> {
    type Error = E;
    fn bolt_write_to<T: Write>(self, buf: &mut T) -> WResult<E> {
        Ok(self.size.bolt_write_to(buf)? + self.inner.bolt_write_to(buf)?)
    }
}

impl<E: From<io::Error>, S: BoltReadable<Error = E> + Into<u64> + Copy> BoltReadable
    for Sized<S, String>
{
    type Error = E;
    fn bolt_read_from<T: Read>(buf: &mut T) -> Result<Self, Self::Error> {
        let size = S::bolt_read_from(buf)?;
        let inner = String::bolt_read_from(&mut buf.take(size.into()))?;
        Ok(Sized { size, inner })
    }
}

/// ```
/// use raio::packing::*;
/// use std::convert::TryFrom;
/// let data : Vec<u8> = vec![127u8; usize::try_from(u16::max_value() as usize + 1).unwrap()];
/// let s = String::from_utf8(data).unwrap();
/// let sized = <Sized<u32, String>>::try_from(s).unwrap();
/// assert_eq!(sized.size(), u16::max_value() as u32 + 1);
/// ```
impl<U: TryFrom<usize, Error = TryFromIntError>> TryFrom<String> for Sized<U, String> {
    type Error = ConversionError;
    fn try_from(input: String) -> Result<Self, Self::Error> {
        let u: U = U::try_from(input.len())?;
        Ok(Sized {
            size: u,
            inner: input,
        })
    }
}

impl<E, U: BoltReadable<Error = E> + Into<u64>, I> UnboundUnpackable for Sized<U, I>
where
    Sized<U, I>: BoltReadable<Error = E>,
    UnpackError: From<E>,
{
    type Marker = MarkerByte;
    fn unpack_as_from<T: BoltRead>(marker: MarkerByte, buf: &mut T) -> Result<Self, UnpackError> {
        let _: MarkerByte = read_expected_marker(marker, buf)?;
        Ok(<Sized<U, I>>::bolt_read_from(buf)?)
    }
}

impl<E, U: BoltWriteable<Error = E>, I> UnboundPackable for Sized<U, I>
where
    Sized<U, I>: BoltWriteable<Error = E>,
    PackError: From<E>,
{
    type Marker = MarkerByte;
    fn pack_as_to<T: BoltWrite>(self, marker: MarkerByte, buf: &mut T) -> WResult<PackError> {
        Ok(buf.bolt_write(marker)? + buf.bolt_write(self)?)
    }
}
