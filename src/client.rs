use auth::AuthMethod;
use crate::ll::pool::Pool;
use crate::ll::manager::Manager;
use async_std::net::ToSocketAddrs;
use crate::client::request::{Run, Pull};
use crate::client::response::{Success, Response};
use crate::client::query::Query;
use crate::ll::connection::{ConnectionConfig};
use crate::client::error::ClientError;
use crate::client::auto_commit::AutoCommit;

pub mod request;
pub mod response;
pub mod auth;
pub mod record_result;
pub mod query;
pub mod auto_commit;
pub mod error;

pub async fn open<A: AuthMethod>() {}

pub struct Client {
    pool: Pool,
}

pub struct ClientConfig {
    pub agent_name: String,
    pub agent_version: String,
    pub connection_config: ConnectionConfig,
    pub max_connections: usize,
}

impl ClientConfig {
    pub fn default(agent_name: &str, agent_version: &str) -> Self {
        ClientConfig {
            agent_name: String::from(agent_name),
            agent_version: String::from(agent_version),
            connection_config: ConnectionConfig::default(),
            max_connections: 10,
        }
    }

    pub fn max_connections(mut self, n: usize) -> Self {
        self.max_connections = n;
        self
    }

    pub fn connection_config(mut self, config: ConnectionConfig) -> Self {
        self.connection_config = config;
        self
    }
}

impl Client {
    /// Creates a client by resolving the provided endpoint (blocks!) and initializes a connection
    /// pool.
    pub fn create<A: AuthMethod, T: ToSocketAddrs>(endpoint: T, auth: A, config: ClientConfig) -> Result<Self, ClientError> {
        // resolves socket address:
        let socket_addr =
            async_std::task::block_on(async {
                endpoint.to_socket_addrs().await
            })?.next().ok_or(ClientError::CannotResolveEndpoint)?;

        // create pool manager:
        let manager =
            Manager::new(
                socket_addr,
                auth,
                &config.agent_name,
                &config.agent_version,
                &config.connection_config,
            );

        // create pool:
        let pool =
            Pool::new(manager, config.max_connections);

        Ok(Client {
            pool
        })
    }

    /// Creates an auto-commit which can be either further configured or committed to run the provided
    /// query.
    pub fn run(&mut self, query: Query) -> AutoCommit {
        AutoCommit::create(&mut self.pool, query)
    }
}