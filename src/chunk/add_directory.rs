use std::fs;
use std::io::Read;

use crate::config::ZiPatchConfig;
use crate::error::{Result, ZiPatchError};
use crate::util::BinaryReaderExt;

/// Add Directory chunk (ADIR)
///
/// Creates a new directory in the game installation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddDirectoryChunk {
    /// Name/path of the directory to create
    pub dir_name: String,
}

impl AddDirectoryChunk {
    pub const CHUNK_TYPE: &'static str = "ADIR";

    /// Reads an AddDirectoryChunk from a reader
    pub fn read<R: Read>(reader: &mut R, _size: u32) -> Result<Self> {
        let dir_name_len = reader.read_u32_be()?;
        let dir_name = reader.read_fixed_string(dir_name_len as usize)?;

        Ok(Self { dir_name })
    }

    /// Applies the chunk by creating the directory
    pub fn apply(&self, config: &mut ZiPatchConfig) -> Result<()> {
        let full_path = config.game_path().join(&self.dir_name);

        fs::create_dir_all(&full_path).map_err(|e| ZiPatchError::DirectoryCreationFailed {
            path: full_path,
            source: e,
        })?;

        Ok(())
    }
}

impl std::fmt::Display for AddDirectoryChunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", Self::CHUNK_TYPE, self.dir_name)
    }
}
