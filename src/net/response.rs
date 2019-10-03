use crate::packing::error::UnpackError;
use crate::packing::ll::{BoltRead, MarkerByte, Signature, TinySizeMarker};
use crate::packing::{Unpackable, Value, ValueList, ValueMap};
use byteorder::ReadBytesExt;
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub enum Response {
    Success(ValueMap<Value>),
    Failure(ValueMap<Value>),
    Record(ValueList<Value>),
    Ignored,
}

impl Unpackable for Response {
    fn unpack_from<T: BoltRead>(buf: &mut T) -> Result<Self, UnpackError> {
        let m_byte = buf.read_u8()?;
        let m: TinySizeMarker = TinySizeMarker::try_from(m_byte)?;

        match m.marker {
            MarkerByte::TinyStruct => {
                // read out signature:
                let sig_byte = buf.read_u8()?;
                let sig = Signature::try_from(sig_byte)?;

                match sig {
                    Signature::Success => Ok(Response::Success(ValueMap::unpack_from(buf)?)),
                    Signature::Failure => Ok(Response::Failure(ValueMap::unpack_from(buf)?)),
                    Signature::Record => Ok(Response::Record(ValueList::unpack_from(buf)?)),
                    Signature::Ignored => Ok(Response::Ignored),
                    _ => Err(UnpackError::UnexpectedSignature(sig, "Response")),
                }
            }
            _ => Err(UnpackError::UnexpectedMarker(m.marker, "Response")),
        }
    }
}

#[cfg(test)]
pub mod test {
    use crate::net::Response;
    use crate::packing::{Unpackable, ValueList, ValueMap};

    #[test]
    pub fn record_response_hex() {
        // Response: RECORD [1]
        let response_bytes: Vec<u8> = vec![0xb1, 0x71, 0x91, 0x01];

        let response: Response = Response::unpack_from(&mut response_bytes.as_slice())
            .expect("Cannot unpack from response bytes.");

        let vl = ValueList::from(vec![1]);

        assert_eq!(response, Response::Record(vl));
    }

    #[test]
    pub fn success_response_hex() {
        // Response: SUCCESS { "fields": ["num"], "result_available_after": 12 }
        let response_bytes: Vec<u8> = vec![
            0xB1, 0x70, 0xA2, 0x86, 0x66, 0x69, 0x65, 0x6C, 0x64, 0x73, 0x91, 0x83, 0x6E, 0x75,
            0x6D, 0xD0, 0x16, 0x72, 0x65, 0x73, 0x75, 0x6C, 0x74, 0x5F, 0x61, 0x76, 0x61, 0x69,
            0x6C, 0x61, 0x62, 0x6C, 0x65, 0x5F, 0x61, 0x66, 0x74, 0x65, 0x72, 0x0C,
        ];

        let response: Response = Response::unpack_from(&mut response_bytes.as_slice())
            .expect("Cannot unpack from response bytes.");

        let mut vl = ValueMap::with_capacity(2);
        vl.insert_value("fields", ValueList::from(vec!["num"]));
        vl.insert_value("result_available_after", 12i64);

        assert_eq!(response, Response::Success(vl));
    }

    #[test]
    pub fn ignored_response_hex() {
        // Response: IGNORED
        let response_bytes: Vec<u8> = vec![0xb0, 0x7e];

        let response: Response = Response::unpack_from(&mut response_bytes.as_slice())
            .expect("Cannot unpack from response bytes.");

        assert_eq!(response, Response::Ignored);
    }

    #[test]
    pub fn failure_response_hex() {
        // Response:
        /*
        FAILURE { "code": "Neo.ClientError.Statement.SyntaxError",
                   "message": "Invalid input 'T': expected <init> (line 1, column 1 (offset: 0))
                           "This will cause a syntax error"
                            ^"}
         */
        let response_bytes: Vec<u8> = vec![
            0xB1, 0x7F, 0xA2, 0x84, 0x63, 0x6F, 0x64, 0x65, 0xD0, 0x25, 0x4E, 0x65, 0x6F, 0x2E,
            0x43, 0x6C, 0x69, 0x65, 0x6E, 0x74, 0x45, 0x72, 0x72, 0x6F, 0x72, 0x2E, 0x53, 0x74,
            0x61, 0x74, 0x65, 0x6D, 0x65, 0x6E, 0x74, 0x2E, 0x53, 0x79, 0x6E, 0x74, 0x61, 0x78,
            0x45, 0x72, 0x72, 0x6F, 0x72, 0x87, 0x6D, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0xD0,
            0x65, 0x49, 0x6E, 0x76, 0x61, 0x6C, 0x69, 0x64, 0x20, 0x69, 0x6E, 0x70, 0x75, 0x74,
            0x20, 0x27, 0x54, 0x27, 0x3A, 0x20, 0x65, 0x78, 0x70, 0x65, 0x63, 0x74, 0x65, 0x64,
            0x20, 0x3C, 0x69, 0x6E, 0x69, 0x74, 0x3E, 0x20, 0x28, 0x6C, 0x69, 0x6E, 0x65, 0x20,
            0x31, 0x2C, 0x20, 0x63, 0x6F, 0x6C, 0x75, 0x6D, 0x6E, 0x20, 0x31, 0x20, 0x28, 0x6F,
            0x66, 0x66, 0x73, 0x65, 0x74, 0x3A, 0x20, 0x30, 0x29, 0x29, 0x0A, 0x22, 0x54, 0x68,
            0x69, 0x73, 0x20, 0x77, 0x69, 0x6C, 0x6C, 0x20, 0x63, 0x61, 0x75, 0x73, 0x65, 0x20,
            0x61, 0x20, 0x73, 0x79, 0x6E, 0x74, 0x61, 0x78, 0x20, 0x65, 0x72, 0x72, 0x6F, 0x72,
            0x22, 0x0A, 0x20, 0x5E,
        ];

        let response: Response = Response::unpack_from(&mut response_bytes.as_slice())
            .expect("Cannot unpack from response bytes.");

        let mut failure_fields: ValueMap = ValueMap::with_capacity(2);
        failure_fields.insert_value("code", "Neo.ClientError.Statement.SyntaxError");
        failure_fields.insert_value(
            "message",
            "Invalid input 'T': expected <init> (line 1, column 1 (offset: 0))\n\"This will cause a syntax error\"\n ^");

        assert_eq!(response, Response::Failure(failure_fields));
    }
}
