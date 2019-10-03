use crate::packing::error::UnpackError;
use crate::packing::ll::BoltRead;
use crate::packing::{Unpackable, Value};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::iter::{FromIterator, IntoIterator};
use std::ops::{Deref, DerefMut};

#[derive(Debug, PartialEq, Clone, Default)]
/// A wrapper around `HashMap<String, I>` which provides a special `get_value`
/// in the case of `I = Value`. Since most of the structs contain some sort of mapping from keys
/// to `Value` this gives some convenience when working directly with returned structs.
/// ```
/// use raio::packing::*;
///
/// let vm : ValueMap<Value> =
///     vec! {
///         (String::from("key1"), Value::Integer(42)),
///         (String::from("key2"), Value::Boolean(true)),
///    }.into_iter().collect();
///
/// let val1 : &i64 = vm.get_value("key1").unwrap();
/// assert_eq!(42, *val1);
///
/// let val2 : &bool = vm.get_value("key2").unwrap();
/// assert_eq!(true, *val2);
///
/// assert!(vm.get_value::<String>("key1").is_none());
/// ```
pub struct ValueMap<I = Value>(pub HashMap<String, I>);

impl<I> Deref for ValueMap<I> {
    type Target = HashMap<String, I>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<I> DerefMut for ValueMap<I> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<I> IntoIterator for ValueMap<I> {
    type Item = (String, I);
    type IntoIter = std::collections::hash_map::IntoIter<String, I>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<I> FromIterator<(String, I)> for ValueMap<I> {
    fn from_iter<T: IntoIterator<Item = (String, I)>>(iter: T) -> Self {
        ValueMap(HashMap::from_iter(iter))
    }
}

impl<I> ValueMap<I> {
    pub fn with_capacity(cap: usize) -> ValueMap<I> {
        ValueMap(<HashMap<String, I>>::with_capacity(cap))
    }

    pub fn new() -> ValueMap<I> {
        ValueMap(<HashMap<String, I>>::new())
    }
}

impl<T: Unpackable> ValueMap<T> {
    pub fn unpack_body<B: BoltRead>(size: usize, buf: &mut B) -> Result<ValueMap<T>, UnpackError> {
        let mut vs = ValueMap::with_capacity(size);
        for _ in 0..size {
            let k = <String>::unpack_from(buf)?;
            let v = <T>::unpack_from(buf)?;
            vs.insert(k, v);
        }
        Ok(vs)
    }
}

impl<'a> ValueMap<Value> {
    pub fn get_value<T>(&'a self, k: &str) -> Option<&'a T>
    where
        &'a T: TryFrom<&'a Value>,
    {
        let key = String::from(k);
        self.0
            .get(&key)
            .and_then(|val| <&'a T>::try_from(&val).ok())
    }

    pub fn insert_value<T>(&'a mut self, k: &str, v: T)
    where
        Value: From<T>,
    {
        let v = <Value>::from(v);
        self.insert(k.to_string(), v);
    }
}
