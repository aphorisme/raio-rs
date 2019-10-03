use crate::packing::{Value, ValueList, ValueMap};
use std::convert::TryFrom;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum QueryResultError {
    RecordMissesField(String),
    RecordHasTooMuchFields,
    UnknownMetaField(String),
    NoFieldsInSuccess,
    InvalidFieldsStructure,
}

impl fmt::Display for QueryResultError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            QueryResultError::RecordMissesField(s) => {
                write!(f, "Received record misses '{}' as its field.", s)
            }
            QueryResultError::RecordHasTooMuchFields => {
                write!(f, "Received record has too much fields.")
            }
            QueryResultError::UnknownMetaField(s) => write!(f, "Unknown meta field '{}'", s),
            QueryResultError::NoFieldsInSuccess => {
                write!(f, "Missing 'fields' meta data in Success response.")
            }
            QueryResultError::InvalidFieldsStructure => {
                write!(f, "'fields' meta data is of wrong structure.")
            }
        }
    }
}

impl std::error::Error for QueryResultError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[derive(Debug)]
/// A positive query result, which consists of the retrieved records (if any) and the meta fields
/// from the opening and final success response. Usually, such a result is produced from [`Client::run`] which
/// sends a cypher statement to the bolt server and returns in several response what is condensed into
/// a `QueryResult`.
///
/// # Example
///
/// ```
/// use raio::packing::*;
/// use raio::net::QueryResult;
///
/// let record_1 : ValueList<Value> = vec!(Value::String("John".to_string()), Value::Integer(42)).into();
/// let record_2 : ValueList<Value> = vec!(Value::String("Jane".to_string()), Value::Integer(42)).into();
///
/// let mut metadata = ValueMap::with_capacity(1);
/// metadata.insert_value("fields", ValueList::from(vec!("name".to_string(), "age".to_string())));
///
///
/// let mut query_result = QueryResult::begin(metadata).unwrap();
/// query_result.push(record_1).unwrap();
/// query_result.push(record_2).unwrap();
///
/// let name_of_first : &String = query_result.get_record(0).unwrap().get_value("name").unwrap();
///
/// assert_eq!(name_of_first, &"John".to_string());
/// ```
/// [`Client:run`]: sync/struct.Client.html#run
pub struct QueryResult {
    meta_fields: ValueMap,
    record_fields: Vec<String>,
    records: Vec<ValueMap>,
}

impl IntoIterator for QueryResult {
    type Item = ValueMap;
    type IntoIter = <Vec<ValueMap> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.records.into_iter()
    }
}

impl QueryResult {
    pub fn get_record(&self, index: usize) -> Option<&ValueMap> {
        self.records.get(index)
    }

    pub fn get_meta<'a, V>(&'a self, meta_name: &str) -> Result<&'a V, QueryResultError>
    where
        &'a V: TryFrom<&'a Value>,
    {
        self.meta_fields
            .get_value(meta_name)
            .ok_or_else(|| QueryResultError::UnknownMetaField(meta_name.to_string()))
    }

    pub fn begin(metadata: ValueMap) -> Result<QueryResult, QueryResultError> {
        let fields: Vec<String> = metadata
            .get_value::<ValueList>("fields")
            .ok_or(QueryResultError::NoFieldsInSuccess)?
            .clone()
            .unify()
            .map_err(|_| QueryResultError::InvalidFieldsStructure)?
            .0;

        let mut meta = metadata;
        meta.remove("fields").unwrap();

        Ok(QueryResult {
            meta_fields: meta,
            record_fields: fields,
            records: Vec::new(),
        })
    }

    pub fn push(&mut self, record: ValueList<Value>) -> Result<(), QueryResultError> {
        let record_len = record.len();
        let field_underflow: i64 = self.record_fields.len() as i64 - record_len as i64;

        if field_underflow < 0 {
            return Err(QueryResultError::RecordHasTooMuchFields);
        }

        if field_underflow > 0 {
            return Err(QueryResultError::RecordMissesField(
                self.record_fields
                    .get(<usize>::try_from(field_underflow - 1).unwrap())
                    .unwrap()
                    .clone(),
            ));
        }

        let mut row = ValueMap::with_capacity(record_len);

        for (ix, value) in record.into_iter().enumerate() {
            let caption: String = self.record_fields.get(ix).unwrap().clone();
            row.insert(caption, value);
        }

        self.records.push(row);

        Ok(())
    }

    pub fn end(&mut self, final_fields: ValueMap) {
        for (key, value) in final_fields {
            self.meta_fields.insert(key, value);
        }
    }
}
