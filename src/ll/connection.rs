use async_std::net::{TcpStream, ToSocketAddrs};
use thiserror::Error;
use packs::{Pack, Unpack};
use async_std::io::{BufReader, BufWriter};
use async_std::prelude::*;
use crate::client::response::{Response, Success};
use crate::ll::message::Message;
use crate::ll::version::Version;
use crate::client::request::{Hello, GoodBye, Reset};

#[derive(Debug, Error)]
/// Possible connection errors, which can happen during connecting, receiving or sending. It also
/// incorporates encoding and decoding errors.
pub enum ConnectionError {
    #[error("IO Error: {0}")]
    IOError(#[from] async_std::io::Error),
    #[error("Cannot pack message: {0}")]
    PackingError(#[from] packs::EncodeError),
    #[error("Cannot unpack message: {0}")]
    UnpackingError(#[from] packs::DecodeError),
    #[error("None of {0:?} are supported by the server.")]
    VersionsNotSupportedByServer([Version; 4]),
    #[error("Authentication failed with code '{1}': {0}")]
    AuthenticationError(String, String),
    #[error("Unexpected response")]
    UnexpectedResponse,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ConnectionConfig {
    initial_chunks: usize,
    chunk_capacity: u16,
}

impl ConnectionConfig {
    pub fn default() -> Self {
        ConnectionConfig {
            initial_chunks: 1,
            chunk_capacity: 1400,
        }
    }

    pub fn initial_chunks(mut self, n: usize) -> Self {
        self.initial_chunks = n;
        self
    }

    pub fn chunk_capacity(mut self, n: u16) -> Self {
        self.chunk_capacity = n;
        self
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum State {
    Connected,
    Ready,
    Closed,
}

/// A `Connection` is the low level abstraction of a bolt protocol connection. It takes care of the
/// sending and receiving of [`Request`](crate::client::request) and [`Response`](crate::client::response::Response)
/// by encoding and packing any request into a [`Message`](crate::ll::message::Message) and vice versa.
pub struct Connection {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
    config: ConnectionConfig,
    pub state: State,
}

impl Connection {
    /// Connects to provided address and returns this established connection. Does **not** send or
    /// receive anything.
    pub async fn connect<A: ToSocketAddrs>(addr: A, config: ConnectionConfig) -> Result<Connection, ConnectionError> {
        let stream = TcpStream::connect(addr).await?;
        let reader = BufReader::new(stream.clone());
        let writer = BufWriter::new(stream);
        Ok(Connection {
            reader,
            writer,
            config,
            state: State::Connected,
        })
    }

    /// Performs a handshake as specified in the bolt protocol. A successful handshake ends in a
    /// negotiated version between the client and a server.
    pub async fn handshake(&mut self, versions: &[Version; 4]) -> Result<Version, ConnectionError> {
        self.writer.write(&[0x60, 0x60, 0xB0, 0x17]).await?;
        for v in versions {
            self.writer.write(&v.encode()).await?;
        }

        self.writer.flush().await?;

        // server responses with a `Version`:
        let mut buffer = [0u8, 0, 0, 0];
        self.reader.read_exact(&mut buffer).await?;
        let version = Version::decode(&buffer);
        if version.is_empty() {
            self.state = State::Closed;
            Err(ConnectionError::VersionsNotSupportedByServer(*versions))
        } else {
            self.state = State::Ready;
            Ok(version)
        }
    }

    /// Sends any value which can be packed into a message, using PackStream,
    /// (c.f. [`packable`](packs::packable)). It returns the number of sent bytes.
    pub async fn send<V: Pack<Message>>(&mut self, value: V) -> Result<usize, ConnectionError> {
        let mut message =
            Message::new_alloc(
                self.config.initial_chunks,
                self.config.chunk_capacity);
        value.encode(&mut message)?;
        Ok(message.pack(&mut self.writer).await?)
    }

    /// Tries to receive any value which can be unpacked from a message, using PackStream. These
    /// are usually the [`responses`](crate::client::response).
    pub async fn recv<T: Unpack<Message>>(&mut self) -> Result<T, ConnectionError> {
        let mut message = Message::unpack(&mut self.reader).await?;
        Ok(T::decode(&mut message)?)
    }

    /// A higher-level function which sends a `HELLO` request to authenticate the connection. Waits
    /// for a response and reports any non `SUCCESS` as an error.
    pub async fn auth_hello(&mut self, agent_name: &str, version: &str, auth_scheme: &str, auth_principal: &str, auth_credentials: &str) -> Result<Success, ConnectionError> {
        self.send(
            Hello::new(agent_name, version, auth_scheme, auth_principal, auth_credentials))
            .await?;

        let response = self.recv::<Response>().await?;
        match response {
            Response::Success(s) => Ok(s),
            Response::Failure(mut f) => {
                self.state = State::Closed;
                Err(ConnectionError::AuthenticationError(f.message().clone(), f.code().clone()))
            },

            _ => {
                self.state = State::Closed;
                Err(ConnectionError::UnexpectedResponse)
            }
        }
    }

    pub async fn goodbye(&mut self) -> Result<(), ConnectionError> {
        self.send(GoodBye {}).await?;
        Ok(())
    }

    pub async fn reset(&mut self) -> Result<(), ConnectionError> {
        self.send(Reset {}).await?;
        Ok(())
    }
}
