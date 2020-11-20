use crate::connectivity::connection::{Connection, ConnectionError};

/// A type alias for a managed pool of connections.
pub type Pool = deadpool::managed::Pool<Connection, ConnectionError>;