use byteorder::{BigEndian, WriteBytesExt};
use std::io;
use std::io::Write;

pub type WResult<E> = Result<usize, E>;

pub trait BoltWriteable {
    type Error;
    fn bolt_write_to<T: Write>(self, buf: &mut T) -> WResult<Self::Error>;
}

impl BoltWriteable for String {
    type Error = io::Error;
    fn bolt_write_to<T: Write>(self, buf: &mut T) -> WResult<Self::Error> {
        buf.write(self.as_bytes())
    }
}

impl BoltWriteable for u8 {
    type Error = io::Error;
    fn bolt_write_to<T: Write>(self, buf: &mut T) -> WResult<Self::Error> {
        buf.write_u8(self)?;
        Ok(1)
    }
}

impl BoltWriteable for u16 {
    type Error = io::Error;
    fn bolt_write_to<T: Write>(self, buf: &mut T) -> WResult<Self::Error> {
        buf.write_u16::<BigEndian>(self)?;
        Ok(2)
    }
}

impl BoltWriteable for u32 {
    type Error = io::Error;
    fn bolt_write_to<T: Write>(self, buf: &mut T) -> WResult<Self::Error> {
        buf.write_u32::<BigEndian>(self)?;
        Ok(4)
    }
}

impl BoltWriteable for i8 {
    type Error = io::Error;
    fn bolt_write_to<T: Write>(self, buf: &mut T) -> WResult<Self::Error> {
        buf.write_i8(self)?;
        Ok(1)
    }
}

impl BoltWriteable for i16 {
    type Error = io::Error;
    fn bolt_write_to<T: Write>(self, buf: &mut T) -> WResult<Self::Error> {
        buf.write_i16::<BigEndian>(self)?;
        Ok(2)
    }
}

impl BoltWriteable for i32 {
    type Error = io::Error;
    fn bolt_write_to<T: Write>(self, buf: &mut T) -> WResult<Self::Error> {
        buf.write_i32::<BigEndian>(self)?;
        Ok(4)
    }
}

impl BoltWriteable for i64 {
    type Error = io::Error;
    fn bolt_write_to<T: Write>(self, buf: &mut T) -> WResult<Self::Error> {
        buf.write_i64::<BigEndian>(self)?;
        Ok(8)
    }
}

impl BoltWriteable for f64 {
    type Error = io::Error;
    fn bolt_write_to<T: Write>(self, buf: &mut T) -> WResult<Self::Error> {
        buf.write_f64::<BigEndian>(self)?;
        Ok(8)
    }
}

pub trait BoltWrite: Write
where
    Self: Sized,
{
    fn bolt_write<T: BoltWriteable>(&mut self, obj: T) -> WResult<<T as BoltWriteable>::Error> {
        obj.bolt_write_to(self)
    }
}
