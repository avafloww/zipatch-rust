use std::io::Read;

use crate::config::ZiPatchConfig;
use crate::error::Result;
use crate::util::{BinaryReaderExt, SqpackDatFile, SqpackIndexFile};

/// SQPK Header command ('H')
///
/// Updates pack file headers
#[derive(Debug, Clone)]
pub struct SqpkHeader {
    /// File kind (Dat or Index)
    pub file_kind: TargetFileKind,
    /// Header kind (Version, Index, or Data)
    pub header_kind: TargetHeaderKind,
    /// Target file (either SqpackDatFile or SqpackIndexFile)
    pub target_file: TargetFile,
    /// Header data (1024 bytes)
    pub header_data: Vec<u8>,
}

/// Kind of target file
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetFileKind {
    /// .dat file
    Dat = b'D',
    /// .index file
    Index = b'I',
}

impl TargetFileKind {
    /// Creates a TargetFileKind from a u8 value
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            b'D' => Some(TargetFileKind::Dat),
            b'I' => Some(TargetFileKind::Index),
            _ => None,
        }
    }
}

/// Kind of header to update
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetHeaderKind {
    /// Version header
    Version = b'V',
    /// Index header
    Index = b'I',
    /// Data header
    Data = b'D',
}

impl TargetHeaderKind {
    /// Creates a TargetHeaderKind from a u8 value
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            b'V' => Some(TargetHeaderKind::Version),
            b'I' => Some(TargetHeaderKind::Index),
            b'D' => Some(TargetHeaderKind::Data),
            _ => None,
        }
    }
}

/// Target file (either Dat or Index)
#[derive(Debug, Clone)]
pub enum TargetFile {
    Dat(SqpackDatFile),
    Index(SqpackIndexFile),
}

impl SqpkHeader {
    pub const COMMAND: char = 'H';
    pub const HEADER_SIZE: usize = 1024;

    /// Reads an SqpkHeader from a reader
    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let file_kind_byte = {
            let mut buf = [0u8; 1];
            reader.read_exact(&mut buf)?;
            buf[0]
        };

        let header_kind_byte = {
            let mut buf = [0u8; 1];
            reader.read_exact(&mut buf)?;
            buf[0]
        };

        // Read and discard alignment byte
        {
            let mut buf = [0u8; 1];
            reader.read_exact(&mut buf)?;
        };

        let file_kind = TargetFileKind::from_u8(file_kind_byte).unwrap_or(TargetFileKind::Dat);
        let header_kind =
            TargetHeaderKind::from_u8(header_kind_byte).unwrap_or(TargetHeaderKind::Version);

        let target_file = match file_kind {
            TargetFileKind::Dat => TargetFile::Dat(SqpackDatFile::read_from(reader)?),
            TargetFileKind::Index => TargetFile::Index(SqpackIndexFile::read_from(reader)?),
        };

        let header_data = reader.read_bytes_required(Self::HEADER_SIZE)?;

        Ok(Self {
            file_kind,
            header_kind,
            target_file,
            header_data,
        })
    }

    /// Applies the command by writing the header data
    pub fn apply(&mut self, config: &mut ZiPatchConfig) -> Result<()> {
        let offset = match self.header_kind {
            TargetHeaderKind::Version => 0,
            _ => Self::HEADER_SIZE as i64,
        };

        let game_path = config.game_path().to_path_buf();

        match &mut self.target_file {
            TargetFile::Dat(dat_file) => {
                dat_file.resolve_path(config.platform);

                if let Some(ref mut store) = config.store {
                    let file = dat_file
                        .sqex_file_mut()
                        .open_stream_with_store(store, &game_path, true, 5, 1)?;
                    file.write_from_offset(&self.header_data, offset)?;
                } else {
                    let mut file = dat_file.sqex_file().open_stream(&game_path, true, 5, 1)?;
                    file.write_from_offset(&self.header_data, offset)?;
                }
            }
            TargetFile::Index(index_file) => {
                index_file.resolve_path(config.platform);

                if let Some(ref mut store) = config.store {
                    let file = index_file
                        .sqex_file_mut()
                        .open_stream_with_store(store, &game_path, true, 5, 1)?;
                    file.write_from_offset(&self.header_data, offset)?;
                } else {
                    let mut file = index_file.sqex_file().open_stream(&game_path, true, 5, 1)?;
                    file.write_from_offset(&self.header_data, offset)?;
                }
            }
        }

        Ok(())
    }
}

impl std::fmt::Display for SqpkHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let target_str = match &self.target_file {
            TargetFile::Dat(dat) => format!("{}", dat),
            TargetFile::Index(index) => format!("{}", index),
        };

        write!(
            f,
            "SQPK:{}:{:?}:{:?}:{}",
            Self::COMMAND,
            self.file_kind,
            self.header_kind,
            target_str
        )
    }
}
