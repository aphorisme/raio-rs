use async_std::prelude::*;
use std::slice::Iter;
use crate::ll::chunk::Chunk;
use std::fmt::Formatter;

#[derive(Debug, Clone, PartialEq)]
/// A `Message` is an array of bytes used to send and receive via the bolt protocol. Outside of
/// transmitting, a message can be seen as just an array of bytes, which can be fed using
/// [`Write`](std::io::Write) and read using [`Read`](std::io::Read):
/// ```
/// # use raio::ll::message::Message;
/// # use std::io::{Read, Write};
/// let mut message = Message::new_alloc(2, 5); // pre-allocates 2 chunks; maximal 5 bytes capacity per chunk.
///
/// // write 11 bytes, creates the two reserved chunks and allocates then another:
/// let mut stream: &[u8] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
/// let written = message.write(&mut stream).unwrap();
/// assert_eq!(written, 11);
/// assert_eq!(message.chunks().len(), 3);
///
/// // read the first 6 bytes:
/// let mut buf_1 = vec![0u8; 6];
/// let mut read = message.read(&mut buf_1).expect("Cannot read into buf_1");
/// assert_eq!(buf_1, &[1, 2, 3, 4, 5, 6][..]);
/// assert_eq!(read, 6);
///
/// // then read the rest:
/// let mut buf_2 = vec![0u8; 10];
/// read += message.read(&mut buf_2).unwrap();
/// assert_eq!(buf_2, &[7, 8, 9, 10, 11, 0, 0, 0, 0, 0][..]);
/// assert_eq!(read, 11);
/// ```
/// But besides, it can be packed
/// and unpacked into a bolt protocol message, which transmits the message in chunks, encoding the
/// chunks size and ending the message properly.
///
/// In conjunction with `packs`, the usual approach in using `Message` is to first fill it by encoding
/// some PackStream value into a message, then using `pack` from `Message` and sending it and the other way,
/// receiving bytes, `unpack` them into a message and using `read` to decode the `Message` into a
/// value:
/// ```
/// use packs::*;
/// use packs::std_structs::*;
/// # use raio::ll::message::Message;
/// # #[async_std::main]
/// # async fn main() -> std::io::Result<()> {
///
/// // create a `Node`:
/// let mut node = Node::new(0);
/// node.add_label("Person");
/// node.properties.add_property("name", "Jane Doe");
///
/// // encode this node into a message, following PackStream:
/// let mut message = Message::new_alloc(2, 15);
/// node.encode(&mut message).unwrap();
///
/// // packing this message bolt specific:
/// let mut buf = Vec::with_capacity(30);
/// message.pack(&mut buf).await?;
///
/// assert_eq!(
///     buf, &[
///         0x00, 0x0F, // first chunk size = 15 = max.
///         0xB3, 0x4E, // struct header
///         0x00, // id
///         0x91, // list header; one entry
///         0x86, 0x50, 0x65, 0x72, 0x73, 0x6F, 0x6E, // entry
///         0xA1, // dict header; one key-value
///         0x84, 0x6E, 0x61, // key part 1 ---
///         0x00, 0x0B, // second chunk size
///         0x6D, 0x65, // -- key part 2
///         0x88, 0x4A, 0x61, 0x6E, 0x65, 0x20, 0x44, 0x6F, 0x65, // value
///         0x00, 0x00 // empty chunk to end message
///         ][..]);
///
/// // assume now, that buf was received, write into a message:
/// let mut recvd_message = Message::unpack(&mut buf.as_slice()).await?;
///
/// // now decode it into a node:
/// let recvd_node = Node::decode(&mut recvd_message).unwrap();
///
/// assert_eq!(node, recvd_node);
/// # Ok(())
/// # }
/// ```
///
/// # Pack und Unpack
/// Packing and unpacking via `pack` and `unpack` is abstracted over `Write` and `Read` from `async_std` but it is
/// recommended to use a buffer in between when sending or receiving directly, since otherwise a
/// lot of syscalls might take place.
pub struct Message {
    chunk_capacity: u16,
    chunks: Vec<Chunk>,
    read_cursor: usize, // index to chunk
    write_cursor: usize, // index to chunk
}

impl Message {
    /// Creates a new message and pre allocates the given number of chunks.
    pub fn new_alloc(pre_alloc_chunks: usize, chunk_capacity: u16) -> Self {
        if chunk_capacity == 0 { panic!("Chunk capacity has to be > 0") };

        let mut chunks = Vec::with_capacity(pre_alloc_chunks);
        for _ in 0..pre_alloc_chunks {
            chunks.push(Chunk::new(chunk_capacity))
        }

        Message {
            chunk_capacity,
            chunks,
            read_cursor: 0,
            write_cursor: 0,
        }
    }

