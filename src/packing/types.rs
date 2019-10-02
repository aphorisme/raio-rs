pub mod fixed;
pub mod sized;
pub mod struct_body;
pub mod tiny_ints;
pub mod tiny_sized;
pub mod tiny_struct;
pub mod value;

pub use fixed::*;
pub use sized::*;
pub use struct_body::*;
pub use tiny_ints::*;
pub use tiny_sized::*;
pub use value::*;

use crate::packing::error::{PackError, UnpackError};
use crate::packing::ll::{MarkerByte, UnboundPack, UnboundUnpack, WResult};
use crate::packing::{Packable, Unpackable};
use std::ops::{Deref, DerefMut};

macro_rules! bolt_type {
    ($name:ident($t:ty) => $m:expr) => {
        pub struct $name($t);

        impl Deref for $name {
            type Target = $t;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl Packable for $name {
            fn pack_to<T: UnboundPack>(self, buf: &mut T) -> WResult<PackError> {
                buf.pack_as($m, self.0)
            }
        }

        impl Unpackable for $name {
            fn unpack_from<T: UnboundUnpack>(buf: &mut T) -> Result<Self, UnpackError> {
                let s = buf.unpack_as::<$t>($m)?;
                Ok($name(s))
            }
        }
    };
}

// Strings:
bolt_type! {
    TinyString(TinySized<String>) => MarkerByte::TinyString
}

bolt_type! {
    String8(Sized<u8, String>) => MarkerByte::String8
}

bolt_type! {
    String16(Sized<u16, String>) => MarkerByte::String16
}

bolt_type! {
    String32(Sized<u32, String>) => MarkerByte::String32
}

// Integers and Floats:
bolt_type! {
    Int8(Fixed<i8>) => MarkerByte::Int8
}

bolt_type! {
    Int16(Fixed<i16>) => MarkerByte::Int16
}

bolt_type! {
    Int32(Fixed<i32>) => MarkerByte::Int32
}

bolt_type! {
    Int64(Fixed<i64>) => MarkerByte::Int64
}

bolt_type! {
    Float64(Fixed<f64>) => MarkerByte::Float64
}
