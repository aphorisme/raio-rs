use packs::{Dictionary, Value, EncodeError, Pack};
use packs::std_structs::StdStruct;
use std::io::Write;

#[derive(Debug, Clone, PartialEq)]
pub struct Query {
    str: String,
    parameters: Dictionary<StdStruct>
}

impl Query {
    pub fn into_inner(self) -> (String, Dictionary<StdStruct>) {
        (self.str, self.parameters)
    }

    pub fn new(query: &str) -> Query {
        Query {
            str: String::from(query),
            parameters: Dictionary::new(),
        }
    }

    pub fn param<V: Into<Value<StdStruct>>>(&mut self, param: &str, value: V){
        self.parameters.add_property(param, value);
    }
}

pub(crate) fn query_pack_flat<T: Write>(query: &Query, writer: &mut T) -> Result<usize, EncodeError> {
    Ok(query.str.encode(writer)? + query.parameters.encode(writer)?)
}
