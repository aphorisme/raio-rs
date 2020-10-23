use packs::*;
use packs::std_structs::StdStruct;
use packs::value::extract_list_ref;

#[derive(Debug, Clone, PartialEq, PackableStruct, Pack, Unpack)]
#[tag = 0x70]
pub struct Success {
    pub metadata: Dictionary<StdStruct>
}

impl Success {
    pub fn fields(&self) -> Option<Vec<&String>> {
        self.metadata.get_property("fields").and_then(extract_list_ref)
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
    metadata: Dictionary<StdStruct>,
}

impl Failure {
    pub fn message(&self) -> Option<&String> {
        self.metadata.get_property_typed("message")
    }

    pub fn code(&self) -> Option<&String> {
        self.metadata.get_property_typed("code")
    }
}

#[derive(Debug, Clone, PartialEq, PackableStruct, Pack, Unpack)]
#[tag = 0x71]
pub struct Record {
    pub data: Vec<Value<StdStruct>>,
}

#[derive(Debug, Clone, PartialEq, PackableStructSum)]
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