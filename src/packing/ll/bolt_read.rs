use std::io;
use std::io::Read;

use byteorder::*;

/// Provides a unified way to write data in a protocol compliant way. This does not give any
/// guarantees besides that any data written with `bolt_read_from` can be recovered by a bolt
/// agent, if known which type it was.
pub trait BoltReadable
where
    Self: Sized,
{
    type Error;
    fn bolt_read_from<T: Read>(buf: &mut T) -> Result<Self, Self::Error>;
}

impl BoltReadable for String {
    type Error = io::Error;
    fn bolt_read_from<T: Read>(buf: &mut T) -> Result<Self, Self::Error> {
        let mut s: String = String::new();
        buf.read_to_string(&mut s)?;
        Ok(s)
    }
}

impl BoltReadable for u8 {
    type Error = io::Error;
    fn bolt_read_from<T: Read>(buf: &mut T) -> Result<Self, Self::Error> {
        buf.read_u8()
    }
}

impl BoltReadable for u16 {
    type Error = io::Error;
    fn bolt_read_from<T: Read>(buf: &mut T) -> Result<Self, Self::Error> {
        buf.read_u16::<BigEndian>()
    }
}

impl BoltReadable for u32 {
    type Error = io::Error;
    fn bolt_read_from<T: Read>(buf: &mut T) -> Result<Self, Self::Error> {
        buf.read_u32::<BigEndian>()
    }
}

impl BoltReadable for i8 {
    type Error = io::Error;
    fn bolt_read_from<T: Read>(buf: &mut T) -> Result<Self, Self::Error> {
        buf.read_i8()
    }
}

impl BoltReadable for i16 {
    type Error = io::Error;
    fn bolt_read_from<T: Read>(buf: &mut T) -> Result<Self, Self::Error> {
        buf.read_i16::<BigEndian>()
    }
}

impl BoltReadable for i32 {
    type Error = io::Error;
    fn bolt_read_from<T: Read>(buf: &mut T) -> Result<Self, Self::Error> {
        buf.read_i32::<BigEndian>()
    }
}

impl BoltReadable for i64 {
    type Error = io::Error;
    fn bolt_read_from<T: Read>(buf: &mut T) -> Result<Self, Self::Error> {
        buf.read_i64::<BigEndian>()
    }
}

impl BoltReadable for f64 {
    type Error = io::Error;
    fn bolt_read_from<T: Read>(buf: &mut T) -> Result<Self, Self::Error> {
        buf.read_f64::<BigEndian>()
    }
}

/// Convenience trait to extend `Read` with `BoltReadable` capacity.
pub trait BoltRead: Read
where
    Self: Sized,
{
    fn bolt_read<T: BoltReadable>(&mut self) -> Result<T, <T as BoltReadable>::Error> {
        T::bolt_read_from(self)
    }

    fn bolt_read_exact<T: BoltReadable>(&mut self, len: u64) -> Result<T, T::Error> {
        T::bolt_read_from(&mut self.take(len))
    }
}

impl<T: Read> BoltRead for T {}
