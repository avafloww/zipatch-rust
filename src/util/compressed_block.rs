use std::io::{Read, Write};

use flate2::read::DeflateDecoder;

use crate::error::{Result, ZiPatchError};
use crate::util::binary_reader::BinaryReaderExt;

/// Represents a compressed data block from SQPK files
///
/// Handles both compressed (deflate) and uncompressed blocks.
#[derive(Debug, Clone)]
pub struct SqpkCompressedBlock {
    /// Size of the block header
    pub header_size: i32,
    /// Size of compressed data (0x7d00 if uncompressed)
    pub compressed_size: i32,
    /// Size of decompressed data
    pub decompressed_size: i32,
    /// The compressed or uncompressed block data
    pub compressed_block: Vec<u8>,
}

impl SqpkCompressedBlock {
    /// Reads a compressed block from a binary reader
    pub fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let header_size = reader.read_i32_le()?;

        // Read and skip padding (4 bytes)
        let _pad = reader.read_u32_le()?;

        let compressed_size = reader.read_i32_le()?;
        let decompressed_size = reader.read_i32_le()?;

        let is_compressed = compressed_size != 0x7d00;
        let compressed_block_length =
            Self::calculate_compressed_block_length(compressed_size, decompressed_size);

        let compressed_block = if is_compressed {
            // Read compressed data
            reader.read_bytes_required((compressed_block_length - header_size) as usize)?
        } else {
            // Read uncompressed data
            let block = reader.read_bytes_required(decompressed_size as usize)?;

            // Read and discard padding
            let padding_size = compressed_block_length - header_size - decompressed_size;
            if padding_size > 0 {
                let _ = reader.read_bytes_required(padding_size as usize)?;
            }

            block
        };

        Ok(Self {
            header_size,
            compressed_size,
            decompressed_size,
            compressed_block,
        })
    }

    /// Checks if this block is compressed
    pub fn is_compressed(&self) -> bool {
        self.compressed_size != 0x7d00
    }

    /// Calculates the total compressed block length including padding
    pub fn compressed_block_length(&self) -> i32 {
        Self::calculate_compressed_block_length(self.compressed_size, self.decompressed_size)
    }

    /// Calculates the compressed block length from sizes
    fn calculate_compressed_block_length(compressed_size: i32, decompressed_size: i32) -> i32 {
        let is_compressed = compressed_size != 0x7d00;
        let size = if is_compressed {
            compressed_size
        } else {
            decompressed_size
        };

        (size + 143) & 0xFFFF_FF80u32 as i32
    }

    /// Decompresses the block into the output stream
    ///
    /// # Arguments
    /// * `out_stream` - The stream to write decompressed data to
    pub fn decompress_into<W: Write>(&self, out_stream: &mut W) -> Result<()> {
        if self.is_compressed() {
            // Decompress using deflate
            let mut decoder = DeflateDecoder::new(&self.compressed_block[..]);
            std::io::copy(&mut decoder, out_stream).map_err(|e| {
                ZiPatchError::DecompressionFailed(format!("Failed to decompress block: {}", e))
            })?;
        } else {
            // Write uncompressed data directly
            out_stream.write_all(&self.compressed_block)?;
        }

        Ok(())
    }

    /// Decompresses the block and returns the decompressed data
    pub fn decompress(&self) -> Result<Vec<u8>> {
        let mut output = Vec::with_capacity(self.decompressed_size as usize);
        self.decompress_into(&mut output)?;
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_compressed() {
        let block = SqpkCompressedBlock {
            header_size: 16,
            compressed_size: 100,
            decompressed_size: 200,
            compressed_block: vec![],
        };
        assert!(block.is_compressed());

        let block2 = SqpkCompressedBlock {
            header_size: 16,
            compressed_size: 0x7d00,
            decompressed_size: 200,
            compressed_block: vec![],
        };
        assert!(!block2.is_compressed());
    }

    #[test]
    fn test_compressed_block_length() {
        let block = SqpkCompressedBlock {
            header_size: 16,
            compressed_size: 100,
            decompressed_size: 200,
            compressed_block: vec![],
        };
        // (100 + 143) & 0xFFFF_FF80 = 243 & 0xFFFF_FF80 = 128
        assert_eq!(block.compressed_block_length(), 128);
    }

    #[test]
    fn test_uncompressed_block() {
        let data = b"Hello, World!";
        let block = SqpkCompressedBlock {
            header_size: 16,
            compressed_size: 0x7d00,
            decompressed_size: data.len() as i32,
            compressed_block: data.to_vec(),
        };

        let decompressed = block.decompress().unwrap();
        assert_eq!(decompressed, data);
    }
}
