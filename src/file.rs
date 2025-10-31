use std::collections::HashSet;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use crate::chunk::{FileHeaderChunk, ZiPatchChunk};
use crate::config::ZiPatchConfig;
use crate::error::{Result, ZiPatchError};
use crate::inspection::{ZiPatchChangeSet, ZiPatchCommandCounts};
use crate::util::{BinaryReaderExt, ChecksumReader, SqexFile};

/// Magic number for ZiPatch files (3 x u32 big-endian)
const ZIPATCH_MAGIC: [u32; 3] = [0x50495A91, 0x48435441, 0x0A1A0A0D];

/// Main ZiPatch file reader
///
/// Reads and parses FFXIV patch files.
pub struct ZiPatchFile<R: Read + Seek> {
    reader: ChecksumReader<R>,
    head_position: u64,
    header: FileHeaderChunk,
}

impl ZiPatchFile<File> {
    /// Opens a ZiPatch file from a file path
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path)?;
        Self::new(file)
    }
}

impl<R: Read + Seek> ZiPatchFile<R> {
    /// Creates a new ZiPatchFile from a reader
    pub fn new(mut reader: R) -> Result<Self> {
        // Read and verify magic number
        let mut magic = [0u32; 3];
        for m in &mut magic {
            *m = reader.read_u32_le()?;
        }

        if magic != ZIPATCH_MAGIC {
            return Err(ZiPatchError::InvalidMagic(magic));
        }

        let head_position = reader.stream_position()?;

        // Wrap in ChecksumReader
        let mut checksum_reader = ChecksumReader::new(reader);

        // Find the file header chunk by reading directly
        let mut header = None;

        loop {
            let chunk = ZiPatchChunk::read(&mut checksum_reader)?;

            if let ZiPatchChunk::FileHeader(fhdr) = chunk {
                header = Some(fhdr);
                break;
            }

            // Stop if we hit EOF without finding header
            if chunk.is_eof() {
                break;
            }
        }

        let header =
            header.ok_or_else(|| ZiPatchError::Custom("Could not find FHDR chunk".to_string()))?;

        // Rewind back to head position
        checksum_reader
            .get_mut()
            .seek(SeekFrom::Start(head_position))?;

        Ok(Self {
            reader: checksum_reader,
            head_position,
            header,
        })
    }

    /// Gets a reference to the file header
    pub fn header(&self) -> &FileHeaderChunk {
        &self.header
    }

    /// Creates an iterator over all chunks in the file
    pub fn chunks(&mut self) -> ChunkIterator<'_, R> {
        // Save current position
        let current_pos = self
            .reader
            .get_mut()
            .stream_position()
            .unwrap_or(self.head_position);

        // Seek to head position
        let _ = self
            .reader
            .get_mut()
            .seek(SeekFrom::Start(self.head_position));

