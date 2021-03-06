use auth::AuthMethod;

use crate::client::auto_commit::{AutoCommit, AutoCommitResult};
use crate::client::error::ClientError;
use crate::messaging::query::Query;
use crate::connectivity::connection::ConnectionConfig;
use crate::connectivity::manager::Manager;
use crate::connectivity::pool::Pool;
use crate::connectivity::stream_result::StreamResult;
use crate::messaging::request::{Amount, Qid, Begin};
use crate::messaging::bookmark::Bookmark;
use crate::messaging::commit_prepare::CommitPrepare;
use crate::client::transaction::Transaction;

pub mod auth;
pub mod auto_commit;
pub mod error;
pub mod record_result;
pub mod transaction;

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
    /// Creates a client, initializes a connection pool and connection manager, but does not connect
    /// anything yet.
    pub fn create<A: AuthMethod>(
        endpoint: &str,
        auth: A,
        config: ClientConfig,
    ) -> Self {
        // create pool manager:
        let manager = Manager::new(
            endpoint.to_owned(),
            auth,
            &config.agent_name,
            &config.agent_version,
            &config.connection_config,
        );

        // create pool:
        let pool = Pool::new(manager, config.max_connections);

        Client { pool }
    }

    /// Runs an `AutoCommit` which allows for commit preparation and is reusable.
    pub async fn run<'a>(&self, auto_commit: &AutoCommit<'a>) -> Result<AutoCommitResult, ClientError> {
        let mut connection = self.pool.get().await?;

        // send a `RUN` and receive a `SUCCESS` containing the fields:
        connection.send(auto_commit.request()).await?;
        let mut stream_begin = connection.recv_success().await?;
        let fields = stream_begin
            .extract_fields()
            .ok_or(ClientError::NoFieldInformation)?;

        // Pull all from last and expect the stream end:
        match connection.pull(Amount::All, Qid::Last).await? {
            StreamResult::Finished(stream_end, records) => {
                Ok(AutoCommitResult::new(&fields, stream_end, records)?)
            }

            _ => Err(ClientError::StreamStillOpen),
        }
    }

    /// Runs the provided query as an auto-commit after the provided bookmark and returns a result.
    pub async fn query_after(&self, query: &Query, before: Bookmark) -> Result<AutoCommitResult, ClientError> {
        let mut auto_commit = AutoCommit::new(query);
        auto_commit.prepare().add_bookmark(before);
        self.run(&auto_commit).await
    }

    /// Runs the provided query as an auto-commit and returns a result.
    pub async fn query(&self, query: &Query) -> Result<AutoCommitResult, ClientError> {
        self.run(&AutoCommit::new(query)).await
    }
    
    /// Opens a transaction with the provided settings.
    pub async fn begin(&self, settings: CommitPrepare) -> Result<Transaction, ClientError> {
        let mut connection = self.pool.get().await?;
        
        connection.send(&Begin::new(settings)).await?;
        let _ = connection.recv_success().await?;
        
        Ok(Transaction {
            connection
        })
    }
}
