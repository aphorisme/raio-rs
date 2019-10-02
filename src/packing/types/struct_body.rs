use crate::packing::error::{PackError, UnpackError};
use crate::packing::ll::{BoltRead, BoltWrite, Signature, WResult};

pub trait TinyStructBody
where
    Self: Sized,
{
    const FIELDS: u8;
    const SIGNATURE: Signature;
    fn write_body_to<T: BoltWrite>(self, buf: &mut T) -> WResult<PackError>;
    fn read_body_from<T: BoltRead>(buf: &mut T) -> Result<Self, UnpackError>;
}
