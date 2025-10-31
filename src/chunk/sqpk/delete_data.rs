use std::io::Read;

use crate::config::ZiPatchConfig;
use crate::error::Result;
use crate::util::{BinaryReaderExt, SqpackDatFile};

/// SQPK Delete Data command ('D')
///
/// Deletes data blocks from .dat files
#[derive(Debug, Clone)]
pub struct SqpkDeleteData {
    /// Target .dat file
    pub target_file: SqpackDatFile,
    /// Block offset (shifted left by 7)
    pub block_offset: i64,
    /// Block number
    pub block_number: u32,
}

impl SqpkDeleteData {
    pub const COMMAND: char = 'D';

    /// Reads an SqpkDeleteData from a reader
    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        // Read and discard 3 bytes of alignment
        let _alignment = reader.read_bytes_required(3)?;

        let target_file = SqpackDatFile::read_from(reader)?;

        let block_offset = (reader.read_u32_be()? as i64) << 7;
        let block_number = reader.read_u32_be()?;

        // Read and discard reserved field
        let _reserved = reader.read_u32_be()?;

        Ok(Self {
            target_file,
            block_offset,
            block_number,
        })
    }

    /// Applies the command by writing an empty file block
    pub fn apply(&mut self, config: &mut ZiPatchConfig) -> Result<()> {
        self.target_file.resolve_path(config.platform);

        let game_path = config.game_path().to_path_buf();

        if let Some(ref mut store) = config.store {
            let file = self
                .target_file
                .sqex_file_mut()
                .open_stream_with_store(store, &game_path, true, 5, 1)?;
            SqpackDatFile::write_empty_file_block_at(
                file,
                self.block_offset,
                self.block_number as i64,
            )?;
        } else {
            let mut file = self
                .target_file
                .sqex_file()
                .open_stream(&game_path, true, 5, 1)?;
            SqpackDatFile::write_empty_file_block_at(
                &mut file,
                self.block_offset,
                self.block_number as i64,
            )?;
        }

        Ok(())
    }
}

impl std::fmt::Display for SqpkDeleteData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SQPK:{}:{}:{}:{}",
            Self::COMMAND,
            self.target_file,
            self.block_offset,
            self.block_number
        )
    }
}
