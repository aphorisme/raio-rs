use std::cmp::min;
use std::fmt::Formatter;
use async_std::prelude::*;

#[derive(Debug, Clone, PartialEq)]
/// A `Chunk` is a part of a [`Message`](crate::connectivity::message::Message), with a fixed capacity.
/// In the same sense as a `Message` can be written to and can be read from, a `Chunk` can, using
/// `write` and `read`. In the same sense a `Chunk` can be packed and unpacked, encoding its
/// actual size following the bolt protocol.
///
/// A `Chunk` is meant to be used via a `Message`.
pub struct Chunk {
    capacity: usize,
    written: usize,
    bytes: Vec<u8>,
    read_cursor: usize,
}

impl Chunk {
    pub fn has_capacity(&self) -> bool {
        self.capacity > self.written
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn written(&self) -> usize {
        self.written
    }

    pub fn eof(&self) -> bool {
        self.written <= self.read_cursor
    }

    /// Creates a new chunk of given maximal size. According to protocol, the size can only be
    /// `u16`; internally the chunk stores all accounting as `usize`.
    pub fn new(max_size: u16) -> Chunk {
        Chunk {
            capacity: max_size as usize,
            written: 0,
            bytes: Vec::with_capacity(max_size as usize),
            read_cursor: 0,
        }
    }

    /// Writes the provided bytes into the chunk and returns all bytes which
    /// are left, if any.
    /// ```
    /// # use raio::messaging::chunk::Chunk;
    /// let mut chunk = Chunk::new(3);
    ///
    /// // can only write 3 bytes, one left over:
    /// let rest = chunk.write(&[1u8, 2, 3, 4]);
    /// assert_eq!(rest, Some(&[4u8][..]));
    /// assert!(!chunk.has_capacity());
    ///
    /// // if nothing is left over, `None` is returned:
    /// chunk = Chunk::new(3);
    /// assert_eq!(chunk.write(&[1u8, 2, 3]), None);
    /// ```
    pub fn write<'a>(&mut self, bytes: &'a [u8]) -> Option<&'a [u8]> {
        let capacity = self.capacity - self.written;
        if capacity > 0 {
            if bytes.len() > capacity as usize {
                let written =
                    <Vec<u8> as std::io::Write>::write(
                        &mut self.bytes,
                        &bytes[0..capacity as usize])
                        .unwrap();
                self.written += written;
                Some(&bytes[capacity as usize..])
            } else {
                self.written +=
                    <Vec<u8> as std::io::Write>::write(
                        &mut self.bytes,
                        bytes)
                        .unwrap();
                None
            }
        } else {
            Some(bytes)
        }
    }

    /// Reads from the chunk into the buffer; returns how many bytes were read.
    /// ```
    /// # use raio::messaging::chunk::Chunk;
    /// let mut chunk = Chunk::new(3);
    /// chunk.write(&[1, 2, 3]);
    ///
    /// let mut buf = vec![0u8; 2];
    /// let written = chunk.read(&mut buf);
    ///
    /// assert_eq!(written, 2);
    /// assert_eq!(buf, &[1, 2]);
    /// ```
    /// The function follows what is to be expected from `read` A `Chunk`
    /// has an internal cursor which keeps track of what was read already:
    /// ```
    /// # use raio::messaging::chunk::Chunk;
    /// let mut chunk = Chunk::new(5);
    /// chunk.write(&[1, 2, 3, 4, 5]);
    ///
    /// let mut buf_1 = vec![0u8; 2];
    /// let mut buf_2 = vec![0u8; 5];
    ///
    /// chunk.read(&mut buf_1);
    /// chunk.read(&mut buf_2);
    ///
    /// assert_eq!(buf_1, &[1, 2]);
    /// assert_eq!(buf_2, &[3, 4, 5, 0, 0]);
    ///
    /// // cursor has reached its end:
    /// assert!(chunk.eof())
    /// ```
    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        let rest = self.written - self.read_cursor;
        let max = min(buf.len(), rest);
        for i in 0..max {
            buf[i] = self.bytes[self.read_cursor];
            self.read_cursor += 1;
        }

        max
    }

    pub fn set_cursor(&mut self, new_cursor: usize) {
        self.read_cursor = new_cursor;
    }

    /// Writes a chunk as part of a bolt message, i.e. adds the size of the chunk at the beginning.
    /// This function is an asynchronous function.
    /// ```
    /// # use raio::messaging::chunk::Chunk;
    /// # #[async_std::main]
    /// # async fn main() -> std::io::Result<()> {
    /// // create a chunk with ten 0's but capacity 13.
    /// let mut chunk = Chunk::new(13);
    /// chunk.write(&vec![0u8; 10]);
    ///
    /// let mut buf = Vec::new();
    /// let written = chunk.pack(&mut buf).await?;
    ///
    /// // two bytes for the `u16` then `10` for the 0's:
    /// assert_eq!(written, 2 + 10);
    ///
    /// // the first two bytes are `10` in big endian:
    /// assert_eq!(&[0x00, 0x0A][..], &buf.as_slice()[0..2]);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn pack<T: async_std::io::Write + Unpin>(&self, writer: &mut T) -> async_std::io::Result<usize> {
        writer.write(&(self.written as u16).to_be_bytes()).await?;
        let written = writer.write(self.bytes.as_slice()).await?;
        Ok(2 + written)
    }

    /// Unpacks a `Chunk` from a bolt stream, i.e. reads out an `u16` then reads as many bytes
    /// as denoted. Sets the size of the `Chunk` to the read `u16`.
    /// This function is an asynchronous function.
    /// ```
    /// # use raio::messaging::chunk::Chunk;
    /// # #[async_std::main]
    /// # async fn main() -> std::io::Result<()> {
    /// // a stream `1, 1, 1, -1` with a encoded size of `3` beforehand:
    /// let mut stream : &[u8] = &[0x00, 0x03, 0x01, 0x01, 0x01, 0xFF];
    ///
    /// // unpack into a perfect sized chunk:
    /// let mut chunk = Chunk::unpack(&mut stream).await?;
    /// assert!(!chunk.has_capacity());
    /// assert_eq!(chunk.written(), 3);
    ///
    /// // check, what is within the chunk:
    /// let mut buf = vec![0u8; 5];
    /// chunk.read(&mut buf);
    /// assert_eq!(buf.as_slice(), &[1, 1, 1, 0, 0][..]);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn unpack<T: async_std::io::Read + Unpin>(reader: &mut T) -> async_std::io::Result<Self> {
        let mut buf_size = [0u8, 0u8];
        reader.read(&mut buf_size).await?;
        let size = u16::from_be_bytes(buf_size) as usize;

        let mut buf = vec![0; size];
        reader.read_exact(&mut buf).await?;
        Ok(
            Chunk {
                read_cursor: 0,
                bytes: buf,
                capacity: size,
                written: size,
            }
        )
    }
}

impl std::fmt::Display for Chunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let size_bytes = (self.written as u16).to_be_bytes();
        write!(f, "[{:X} {:X}]", size_bytes[0], size_bytes[1])?;
        for b in &self.bytes {
            write!(f, " {:X}", b)?;
        }

        Ok(())
    }
}