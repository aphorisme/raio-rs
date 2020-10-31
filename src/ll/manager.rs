use async_std::net::{SocketAddr};
use crate::ll::connection::{Connection, ConnectionError, ConnectionConfig, State};
use deadpool::managed::{RecycleResult, RecycleError};
use async_trait::async_trait;
use crate::ll::version::Version;
use crate::client::auth::{AuthData, AuthMethod};

/// Handles the opening and recycling of connections.
pub struct Manager {
    endpoint: SocketAddr,
    connection_config: ConnectionConfig,
    authentication: AuthData,
    agent_name: String,
    agent_version: String,
}

impl Manager {
    pub fn new<A: AuthMethod>(
        endpoint: SocketAddr,
        auth: A,
        agent_name: &str,
        agent_version: &str,
        connection_config: &ConnectionConfig) -> Self {
        Manager {
            endpoint,
            connection_config: *connection_config,
            authentication: auth.into_auth_data(),
            agent_version: String::from(agent_version),
            agent_name: String::from(agent_name),
        }
    }
}

#[async_trait]
impl deadpool::managed::Manager<Connection, ConnectionError> for Manager {
    async fn create(&self) -> Result<Connection, ConnectionError> {
        // connect:
        let mut connection = Connection::connect(self.endpoint, self.connection_config).await?;

        // handshake with fixed supported versions:
        let _ = connection.handshake(
            &[
                Version::new(4,1),
                Version::new(4,0),
                Version::empty(),
                Version::empty()]).await?;

        // authenticate:
        let _ = connection
            .auth_hello(
                &self.agent_name,
                &self.agent_version,
                &self.authentication.scheme,
                &self.authentication.principal,
                &self.authentication.credentials).await?;

        Ok(connection)
    }

    async fn recycle(&self, obj: &mut Connection) -> RecycleResult<ConnectionError> {
        match obj.state {
            State::Ready => {
                obj.reset().await?;
                Ok(())
            },
            _ => Err(
                RecycleError::Message(String::from("Cannot recycle connection, connection not established or closed.")))
        }
    }
}