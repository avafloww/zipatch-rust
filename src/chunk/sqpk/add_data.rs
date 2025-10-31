use std::io::Read;

use crate::config::ZiPatchConfig;
use crate::error::Result;
use crate::util::{BinaryReaderExt, SqpackDatFile};

/// SQPK Add Data command ('A')
///
/// Adds data blocks to .dat files
#[derive(Debug, Clone)]
pub struct SqpkAddData {
    /// Target .dat file
    pub target_file: SqpackDatFile,
    /// Block offset (shifted left by 7)
    pub block_offset: i64,
    /// Block number/size (shifted left by 7)
    pub block_number: i64,
    /// Block delete number (shifted left by 7)
    pub block_delete_number: i64,
    /// Block data
    pub block_data: Vec<u8>,
}

impl SqpkAddData {
    pub const COMMAND: char = 'A';

    /// Reads an SqpkAddData from a reader
    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        // Read and discard 3 bytes of alignment
        let _alignment = reader.read_bytes_required(3)?;

        let target_file = SqpackDatFile::read_from(reader)?;

        let block_offset = (reader.read_u32_be()? as i64) << 7;
        let block_number = (reader.read_u32_be()? as i64) << 7;
        let block_delete_number = (reader.read_u32_be()? as i64) << 7;

        // Read block data
        let block_data = reader.read_bytes_required(block_number as usize)?;

        Ok(Self {
            target_file,
            block_offset,
            block_number,
            block_delete_number,
            block_data,
        })
    }

    /// Applies the command by writing block data and wiping deleted data
    pub fn apply(&mut self, config: &mut ZiPatchConfig) -> Result<()> {
        self.target_file.resolve_path(config.platform);

        let game_path = config.game_path().to_path_buf();

        if let Some(ref mut store) = config.store {
            let file = self
                .target_file
                .sqex_file_mut()
                .open_stream_with_store(store, &game_path, true, 5, 1)?;
            file.write_from_offset(&self.block_data, self.block_offset)?;
            file.wipe(self.block_delete_number as u64)?;
        } else {
            let mut file = self
                .target_file
                .sqex_file()
                .open_stream(&game_path, true, 5, 1)?;
            file.write_from_offset(&self.block_data, self.block_offset)?;
            file.wipe(self.block_delete_number as u64)?;
        }

        Ok(())
    }
}

impl std::fmt::Display for SqpkAddData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SQPK:{}:{}:{}:{}:{}",
            Self::COMMAND,
            self.target_file,
            self.block_offset,
            self.block_number,
            self.block_delete_number
        )
    }
}