        ChunkIterator::new(&mut self.reader, current_pos)
    }

    /// Calculates which files were changed by this patch
    pub fn calculate_changed_files(&mut self, config: &ZiPatchConfig) -> Result<ZiPatchChangeSet> {
        let start_pos = self.reader.get_mut().stream_position()?;
        self.reader
            .get_mut()
            .seek(SeekFrom::Start(self.head_position))?;

        let mut added = HashSet::new();
        let mut deleted = HashSet::new();
        let mut modified = HashSet::new();

        loop {
            let chunk = ZiPatchChunk::read(&mut self.reader)?;

            if chunk.is_eof() {
                break;
            }

            match chunk {
                ZiPatchChunk::AddDirectory(ref adir) => {
                    added.insert(adir.dir_name.clone());
                }
                ZiPatchChunk::DeleteDirectory(ref deld) => {
                    deleted.insert(deld.dir_name.clone());
                }
                ZiPatchChunk::Sqpk(ref sqpk) => match sqpk {
                    crate::chunk::SqpkCommand::Header(ref h) => {
                        use crate::chunk::sqpk::TargetFile;
                        let filename = match &h.target_file {
                            TargetFile::Dat(dat) => dat.get_file_name(config.platform),
                            TargetFile::Index(idx) => idx.get_file_name(config.platform),
                        };
                        modified.insert(filename);
                    }
                    crate::chunk::SqpkCommand::File(ref f) => {
                        use crate::chunk::sqpk::OperationKind;
                        match f.operation {
                            OperationKind::AddFile => {
                                if f.file_offset == 0 {
                                    added.insert(f.target_file.relative_path.clone());
                                } else {
                                    modified.insert(f.target_file.relative_path.clone());
                                }
                            }
                            OperationKind::DeleteFile => {
                                deleted.insert(f.target_file.relative_path.clone());
                            }
                            OperationKind::RemoveAll => {
                                let expansion =
                                    SqexFile::get_expansion_folder(f.expansion_id as u8);
                                deleted.insert(format!("sqpack/{}/", expansion));
                                deleted.insert(format!("movie/{}/", expansion));
                            }
                            OperationKind::MakeDirTree => {
                                added.insert(f.target_file.relative_path.clone());
                            }
                        }
                    }
                    crate::chunk::SqpkCommand::AddData(ref a) => {
                        modified.insert(a.target_file.get_file_name(config.platform));
                    }
                    crate::chunk::SqpkCommand::DeleteData(ref d) => {
                        modified.insert(d.target_file.get_file_name(config.platform));
                    }
                    crate::chunk::SqpkCommand::ExpandData(ref e) => {
                        modified.insert(e.target_file.get_file_name(config.platform));
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        // Remove items from 'added' that are in 'modified'
        added.retain(|item| !modified.contains(item));

        self.reader.get_mut().seek(SeekFrom::Start(start_pos))?;

        Ok(ZiPatchChangeSet::with_changes(
            added.into_iter().collect(),
            deleted.into_iter().collect(),
            modified.into_iter().collect(),
        ))
    }

    /// Calculates actual command counts in the patch file
    pub fn calculate_actual_counts(&mut self) -> Result<ZiPatchCommandCounts> {
        let start_pos = self.reader.get_mut().stream_position()?;
        self.reader
            .get_mut()
            .seek(SeekFrom::Start(self.head_position))?;

        let mut total = 0u32;
        let mut adir = 0u32;
        let mut deld = 0u32;
        let mut sqpk_h = 0u32;
        let mut sqpk_f = 0u32;
        let mut sqpk_a = 0u32;
        let mut sqpk_d = 0u32;
        let mut sqpk_e = 0u32;

        loop {
            let chunk = ZiPatchChunk::read(&mut self.reader)?;

            if chunk.is_eof() {
                break;
            }

            total += 1;

            match chunk {
                ZiPatchChunk::AddDirectory(_) => adir += 1,
                ZiPatchChunk::DeleteDirectory(_) => deld += 1,
                ZiPatchChunk::Sqpk(ref sqpk) => match sqpk {
                    crate::chunk::SqpkCommand::Header(_) => sqpk_h += 1,
                    crate::chunk::SqpkCommand::File(_) => sqpk_f += 1,
                    crate::chunk::SqpkCommand::AddData(_) => sqpk_a += 1,
                    crate::chunk::SqpkCommand::DeleteData(_) => sqpk_d += 1,
                    crate::chunk::SqpkCommand::ExpandData(_) => sqpk_e += 1,
                    _ => {}
                },
                _ => {}
            }
        }

        self.reader.get_mut().seek(SeekFrom::Start(start_pos))?;

        Ok(ZiPatchCommandCounts::with_counts(
            adir, deld, total, sqpk_a, sqpk_d, sqpk_e, sqpk_h, sqpk_f,
        ))
    }
}

/// Iterator over chunks in a ZiPatch file
pub struct ChunkIterator<'a, R: Read + Seek> {
    reader: &'a mut ChecksumReader<R>,
    done: bool,
    restore_position: u64,
}

impl<'a, R: Read + Seek> ChunkIterator<'a, R> {
    fn new(reader: &'a mut ChecksumReader<R>, restore_position: u64) -> Self {
        Self {
            reader,
            done: false,
            restore_position,
        }
    }
}

impl<'a, R: Read + Seek> Iterator for ChunkIterator<'a, R> {
    type Item = Result<ZiPatchChunk>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        match ZiPatchChunk::read(self.reader) {
            Ok(chunk) => {
                let is_eof = chunk.is_eof();
                let result = Some(Ok(chunk));

                if is_eof {
                    self.done = true;
                }

                result
            }
            Err(e) => {
                self.done = true;
                Some(Err(e))
            }
        }
    }
}

impl<'a, R: Read + Seek> Drop for ChunkIterator<'a, R> {
    fn drop(&mut self) {
        // Restore original position
        let _ = self
            .reader
            .get_mut()
            .seek(SeekFrom::Start(self.restore_position));
    }
}
