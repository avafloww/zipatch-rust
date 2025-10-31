use std::io::Read;

use crate::config::ZiPatchConfig;
use crate::error::Result;

/// End of File chunk (EOF_)
///
/// Marks the end of a patch file. No data is contained in this chunk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndOfFileChunk;

impl EndOfFileChunk {
    pub const CHUNK_TYPE: &'static str = "EOF_";

    /// Reads an EndOfFileChunk from a reader
    pub fn read<R: Read>(_reader: &mut R, _size: u32) -> Result<Self> {
        // EOF chunk contains no data
        Ok(Self)
    }

    /// Applies the chunk (no-op for EOF)
    pub fn apply(&self, _config: &mut ZiPatchConfig) -> Result<()> {
        // EOF chunk does nothing
        Ok(())
    }
}

impl std::fmt::Display for EndOfFileChunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Self::CHUNK_TYPE)
    }
}
