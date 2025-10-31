use std::io::Read;

use crate::config::{Platform, ZiPatchConfig};
use crate::error::Result;
use crate::util::BinaryReaderExt;

/// SQPK Target Info command ('T')
///
/// Sets platform and region information. Only Platform is used on recent patcher versions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqpkTargetInfo {
    /// Target platform
    pub platform: Platform,
    /// Region ID
    pub region: RegionId,
    /// Debug flag
    pub is_debug: bool,
    /// Version
    pub version: u16,
    /// Deleted data size
    pub deleted_data_size: u64,
    /// Seek count
    pub seek_count: u64,
}

/// Region identifier
#[repr(i16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionId {
    /// Global region (US/EU/JP/ZH)
    Global = -1,
}

impl RegionId {
    /// Creates a RegionId from an i16 value
    pub fn from_i16(value: i16) -> Self {
        match value {
            -1 => RegionId::Global,
            _ => RegionId::Global, // Default to Global for unknown values
        }
    }
}

impl SqpkTargetInfo {
    pub const COMMAND: char = 'T';

    /// Reads an SqpkTargetInfo from a reader
    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        // Read and discard 3 bytes of reserved data
        let _reserved = reader.read_bytes_required(3)?;

        let platform = Platform::from_u16(reader.read_u16_be()?)?;
        let region = RegionId::from_i16(reader.read_i16_be()?);
        let is_debug = reader.read_i16_be()? != 0;
        let version = reader.read_u16_be()?;

        // Note: These are little-endian (not BE)
        let deleted_data_size = {
            let mut buf = [0u8; 8];
            reader.read_exact(&mut buf)?;
            u64::from_le_bytes(buf)
        };

        let seek_count = {
            let mut buf = [0u8; 8];
            reader.read_exact(&mut buf)?;
            u64::from_le_bytes(buf)
        };

        // Note: There are 32 + 64 bytes of empty data at the end

        Ok(Self {
            platform,
            region,
            is_debug,
            version,
            deleted_data_size,
            seek_count,
        })
    }

    /// Applies the command by setting the platform in the config
    pub fn apply(&self, config: &mut ZiPatchConfig) -> Result<()> {
        config.platform = self.platform;
        Ok(())
    }
}

impl std::fmt::Display for SqpkTargetInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SQPK:{}:{:?}:{:?}:{}:{}:{}:{}",
            Self::COMMAND,
            self.platform,
            self.region,
            self.is_debug,
            self.version,
            self.deleted_data_size,
            self.seek_count
        )
    }
}
