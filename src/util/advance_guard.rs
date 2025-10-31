use std::io::{self, Read, Seek, SeekFrom};

/// RAII guard that ensures a stream advances to a specific position on drop
///
/// When dropped, it automatically seeks the stream to the expected end position,
/// ensuring consistent stream positioning even if errors occur during chunk reading.
pub struct AdvanceGuard<'a, S: Read + Seek> {
    stream: &'a mut S,
    offset_before: u64,
    offset_after: u64,
}

impl<'a, S: Read + Seek> AdvanceGuard<'a, S> {
    /// Creates a new AdvanceGuard
    ///
    /// # Arguments
    /// * `stream` - The stream to guard
    /// * `size` - The number of bytes expected to be read
    pub fn new(stream: &'a mut S, size: u64) -> io::Result<Self> {
        let offset_before = stream.stream_position()?;
        let offset_after = offset_before + size;

        Ok(Self {
            stream,
            offset_before,
            offset_after,
        })
    }

    /// Gets the offset before reading started
    pub fn offset_before(&self) -> u64 {
        self.offset_before
    }

    /// Gets the expected offset after reading
    pub fn offset_after(&self) -> u64 {
        self.offset_after
    }

    /// Gets the number of bytes remaining to read
    pub fn num_bytes_remaining(&mut self) -> io::Result<u64> {
        let current = self.stream.stream_position()?;
        Ok(self.offset_after.saturating_sub(current))
    }

    /// Manually advances the stream to the expected position
    pub fn advance(&mut self) -> io::Result<()> {
        self.stream.seek(SeekFrom::Start(self.offset_after))?;
        Ok(())
    }
}

impl<'a, S: Read + Seek> Read for AdvanceGuard<'a, S> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buf)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.stream.read_exact(buf)
    }
}

impl<'a, S: Read + Seek> Seek for AdvanceGuard<'a, S> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.stream.seek(pos)
    }

    fn stream_position(&mut self) -> io::Result<u64> {
        self.stream.stream_position()
    }
}

impl<'a, S: Read + Seek> Drop for AdvanceGuard<'a, S> {
    fn drop(&mut self) {
        // Read any remaining bytes to ensure they're included in checksum calculation
        // (seeking would bypass the ChecksumReader)
        if let Ok(current) = self.stream.stream_position() {
            if current < self.offset_after {
                let remaining = (self.offset_after - current) as usize;
                // Use a reasonable buffer size to avoid huge allocations
                let mut buf = vec![0u8; remaining.min(8192)];
                let mut left = remaining;

                while left > 0 {
                    let to_read = left.min(buf.len());
                    match self.stream.read_exact(&mut buf[..to_read]) {
                        Ok(_) => left -= to_read,
                        Err(_) => {
                            // If read fails, try seeking as fallback
                            let _ = self.stream.seek(SeekFrom::Start(self.offset_after));
                            break;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_advance_guard_normal_read() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let mut cursor = Cursor::new(data);

        {
            let mut guard = AdvanceGuard::new(&mut cursor, 4).unwrap();
            let mut buf = [0u8; 4];
            guard.read_exact(&mut buf).unwrap();
        }

        // After drop, the cursor should be at position 4
        assert_eq!(cursor.stream_position().unwrap(), 4);
    }

    #[test]
    fn test_advance_guard_partial_read() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let mut cursor = Cursor::new(data);

        {
            let mut guard = AdvanceGuard::new(&mut cursor, 8).unwrap();
            let mut buf = [0u8; 2];
            // Only read 2 bytes instead of 8
            guard.read_exact(&mut buf).unwrap();
            // Guard will advance to position 8 on drop
        }

        assert_eq!(cursor.stream_position().unwrap(), 8);
    }

    #[test]
    fn test_num_bytes_remaining() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let mut cursor = Cursor::new(data);

        {
            let mut guard = AdvanceGuard::new(&mut cursor, 6).unwrap();
            assert_eq!(guard.num_bytes_remaining().unwrap(), 6);
        }

        // Position should be advanced to 6
        assert_eq!(cursor.stream_position().unwrap(), 6);
    }
}
