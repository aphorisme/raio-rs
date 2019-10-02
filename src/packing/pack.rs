use crate::packing::error::PackError;
use crate::packing::ll::{UnboundPack, WResult};
use std::{fmt, io};

pub trait Packable {
    fn pack_to<T: UnboundPack>(self, buf: &mut T) -> WResult<PackError>;
}

pub trait Pack: UnboundPack {
    fn pack<T: Packable>(&mut self, obj: T) -> WResult<PackError> {
        obj.pack_to(self)
    }
}
