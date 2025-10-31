use std::io::Read;

use crate::config::ZiPatchConfig;
use crate::error::{Result, ZiPatchError};
use crate::inspection::ZiPatchCommandCounts;
use crate::util::BinaryReaderExt;

/// File Header chunk (FHDR)
///
/// Contains metadata about the patch file, including version and command counts
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileHeaderChunk {
    /// Version of the patch file format (2 or 3)
    pub version: u8,
    /// Patch type identifier (4-character string)
    pub patch_type: String,
    /// Number of entry files
    pub entry_files: u32,
    /// Command counts (only present in V3)
    pub command_counts: Option<ZiPatchCommandCounts>,
    /// Number of add directory commands (V3 only)
    pub add_directories: u32,
    /// Number of delete directory commands (V3 only)
    pub delete_directories: u32,
    /// Size of deleted data (V3 only)
    pub delete_data_size: i64,
    /// Minor version (V3 only)
    pub minor_version: u32,
    /// Repository name (V3 only)
    pub repository_name: u32,
}

impl FileHeaderChunk {
    pub const CHUNK_TYPE: &'static str = "FHDR";

    /// Reads a FileHeaderChunk from a reader
    pub fn read<R: Read>(reader: &mut R, _size: u32) -> Result<Self> {
        // Read version from upper 16 bits of a u32 (little-endian)
        let version_field = reader.read_u32_le()?;
        let version = (version_field >> 16) as u8;

        // Validate version
        if version != 2 && version != 3 {
            return Err(ZiPatchError::InvalidFileHeaderVersion(version));
        }

        let patch_type = reader.read_fixed_string(4)?;
        let entry_files = reader.read_u32_be()?;

        let (
            command_counts,
            add_directories,
            delete_directories,
            delete_data_size,
            minor_version,
            repository_name,
        ) = if version == 3 {
            let adir = reader.read_u32_be()?;
            let deld = reader.read_u32_be()?;

            // Delete data size is stored as two u32s (low, high)
            let delete_size_low = reader.read_u32_be()? as i64;
            let delete_size_high = reader.read_u32_be()? as i64;
            let delete_size = delete_size_low | (delete_size_high << 32);

            let minor_ver = reader.read_u32_be()?;
            let repo_name = reader.read_u32_be()?;

            // Read command counts
            let total = reader.read_u32_be()?;
            let sqpk_add = reader.read_u32_be()?;
            let sqpk_delete = reader.read_u32_be()?;
            let sqpk_expand = reader.read_u32_be()?;
            let sqpk_header = reader.read_u32_be()?;
            let sqpk_file = reader.read_u32_be()?;

            let counts = ZiPatchCommandCounts::with_counts(
                adir,
                deld,
                total,
                sqpk_add,
                sqpk_delete,
                sqpk_expand,
                sqpk_header,
                sqpk_file,
            );

            (Some(counts), adir, deld, delete_size, minor_ver, repo_name)
        } else {
            (None, 0, 0, 0, 0, 0)
        };

        // nb: 0xB8 bytes of unknown data for V3 and 0x08 bytes for V2, but we don't need to read it
        // as we use the advance guard pattern to skip remaining bytes

        Ok(Self {
            version,
            patch_type,
            entry_files,
            command_counts,
            add_directories,
            delete_directories,
            delete_data_size,
            minor_version,
            repository_name,
        })
    }

    /// Applies the chunk (no-op for file header)
    pub fn apply(&self, _config: &mut ZiPatchConfig) -> Result<()> {
        // File header doesn't modify anything
        Ok(())
    }
}

impl std::fmt::Display for FileHeaderChunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:V{}:{}",
            Self::CHUNK_TYPE,
            self.version,
            self.repository_name
        )
    }
}
