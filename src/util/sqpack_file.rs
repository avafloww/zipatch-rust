use std::io::{Read, Write};

use super::{SqexFile, SqexFileStream};
use crate::config::Platform;
use crate::error::Result;
use crate::util::binary_reader::BinaryReaderExt;

/// Base structure for Sqpack files (index and data files)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqpackFile {
    /// Main ID (first part of filename)
    pub main_id: u16,
    /// Sub ID (second part of filename, contains expansion ID in high byte)
    pub sub_id: u16,
    /// File ID (third part of filename)
    pub file_id: u32,
    /// Underlying SqexFile for path management
    pub sqex_file: SqexFile,
}

impl SqpackFile {
    /// Creates a new SqpackFile by reading from a binary reader
    pub fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let main_id = reader.read_u16_be()?;
        let sub_id = reader.read_u16_be()?;
        let file_id = reader.read_u32_be()?;

        let expansion_id = (sub_id >> 8) as u8;
        let expansion_path = Self::get_expansion_path(expansion_id);

        Ok(Self {
            main_id,
            sub_id,
            file_id,
            sqex_file: SqexFile::new(expansion_path),
        })
    }

    /// Gets the expansion ID from the sub_id
    pub fn expansion_id(&self) -> u8 {
        (self.sub_id >> 8) as u8
    }

    /// Gets the expansion path (e.g., "/sqpack/ffxiv/", "/sqpack/ex1/")
    fn get_expansion_path(expansion_id: u8) -> String {
        format!("/sqpack/{}/", SqexFile::get_expansion_folder(expansion_id))
    }

    /// Gets the base filename without extension (e.g., "0a0000.win32")
    pub fn get_base_filename(&self, platform: Platform) -> String {
        let platform_str = match platform {
            Platform::Win32 => "win32",
            Platform::Ps3 => "ps3",
            Platform::Ps4 => "ps4",
            Platform::Unknown => "unknown",
        };

        format!("{:02x}{:04x}.{}", self.main_id, self.sub_id, platform_str)
    }
}

/// Sqpack data file (.dat0, .dat1, etc.)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqpackDatFile {
    /// Base sqpack file info
    pub sqpack: SqpackFile,
}

impl SqpackDatFile {
    /// Creates a new SqpackDatFile by reading from a binary reader
    pub fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let sqpack = SqpackFile::read_from(reader)?;
        Ok(Self { sqpack })
    }

    /// Gets the full filename including .dat{N} extension
    pub fn get_file_name(&self, platform: Platform) -> String {
        format!(
            "{}{}.dat{}",
            Self::get_expansion_path(self.sqpack.expansion_id()),
            self.sqpack.get_base_filename(platform),
            self.sqpack.file_id
        )
    }

    /// Resolves the relative path for this file
    pub fn resolve_path(&mut self, platform: Platform) {
        self.sqpack.sqex_file.relative_path = self.get_file_name(platform);
    }

    /// Gets the expansion path (e.g., "/sqpack/ffxiv/", "/sqpack/ex1/")
    fn get_expansion_path(expansion_id: u8) -> String {
        format!("/sqpack/{}/", SqexFile::get_expansion_folder(expansion_id))
    }

    /// Gets a reference to the underlying SqexFile
    pub fn sqex_file(&self) -> &SqexFile {
        &self.sqpack.sqex_file
    }

    /// Gets a mutable reference to the underlying SqexFile
    pub fn sqex_file_mut(&mut self) -> &mut SqexFile {
        &mut self.sqpack.sqex_file
    }

    /// Writes an empty file block at the specified offset
    ///
    /// This creates a file block header with zeroed data
    pub fn write_empty_file_block_at(
        stream: &mut SqexFileStream,
        offset: i64,
        block_number: i64,
    ) -> Result<()> {
        // Wipe the block area
        stream.wipe_from_offset((block_number << 7) as u64, offset)?;
        stream.seek_to(offset as u64)?;

        // Write file block header
        // Block size
        stream.write_all(&(1i32 << 7).to_le_bytes())?;
        // Unknown field (0)
        stream.write_all(&0i32.to_le_bytes())?;
        // File size (0)
        stream.write_all(&0i32.to_le_bytes())?;
        // Total number of blocks
        stream.write_all(&((block_number - 1) as i32).to_le_bytes())?;
        // Used number of blocks (0)
        stream.write_all(&0i32.to_le_bytes())?;

        Ok(())
    }
}

