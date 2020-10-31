use crate::client::error::ClientError;
use crate::client::query::Query;
use crate::client::record_result::RecordResult;
use crate::client::request::{Pull};
use crate::client::response::{Response, Success};
use crate::ll::pool::Pool;

pub struct AutoCommitResult {
    success: Success,
    records: Vec<RecordResult>,

}

impl AutoCommitResult {
    pub fn bookmark(&self) -> &String {
        // AutoCommitResult is created from `AutoCommit::commit` which checks if the bookmark field
        // is set.
        self.success.bookmark().unwrap()
    }

    pub fn records(&self) -> &Vec<RecordResult> {
        &self.records
    }
}

pub struct AutoCommit<'pool> {
    pool: &'pool mut Pool,
    after_commits: Vec<String>,
    query: Query,
}

impl<'pool> AutoCommit<'pool> {
    pub(crate) fn create(pool: &'pool mut Pool, query: Query) -> Self {
        AutoCommit {
            pool,
            after_commits: Vec::new(),
            query,
        }
    }

    /// Can be used to establish [Causal Consistency](https://neo4j.com/developer/kb/when-to-use-bookmarks/)
    /// using bookmarks.
    pub fn after(mut self, previous: &AutoCommitResult) -> Self {
        self.after_commits.push(previous.bookmark().clone());
        self
    }

    /// Commits the AutoCommit and consumes it. This sends a `RUN` request and receives the hole `RECORD`
    /// stream. Requires connection resources and frees them afterwards.
    pub async fn commit(self) -> Result<AutoCommitResult, ClientError> {
        let mut connection = self.pool.get().await?;

        // send run:
        let mut run = self.query.into_run();
        run.add_bookmarks(self.after_commits);
        connection.send(run).await?;

        // receive fields:
        let fields =
            connection
                .recv::<Success>()
                .await?
                .extract_fields()
                .ok_or(ClientError::NoFieldInformation)?;

        // pull all:
        connection.send(Pull::all_from_last()).await?;

        // receive all records:
        let mut results = Vec::new();
        // a successful stream ends with a 'SUCCESS' which contains the bookmark of the commit
        // or fails with an error.
        let success =
            loop {
                let response = connection.recv::<Response>().await?;
                match response {
                    Response::Record(r) =>
                        results.push(RecordResult::new(&fields, r)?),
                    Response::Success(s) => {
                        if s.has_more() {
                            // unexpected has_more?
                            connection.send(Pull::all_from_last()).await?;
                        } else {
                            break s;
                        }
                    }
                    Response::Failure(mut f) =>
                        return Err(ClientError::FailureResponse(f.code().clone(), f.message().clone())),
                    Response::Ignored(_) =>
                        return Err(ClientError::CommitPullIgnored)
                }
            };

        // protocol sanity check:
        if !success.has_bookmark() {
            return Err(ClientError::CannotExtractBookmarkFromAutoCommit);
        }

        Ok(AutoCommitResult {
            success,
            records: results,
        })
    }
}

