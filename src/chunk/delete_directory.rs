use std::fs;
use std::io::Read;

use crate::config::ZiPatchConfig;
use crate::error::{Result, ZiPatchError};
use crate::util::BinaryReaderExt;

/// Delete Directory chunk (DELD)
///
/// Deletes a directory from the game installation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteDirectoryChunk {
    /// Name/path of the directory to delete
    pub dir_name: String,
}

impl DeleteDirectoryChunk {
    pub const CHUNK_TYPE: &'static str = "DELD";

    /// Reads a DeleteDirectoryChunk from a reader
    pub fn read<R: Read>(reader: &mut R, _size: u32) -> Result<Self> {
        let dir_name_len = reader.read_u32_be()?;
        let dir_name = reader.read_fixed_string(dir_name_len as usize)?;

        Ok(Self { dir_name })
    }

    /// Applies the chunk by deleting the directory
    pub fn apply(&self, config: &mut ZiPatchConfig) -> Result<()> {
        let full_path = config.game_path().join(&self.dir_name);

        // Only delete if the directory exists
        if full_path.exists() && full_path.is_dir() {
            fs::remove_dir(&full_path).map_err(|e| ZiPatchError::FileOperationFailed {
                path: full_path,
                source: e,
            })?;
        }

        Ok(())
    }
}

impl std::fmt::Display for DeleteDirectoryChunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", Self::CHUNK_TYPE, self.dir_name)
    }
}
