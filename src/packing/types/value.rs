use std::convert::TryFrom;

use byteorder::ReadBytesExt;

use crate::packing::error::{PackError, UnpackError};
use crate::packing::ll::{BoltRead, BoltWrite, MarkerByte, Signature, TinySizeMarker, ValueMap};
use crate::packing::structs::{Node, Path, Relationship, UnboundRelationship};
use crate::packing::types::*;
use crate::packing::{Packable, Unpackable, ValueList};

#[derive(Debug, PartialEq, Clone)]
/// A type which represents all possible bolt values in their most general form.
pub enum Value {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    List(ValueList<Value>),
    Map(ValueMap<Value>),
    Node(Node),
    Relationship(Relationship),
    UnboundRelationship(UnboundRelationship),
    Path(Path),
}

pub enum ValueConversionError {
    WrongValueError(Value, &'static str),
}

macro_rules! embeeded_types {
    ($($v:ident : $e:expr => $t:ty);+) => {
        $(impl<'a> TryFrom<&'a Value> for &'a $t {
            type Error = ValueConversionError;
            fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
                match value {
                    Value::$v(i) => Ok(i),
                    _ => Err(ValueConversionError::WrongValueError(value.clone(), $e))
                }
            }
        }

        impl TryFrom<Value> for $t {
            type Error = ValueConversionError;
            fn try_from(value: Value) -> Result<Self, Self::Error> {
                match value {
                    Value::$v(i) => Ok(i),
                    _ => Err(ValueConversionError::WrongValueError(value.clone(), $e))
                }
            }
        }

        impl TryFrom<Value> for Option<$t> {
            type Error = ValueConversionError;
            fn try_from(value: Value) -> Result<Self, Self::Error> {
                if let Value::Null = value {
                    Ok(None)
                } else {
                    Ok(Some(<$t>::try_from(value)?))
                }
            }
        }

        impl From<$t> for Value {
            fn from(input: $t) -> Value {
                Value::$v(input.into())
            }
        })+
    }
}

embeeded_types! {
    Integer : "Integer" => i64;
    String  : "String"  => String;
    Float   : "Float"   => f64;
    Boolean : "Boolean" => bool;
    Map     : "Map"     => ValueMap<Value>;
    List    : "List"    => ValueList<Value>;
    Node    : "Node"    => Node;
    Relationship : "Relationship" => Relationship;
    UnboundRelationship : "UnboundRelationship" => UnboundRelationship;
    Path : "Path" => Path
}

/// Implementation for in `Option` wrapped embedded types. For `Some(x)` this is `x`
/// interpreted as `Value`. For `None` this is `Value::Null`.
impl<T> From<Option<T>> for Value
where
    Value: From<T>,
{
    fn from(input: Option<T>) -> Self {
        if let Some(x) = input {
            Value::from(x)
        } else {
            Value::Null
        }
    }
}

impl From<&str> for Value {
    fn from(input: &str) -> Self {
        <Value>::from(input.to_string())
    }
}

impl<V> From<Vec<V>> for Value
where
    Value: From<V>,
{
    fn from(input: Vec<V>) -> Self {
        Value::List(ValueList(
            input.into_iter().map(<Value>::from).collect(),
        ))
    }
}

impl<V> TryFrom<Value> for ValueList<V>
where
    V: TryFrom<Value, Error = ValueConversionError>,
{
    type Error = ValueConversionError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::List(vl) => {
                let mut new_vl = ValueList::with_capacity(vl.len());
                for v in vl {
                    new_vl.push(<V>::try_from(v)?);
                }
                Ok(new_vl)
            }
            _ => Err(ValueConversionError::WrongValueError(value, "List")),
        }
    }
}

impl Packable for Value {
    fn pack_to<T: BoltWrite>(&self, buf: &mut T) -> Result<usize, PackError> {
        match self {
            Value::Null => Ok(buf.bolt_write(MarkerByte::Null)?),
            Value::Boolean(b) => b.pack_to(buf),
            Value::Integer(i) => i.pack_to(buf),
            Value::Float(f) => f.pack_to(buf),
            Value::String(s) => s.pack_to(buf),
            Value::List(v) => v.pack_to(buf),
            Value::Map(m) => m.pack_to(buf),
            Value::Node(n) => n.pack_to(buf),
            Value::Relationship(r) => r.pack_to(buf),
            Value::UnboundRelationship(ur) => ur.pack_to(buf),
            Value::Path(p) => p.pack_to(buf),
        }
    }
}

