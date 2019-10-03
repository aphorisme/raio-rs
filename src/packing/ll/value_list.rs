use crate::packing::error::UnpackError;
use crate::packing::ll::BoltRead;
use crate::packing::{Unpackable, Value, ValueConversionError};
use std::convert::TryFrom;
use std::iter::FromIterator;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, PartialEq, Default)]
/// Gives a convenience wrapper around `Vec<T>` which provides `value_at` for `Vec<Value>` to
/// retrieve a value unwrapped to its general type.
/// ```
/// use raio::packing::*;
///
/// let v : ValueList<Value> = ValueList(vec! {
///     Value::Integer(42),
///     Value::Boolean(true),
///     Value::String(String::from("Hello World"))
/// });
///
/// let i : &i64 = v.value_at(0).unwrap();
/// assert_eq!(42, *i);
///
/// let b : &bool = v.value_at(1).unwrap();
/// assert_eq!(true, *b);
///
/// let s : &String = v.value_at(2).unwrap();
/// assert_eq!(String::from("Hello World"), s.to_owned());
///
/// // at `ix = 2` is a `Value::String` which has as its general type `String`, hence
/// // trying to get a `i64` yields none.
/// assert!(v.value_at::<i64>(2).is_none())
/// ```
pub struct ValueList<T = Value>(pub Vec<T>);

impl<T> ValueList<T> {
    pub fn with_capacity(cap: usize) -> ValueList<T> {
        ValueList(<Vec<T>>::with_capacity(cap))
    }

    pub fn new() -> ValueList<T> {
        ValueList(<Vec<T>>::new())
    }
}

impl<T: Unpackable> ValueList<T> {
    pub fn unpack_body<B: BoltRead>(size: usize, buf: &mut B) -> Result<ValueList<T>, UnpackError> {
        let mut vs = ValueList::with_capacity(size);
        for _ in 0..size {
            let v = <T>::unpack_from(buf)?;
            vs.push(v)
        }
        Ok(vs)
    }
}

impl<T> IntoIterator for ValueList<T> {
    type Item = T;
    type IntoIter = <Vec<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<V> FromIterator<V> for ValueList<V> {
    fn from_iter<T: IntoIterator<Item = V>>(iter: T) -> Self {
        ValueList(<Vec<V>>::from_iter(iter))
    }
}

impl<T> Deref for ValueList<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for ValueList<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<V> From<Vec<V>> for ValueList<Value>
where
    Value: From<V>,
{
    fn from(input: Vec<V>) -> Self {
        ValueList(input.into_iter().map(Value::from).collect())
    }
}

impl<'a> ValueList<Value> {
    pub fn value_at<T>(&'a self, ix: usize) -> Option<&'a T>
    where
        &'a T: TryFrom<&'a Value>,
    {
        self.0.get(ix).and_then(|x| <&T>::try_from(x).ok())
    }

    pub fn unify<V: TryFrom<Value, Error = ValueConversionError>>(
        self,
    ) -> Result<ValueList<V>, ValueConversionError> {
        let mut unified = ValueList::with_capacity(self.len());

        for value in self {
            let v = <V>::try_from(value)?;
            unified.push(v);
        }

        Ok(unified)
    }
}
