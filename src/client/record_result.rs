use packs::{Dictionary, ExtractRef, Value};
use packs::std_structs::StdStruct;
use crate::client::response::{Record};
use crate::client::error::ClientError;

pub struct RecordResult {
    data: Dictionary<StdStruct>,
}

impl RecordResult {
    pub fn new(success_fields: &Vec<String>, record: Record) -> Result<Self, ClientError> {
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

    pub fn get_field(&self, field: &str) -> Option<&Value<StdStruct>> {
        self.data.get_property(field)
    }

    pub fn get_field_typed<T: ExtractRef<StdStruct>>(&self, field: &str) -> Option<&T> {
        self.data.get_property_typed(field)
    }
}

