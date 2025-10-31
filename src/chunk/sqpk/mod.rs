mod add_data;
mod delete_data;
mod expand_data;
mod file;
mod header;
mod index;
mod patch_info;
mod target_info;

pub use add_data::SqpkAddData;
pub use delete_data::SqpkDeleteData;
pub use expand_data::SqpkExpandData;
pub use file::{OperationKind, SqpkFile};
pub use header::{SqpkHeader, TargetFile, TargetFileKind, TargetHeaderKind};
pub use index::{IndexCommandKind, SqpkIndex};
pub use patch_info::SqpkPatchInfo;
pub use target_info::{RegionId, SqpkTargetInfo};

use std::io::Read;

use crate::config::ZiPatchConfig;
use crate::error::{Result, ZiPatchError};
use crate::util::BinaryReaderExt;

/// SQPK command variants
#[derive(Debug, Clone)]
pub enum SqpkCommand {
    AddData(SqpkAddData),
    DeleteData(SqpkDeleteData),
    ExpandData(SqpkExpandData),
    File(SqpkFile),
    Header(SqpkHeader),
    Index(SqpkIndex),
    PatchInfo(SqpkPatchInfo),
    TargetInfo(SqpkTargetInfo),
}

impl SqpkCommand {
    /// Reads an SQPK command from a reader
    ///
    /// # Arguments
    /// * `reader` - The reader to read from
    /// * `outer_size` - The size from the outer SQPK chunk
    pub fn read<R: Read>(reader: &mut R, outer_size: u32, offset: u64) -> Result<Self> {
        // Read inner size (should match outer size)
        let inner_size = reader.read_i32_be()?;

        // Validate size match
        if inner_size != outer_size as i32 {
            return Err(ZiPatchError::SqpkSizeMismatch {
                outer: outer_size,
                inner: inner_size,
            });
        }

        // Read command character
        let command_char = {
            let mut buf = [0u8; 1];
            reader.read_exact(&mut buf)?;
            buf[0] as char
        };

        // Calculate remaining size for the command data
        // We've read 4 bytes (inner_size) + 1 byte (command char) = 5 bytes
        let remaining_size = outer_size.saturating_sub(5) as u64;

        // Dispatch to appropriate command based on command character
        let command = match command_char {
            'A' => SqpkCommand::AddData(SqpkAddData::read(reader)?),
            'D' => SqpkCommand::DeleteData(SqpkDeleteData::read(reader)?),
            'E' => SqpkCommand::ExpandData(SqpkExpandData::read(reader)?),
            'F' => SqpkCommand::File(SqpkFile::read(reader, remaining_size)?),
            'H' => SqpkCommand::Header(SqpkHeader::read(reader)?),
            'I' => SqpkCommand::Index(SqpkIndex::read(reader)?),
            'X' => SqpkCommand::PatchInfo(SqpkPatchInfo::read(reader)?),
            'T' => SqpkCommand::TargetInfo(SqpkTargetInfo::read(reader)?),
            _ => {
                return Err(ZiPatchError::UnknownSqpkCommand(command_char, offset));
            }
        };

        Ok(command)
    }

    /// Applies the SQPK command
    pub fn apply(&mut self, config: &mut ZiPatchConfig) -> Result<()> {
        match self {
            SqpkCommand::AddData(cmd) => cmd.apply(config),
            SqpkCommand::DeleteData(cmd) => cmd.apply(config),
            SqpkCommand::ExpandData(cmd) => cmd.apply(config),
            SqpkCommand::File(cmd) => cmd.apply(config),
            SqpkCommand::Header(cmd) => cmd.apply(config),
            SqpkCommand::Index(cmd) => cmd.apply(config),
            SqpkCommand::PatchInfo(cmd) => cmd.apply(config),
            SqpkCommand::TargetInfo(cmd) => cmd.apply(config),
        }
    }

    /// Gets the command character
    pub fn command_char(&self) -> char {
        match self {
            SqpkCommand::AddData(_) => SqpkAddData::COMMAND,
            SqpkCommand::DeleteData(_) => SqpkDeleteData::COMMAND,
            SqpkCommand::ExpandData(_) => SqpkExpandData::COMMAND,
            SqpkCommand::File(_) => SqpkFile::COMMAND,
            SqpkCommand::Header(_) => SqpkHeader::COMMAND,
            SqpkCommand::Index(_) => SqpkIndex::COMMAND,
            SqpkCommand::PatchInfo(_) => SqpkPatchInfo::COMMAND,
            SqpkCommand::TargetInfo(_) => SqpkTargetInfo::COMMAND,
        }
    }
}

impl std::fmt::Display for SqpkCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SqpkCommand::AddData(cmd) => write!(f, "{}", cmd),
            SqpkCommand::DeleteData(cmd) => write!(f, "{}", cmd),
            SqpkCommand::ExpandData(cmd) => write!(f, "{}", cmd),
            SqpkCommand::File(cmd) => write!(f, "{}", cmd),
            SqpkCommand::Header(cmd) => write!(f, "{}", cmd),
            SqpkCommand::Index(cmd) => write!(f, "{}", cmd),
            SqpkCommand::PatchInfo(cmd) => write!(f, "{}", cmd),
            SqpkCommand::TargetInfo(cmd) => write!(f, "{}", cmd),
        }
    }
}
