use crate::messaging::response::{Record, Success};

/// The possible result of a `PULL`, which is either an `IGNORED`,
/// a `SUCCESS` with `has_more = true`, or a `SUCCESS` with `has_more = false`.
/// If the stream has finished, the resulting `SUCCESS` is returned as a marker for
/// the end of the stream (which should contain a `bookmark`).
pub enum StreamResult {
    Ignored,
    HasMore(Vec<Record>),
    Finished(Success, Vec<Record>),
}