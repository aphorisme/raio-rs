use crate::packing::error::{PackError, UnpackError};
use crate::packing::ll::{BoltReadable, BoltWrite};
use crate::packing::{Packable, Unpackable};
use std::convert::TryFrom;
use std::fmt::Formatter;
use std::io::Error;
use std::{fmt, io};

#[derive(Debug)]
pub enum MessageWriteError {
    DataTooLong,
    PackError(PackError),
    WriteIOError(io::Error),
}

impl fmt::Display for MessageWriteError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            MessageWriteError::DataTooLong => write!(f, "Data is too long to write"),
            MessageWriteError::PackError(e) => write!(f, "Pack error: {}", e),
            MessageWriteError::WriteIOError(e) => write!(f, "Write IO error: {}", e),
        }
    }
}

impl std::error::Error for MessageWriteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MessageWriteError::PackError(e) => Some(e),
            MessageWriteError::WriteIOError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for MessageWriteError {
    fn from(input: io::Error) -> MessageWriteError {
        MessageWriteError::WriteIOError(input)
    }
}

impl From<PackError> for MessageWriteError {
    fn from(input: PackError) -> Self {
        MessageWriteError::PackError(input)
    }
}

#[derive(Debug)]
pub enum MessageReadError {
    UnpackError(UnpackError),
    ReadIOError(io::Error),
    MessageEndWasExpected,
    EmptyChunk,
}

impl fmt::Display for MessageReadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            MessageReadError::UnpackError(e) => {
                write!(f, "Unpack error while reading message: {}", e)
            }
            MessageReadError::ReadIOError(e) => write!(f, "IO error while reading message: {}", e),
            MessageReadError::MessageEndWasExpected => write!(f, "Message end was expected."),
            MessageReadError::EmptyChunk => {
                write!(f, "Empty chunk reached, but end wasn't expected.")
            }
        }
    }
}

impl std::error::Error for MessageReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MessageReadError::ReadIOError(e) => Some(e),
            MessageReadError::UnpackError(e) => Some(e),
            MessageReadError::EmptyChunk => None,
            MessageReadError::MessageEndWasExpected => None,
        }
    }
}

impl From<io::Error> for MessageReadError {
    fn from(input: Error) -> Self {
        MessageReadError::ReadIOError(input)
    }
}

impl From<UnpackError> for MessageReadError {
    fn from(input: UnpackError) -> Self {
        MessageReadError::UnpackError(input)
    }
}

pub trait MessageWrite: io::Write
where
    Self: Sized,
{
    fn write_as_message<V: Packable>(&mut self, value: &V) -> Result<usize, MessageWriteError> {
        let written = self.write_as_chunk(value)? + self.write_message_end()?;
        Ok(written)
    }

    fn write_as_chunk<V: Packable>(&mut self, value: &V) -> Result<usize, MessageWriteError> {
        let mut v_bytes = Vec::new();
        let v_written = <u16>::try_from(value.pack_to(&mut v_bytes)?)
            .map_err(|_| MessageWriteError::DataTooLong)?;
        let written = self.bolt_write(v_written)? + self.write(v_bytes.as_slice())?;
        Ok(written)
    }

    fn write_message_end(&mut self) -> Result<usize, io::Error> {
        self.write(&[0, 0])
    }
}

impl<T: io::Write> MessageWrite for T {}

pub trait MessageRead: io::Read
where
    Self: Sized,
{
    fn read_from_message<V: Unpackable>(&mut self) -> Result<V, MessageReadError> {
        let mut data: Vec<u8> = Vec::new();

        // read in chunks until an empty chunk is encountered:
        loop {
            let chunk_size = <u16>::bolt_read_from(self)?;
            if chunk_size == 0 {
                // end of message reached:
                break;
            }
            let mut chunk_data = vec![0; chunk_size as usize];
            self.read_exact(&mut chunk_data)?;

            data.append(&mut chunk_data);
        }

        if data.is_empty() {
            return Err(MessageReadError::EmptyChunk);
        }

        let v = <V>::unpack_from(&mut data.as_slice())?;
        Ok(v)
    }

    fn read_from_chunk<V: Unpackable>(&mut self) -> Result<V, MessageReadError> {
        // read in chunk:
        let chunk_size = <u16>::bolt_read_from(self)?;

        if chunk_size == 0 {
            return Err(MessageReadError::EmptyChunk);
        }

        let mut chunk_data: Vec<u8> = Vec::with_capacity(<usize>::try_from(chunk_size).unwrap());
        self.read_exact(&mut chunk_data)?;

        let v = <V>::unpack_from(&mut chunk_data.as_slice())?;
        Ok(v)
    }

    fn read_end(&mut self) -> Result<(), MessageReadError> {
        // read in end:
        let mut end: [u8; 2] = [0, 0];
        self.read_exact(&mut end)
            .map_err(|_| MessageReadError::MessageEndWasExpected)?;
        if end[0] != 0x00 || end[1] != 0x00 {
            Err(MessageReadError::MessageEndWasExpected)
        } else {
            Ok(())
        }
    }
}

