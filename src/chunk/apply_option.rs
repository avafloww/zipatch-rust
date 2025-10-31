use std::io::Read;

use crate::config::ZiPatchConfig;
use crate::error::Result;
use crate::util::BinaryReaderExt;

/// Apply Option chunk (APLY)
///
/// Sets configuration options for patch application
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyOptionChunk {
    /// The option being set
    pub option_kind: ApplyOptionKind,
    /// The value for the option
    pub option_value: bool,
}

/// Kind of apply option
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyOptionKind {
    /// Ignore missing files
    IgnoreMissing = 1,
    /// Ignore old file mismatches
    IgnoreOldMismatch = 2,
    /// Unknown option kind
    Unknown = 0,
}

impl ApplyOptionKind {
    /// Creates an ApplyOptionKind from a u32 value
    pub fn from_u32(value: u32) -> Self {
        match value {
            1 => ApplyOptionKind::IgnoreMissing,
            2 => ApplyOptionKind::IgnoreOldMismatch,
            _ => ApplyOptionKind::Unknown,
        }
    }
}

impl ApplyOptionChunk {
    pub const CHUNK_TYPE: &'static str = "APLY";

    /// Reads an ApplyOptionChunk from a reader
    pub fn read<R: Read>(reader: &mut R, _size: u32) -> Result<Self> {
        let option_kind_value = reader.read_u32_be()?;
        let option_kind = ApplyOptionKind::from_u32(option_kind_value);

        // Read and discard padding (always 0x0000_0004 as far as observed)
        let _padding = reader.read_bytes_required(4)?;

        let value_raw = reader.read_u32_be()?;
        let value = value_raw != 0;

        // Only set the value if the option kind is valid
        let option_value = match option_kind {
            ApplyOptionKind::IgnoreMissing | ApplyOptionKind::IgnoreOldMismatch => value,
            ApplyOptionKind::Unknown => false,
        };

        Ok(Self {
            option_kind,
            option_value,
        })
    }

    /// Applies the chunk by setting the configuration option
    pub fn apply(&self, config: &mut ZiPatchConfig) -> Result<()> {
        match self.option_kind {
            ApplyOptionKind::IgnoreMissing => {
                config.ignore_missing = self.option_value;
            }
            ApplyOptionKind::IgnoreOldMismatch => {
                config.ignore_old_mismatch = self.option_value;
            }
            ApplyOptionKind::Unknown => {
                // Do nothing for unknown options
            }
        }

        Ok(())
    }
}

impl std::fmt::Display for ApplyOptionChunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind_str = match self.option_kind {
            ApplyOptionKind::IgnoreMissing => "IgnoreMissing",
            ApplyOptionKind::IgnoreOldMismatch => "IgnoreOldMismatch",
            ApplyOptionKind::Unknown => "Unknown",
        };

        write!(f, "{}:{}:{}", Self::CHUNK_TYPE, kind_str, self.option_value)
    }
}
