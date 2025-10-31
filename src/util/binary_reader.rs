use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use std::io::Read;

use crate::error::Result;

/// Extension trait for reading binary data
pub trait BinaryReaderExt: Read {
    /// Reads a u16 in big-endian byte order
    fn read_u16_be(&mut self) -> Result<u16> {
        Ok(self.read_u16::<BigEndian>()?)
    }

    /// Reads an i16 in big-endian byte order
    fn read_i16_be(&mut self) -> Result<i16> {
        Ok(self.read_i16::<BigEndian>()?)
    }

    /// Reads a u32 in big-endian byte order
    fn read_u32_be(&mut self) -> Result<u32> {
        Ok(self.read_u32::<BigEndian>()?)
    }

    /// Reads an i32 in big-endian byte order
    fn read_i32_be(&mut self) -> Result<i32> {
        Ok(self.read_i32::<BigEndian>()?)
    }

    /// Reads a u64 in big-endian byte order
    fn read_u64_be(&mut self) -> Result<u64> {
        Ok(self.read_u64::<BigEndian>()?)
    }

    /// Reads an i64 in big-endian byte order
    fn read_i64_be(&mut self) -> Result<i64> {
        Ok(self.read_i64::<BigEndian>()?)
    }

    /// Reads a u16 in little-endian byte order
    fn read_u16_le(&mut self) -> Result<u16> {
        Ok(self.read_u16::<LittleEndian>()?)
    }

    /// Reads an i16 in little-endian byte order
    fn read_i16_le(&mut self) -> Result<i16> {
        Ok(self.read_i16::<LittleEndian>()?)
    }

    /// Reads a u32 in little-endian byte order
    fn read_u32_le(&mut self) -> Result<u32> {
        Ok(self.read_u32::<LittleEndian>()?)
    }

    /// Reads an i32 in little-endian byte order
    fn read_i32_le(&mut self) -> Result<i32> {
        Ok(self.read_i32::<LittleEndian>()?)
    }

    /// Reads a u64 in little-endian byte order
    fn read_u64_le(&mut self) -> Result<u64> {
        Ok(self.read_u64::<LittleEndian>()?)
    }

    /// Reads an i64 in little-endian byte order
    fn read_i64_le(&mut self) -> Result<i64> {
        Ok(self.read_i64::<LittleEndian>()?)
    }

    /// Reads a fixed-length ASCII string, trimming null bytes
    fn read_fixed_string(&mut self, length: usize) -> Result<String> {
        let mut buffer = vec![0u8; length];
        self.read_exact(&mut buffer)?;

        // Convert to string, trimming null bytes
        let trimmed = buffer
            .iter()
            .position(|&b| b == 0)
            .map(|pos| &buffer[..pos])
            .unwrap_or(&buffer);

        Ok(String::from_utf8_lossy(trimmed).into_owned())
    }

    /// Reads exactly the specified number of bytes or returns an error
    fn read_bytes_required(&mut self, length: usize) -> Result<Vec<u8>> {
        let mut buffer = vec![0u8; length];
        self.read_exact(&mut buffer)?;
        Ok(buffer)
    }

    /// Reads a 4-character type identifier (e.g., "FHDR", "SQPK")
    fn read_chunk_type(&mut self) -> Result<String> {
        self.read_fixed_string(4)
    }

    /// Dumps bytes as hex for debugging (reads without consuming from source)
    fn dump(&mut self, length: usize) -> Result<String> {
        let bytes = self.read_bytes_required(length)?;
        let hex: Vec<String> = bytes.iter().map(|b| format!("{:02X}", b)).collect();
        Ok(hex.join(" "))
    }
}

// Implement for all types that implement Read
impl<R: Read> BinaryReaderExt for R {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_u32_be() {
        let data = vec![0x00, 0x00, 0x00, 0x2A];
        let mut cursor = Cursor::new(data);
        assert_eq!(cursor.read_u32_be().unwrap(), 42);
    }

    #[test]
    fn test_read_fixed_string() {
        let data = b"FHDR\0\0\0\0";
        let mut cursor = Cursor::new(data);
        let s = cursor.read_fixed_string(8).unwrap();
        assert_eq!(s, "FHDR");
    }

    #[test]
    fn test_read_fixed_string_no_null() {
        let data = b"SQPK";
        let mut cursor = Cursor::new(data);
        let s = cursor.read_fixed_string(4).unwrap();
        assert_eq!(s, "SQPK");
    }

    #[test]
    fn test_read_chunk_type() {
        let data = b"EOF_";
        let mut cursor = Cursor::new(data);
        let chunk_type = cursor.read_chunk_type().unwrap();
        assert_eq!(chunk_type, "EOF_");
    }

    #[test]
    fn test_read_bytes_required() {
        let data = vec![1, 2, 3, 4, 5];
        let mut cursor = Cursor::new(data);
        let bytes = cursor.read_bytes_required(3).unwrap();
        assert_eq!(bytes, vec![1, 2, 3]);
    }
}
