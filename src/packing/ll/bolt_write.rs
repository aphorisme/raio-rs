use std::io;
use std::io::Write;

use byteorder::{BigEndian, WriteBytesExt};

pub type WResult<E> = Result<usize, E>;

/// Counterpart to `BoltReadable`, i.e. provides a unified way to read data which is encoded
/// in a bolt protocol compliant way.
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

impl BoltWriteable for &String {
    type Error = io::Error;
    fn bolt_write_to<T: Write>(self, buf: &mut T) -> Result<usize, Self::Error> {
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

/// Convenience trait to extend a `Write` with `BoltWritable` capabilities.
pub trait BoltWrite: Write
where
    Self: Sized,
{
    fn bolt_write<T: BoltWriteable>(&mut self, obj: T) -> WResult<<T as BoltWriteable>::Error> {
        obj.bolt_write_to(self)
    }
}

impl<T: Write> BoltWrite for T {}

// tests:
#[cfg(test)]
mod tests {

    macro_rules! written_bytes_is_exact {
        ($e:expr => $t:ident) => {
            #[allow(non_snake_case)]
            mod $t {
                use crate::packing::ll::BoltWrite;

                #[test]
                fn written_bytes_is_exact() {
                    let mut data = Vec::new();
                    let w = data.bolt_write::<$t>($e).unwrap();
                    assert_eq!(data.len(), w);
                }
            }
        };
    }

    written_bytes_is_exact!(42u8 => u8);
    written_bytes_is_exact!(42u16 => u16);
    written_bytes_is_exact!(42u32 => u32);
    written_bytes_is_exact!(42i8 => i8);
    written_bytes_is_exact!(42i16 => i16);
    written_bytes_is_exact!(42i32 => i32);
    written_bytes_is_exact!(42i64 => i64);
    written_bytes_is_exact!(42f64 => f64);
    written_bytes_is_exact!(String::from("Hello World!☎") => String);
}