    fn new_chunk(&mut self) -> &mut Chunk {
        self.chunks.push(Chunk::new(self.chunk_capacity));
        self.chunks.last_mut().unwrap()
    }

    /// Returns a chunk with capacity or creates a new one.
    /// ```
    /// # use raio::ll::message::Message;
    /// # use std::io::Write;
    ///
    /// // no pre allocated chunks, chunks are limited to 2 byte:
    /// let mut message = Message::new_alloc(0, 2);
    ///
    /// // generate a chunk and write one byte:
    /// let mut chunk_1 = message.pull_chunk();
    /// chunk_1.write(&[0x42]);
    /// assert_eq!(message.chunks().len(), 1);
    ///
    /// // still same chunk, since it has capacity:
    /// let mut chunk_2 = message.pull_chunk();
    /// let mut buf = vec![0u8; 4];
    /// chunk_2.read(&mut buf);
    /// assert_eq!(buf, &[0x42, 0, 0, 0]);
    ///
    /// // fill chunk:
    /// chunk_2.write(&[0x43]);
    /// assert!(!chunk_2.has_capacity());
    ///
    /// // now, a new chunk has to be created:
    /// let chunk_3 = message.pull_chunk();
    /// assert!(chunk_3.has_capacity());
    /// assert_eq!(message.chunks().len(), 2);
    /// ```
    pub fn pull_chunk(&mut self) -> &mut Chunk {
        while let Some(chunk) = self.chunks.get_mut(self.write_cursor) {
            if chunk.has_capacity() {
                return self.chunks.get_mut(self.write_cursor).unwrap();
            }

            self.write_cursor += 1;
        }

        self.new_chunk()
    }

    /// Gives an iterator over the chunks of a message.
    pub fn chunks(&self) -> Iter<Chunk> {
        self.chunks.iter()
    }

    /// Packs chunk by chunk of a message according to the bolt specification. Each chunk is written
    /// into the writer by first encoding its size and then write out its content.
    /// ```
    /// # use raio::ll::message::Message;
    /// # use std::io::Write;
    /// # #[async_std::main]
    /// # async fn main() -> std::io::Result<()> {
    /// let mut message = Message::new_alloc(2, 3);
    /// let mut stream: &[u8] = &[1, 2, 3, 4, 5];
    /// message.write(&mut stream).unwrap();
    ///
    /// let mut target = Vec::with_capacity(11);
    /// message.pack(&mut target).await?;
    ///
    /// assert_eq!(target.as_slice(), &[0x00, 0x03, 1, 2, 3, 0x00, 0x02, 4, 5, 0x00, 0x00]);
    /// # Ok(())
    /// # }
    /// ```
    /// The message ends with a chunk of empty size, i.e. `0 : u16` encoded.
    pub async fn pack<T: async_std::io::Write + Unpin>(&self, writer: &mut T) -> async_std::io::Result<usize> {
        let mut written = 0;
        for chunk in &self.chunks {
            written += chunk.pack(writer).await?;
            writer.flush().await?;
        }

        writer.write(&[0, 0]).await?;
        writer.flush().await?;
        Ok(2 + written)
    }

    /// Unpacks from a `Read` into a message. Reads in the chunks as given by the reader. The set
    /// chunk capacity for new chunks of the returned `Message` is the size of the first chunk.
    pub async fn unpack<T: async_std::io::Read + Unpin>(reader: &mut T) -> async_std::io::Result<Message> {
        let mut chunks = Vec::new();
        let mut chunk = Chunk::unpack(reader).await?;
        let first_cap = chunk.capacity();
        while chunk.capacity() != 0 {
            chunks.push(chunk);
            chunk = Chunk::unpack(reader).await?;
        }

        Ok(Message {
            write_cursor: 0,
            read_cursor: 0,
            chunk_capacity: first_cap as u16,
            chunks,
        })
    }
}

impl std::io::Write for Message {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut rest = buf;
        while let Some(next_rest) = self.pull_chunk().write(rest) {
            rest = next_rest;
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl std::io::Read for Message {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut written = 0;
        while let Some(chunk) = self.chunks.get_mut(self.read_cursor)  {
            if buf.len() > written {
                written += chunk.read(&mut buf[written..]);
                if chunk.eof() {
                    self.read_cursor += 1;
                }
                if buf.len() <= written {
                    return Ok(written)
                }
            } else {
                return Ok(written);
            }
        }

        Ok(written)
    }
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for c in &self.chunks {
            write!(f, " {} |", c)?;
        }

        write!(f, " [end]")
    }
}