impl Unpackable for Value {
    fn unpack_from<T: BoltRead>(buf: &mut T) -> Result<Self, UnpackError> {
        let m_byte = buf.read_u8()?;
        let m: TinySizeMarker = TinySizeMarker::try_from(m_byte)?;

        match m.marker {
            MarkerByte::Null => Ok(Value::Null),
            MarkerByte::BoolFalse => Ok(Value::Boolean(false)),
            MarkerByte::BoolTrue => Ok(Value::Boolean(true)),

            MarkerByte::TinyString => Ok(Value::String(String::tiny_sized_body_from(
                m.tiny_size,
                buf,
            )?)),
            MarkerByte::String8 => Ok(Value::String(
                <String as SizedUnpackableAs<u8>>::sized_body_from(buf)?,
            )),
            MarkerByte::String16 => Ok(Value::String(
                <String as SizedUnpackableAs<u16>>::sized_body_from(buf)?,
            )),
            MarkerByte::String32 => Ok(Value::String(
                <String as SizedUnpackableAs<u32>>::sized_body_from(buf)?,
            )),

            MarkerByte::TinyList => Ok(Value::List(<ValueList<Value>>::tiny_sized_body_from(
                m.tiny_size,
                buf,
            )?)),
            MarkerByte::List8 => Ok(Value::List(
                <ValueList<Value> as SizedUnpackableAs<u8>>::sized_body_from(buf)?,
            )),
            MarkerByte::List16 => Ok(Value::List(
                <ValueList<Value> as SizedUnpackableAs<u16>>::sized_body_from(buf)?,
            )),
            MarkerByte::List32 => Ok(Value::List(
                <ValueList<Value> as SizedUnpackableAs<u32>>::sized_body_from(buf)?,
            )),

            MarkerByte::TinyMap => Ok(Value::Map(<ValueMap<Value>>::tiny_sized_body_from(
                m.tiny_size,
                buf,
            )?)),
            MarkerByte::Map8 => Ok(Value::Map(
                <ValueMap<Value> as SizedUnpackableAs<u8>>::sized_body_from(buf)?,
            )),
            MarkerByte::Map16 => Ok(Value::Map(
                <ValueMap<Value> as SizedUnpackableAs<u16>>::sized_body_from(buf)?,
            )),
            MarkerByte::Map32 => Ok(Value::Map(
                <ValueMap<Value> as SizedUnpackableAs<u32>>::sized_body_from(buf)?,
            )),

            MarkerByte::MinusTinyInt => {
                let mut r: &[u8] = &[m_byte];
                let mty = MinusTinyInt::unpack_from(&mut r)?;
                Ok(Value::Integer(i64::from(mty)))
            }
            MarkerByte::PlusTinyInt => Ok(Value::Integer(i64::from(m_byte))),
            MarkerByte::Int8 => Ok(Value::Integer(i64::from(i8::fixed_body_from(buf)?))),
            MarkerByte::Int16 => Ok(Value::Integer(i64::from(i16::fixed_body_from(buf)?))),
            MarkerByte::Int32 => Ok(Value::Integer(i64::from(i32::fixed_body_from(buf)?))),
            MarkerByte::Int64 => Ok(Value::Integer(i64::fixed_body_from(buf)?)),
            MarkerByte::Float64 => Ok(Value::Float(f64::fixed_body_from(buf)?)),

            MarkerByte::TinyStruct => {
                // read struct signature:
                let sig_byte = buf.read_u8()?;
                let sig = Signature::try_from(sig_byte)?;

                match sig {
                    Signature::Node => Ok(Value::Node(Node::read_body_from(buf)?)),
                    Signature::Path => Ok(Value::Path(Path::read_body_from(buf)?)),
                    Signature::Relationship => {
                        Ok(Value::Relationship(Relationship::read_body_from(buf)?))
                    }
                    Signature::UnboundRelationship => Ok(Value::UnboundRelationship(
                        UnboundRelationship::read_body_from(buf)?,
                    )),
                    _ => Err(UnpackError::UnexpectedSignature(sig, "Value")),
                }
            }

            MarkerByte::Struct8 | MarkerByte::Struct16 => unimplemented!(),
        }
    }
}
