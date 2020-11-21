use deadpool::managed::Object;
use crate::connectivity::connection::{Connection, ConnectionError};
use crate::messaging::query::Query;
use crate::client::error::ClientError;
use crate::client::record_result::RecordResult;
use crate::messaging::request::{Run, Amount, Qid, Commit, RollBack};
use crate::connectivity::stream_result::StreamResult;
use crate::messaging::bookmark::Bookmark;

pub struct Transaction {
    pub(crate) connection: Object<Connection, ConnectionError>
}

impl Transaction {
    pub async fn run(&mut self, query: &Query) -> Result<Vec<RecordResult>, ClientError> {
        self.connection.send(&Run::new(query)).await?;
        let mut run_success = self.connection.recv_success().await?;
        
        let qid = 
            run_success.extract_qid().ok_or(ClientError::NoQidInformation)?;
        let fields = 
            run_success.extract_fields().ok_or(ClientError::NoFieldInformation)?;
        
        let pull_result =
            self.connection.pull(Amount::All, Qid::Exact(qid)).await?;
        
        match pull_result {
            StreamResult::Finished(_, records) => 
                RecordResult::from_results(&fields, records),
            
            _ => Err(ClientError::StreamStillOpen)
        }
    }
    
    pub async fn commit(mut self) -> Result<Bookmark, ClientError> {
        self.connection.send(&Commit {}).await?;
        Bookmark::from_success(
            self.connection.recv_success().await?
        )
    }
    
    pub async fn rollback(mut self) -> Result<(), ClientError> {
        self.connection.send(&RollBack {}).await?;
        Ok(())
    }
}