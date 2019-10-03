use crate::packing::error::{PackError, UnpackError};
use crate::packing::ll::{
    read_expected_marker, BoltRead, BoltWrite, MarkerByte, Signature, TinySizeMarker, WResult,
};
use crate::packing::{Packable, Unpackable};

// TODO: Move this to ll
pub trait TinyStructBody
where
    Self: Sized,
{
    const FIELDS: u8;
    const SIGNATURE: Signature;
    fn write_body_to<T: BoltWrite>(&self, buf: &mut T) -> WResult<PackError>;
    fn read_body_from<T: BoltRead>(buf: &mut T) -> Result<Self, UnpackError>;
}

impl<V: TinyStructBody> Packable for V {
    fn pack_to<T: BoltWrite>(&self, buf: &mut T) -> WResult<PackError> {
        let written = buf.bolt_write(TinySizeMarker {
            tiny_size: Self::FIELDS,
            marker: MarkerByte::TinyStruct,
        })? + buf.bolt_write(Self::SIGNATURE)?
            + self.write_body_to::<T>(buf)?;
        Ok(written)
    }
}

impl<V: TinyStructBody> Unpackable for V {
    fn unpack_from<T: BoltRead>(buf: &mut T) -> Result<Self, UnpackError> {
        // read marker and check size:
        let marker: TinySizeMarker = read_expected_marker(MarkerByte::TinyStruct, buf)?;
        if marker.tiny_size != Self::FIELDS {
            return Err(UnpackError::UnexpectedSignatureSize(
                Self::FIELDS,
                marker.tiny_size,
            ));
        }

        // signature:
        Self::SIGNATURE.read_expected(buf)?;

        // finally, body:
        let b = Self::read_body_from(buf)?;
        Ok(b)
    }
}
