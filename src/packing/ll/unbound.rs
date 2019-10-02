use crate::packing::error::{PackError, UnpackError};
use crate::packing::ll::{BoltRead, BoltWrite, MarkerByte, WResult};

pub trait UnboundPackable {
    type Marker;
    fn pack_as_to<T: BoltWrite>(self, marker: MarkerByte, buf: &mut T) -> WResult<PackError>;
}

pub trait UnboundPack: BoltWrite {
    fn pack_as<I: UnboundPackable>(&mut self, marker: MarkerByte, obj: I) -> WResult<PackError> {
        obj.pack_as_to(marker, self)
    }
}

pub trait UnboundUnpackable
where
    Self: Sized,
{
    type Marker;
    fn unpack_as_from<T: BoltRead>(marker: MarkerByte, buf: &mut T) -> Result<Self, UnpackError>;
}

pub trait UnboundUnpack: BoltRead {
    fn unpack_as<I: UnboundUnpackable>(&mut self, marker: MarkerByte) -> Result<I, UnpackError> {
        I::unpack_as_from(marker, self)
    }
}
