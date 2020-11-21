use crate::messaging::request::{Run};
use crate::messaging::commit_prepare::CommitPrepare;
use crate::messaging::query::Query;
use crate::messaging::bookmark::Bookmark;
use crate::client::record_result::RecordResult;
use crate::messaging::response::{Success, Record};
use crate::client::error::ClientError;

/// A thin wrapper around a `RUN` message in an auto-commit context. Can be used to prepare a
/// common auto-commit, i.e. a query and a few commit options.
/// ```
/// # use raio::messaging::query::Query;
/// # use raio::client::auto_commit::AutoCommit;
/// # use raio::messaging::commit_prepare::CommitMode;
/// let mut query = Query::new("RETURN $x as x");
/// query.param("x", 42);
///
/// let mut auto_commit = AutoCommit::new(&query);
/// auto_commit
///     .prepare()
///     .set_db("my_database")
///     .set_mode(Some(CommitMode::Read))
///     .set_timeout(Some(10));
/// ```
/// The `run` function of a [`Client`](crate::client::Client) runs an `AutoCommit`.
pub struct AutoCommit<'a> {
    run: Run<'a>
}

impl<'a> AutoCommit<'a> {
    /// Creates a new `AutoCommit` out of a query. Does not set any `CommitPrepare` options like
    /// timeout or database name.
    pub fn new(query: &'a Query) -> Self {
        let run = Run::new(query);
        AutoCommit {
            run
        }
    }

    /// Gives access to the `CommitPrepare` of the underlying `RUN` message to set commit
    /// settings.
    pub fn prepare(&mut self) -> &mut CommitPrepare {
        self.run.commit_prepare()
    }

    /// Return the `AutoCommit` as a request, which can be sent to the server.
    pub fn request(&self) -> &Run {
        &self.run
    }
}

pub struct AutoCommitResult {
    bookmark: Bookmark,
    records: Vec<RecordResult>,
}

impl AutoCommitResult {
    /// Creates a new `CommitResult` from a final `SUCCESS` message, and a list of `RECORD`s.
    pub fn new(fields: &[String], stream_end: Success, records: Vec<Record>) -> Result<Self, ClientError> {
        let bookmark = Bookmark::from_success(stream_end)?;

        // build up record results:
        let records = RecordResult::from_results(fields, records)?;


        Ok(AutoCommitResult {
            bookmark,
            records,
        })
    }

    pub fn bookmark(&self) -> &Bookmark {
        &self.bookmark
    }

    pub fn records(&self) -> &Vec<RecordResult> {
        &self.records
    }

    pub fn into_records(self) -> Vec<RecordResult> {
        self.records
    }
}
