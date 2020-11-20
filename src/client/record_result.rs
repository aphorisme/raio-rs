use packs::{Dictionary, ExtractRef, Value};
use packs::std_structs::StdStruct;
use crate::messaging::response::{Record};
use crate::client::error::ClientError;

#[derive(Debug, Clone)]
/// A structure which captures a `RECORD` response into a result row.
pub struct RecordResult {
    pub data: Dictionary<StdStruct>,
}

impl RecordResult {
    /// Uses the `fields` information to augment a `RECORD` with field names.
    pub fn new(success_fields: &[String], record: Record) -> Result<Self, ClientError> {
        if success_fields.len() != record.data.len() {
            return Err(ClientError::FieldsToRecordMismatch)
        }

        let mut data = Dictionary::with_capacity(success_fields.len());
        for (f, v) in success_fields.into_iter().zip(record.data.into_iter()) {
            data.add_property(f, v);
        }

        Ok(RecordResult {
            data,
        })
    }
    
    pub fn from_results(fields: &[String], records: Vec<Record>) -> Result<Vec<Self>, ClientError> {
        let mut results = Vec::with_capacity(records.len());
        for r in records.into_iter() {
            results.push(RecordResult::new(fields, r)?);
        }
        Ok(results)
    }
    
    pub fn get_field_typed<T: ExtractRef<StdStruct>>(&self, key: &str) -> Option<&T> {
        self.data.get_property_typed(key)
    }

    pub fn get_field(&self, key: &str) -> Option<&Value<StdStruct>> {
        self.data.get_property(key)
    }
}


