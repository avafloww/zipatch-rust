use std::io::{self, Read, Seek};

use super::crc32::Crc32;

/// A reader wrapper that calculates CRC32 checksum of all bytes read
///
/// All read operations automatically update the internal CRC32 state.
#[derive(Debug)]
pub struct ChecksumReader<R: Read> {
    inner: R,
    crc32: Crc32,
}

impl<R: Read> ChecksumReader<R> {
    /// Creates a new ChecksumReader wrapping the given reader
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            crc32: Crc32::new(),
        }
    }

    /// Initializes/resets the CRC32 checksum
    pub fn init_crc32(&mut self) {
        self.crc32.init();
    }

    /// Gets the current CRC32 checksum value
    pub fn get_crc32(&self) -> u32 {
        self.crc32.finalize()
    }

    /// Consumes the ChecksumReader and returns the inner reader
    pub fn into_inner(self) -> R {
        self.inner
    }

    /// Gets a reference to the inner reader
    pub fn get_ref(&self) -> &R {
        &self.inner
    }

    /// Gets a mutable reference to the inner reader
    pub fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }
}

impl<R: Read> Read for ChecksumReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.inner.read(buf)?;
        if n > 0 {
            self.crc32.update(&buf[..n]);
        }
        Ok(n)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.inner.read_exact(buf)?;
        self.crc32.update(buf);
        Ok(())
    }
}

impl<R: Read + Seek> Seek for ChecksumReader<R> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.inner.seek(pos)
    }

    fn stream_position(&mut self) -> io::Result<u64> {
        self.inner.stream_position()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_checksum_reader() {
        let data = b"Hello, World!";
        let cursor = Cursor::new(data);
        let mut reader = ChecksumReader::new(cursor);

        let mut buf = [0u8; 13];
        reader.read_exact(&mut buf).unwrap();

        let checksum = reader.get_crc32();
        let expected = Crc32::calculate(data);

        assert_eq!(checksum, expected);
    }

    #[test]
    fn test_checksum_reader_reset() {
        let data = b"Test data here";
        let cursor = Cursor::new(data);
        let mut reader = ChecksumReader::new(cursor);

        // Read some data
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf).unwrap();

        // Reset checksum
        reader.init_crc32();

        // Read more data
        let mut buf2 = [0u8; 5];
        reader.read_exact(&mut buf2).unwrap();

        // Checksum should only include data after reset
        let checksum = reader.get_crc32();
        let expected = Crc32::calculate(&data[4..9]);

        assert_eq!(checksum, expected);
    }
}
