use std::fs;
use std::io::{Read, Seek, SeekFrom};

use crate::config::ZiPatchConfig;
use crate::error::Result;
use crate::util::{BinaryReaderExt, SqexFile, SqpkCompressedBlock};

/// SQPK File command ('F')
///
/// Performs file operations (add, delete, remove all, make directories)
#[derive(Debug, Clone)]
pub struct SqpkFile {
    /// Operation to perform
    pub operation: OperationKind,
    /// File offset
    pub file_offset: i64,
    /// File size
    pub file_size: i64,
    /// Expansion ID
    pub expansion_id: u16,
    /// Target file
    pub target_file: SqexFile,
    /// Compressed data blocks (only for AddFile operation)
    pub compressed_data: Vec<SqpkCompressedBlock>,
}

/// Kind of file operation
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationKind {
    /// Add a file
    AddFile = b'A',
    /// Remove all files in an expansion
    RemoveAll = b'R',
    /// Delete a specific file (rarely seen)
    DeleteFile = b'D',
    /// Make directory tree (rarely seen)
    MakeDirTree = b'M',
}

impl OperationKind {
    /// Creates an OperationKind from a u8 value
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            b'A' => Some(OperationKind::AddFile),
            b'R' => Some(OperationKind::RemoveAll),
            b'D' => Some(OperationKind::DeleteFile),
            b'M' => Some(OperationKind::MakeDirTree),
            _ => None,
        }
    }
}

impl SqpkFile {
    pub const COMMAND: char = 'F';

    /// Reads an SqpkFile from a reader
    pub fn read<R: Read>(reader: &mut R, remaining_size: u64) -> Result<Self> {
        let operation_byte = {
            let mut buf = [0u8; 1];
            reader.read_exact(&mut buf)?;
            buf[0]
        };

        let operation = OperationKind::from_u8(operation_byte).unwrap_or(OperationKind::AddFile);

        // Read and discard 2 bytes of alignment
        let _alignment = reader.read_bytes_required(2)?;

        let file_offset = reader.read_i64_be()?;
        let file_size = reader.read_i64_be()?;

        let path_len = reader.read_u32_be()?;
        let expansion_id = reader.read_u16_be()?;

        // Read and discard 2 bytes of padding
        let _padding = reader.read_bytes_required(2)?;

        let target_file = SqexFile::new(reader.read_fixed_string(path_len as usize)?);

        let mut compressed_data = Vec::new();

        // Calculate bytes consumed so far
        let header_bytes = 1 + 2 + 8 + 8 + 4 + 2 + 2 + path_len;
        let mut bytes_remaining = remaining_size.saturating_sub(header_bytes as u64);

        if operation == OperationKind::AddFile {
            while bytes_remaining > 0 {
                let block = SqpkCompressedBlock::read_from(reader)?;
                let block_bytes = block.header_size as u64
                    + (block.compressed_block_length() - block.header_size) as u64;

                bytes_remaining = bytes_remaining.saturating_sub(block_bytes);
                compressed_data.push(block);
            }
        }

        Ok(Self {
            operation,
            file_offset,
            file_size,
            expansion_id,
            target_file,
            compressed_data,
        })
    }

    /// Filter for RemoveAll operation - excludes .var files and specific .bk2 files
    fn remove_all_filter(file_path: &str) -> bool {
        let exclusions = [".var", "00000.bk2", "00001.bk2", "00002.bk2", "00003.bk2"];
        !exclusions.iter().any(|ext| file_path.ends_with(ext))
    }

    /// Applies the command by performing the file operation
    pub fn apply(&mut self, config: &mut ZiPatchConfig) -> Result<()> {
        match self.operation {
            OperationKind::AddFile => {
                let game_path = config.game_path().to_path_buf();

                // Create directory tree
                self.target_file.create_directory_tree(&game_path)?;

                if let Some(ref mut store) = config.store {
                    // Use store
                    let file_stream = self
                        .target_file
                        .open_stream_with_store(store, &game_path, true, 5, 1)?;

                    // If starting at offset 0, truncate the file
                    if self.file_offset == 0 {
                        file_stream.get_mut().set_len(0)?;
                    }

                    // Seek to the file offset
                    file_stream.seek(SeekFrom::Start(self.file_offset as u64))?;

                    // Decompress all blocks into the file
                    for block in &self.compressed_data {
                        block.decompress_into(file_stream)?;
                    }
                } else {
                    // Open directly
                    let mut file_stream = self.target_file.open_stream(&game_path, true, 5, 1)?;

                    // If starting at offset 0, truncate the file
                    if self.file_offset == 0 {
                        file_stream.get_mut().set_len(0)?;
                    }

                    // Seek to the file offset
                    file_stream.seek(SeekFrom::Start(self.file_offset as u64))?;

                    // Decompress all blocks into the file
                    for block in &self.compressed_data {
                        block.decompress_into(&mut file_stream)?;
                    }
                }
            }

            OperationKind::RemoveAll => {
                // Get all files for the expansion
                let files =
                    SqexFile::get_all_expansion_files(config.game_path(), self.expansion_id)?;

                // Delete all files that pass the filter
                for file_path in files {
                    if let Some(path_str) = file_path.to_str() {
                        if Self::remove_all_filter(path_str) {
                            let _ = fs::remove_file(&file_path); // Ignore errors
                        }
                    }
                }
            }

            OperationKind::DeleteFile => {
                let full_path = config.game_path().join(&self.target_file.relative_path);
                if full_path.exists() {
                    fs::remove_file(&full_path)?;
                }
            }

            OperationKind::MakeDirTree => {
                let full_path = config.game_path().join(&self.target_file.relative_path);
                fs::create_dir_all(&full_path)?;
            }
        }

        Ok(())
    }
}

impl std::fmt::Display for SqpkFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SQPK:{}:{:?}:{}:{}:{}:{}",
            Self::COMMAND,
            self.operation,
            self.file_offset,
            self.file_size,
            self.expansion_id,
            self.target_file
        )
    }
}
