use deadpool::managed::Object;
use crate::connectivity::connection::{ConnectionError, Connection};
use crate::messaging::query::Query;
use crate::messaging::request::{RunInTx, Commit, RollBack, Amount, Qid, Discard};
use crate::client::error::ClientError;
use crate::connectivity::stream_result::StreamResult;
use crate::client::record_result::RecordResult;
use crate::messaging::bookmark::Bookmark;

pub struct Transaction {
    connection: Object<Connection, ConnectionError>,
}

impl Transaction {
    pub(crate) fn new(connection: Object<Connection, ConnectionError>) -> Self {
        Transaction {
            connection,
        }
    }

    pub async fn run<'a>(&'a mut self, query: Query) ->  Result<TransactionQuery<'a>, ClientError> {
        let (query_str, params) = query.into_inner();

        // send the query and receive a response with the corresponding qid:
        self.connection.send(&RunInTx::new(query_str, params)).await?;
        let mut success = self.connection.recv_success().await?;

        Ok(TransactionQuery {
            qid: success.extract_qid().ok_or(ClientError::NoQidInformation)?,
            fields: success.extract_fields().ok_or(ClientError::NoFieldInformation)?,
            connection: &mut self.connection
        })

    }

    pub async fn commit(mut self) -> Result<Bookmark, ClientError> {
        self.connection.send(&Commit {}).await?;
        let success = self.connection.recv_success().await?;
        Bookmark::from_success(success)
    }

    pub async fn rollback(mut self) -> Result<(), ClientError> {
        self.connection.send(&RollBack {}).await?;
        let _ = self.connection.recv_success().await?;
        Ok(())
    }
}

pub struct TransactionQuery<'a> {
    qid: i64,
    fields: Vec<String>,
    connection: &'a mut Object<Connection, ConnectionError>,
}

impl<'a> TransactionQuery<'a> {
    pub async fn pull(self) -> Result<Vec<RecordResult>, ClientError> {
        let pull_response =
            self.connection.pull(Amount::All, Qid::Exact(self.qid)).await?;
        match pull_response {
            StreamResult::Finished(_, records) => {
                Ok(RecordResult::from_results(&self.fields, records)?)
            },
            _ => Err(ClientError::StreamStillOpen)
        }
    }

    pub async fn discard(self) -> Result<(), ClientError> {
        self.connection.send(&Discard::new(Amount::All, Qid::Exact(self.qid))).await?;
        Ok(())
    }
}
