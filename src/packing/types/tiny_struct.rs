use crate::packing::error::{PackError, UnpackError};
use crate::packing::ll::{
    read_expected_marker, MarkerByte, TinySizeMarker, UnboundPack, UnboundUnpack, WResult,
};
use crate::packing::{Packable, TinyStructBody, Unpackable};

pub struct TinyStruct<S: TinyStructBody>(S);

impl<S: TinyStructBody> Packable for TinyStruct<S> {
    fn pack_to<T: UnboundPack>(self, buf: &mut T) -> WResult<PackError> {
        let written = buf.bolt_write(TinySizeMarker {
            tiny_size: S::FIELDS,
            marker: MarkerByte::TinyStruct,
        })? + buf.bolt_write(S::SIGNATURE)?
            + self.0.write_body_to::<T>(buf)?;
        Ok(written)
    }
}

impl<S: TinyStructBody> Unpackable for TinyStruct<S> {
    fn unpack_from<T: UnboundUnpack>(buf: &mut T) -> Result<Self, UnpackError> {
        // read marker and check size:
        let marker: TinySizeMarker = read_expected_marker(MarkerByte::TinyStruct, buf)?;
        if marker.tiny_size != S::FIELDS {
            return Err(UnpackError::UnexpectedSignatureSize(
                S::FIELDS,
                marker.tiny_size,
            ));
        }

        // signature:
        S::SIGNATURE.read_expected(buf)?;

        // finally, body:
        let b = S::read_body_from(buf)?;
        Ok(TinyStruct(b))
    }
}
