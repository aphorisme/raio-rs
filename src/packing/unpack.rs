use crate::packing::error::UnpackError;
use crate::packing::ll::UnboundUnpack;
use std::io;

pub trait Unpackable
where
    Self: Sized,
{
    fn unpack_from<T: UnboundUnpack>(buf: &mut T) -> Result<Self, UnpackError>;
}

pub trait Unpack: UnboundUnpack {
    fn unpack<T: Unpackable>(&mut self) -> Result<T, UnpackError> {
        <T>::unpack_from(self)
    }
}
