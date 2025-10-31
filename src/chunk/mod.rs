mod add_directory;
mod apply_free_space;
mod apply_option;
mod delete_directory;
mod end_of_file;
mod file_header;
pub mod sqpk;
mod xxxx;

pub use add_directory::AddDirectoryChunk;
pub use apply_free_space::ApplyFreeSpaceChunk;
pub use apply_option::{ApplyOptionChunk, ApplyOptionKind};
pub use delete_directory::DeleteDirectoryChunk;
pub use end_of_file::EndOfFileChunk;
pub use file_header::FileHeaderChunk;
pub use sqpk::SqpkCommand;
pub use xxxx::XXXXChunk;

use std::io::{Read, Seek};

use crate::config::ZiPatchConfig;
use crate::error::{Result, ZiPatchError};
use crate::util::{AdvanceGuard, BinaryReaderExt, ChecksumReader};

/// ZiPatch chunk variants
#[derive(Debug, Clone)]
pub enum ZiPatchChunk {
    FileHeader(FileHeaderChunk),
    ApplyOption(ApplyOptionChunk),
    ApplyFreeSpace(ApplyFreeSpaceChunk),
    AddDirectory(AddDirectoryChunk),
    DeleteDirectory(DeleteDirectoryChunk),
    Sqpk(SqpkCommand),
    EndOfFile(EndOfFileChunk),
    XXXX(XXXXChunk),
}

impl ZiPatchChunk {
    /// Reads a chunk from a checksummed reader
    ///
    /// # Arguments
    /// * `reader` - Checksummed reader at the start of a chunk
    pub fn read<R: Read + Seek>(reader: &mut ChecksumReader<R>) -> Result<Self> {
        let offset = reader.get_mut().stream_position()?;

        // Read chunk size (big-endian)
        let size = reader.read_u32_be()?;

        // Read chunk type (4-character string)
        reader.init_crc32();
        let chunk_type = reader.read_chunk_type()?;

        // Parse the chunk based on type
        // The guard ensures we advance to the correct position even if reading fails
        // All reads go through the guard, which delegates to ChecksumReader
        let chunk = {
            let mut guard = AdvanceGuard::new(reader, size as u64)?;

            match chunk_type.as_str() {
                "FHDR" => ZiPatchChunk::FileHeader(FileHeaderChunk::read(&mut guard, size)?),
                "APLY" => ZiPatchChunk::ApplyOption(ApplyOptionChunk::read(&mut guard, size)?),
                "APFS" => {
                    ZiPatchChunk::ApplyFreeSpace(ApplyFreeSpaceChunk::read(&mut guard, size)?)
                }
                "ADIR" => ZiPatchChunk::AddDirectory(AddDirectoryChunk::read(&mut guard, size)?),
                "DELD" => {
                    ZiPatchChunk::DeleteDirectory(DeleteDirectoryChunk::read(&mut guard, size)?)
                }
                "SQPK" => ZiPatchChunk::Sqpk(SqpkCommand::read(&mut guard, size, offset)?),
                "EOF_" => ZiPatchChunk::EndOfFile(EndOfFileChunk::read(&mut guard, size)?),
                "XXXX" => ZiPatchChunk::XXXX(XXXXChunk::read(&mut guard, size)?),
                _ => {
                    return Err(ZiPatchError::UnknownChunkType(chunk_type, offset));
                }
            }
        };

        // Verify checksum
        let calculated_checksum = reader.get_crc32();
        let expected_checksum = reader.read_u32_be()?;

        if calculated_checksum != expected_checksum {
            return Err(ZiPatchError::ChecksumMismatch {
                offset,
                expected: expected_checksum,
                actual: calculated_checksum,
            });
        }

        Ok(chunk)
    }

    /// Applies the chunk to the configuration
    pub fn apply(&mut self, config: &mut ZiPatchConfig) -> Result<()> {
        match self {
            ZiPatchChunk::FileHeader(chunk) => chunk.apply(config),
            ZiPatchChunk::ApplyOption(chunk) => chunk.apply(config),
            ZiPatchChunk::ApplyFreeSpace(chunk) => chunk.apply(config),
            ZiPatchChunk::AddDirectory(chunk) => chunk.apply(config),
            ZiPatchChunk::DeleteDirectory(chunk) => chunk.apply(config),
            ZiPatchChunk::Sqpk(chunk) => chunk.apply(config),
            ZiPatchChunk::EndOfFile(chunk) => chunk.apply(config),
            ZiPatchChunk::XXXX(chunk) => chunk.apply(config),
        }
    }

    /// Checks if this is an EOF chunk
    pub fn is_eof(&self) -> bool {
        matches!(self, ZiPatchChunk::EndOfFile(_))
    }

    /// Gets the chunk type string
    pub fn chunk_type(&self) -> &'static str {
        match self {
            ZiPatchChunk::FileHeader(_) => FileHeaderChunk::CHUNK_TYPE,
            ZiPatchChunk::ApplyOption(_) => ApplyOptionChunk::CHUNK_TYPE,
            ZiPatchChunk::ApplyFreeSpace(_) => ApplyFreeSpaceChunk::CHUNK_TYPE,
            ZiPatchChunk::AddDirectory(_) => AddDirectoryChunk::CHUNK_TYPE,
            ZiPatchChunk::DeleteDirectory(_) => DeleteDirectoryChunk::CHUNK_TYPE,
            ZiPatchChunk::Sqpk(_) => "SQPK",
            ZiPatchChunk::EndOfFile(_) => EndOfFileChunk::CHUNK_TYPE,
            ZiPatchChunk::XXXX(_) => XXXXChunk::CHUNK_TYPE,
        }
    }
}

impl std::fmt::Display for ZiPatchChunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZiPatchChunk::FileHeader(chunk) => write!(f, "{}", chunk),
            ZiPatchChunk::ApplyOption(chunk) => write!(f, "{}", chunk),
            ZiPatchChunk::ApplyFreeSpace(chunk) => write!(f, "{}", chunk),
            ZiPatchChunk::AddDirectory(chunk) => write!(f, "{}", chunk),
            ZiPatchChunk::DeleteDirectory(chunk) => write!(f, "{}", chunk),
            ZiPatchChunk::Sqpk(chunk) => write!(f, "{}", chunk),
            ZiPatchChunk::EndOfFile(chunk) => write!(f, "{}", chunk),
            ZiPatchChunk::XXXX(chunk) => write!(f, "{}", chunk),
        }
    }
}
