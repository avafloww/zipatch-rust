use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::thread;
use std::time::Duration;

use crate::error::{Result, ZiPatchError};

/// Buffer size for file operations (64KB)
const BUFFER_SIZE: usize = 1 << 16;

/// Specialized file stream for Square Enix game files
///
/// Provides retry logic for opening files and utilities for wiping file regions.
#[derive(Debug)]
pub struct SqexFileStream {
    file: File,
}

impl SqexFileStream {
    /// Creates a new SqexFileStream for the given path
    ///
    /// # Arguments
    /// * `path` - Path to the file
    /// * `write` - If true, open for writing; if false, open for reading
    pub fn new<P: AsRef<Path>>(path: P, write: bool) -> Result<Self> {
        let file = if write {
            OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(path)?
        } else {
            OpenOptions::new().read(true).open(path)?
        };

        Ok(Self { file })
    }

    /// Waits for a file stream to be available with retry logic
    ///
    /// # Arguments
    /// * `path` - Path to the file
    /// * `write` - If true, open for writing; if false, open for reading
    /// * `tries` - Number of attempts (default: 5)
    /// * `sleeptime` - Sleep time in seconds between retries (default: 1)
    pub fn wait_for_stream<P: AsRef<Path>>(
        path: P,
        write: bool,
        tries: u32,
        sleeptime: u64,
    ) -> Result<Self> {
        let path = path.as_ref();
        let mut remaining_tries = tries;

        loop {
            match Self::new(path, write) {
                Ok(stream) => return Ok(stream),
                Err(ZiPatchError::Io(e)) if remaining_tries > 1 => {
                    // Only retry on I/O errors
                    if e.kind() == io::ErrorKind::PermissionDenied
                        || e.kind() == io::ErrorKind::WouldBlock
                    {
                        remaining_tries -= 1;
                        thread::sleep(Duration::from_secs(sleeptime));
                        continue;
                    }
                    return Err(ZiPatchError::Io(e));
                }
                Err(e) => {
                    if remaining_tries <= 1 {
                        return Err(ZiPatchError::FileStreamRetryExhausted {
                            path: path.to_path_buf(),
                            tries,
                        });
                    }
                    return Err(e);
                }
            }
        }
    }

    /// Writes data at the specified offset
    ///
    /// # Arguments
    /// * `data` - Data to write
    /// * `offset` - Offset in the file
    pub fn write_from_offset(&mut self, data: &[u8], offset: i64) -> Result<()> {
        self.file.seek(SeekFrom::Start(offset as u64))?;
        self.file.write_all(data)?;
        Ok(())
    }

    /// Wipes (zeros) a region of the file
    ///
    /// # Arguments
    /// * `length` - Number of bytes to wipe
    pub fn wipe(&mut self, length: u64) -> Result<()> {
        let wipe_buffer = vec![0u8; BUFFER_SIZE];
        let num_full_chunks = length / BUFFER_SIZE as u64;

        for _ in 0..num_full_chunks {
            self.file.write_all(&wipe_buffer)?;
        }

        let remaining = (length % BUFFER_SIZE as u64) as usize;
        if remaining > 0 {
            self.file.write_all(&wipe_buffer[..remaining])?;
        }

        Ok(())
    }

    /// Wipes (zeros) a region of the file starting at the specified offset
    ///
    /// # Arguments
    /// * `length` - Number of bytes to wipe
    /// * `offset` - Offset in the file
    pub fn wipe_from_offset(&mut self, length: u64, offset: i64) -> Result<()> {
        self.file.seek(SeekFrom::Start(offset as u64))?;
        self.wipe(length)
    }

    /// Seeks to a specific position in the file
    pub fn seek_to(&mut self, offset: u64) -> Result<u64> {
        Ok(self.file.seek(SeekFrom::Start(offset))?)
    }

    /// Gets the current position in the file
    pub fn position(&mut self) -> Result<u64> {
        Ok(self.file.stream_position()?)
    }

    /// Gets a reference to the underlying file
    pub fn get_ref(&self) -> &File {
        &self.file
    }

    /// Gets a mutable reference to the underlying file
    pub fn get_mut(&mut self) -> &mut File {
        &mut self.file
    }
}

impl Read for SqexFileStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.file.read(buf)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.file.read_exact(buf)
    }
}

impl Write for SqexFileStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file.write(buf)
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.file.write_all(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

impl Seek for SqexFileStream {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.file.seek(pos)
    }
}
