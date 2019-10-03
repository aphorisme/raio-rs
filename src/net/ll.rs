use crate::packing::ll::{BoltWrite, BoltWriteable};
use std::io;
use std::io::Write;

// Denotes the magic number used in the handshake.
pub const MAGIC_NUMBER: [u8; 4] = [0x60, 0x60, 0xB0, 0x17];

pub const DEFAULT_PORT: u16 = 7678;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct VersionHandshake {
    pub version1: Option<u32>,
    pub version2: Option<u32>,
    pub version3: Option<u32>,
    pub version4: Option<u32>,
}

impl Default for VersionHandshake {
    fn default() -> Self {
        VersionHandshake {
            version1: None,
            version2: None,
            version3: None,
            version4: None,
        }
    }
}

impl VersionHandshake {
    pub fn just_version(version: u32) -> VersionHandshake {
        VersionHandshake {
            version1: Some(version),
            version2: None,
            version3: None,
            version4: None,
        }
    }
}

impl BoltWriteable for VersionHandshake {
    type Error = io::Error;

    fn bolt_write_to<T: Write>(self, buf: &mut T) -> Result<usize, Self::Error> {
        Ok(buf.bolt_write(self.version1.unwrap_or_default())?
            + buf.bolt_write(self.version2.unwrap_or_default())?
            + buf.bolt_write(self.version3.unwrap_or_default())?
            + buf.bolt_write(self.version4.unwrap_or_default())?)
    }
}
