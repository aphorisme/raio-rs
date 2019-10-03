use std::convert::TryFrom;

use crate::packing::error::PackError;
use crate::packing::ll::{BoltWrite, BoltWriteable, MarkerByte, ValueMap, WResult};
use crate::packing::types::*;
use crate::packing::ValueList;

pub trait Packable {
    fn pack_to<T: BoltWrite>(&self, buf: &mut T) -> WResult<PackError>;
}

pub trait Pack: BoltWrite {
    fn pack<T: Packable>(&mut self, obj: &T) -> WResult<PackError> {
        obj.pack_to(self)
    }
}

impl<W: BoltWrite> Pack for W {}

// ------------------------
// GENERIC IMPLEMENTATIONS
// ------------------------

impl Packable for i64 {
    fn pack_to<T: BoltWrite>(&self, buf: &mut T) -> Result<usize, PackError> {
        let local_self = *self;
        // find suitable bolt type:

        // PlusTinyInt:
        if local_self <= i64::from(MAX_PLUS_TINY_INT) && local_self >= 0 {
            let t = unsafe { PlusTinyInt::new_unchecked(local_self as u8) };
            return t.pack_to(buf);
        }

        // MinusTinyInt:
        if local_self <= 0 && local_self >= i64::from(MIN_MINUS_TINY_INT) {
            return MinusTinyInt::try_from(local_self).unwrap().pack_to(buf);
        }

        // Int8, Int16, Int32, Int64 stack:
        if let Ok(i) = i8::try_from(local_self) {
            i.as_fixed_to(Int8::MARKER_BYTE, buf)
        } else if let Ok(i) = i16::try_from(local_self) {
            i.as_fixed_to(Int16::MARKER_BYTE, buf)
        } else if let Ok(i) = i32::try_from(local_self) {
            i.as_fixed_to(Int32::MARKER_BYTE, buf)
        } else {
            local_self.as_fixed_to(Int64::MARKER_BYTE, buf)
        }
    }
}

impl Packable for String {
    fn pack_to<T: BoltWrite>(&self, buf: &mut T) -> Result<usize, PackError> {
        let len = self.len();
        if len <= TINY_SIZE_MAX as usize {
            return self.as_tiny_sized_to(TinyString::MARKER_BYTE, buf);
        }

        if len <= u8::max_value() as usize {
            <String as SizedPackableAs<u8>>::as_sized_to(self, String8::MARKER_BYTE, buf)
        } else if len <= u16::max_value() as usize {
            <String as SizedPackableAs<u16>>::as_sized_to(self, String16::MARKER_BYTE, buf)
        } else if len <= u32::max_value() as usize {
            <String as SizedPackableAs<u32>>::as_sized_to(self, String32::MARKER_BYTE, buf)
        } else {
            Err(PackError::GenericTooLarge("String"))
        }
    }
}

impl Packable for &str {
    fn pack_to<T: BoltWrite>(&self, buf: &mut T) -> Result<usize, PackError> {
        String::from(*self).pack_to(buf)
    }
}

impl Packable for f64 {
    fn pack_to<T: BoltWrite>(&self, buf: &mut T) -> Result<usize, PackError> {
        self.as_fixed_to(Float64::MARKER_BYTE, buf)
    }
}

impl<I: Packable> Packable for ValueList<I> {
    fn pack_to<T: BoltWrite>(&self, buf: &mut T) -> Result<usize, PackError> {
        let len = self.len();
        if len <= TINY_SIZE_MAX as usize {
            self.as_tiny_sized_to(<TinyList<I>>::MARKER_BYTE, buf)
        } else if len <= u8::max_value() as usize {
            <Self as SizedPackableAs<u8>>::as_sized_to(self, <List8<I>>::MARKER_BYTE, buf)
        } else if len <= u16::max_value() as usize {
            <Self as SizedPackableAs<u16>>::as_sized_to(self, <List16<I>>::MARKER_BYTE, buf)
        } else if len <= u32::max_value() as usize {
            <Self as SizedPackableAs<u32>>::as_sized_to(self, <List32<I>>::MARKER_BYTE, buf)
        } else {
            Err(PackError::GenericTooLarge(stringify!(Self)))
        }
    }
}

impl<I: Packable> Packable for ValueMap<I> {
    fn pack_to<T: BoltWrite>(&self, buf: &mut T) -> Result<usize, PackError> {
        let len = self.len();
        if len <= TINY_SIZE_MAX as usize {
            self.as_tiny_sized_to(<TinyMap<I>>::MARKER_BYTE, buf)
        } else if len <= u8::max_value() as usize {
            <Self as SizedPackableAs<u8>>::as_sized_to(self, <Map8<I>>::MARKER_BYTE, buf)
        } else if len <= u16::max_value() as usize {
            <Self as SizedPackableAs<u16>>::as_sized_to(self, <Map16<I>>::MARKER_BYTE, buf)
        } else if len <= u32::max_value() as usize {
            <Self as SizedPackableAs<u32>>::as_sized_to(self, <Map32<I>>::MARKER_BYTE, buf)
        } else {
            Err(PackError::GenericTooLarge(stringify!(Self)))
        }
    }
}

impl Packable for bool {
    fn pack_to<T: BoltWrite>(&self, buf: &mut T) -> Result<usize, PackError> {
        if *self {
            Ok(MarkerByte::BoolTrue.bolt_write_to(buf)?)
        } else {
            Ok(MarkerByte::BoolFalse.bolt_write_to(buf)?)
        }
    }
}
