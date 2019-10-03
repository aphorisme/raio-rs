use std::convert::TryFrom;
use std::ops::Deref;

pub use fixed::*;
pub use sized::*;
pub use struct_body::*;
pub use structs::*;
pub use tiny_ints::*;
pub use tiny_sized::*;
pub use value::*;

use crate::packing::error::{ConversionError, PackError, UnpackError};
use crate::packing::ll::{
    read_expected_marker, BoltRead, BoltWrite, MarkerByte, TinySizeMarker, ValueList, ValueMap,
    WResult,
};
use crate::packing::{Packable, Unpackable};

pub mod fixed;
pub mod sized;
pub mod struct_body;
pub mod structs;
pub mod tiny_ints;
pub mod tiny_sized;
pub mod value;

macro_rules! bolt_type {
    ($name:ident$(<$var:ident>)*($t:ty) => $m:expr) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $name$(<$var>)*(pub $t);

        impl$(<$var>)* $name$(<$var>)* {
            pub const MARKER_BYTE: MarkerByte = $m;
            pub fn into_inner(self) -> $t {
                self.0
            }
        }

        impl$(<$var>)* Deref for $name$(<$var>)* {
            type Target = $t;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    }
}

macro_rules! sized {
    ($name:ident$(<$var:ident>)*($t:ty) => $u:ty : $m:expr) => {
        bolt_type!($name$(<$var>)*($t) => $m);

        impl$(<$var>)* TryFrom<$t> for $name$(<$var>)* {
            type Error = ConversionError;
            fn try_from(input: $t) -> Result<Self, Self::Error> {
                let _ : $u = <$u>::try_from(input.len())?;
                Ok($name(input))
            }
        }

        impl$(<$var: Packable>)* Packable for $name$(<$var>)* {
            fn pack_to<T: BoltWrite>(&self, buf: &mut T) -> WResult<PackError> {
                <$t as SizedPackableAs<$u>>::as_sized_to(&self.0, $m, buf)
            }
        }

        impl$(<$var: Unpackable>)* Unpackable for $name$(<$var>)* {
            fn unpack_from<T: BoltRead>(buf: &mut T) -> Result<Self, UnpackError> {
                let _ : MarkerByte = read_expected_marker($m, buf)?;
                let inner = <$t as SizedUnpackableAs<$u>>::sized_body_from(buf)?;
                Ok($name(inner))
            }
        }
    }
}

sized!(String8(String) => u8 : MarkerByte::String8);
sized!(String16(String) => u16 : MarkerByte::String16);
sized!(String32(String) => u32 : MarkerByte::String32);
sized!(List8<I>(ValueList<I>) => u8 : MarkerByte::List8);
sized!(List16<I>(ValueList<I>) => u16 : MarkerByte::List16);
sized!(List32<I>(ValueList<I>) => u32 : MarkerByte::List32);
sized!(Map8<I>(ValueMap<I>) => u8 : MarkerByte::Map8);
sized!(Map16<I>(ValueMap<I>) => u16 : MarkerByte::Map16);
sized!(Map32<I>(ValueMap<I>) => u32 : MarkerByte::Map32);

macro_rules! tiny_sized {
    ($name:ident$(<$var:ident>)*($t:ty) => $m:expr) => {
        bolt_type!($name$(<$var>)*($t) => $m);

        impl$(<$var: Packable>)* Packable for $name$(<$var>)* {
            fn pack_to<T: BoltWrite>(&self, buf: &mut T) -> WResult<PackError> {
                self.0.as_tiny_sized_to($m, buf)
            }
        }

        impl$(<$var: Unpackable>)* Unpackable for $name$(<$var>)* {
            fn unpack_from<T: BoltRead>(buf: &mut T) -> Result<Self, UnpackError> {
                let tm : TinySizeMarker = read_expected_marker($m, buf)?;
                let inner = <$t as TinySizedUnpackableAs>::tiny_sized_body_from(tm.tiny_size, buf)?;
                Ok($name(inner))
            }
        }
    }
}

tiny_sized!(TinyString(String) => MarkerByte::TinyString);
tiny_sized!(TinyMap<I>(ValueMap<I>) => MarkerByte::TinyMap);
tiny_sized!(TinyList<I>(ValueList<I>) => MarkerByte::TinyList);

macro_rules! fixed {
    ($name:ident($t:ty) => $m:expr) => {
        bolt_type!($name($t) => $m);

        impl From<$t> for $name {
            fn from(input: $t) -> Self {
                $name(input)
            }
        }

        impl Packable for $name {
            fn pack_to<T: BoltWrite>(&self, buf: &mut T) -> WResult<PackError> {
                self.as_fixed_to($m, buf)
            }
        }

        impl Unpackable for $name {
            fn unpack_from<T: BoltRead>(buf: &mut T) -> Result<Self, UnpackError> {
                let _ : MarkerByte = read_expected_marker($m, buf)?;
                let inner = <$t as FixedUnpackableAs>::fixed_body_from(buf)?;
                Ok($name(inner))
            }
        }
    }
}

fixed!(Int8(i8) => MarkerByte::Int8);
fixed!(Int16(i16) => MarkerByte::Int16);
fixed!(Int32(i32) => MarkerByte::Int32);
fixed!(Int64(i64) => MarkerByte::Int64);
fixed!(Float64(f64) => MarkerByte::Float64);
