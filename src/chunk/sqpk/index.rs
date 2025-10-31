use std::io::Read;

use crate::config::ZiPatchConfig;
use crate::error::Result;
use crate::util::{BinaryReaderExt, SqpackIndexFile};

/// SQPK Index command ('I')
///
/// This is a NOP (no operation) on modern patchers.
#[derive(Debug, Clone)]
pub struct SqpkIndex {
    /// Index command kind (Add or Delete)
    pub index_command: IndexCommandKind,
    /// Whether this is a synonym
    pub is_synonym: bool,
    /// Target index file
    pub target_file: SqpackIndexFile,
    /// File hash
    pub file_hash: u64,
    /// Block offset
    pub block_offset: u32,
    /// Block number (purpose unknown)
    pub block_number: u32,
}

/// Kind of index command
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexCommandKind {
    /// Add to index
    Add = b'A',
    /// Delete from index
    Delete = b'D',
}

impl IndexCommandKind {
    /// Creates an IndexCommandKind from a u8 value
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            b'A' => Some(IndexCommandKind::Add),
            b'D' => Some(IndexCommandKind::Delete),
            _ => None,
        }
    }
}

impl SqpkIndex {
    pub const COMMAND: char = 'I';

    /// Reads an SqpkIndex from a reader
    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let index_command_byte = {
            let mut buf = [0u8; 1];
            reader.read_exact(&mut buf)?;
            buf[0]
        };

        let index_command =
            IndexCommandKind::from_u8(index_command_byte).unwrap_or(IndexCommandKind::Add);

        let is_synonym = {
            let mut buf = [0u8; 1];
            reader.read_exact(&mut buf)?;
            buf[0] != 0
        };

        // Read and discard alignment byte
        {
            let mut buf = [0u8; 1];
            reader.read_exact(&mut buf)?;
        };

        let target_file = SqpackIndexFile::read_from(reader)?;
        let file_hash = reader.read_u64_be()?;
        let block_offset = reader.read_u32_be()?;
        let block_number = reader.read_u32_be()?;

        Ok(Self {
            index_command,
            is_synonym,
            target_file,
            file_hash,
            block_offset,
            block_number,
        })
    }

    /// Applies the command (NOP - does nothing)
    pub fn apply(&self, _config: &mut ZiPatchConfig) -> Result<()> {
        // NOP on modern patchers
        Ok(())
    }
}

impl std::fmt::Display for SqpkIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SQPK:{}:{:?}:{}:{}:{:X}:{}:{}",
            Self::COMMAND,
            self.index_command,
            self.is_synonym,
            self.target_file,
            self.file_hash,
            self.block_offset,
            self.block_number
        )
    }
}
