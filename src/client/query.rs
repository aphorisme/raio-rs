use packs::{Dictionary, Value};
use packs::std_structs::StdStruct;
use crate::client::request::Run;

pub struct Query {
    str: String,
    parameters: Dictionary<StdStruct>
}

impl Query {
    pub fn into_run(self) -> Run {
        Run {
            query: self.str,
            parameters: self.parameters,
            extra: Dictionary::new(),
        }
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