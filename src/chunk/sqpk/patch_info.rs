use std::io::Read;

use crate::config::ZiPatchConfig;
use crate::error::Result;
use crate::util::BinaryReaderExt;

/// SQPK Patch Info command ('X')
///
/// This is a NOP (no operation) on modern patchers.
/// The purpose of these fields is unknown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqpkPatchInfo {
    /// Status byte
    pub status: u8,
    /// Version byte
    pub version: u8,
    /// Install size
    pub install_size: u64,
}

impl SqpkPatchInfo {
    pub const COMMAND: char = 'X';

    /// Reads an SqpkPatchInfo from a reader
    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let status = {
            let mut buf = [0u8; 1];
            reader.read_exact(&mut buf)?;
            buf[0]
        };

        let version = {
            let mut buf = [0u8; 1];
            reader.read_exact(&mut buf)?;
            buf[0]
        };

        // Read and discard alignment byte
        let _alignment = {
            let mut buf = [0u8; 1];
            reader.read_exact(&mut buf)?;
            buf[0]
        };

        let install_size = reader.read_u64_be()?;

        Ok(Self {
            status,
            version,
            install_size,
        })
    }

    /// Applies the command (NOP - does nothing)
    pub fn apply(&self, _config: &mut ZiPatchConfig) -> Result<()> {
        // NOP on modern patchers
        Ok(())
    }
}

impl std::fmt::Display for SqpkPatchInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SQPK:{}:{}:{}:{}",
            Self::COMMAND,
            self.status,
            self.version,
            self.install_size
        )
    }
}
