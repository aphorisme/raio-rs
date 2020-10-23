use async_std::net::ToSocketAddrs;
use crate::ll::connection::{Connection, Error};
use deadpool::managed::RecycleResult;

/// Handles the opening and recycling of connections.
pub struct Manager<A> {
    endpoint: A,
}

/*
#[async_trait]
impl<A: ToSocketAddrs> deadpool::managed::Manager<Connection, Error> for Manager<A> {
    async fn create(&self) -> Result<Connection, Error> {
        todo!()
    }

    async fn recycle(&self, obj: &mut Connection) -> RecycleResult<Error> {
        todo!()
    }
}
 */