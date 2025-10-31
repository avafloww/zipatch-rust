use std::io::Read;

use crate::config::ZiPatchConfig;
use crate::error::Result;
use crate::util::BinaryReaderExt;

/// Apply Free Space chunk (APFS)
///
/// This is a NOP (no operation) in modern patchers.
/// No real-world samples have been found, so the fields are theoretical.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyFreeSpaceChunk {
    /// Unknown field A
    pub unknown_field_a: i64,
    /// Unknown field B
    pub unknown_field_b: i64,
}

impl ApplyFreeSpaceChunk {
    pub const CHUNK_TYPE: &'static str = "APFS";

    /// Reads an ApplyFreeSpaceChunk from a reader
    pub fn read<R: Read>(reader: &mut R, _size: u32) -> Result<Self> {
        let unknown_field_a = reader.read_i64_be()?;
        let unknown_field_b = reader.read_i64_be()?;

        Ok(Self {
            unknown_field_a,
            unknown_field_b,
        })
    }

    /// Applies the chunk (NOP - does nothing)
    pub fn apply(&self, _config: &mut ZiPatchConfig) -> Result<()> {
        // NOP on modern patchers
        Ok(())
    }
}

impl std::fmt::Display for ApplyFreeSpaceChunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}",
            Self::CHUNK_TYPE,
            self.unknown_field_a,
            self.unknown_field_b
        )
    }
}