impl<T: io::Read> MessageRead for T {}

#[cfg(test)]
mod test {
    use crate::packing::ValueMap;
    use crate::packing::{Init, MessageRead, MessageWrite, Run};

    #[test]
    fn init_message_hex() {
        let mut data = Vec::new();

        let mut auth_options = ValueMap::with_capacity(3);
        auth_options.insert_value("scheme", "basic");
        auth_options.insert_value("principal", "neo4j");
        auth_options.insert_value("credentials", "secret");

        let init_struct = Init {
            client_name: "MyClient/1.0".to_string(),
            auth_token: auth_options,
        };

        data.write_as_message(&init_struct)
            .expect("Cannot write to buffer.");

        let control_bytes: Vec<u8> = vec![
            0x00, 0x40, // chunk size
            0xB2, 0x01, 0x8C, 0x4D, 0x79, 0x43, 0x6C, 0x69, 0x65, 0x6E, 0x74, 0x2F, 0x31, 0x2E,
            0x30, 0xA3, // TinyMap, 3 items
            0x8B, 0x63, 0x72, 0x65, 0x64, 0x65, 0x6E, 0x74, 0x69, 0x61, 0x6C, 0x73, // Key
            0x86, 0x73, 0x65, 0x63, 0x72, 0x65, 0x74, // Value
            0x86, 0x73, 0x63, 0x68, 0x65, 0x6D, 0x65, // Key
            0x85, 0x62, 0x61, 0x73, 0x69, 0x63, // Value
            0x89, 0x70, 0x72, 0x69, 0x6E, 0x63, 0x69, 0x70, 0x61, 0x6C, // Key
            0x85, 0x6E, 0x65, 0x6F, 0x34, 0x6A, // Value
            0x00, 0x00, // message end
        ];

        let control_init: Init = control_bytes
            .as_slice()
            .read_from_message()
            .expect("Cannot unpack init from control bytes");

        assert_eq!(init_struct, control_init);
    }

    #[test]
    fn run_simple_message_hex() {
        let mut data = Vec::new();

        let run_struct = Run::statement("RETURN 1 AS num");

        data.write_as_message(&run_struct)
            .expect("Cannot write to buffer.");

        let control_bytes: Vec<u8> = vec![
            0x00, 0x14, 0xb3, 0x10, 0x8f, 0x52, 0x45, 0x54, 0x55, 0x52, 0x4e, 0x20, 0x31, 0x20,
            0x41, 0x53, 0x20, 0x6e, 0x75, 0x6d, 0xa0, 0xa0, 0x00, 0x00,
        ];

        assert_eq!(format!("{:x?}", data), format!("{:x?}", control_bytes));

        let run_control: Run = control_bytes
            .as_slice()
            .read_from_message()
            .expect("Cannot unpack init from control bytes");

        assert_eq!(run_struct, run_control);
    }

    #[test]
    fn run_message_hex() {
        let mut data = Vec::new();

        let run_struct = Run::statement("MATCH (n: Tag) RETURN n");

        data.write_as_message(&run_struct)
            .expect("Cannot write to buffer.");

        let control_bytes: Vec<u8> = vec![
            0x00, 0x1d, 0xb3, 0x10, 0xd0, 0x17, 0x4d, 0x41, 0x54, 0x43, 0x48, 0x20, 0x28, 0x6e,
            0x3a, 0x20, 0x54, 0x61, 0x67, 0x29, 0x20, 0x52, 0x45, 0x54, 0x55, 0x52, 0x4e, 0x20,
            0x6e, 0xa0, 0xa0, 0x00, 0x00,
        ];

        assert_eq!(format!("{:x?}", data), format!("{:x?}", control_bytes));

        let run_control: Run = control_bytes
            .as_slice()
            .read_from_message()
            .expect("Cannot unpack run from control bytes");

        assert_eq!(run_struct, run_control);
    }
}
