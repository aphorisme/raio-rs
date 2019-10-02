use std::collections::HashMap;
use std::convert::TryFrom;
use std::io::Write;
use std::num::TryFromIntError;
use std::{fmt, io};

use crate::packing::error::{ConversionError, PackError, UnpackError};
use crate::packing::ll::{
    combine_nibble, read_expected_marker, BoltRead, BoltReadable, BoltWrite, BoltWriteable,
    MarkerByte, TinySizeMarker, UnboundPackable, UnboundUnpackable, WResult,
};
use byteorder::WriteBytesExt;

const TINY_SIZE_MAX: u8 = 0x0F;

pub struct TinySized<I> {
    tiny_size: u8,
    inner: I,
}

impl<I> TinySized<I> {
    pub fn size(&self) -> u8 {
        self.tiny_size
    }

    pub fn into_inner(self) -> I {
        self.inner
    }

    pub fn inner_ref(&self) -> &I {
        &self.inner
    }

    pub fn inner_mut_ref(&mut self) -> &mut I {
        &mut self.inner
    }

    pub fn write_header<T: Write>(&self, buf: &mut T, marker: MarkerByte) -> WResult<io::Error> {
        buf.write_u8(combine_nibble(marker as u8, self.tiny_size))?;
        Ok(1)
    }
}

impl<I: BoltWriteable> TinySized<I> {
    pub fn write_body<T: BoltWrite>(self, buf: &mut T) -> WResult<<I as BoltWriteable>::Error> {
        buf.bolt_write::<I>(self.inner)
    }
}

impl<I: BoltReadable<Error = io::Error>> TinySized<I> {
    pub fn read_body<T: BoltRead>(
        marker: TinySizeMarker,
        buf: &mut T,
    ) -> Result<TinySized<I>, io::Error> {
        let inner = buf.bolt_read_exact::<I>(marker.tiny_size as u64)?;
        Ok(TinySized {
            tiny_size: marker.tiny_size,
            inner,
        })
    }
}

impl<E, I: BoltWriteable<Error = E>> UnboundPackable for TinySized<I>
where
    PackError: From<E>,
{
    type Marker = TinySizeMarker;
    fn pack_as_to<T: BoltWrite>(self, marker: MarkerByte, buf: &mut T) -> WResult<PackError> {
        let written = TinySizeMarker {
            marker,
            tiny_size: self.tiny_size,
        }
        .bolt_write_to(buf)?
            + self.inner.bolt_write_to(buf)?;
        Ok(written)
    }
}

impl<I: BoltReadable<Error = io::Error>> UnboundUnpackable for TinySized<I> {
    type Marker = TinySizeMarker;
    fn unpack_as_from<T: BoltRead>(marker: MarkerByte, buf: &mut T) -> Result<Self, UnpackError> {
        let marker: TinySizeMarker = read_expected_marker(marker, buf)?;
        let inner = buf.bolt_read_exact(marker.tiny_size as u64)?;
        Ok(TinySized {
            tiny_size: marker.tiny_size,
            inner,
        })
    }
}

impl TryFrom<String> for TinySized<String> {
    type Error = ConversionError;
    fn try_from(input: String) -> Result<Self, Self::Error> {
        let u = u8::try_from(input.len())?;
        if u <= TINY_SIZE_MAX {
            Ok(TinySized {
                tiny_size: u,
                inner: input,
            })
        } else {
            Err(ConversionError::SourceTooLarge)
        }
    }
}

impl<V> TryFrom<Vec<V>> for TinySized<Vec<V>> {
    type Error = ConversionError;
    fn try_from(input: Vec<V>) -> Result<Self, Self::Error> {
        let u: u8 = u8::try_from(input.len())?;
        if u <= TINY_SIZE_MAX {
            Ok(TinySized {
                tiny_size: u,
                inner: input,
            })
        } else {
            Err(ConversionError::SourceTooLarge)
        }
    }
}

impl<V, K> TryFrom<HashMap<K, V>> for TinySized<HashMap<K, V>> {
    type Error = ConversionError;
    fn try_from(input: HashMap<K, V>) -> Result<Self, Self::Error> {
        let u: u8 = u8::try_from(input.len())?;
        if u <= TINY_SIZE_MAX {
            Ok(TinySized {
                tiny_size: u,
                inner: input,
            })
        } else {
            Err(ConversionError::SourceTooLarge)
        }
    }
}
