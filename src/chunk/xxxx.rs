use std::io::Read;

use crate::config::ZiPatchConfig;
use crate::error::Result;

/// XXXX chunk
///
/// This chunk type has never been observed in practice.
/// It's included for completeness but is essentially a placeholder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XXXXChunk;

impl XXXXChunk {
    pub const CHUNK_TYPE: &'static str = "XXXX";

    /// Reads an XXXXChunk from a reader
    pub fn read<R: Read>(_reader: &mut R, _size: u32) -> Result<Self> {
        // XXXX chunk contains no data
        Ok(Self)
    }

    /// Applies the chunk (no-op)
    pub fn apply(&self, _config: &mut ZiPatchConfig) -> Result<()> {
        // XXXX chunk does nothing
        Ok(())
    }
}

impl std::fmt::Display for XXXXChunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Self::CHUNK_TYPE)
    }
}
