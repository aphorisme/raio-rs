use async_std::net::{TcpStream, ToSocketAddrs};
use thiserror::Error;
use packs::{Pack, Unpack};
use async_std::io::{BufReader, BufWriter};
use async_std::prelude::*;
use crate::client::response::Response;
use crate::ll::message::Message;
use crate::ll::version::Version;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO Error: {0}")]
    IOError(#[from] async_std::io::Error),
    #[error("Cannot pack message: {0}")]
    PackingError(#[from] packs::EncodeError),
    #[error("Cannot unpack message: {0}")]
    UnpackingError(#[from] packs::DecodeError),
}

pub struct Config {
    initial_chunks: usize,
    chunk_capacity: u16,
}

impl Config {
    pub fn default() -> Self {
        Config {
            initial_chunks: 1,
            chunk_capacity: 1400,
        }
    }
}

pub struct Connection {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
    config: Config,
}

impl Connection {
    /// Connects to provided address and returns this established connection. Does **not** send or
    /// receive anything.
    pub async fn connect<A: ToSocketAddrs>(addr: A) -> Result<Connection, Error> {
        let stream = TcpStream::connect(addr).await?;
        let reader = BufReader::new(stream.clone());
        let writer = BufWriter::new(stream);
        Ok(Connection {
            reader,
            writer,
            config: Config::default(),
        })
    }

    /// Performs a handshake as specified in the bolt protocol. A successful handshake ends in a
    /// negotiated version between client and server.
    pub async fn handshake(&mut self, versions: &[Version; 4]) -> Result<Version, Error> {
        self.writer.write(&[0x60, 0x60, 0xB0, 0x17]).await?;
        for v in versions {
            self.writer.write(&v.encode()).await?;
        }

        self.writer.flush().await?;

        let mut buffer = [0u8, 0, 0, 0];
        self.reader.read_exact(&mut buffer).await?;
        Ok(Version::decode(&buffer))
    }

    /// Sends any value which can be packed into a message, using PackStream,
    /// (c.f. [`packable`](packs::packable)). It returns the number of sent bytes.
    pub async fn send<V: Pack<Message>>(&mut self, value: V) -> Result<usize, Error> {
        let mut message =
            Message::new_alloc(
                self.config.initial_chunks,
                self.config.chunk_capacity);
        value.encode(&mut message)?;
        Ok(message.pack(&mut self.writer).await?)
    }

    pub async fn recv_response(&mut self) -> Result<Response, Error> {
        let mut message = Message::unpack(&mut self.reader).await?;
        Ok(Response::decode(&mut message)?)
    }
}
