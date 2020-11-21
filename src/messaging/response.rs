use packs::std_structs::{StdStructPrimitive, StdStruct};
use packs::*;

#[derive(Debug, Clone, PartialEq, Unpack)]
#[tag = 0x70]
pub struct Success {
    pub metadata: Dictionary<StdStructPrimitive>
}

impl Success {
    pub fn fields(&self) -> Option<Vec<&String>> {
        self.metadata.get_property("fields").and_then(extract_list_ref)
    }

    pub fn extract_fields(&mut self) -> Option<Vec<String>> {
        self.metadata.extract_property("fields").and_then(extract_list)
    }

    pub fn extract_qid(&mut self) -> Option<i64> {
        self.metadata.extract_property_typed("qid")
    }

    pub fn into_raw_bookmark(mut self) -> Option<String> {
        self.metadata.extract_property_typed("bookmark")
    }

    /// This denotes if there are more records to pull. According to spec, this defaults to
    /// false, even if the property isn't set.
    pub fn has_more(&self) -> bool {
        if let Some(b) = self.metadata.get_property_typed("has_more") {
            *b
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, PartialEq, Unpack)]
#[tag = 0x7E]
pub struct Ignored {}

#[derive(Debug, Clone, PartialEq, Unpack)]
#[tag = 0x7F]
pub struct Failure {
    metadata: Dictionary<StdStructPrimitive>,
}

impl Failure {
    pub fn message(&mut self) -> String {
        self.metadata.extract_property_typed("message").unwrap_or(String::from("<unknown>"))
    }

    pub fn code(&mut self) -> String {
        self.metadata.extract_property_typed("code").unwrap_or(String::from("<unknown>"))
    }
}

#[derive(Debug, Clone, PartialEq, Unpack)]
#[tag = 0x71]
pub struct Record {
    pub data: Vec<Value<StdStruct>>,
}

#[derive(Debug, Clone, PartialEq, Unpack)]
pub enum Response {
    #[tag = 0x70]
    Success(Success),
    #[tag = 0x7E]
    Ignored(Ignored),
    #[tag = 0x7F]
    Failure(Failure),
    #[tag = 0x71]
    Record(Record),
}
