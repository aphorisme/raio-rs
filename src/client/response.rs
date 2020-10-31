use packs::*;
use packs::std_structs::{StdStructPrimitive, StdStruct};
use packs::value::{extract_list_ref, extract_list};

#[derive(Debug, Clone, PartialEq, PackableStruct, Pack, Unpack)]
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

    pub fn bookmark(&self) -> Option<&String> {
        self.metadata.get_property_typed("bookmark")
    }

    pub fn has_bookmark(&self) -> bool {
        self.metadata.has_property("bookmark")
    }

    pub fn qid(&self) -> Option<&i64> {
        self.metadata.get_property_typed("qid")
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

#[derive(Debug, Clone, PartialEq, PackableStruct, Pack, Unpack)]
#[tag = 0x7E]
pub struct Ignored {}

#[derive(Debug, Clone, PartialEq, PackableStruct, Pack, Unpack)]
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

#[derive(Debug, Clone, PartialEq, PackableStruct, Pack, Unpack)]
#[tag = 0x71]
pub struct Record {
    pub data: Vec<Value<StdStruct>>,
}

#[derive(Debug, Clone, PartialEq, PackableStructSum, Pack, Unpack)]
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

impl Response {
    pub fn is_success(&self) -> bool {
        match self {
            Response::Success(_) => true,
            _ => false,
        }
    }
}