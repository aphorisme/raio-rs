use crate::messaging::response::Success;
use crate::client::error::ClientError;

#[derive(Debug, Clone, PartialEq)]
pub struct Bookmark(String);

impl Bookmark {
    pub fn from_success(s: Success) -> Result<Self, ClientError> {
        s.into_raw_bookmark().ok_or(ClientError::NoBookmarkInformationInCommit).map(Bookmark)
    }

    pub fn value(&self) -> &String {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}
