use crate::net::ll::{VersionHandshake, MAGIC_NUMBER};
use crate::net::{Error, QueryResult, Response};
use crate::packing::ll::{BoltRead, BoltWrite};
use crate::packing::{Hello, MessageRead, MessageWrite, Packable, PullAll, Run, Value, ValueMap};
use std::io::Write;
use std::net::{TcpStream, ToSocketAddrs};

pub const VERSION: u32 = 3;
pub const USER_AGENT: &str = "raio-sync/alpha";

/// A `neo4j/bolt` client, which can be [`Client:open`] to connect to a neo4j database and then
/// be used with [`Client:run`] to ask the server to execute a cypher query.
///
/// [`Client:run`]: sync/struct.Client.html#run
/// [`Client:open`]: sync/struct.Client.html#open
pub struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn send<V: Packable>(&mut self, obj: &V) -> Result<usize, Error> {
        Ok(self.stream.write_as_message(obj)?)
    }

    pub fn recv(&mut self) -> Result<Response, Error> {
        let response: Response = self.stream.read_from_message()?;
        Ok(response)
    }

    pub fn recv_success(&mut self) -> Result<ValueMap<Value>, Error> {
        let response: Response = self.stream.read_from_message()?;
        match response {
            Response::Success(vm) => Ok(vm),
            _ => Err(Error::UnexpectedResponse(response, "Success")),
        }
    }

    pub fn open<A: ToSocketAddrs>(addr: A, user: &str, pass: &str) -> Result<Client, Error> {
        let mut stream = TcpStream::connect(addr)?;

        // open with magic number and version
        stream.write_all(&MAGIC_NUMBER)?;
        stream.bolt_write(VersionHandshake::just_version(VERSION))?;

        // hard check version:
        let server_version: u32 = stream.bolt_read()?;
        if server_version != VERSION {
            return Err(Error::UnsupportedServerVersion(server_version));
        }

        let mut client = Client { stream };

        // send init message:
        let mut map = ValueMap::with_capacity(3);
        map.insert_value("user_agent", USER_AGENT);
        map.insert_value("scheme", "basic");
        map.insert_value("principal", user);
        map.insert_value("credentials", pass);

        client.send(&Hello { auth_token: map })?;

        // only an `Success {}` is a positive answer:
        let _ = client.recv_success()?;

        Ok(client)
    }

    pub fn run(&mut self, statement: &Run) -> Result<QueryResult, Error> {
        self.send(statement)?;
        self.send(&PullAll {})?;

        let metadata = self.recv_success()?;

        let mut result = QueryResult::begin(metadata)?;

        loop {
            let response = self.recv()?;
            match response {
                Response::Success(final_fields) => {
                    result.end(final_fields);
                    return Ok(result);
                }

                Response::Record(record) => result.push(record)?,

                _ => return Err(Error::UnexpectedResponse(response, "Success or Record")),
            }
        }
    }
}