impl std::fmt::Display for SqpackDatFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Default to Win32 for display
        write!(f, "{}", self.get_file_name(Platform::Win32))
    }
}

/// Sqpack index file (.index, .index2, etc.)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqpackIndexFile {
    /// Base sqpack file info
    pub sqpack: SqpackFile,
}

impl SqpackIndexFile {
    /// Creates a new SqpackIndexFile by reading from a binary reader
    pub fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let sqpack = SqpackFile::read_from(reader)?;
        Ok(Self { sqpack })
    }

    /// Gets the full filename including .index or .index{N} extension
    pub fn get_file_name(&self, platform: Platform) -> String {
        let index_suffix = if self.sqpack.file_id == 0 {
            String::new()
        } else {
            self.sqpack.file_id.to_string()
        };

        format!(
            "{}{}.index{}",
            Self::get_expansion_path(self.sqpack.expansion_id()),
            self.sqpack.get_base_filename(platform),
            index_suffix
        )
    }

    /// Resolves the relative path for this file
    pub fn resolve_path(&mut self, platform: Platform) {
        self.sqpack.sqex_file.relative_path = self.get_file_name(platform);
    }

    /// Gets the expansion path (e.g., "/sqpack/ffxiv/", "/sqpack/ex1/")
    fn get_expansion_path(expansion_id: u8) -> String {
        format!("/sqpack/{}/", SqexFile::get_expansion_folder(expansion_id))
    }

    /// Gets a reference to the underlying SqexFile
    pub fn sqex_file(&self) -> &SqexFile {
        &self.sqpack.sqex_file
    }

    /// Gets a mutable reference to the underlying SqexFile
    pub fn sqex_file_mut(&mut self) -> &mut SqexFile {
        &mut self.sqpack.sqex_file
    }
}

impl std::fmt::Display for SqpackIndexFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Default to Win32 for display
        write!(f, "{}", self.get_file_name(Platform::Win32))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_sqpack_file_read() {
        // MainId=0x0A, SubId=0x0000, FileId=0
        let data = vec![0x00, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let mut cursor = Cursor::new(data);

        let sqpack = SqpackFile::read_from(&mut cursor).unwrap();
        assert_eq!(sqpack.main_id, 0x0A);
        assert_eq!(sqpack.sub_id, 0x0000);
        assert_eq!(sqpack.file_id, 0);
        assert_eq!(sqpack.expansion_id(), 0);
    }

    #[test]
    fn test_sqpack_dat_file_name() {
        let data = vec![0x00, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let mut cursor = Cursor::new(data);

        let dat = SqpackDatFile::read_from(&mut cursor).unwrap();
        let filename = dat.get_file_name(Platform::Win32);

        assert_eq!(filename, "/sqpack/ffxiv/0a0000.win32.dat0");
    }

    #[test]
    fn test_sqpack_index_file_name() {
        let data = vec![0x00, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let mut cursor = Cursor::new(data);

        let index = SqpackIndexFile::read_from(&mut cursor).unwrap();
        let filename = index.get_file_name(Platform::Win32);

        assert_eq!(filename, "/sqpack/ffxiv/0a0000.win32.index");
    }

    #[test]
    fn test_sqpack_index_file_name_with_id() {
        let data = vec![0x00, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02];
        let mut cursor = Cursor::new(data);

        let index = SqpackIndexFile::read_from(&mut cursor).unwrap();
        let filename = index.get_file_name(Platform::Win32);

        assert_eq!(filename, "/sqpack/ffxiv/0a0000.win32.index2");
    }

    #[test]
    fn test_expansion_id_extraction() {
        let data = vec![0x00, 0x0A, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00];
        let mut cursor = Cursor::new(data);

        let sqpack = SqpackFile::read_from(&mut cursor).unwrap();
        assert_eq!(sqpack.expansion_id(), 1);
    }
}
