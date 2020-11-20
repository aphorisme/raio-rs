use async_std::io::{BufReader, BufWriter};
use async_std::net::{TcpStream, ToSocketAddrs};
use async_std::prelude::*;
use packs::{Pack, Unpack};
use thiserror::Error;

use crate::connectivity::stream_result::StreamResult;
use crate::connectivity::version::Version;
use crate::messaging::response::{Failure, Success, Response};
use crate::messaging::request::{Hello, Pull, GoodBye, Reset, Amount, Qid};
use crate::messaging::message::Message;

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
    #[error("Failure response '{0}' with message: '{1}")]
    FailureResponse(String, String),
}

impl From<Failure> for ConnectionError {
    fn from(mut f: Failure) -> Self {
        ConnectionError::FailureResponse(f.code().clone(), f.message().clone())
    }
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
/// by encoding and packing any request into a [`Message`](crate::connectivity::message::Message) and vice versa.
pub struct Connection {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
    config: ConnectionConfig,
    state: State,
}

impl Connection {
    pub fn state(&self) -> State {
        self.state
    }

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
    pub async fn send<V: Pack>(&mut self, value: &V) -> Result<usize, ConnectionError> {
        let mut message =
            Message::new_alloc(
                self.config.initial_chunks,
                self.config.chunk_capacity);
        value.encode(&mut message)?;
        Ok(message.pack(&mut self.writer).await?)
    }

    /// Tries to receive any value which can be unpacked from a message, using PackStream. These
    /// are usually the [`responses`](crate::client::response).
    pub async fn recv<T: Unpack>(&mut self) -> Result<T, ConnectionError> {
        let mut message = Message::unpack(&mut self.reader).await?;
        Ok(T::decode(&mut message)?)
    }

    /// Tries to receive a `SUCCESS`. Turns a `FAILURE` into a `ConnectionError` and every other
    /// response to an `UnexpectedResponse`.
    pub async fn recv_success(&mut self) -> Result<Success, ConnectionError> {
        let response = self.recv::<Response>().await?;
        match response {
            Response::Success(s) => Ok(s),
            Response::Failure(f) => Err(f.into()),
            _ => Err(ConnectionError::UnexpectedResponse),
        }
    }

    /// A higher-level function which sends a `HELLO` request to authenticate the connection. Waits
    /// for a response and reports any non `SUCCESS` as an error.
    pub async fn auth_hello(&mut self, agent_name: &str, version: &str, auth_scheme: &str, auth_principal: &str, auth_credentials: &str) -> Result<Success, ConnectionError> {
        self.send(
            &Hello::new(agent_name, version, auth_scheme, auth_principal, auth_credentials))
            .await?;

        let response = self.recv::<Response>().await?;
        match response {
            Response::Success(s) => Ok(s),
            Response::Failure(mut f) => {
                self.state = State::Closed;
                Err(ConnectionError::AuthenticationError(f.message().clone(), f.code().clone()))
            }

            _ => {
                self.state = State::Closed;
                Err(ConnectionError::UnexpectedResponse)
            }
        }
    }

    /// A higher-level function which sends a `Pull` and receives all `RECORD` until either an error
    /// occurs, a `FAILURE` was received, or a `SUCCESS` denotes the (intermediate) end of the pull.
    /// A pull might got ignored, in this case, the [`StreamResult`](crate::connectivity::stream_result) shows
    /// this.
    pub async fn pull(&mut self, n: Amount, qid: Qid) -> Result<StreamResult, ConnectionError> {
        self.send(&Pull::new(n, qid)).await?;

        // receive all records:
        let mut results = Vec::new();
        // a successful stream ends with a 'SUCCESS' which contains the bookmark of the commit
        // or fails with an error.
        loop {
            let response = self.recv::<Response>().await?;
            match response {
                Response::Record(r) =>
                    results.push(r),
                Response::Success(s) => {
                    return if s.has_more() {
                        Ok(StreamResult::HasMore(results))
                    } else {
                        Ok(StreamResult::Finished(s, results))
                    }
                }
                Response::Failure(f) =>
                    return Err(f.into()),
                Response::Ignored(_) =>
                    return Ok(StreamResult::Ignored),
            }
        }
    }

    pub async fn goodbye(&mut self) -> Result<(), ConnectionError> {
        self.send(&GoodBye {}).await?;
        Ok(())
    }

    pub async fn reset(&mut self) -> Result<(), ConnectionError> {
        self.send(&Reset {}).await?;
        Ok(())
    }
}
