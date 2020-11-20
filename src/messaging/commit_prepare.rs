use packs::{Dictionary, Pack, EncodeError, Marker};
use packs::std_structs::StdStruct;
use packs::utils::encode_property;
use crate::messaging::bookmark::Bookmark;
use std::io::Write;

#[derive(Debug, Clone, PartialEq)]
/// A structure which makes the `extra: Dictionary` of a `RUN` (auto-commit), or a `BEGIN` explicit.
/// It contains all possible parameters as optional members. Using `encode` from `Pack` on it packs
/// this structure as if it was an `Dictionary<StdStruct>` to follow the bolt protocol:
/// ```
/// # use raio::messaging::commit_prepare::{CommitPrepare, CommitMode};
/// use packs::{Unpack, Pack, Dictionary};
/// use packs::std_structs::StdStruct;
///
/// let mut cp =
///     CommitPrepare::new();
/// cp
///     .set_mode(Some(CommitMode::Read))
///     .set_timeout(Some(42));
///
/// let mut buf : Vec<u8> = Vec::new();
/// cp.encode(&mut buf).unwrap();
///
/// let dict = <Dictionary<StdStruct>>::decode(&mut buf.as_slice()).unwrap();
///
/// assert_eq!(dict.get_property_typed("mode"), Some(&String::from("r")));
/// assert_eq!(dict.get_property_typed("tx_timeout"), Some(&42));
/// assert!(!dict.has_property("tx_metadata"));
/// assert!(!dict.has_property("db"));
/// assert!(!dict.has_property("bookmarks"));
/// ```
pub struct CommitPrepare {
   pub bookmarks: Vec<String>,
   pub tx_timeout: Option<i64>,
   pub tx_metadata: Dictionary<StdStruct>,
   pub mode: Option<CommitMode>,
   pub db: Option<String>,
}

impl CommitPrepare {
   pub fn new() -> Self {
      CommitPrepare {
         bookmarks: Vec::new(),
         tx_timeout: None,
         tx_metadata: Dictionary::new(),
         mode: None,
         db: None,
      }
   }

   pub fn set_timeout(&mut self, secs: Option<i64>) -> &mut Self {
      self.tx_timeout = secs;
      self
   }

   pub fn set_mode(&mut self, mode: Option<CommitMode>) -> &mut Self {
      self.mode = mode;
      self
   }

   pub fn set_db(&mut self, db_name: &str) -> &mut Self {
      self.db = Some(String::from(db_name));
      self
   }

   pub fn metadata(&mut self) -> &mut Dictionary<StdStruct> {
      &mut self.tx_metadata
   }

   pub fn add_bookmark(&mut self, bookmark: Bookmark) -> &mut Self {
      self.bookmarks.push(bookmark.into_inner());
      self
   }
}

impl Pack for CommitPrepare {
   fn encode<T: Write>(&self, writer: &mut T) -> Result<usize, EncodeError> {
      let fields =
         if self.bookmarks.len() > 0 { 1 } else { 0 } +
             if self.tx_timeout.is_some() { 1 } else { 0 } +
             if self.tx_metadata.len() > 0 { 1 } else { 0 } +
             if self.mode.is_some() { 1 } else { 0 } +
             if self.db.is_some() { 1 } else { 0 };
      let mut written = Marker::TinyDictionary(fields).encode(writer)?;

      if self.bookmarks.len() > 0 {
         written += encode_property("bookmarks", &self.bookmarks, writer)?;
      }

      if let Some(tx_timeout) = self.tx_timeout {
         written += encode_property("tx_timeout", &tx_timeout, writer)?;
      }

      if self.tx_metadata.len() > 0 {
         written += encode_property("tx_metadata", &self.tx_metadata, writer)?;
      }

      if let Some(mode) = &self.mode {
         written += encode_property("mode", mode, writer)?;
      }

      if let Some(db) = &self.db {
         written += encode_property("db", db, writer)?;
      }

      Ok(written)
   }
}

#[derive(Debug, Copy, Clone, PartialEq)]
/// The different commit modes, which is either read or write.
pub enum CommitMode {
   Read,
   Write,
}

impl Pack for CommitMode {
   fn encode<T: Write>(&self, writer: &mut T) -> Result<usize, EncodeError> {
      let str = match self {
         Self::Read => String::from("r"),
         Self::Write => String::from("w"),
      };
       str.encode(writer)
   }
}
