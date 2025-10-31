use crc32fast::Hasher;

/// CRC32 calculator for chunk checksums
#[derive(Debug, Clone)]
pub struct Crc32 {
    hasher: Hasher,
}

impl Crc32 {
    /// Creates a new CRC32 calculator with initial state
    pub fn new() -> Self {
        Self {
            hasher: Hasher::new(),
        }
    }

    /// Initializes/resets the CRC32 state
    pub fn init(&mut self) {
        self.hasher = Hasher::new();
    }

    /// Updates the CRC32 with the given bytes
    pub fn update(&mut self, bytes: &[u8]) {
        self.hasher.update(bytes);
    }

    /// Finalizes and returns the CRC32 checksum
    pub fn finalize(&self) -> u32 {
        self.hasher.clone().finalize()
    }

    /// Calculates CRC32 for the given bytes in one step
    pub fn calculate(bytes: &[u8]) -> u32 {
        crc32fast::hash(bytes)
    }
}

impl Default for Crc32 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc32_empty() {
        let crc = Crc32::calculate(&[]);
        assert_eq!(crc, 0);
    }

    #[test]
    fn test_crc32_incremental() {
        let data = b"Hello, World!";

        let mut crc = Crc32::new();
        crc.update(data);
        let result1 = crc.finalize();

        let result2 = Crc32::calculate(data);

        assert_eq!(result1, result2);
    }

    #[test]
    fn test_crc32_reset() {
        let mut crc = Crc32::new();
        crc.update(b"test");
        crc.init();
        crc.update(b"data");

        let expected = Crc32::calculate(b"data");
        assert_eq!(crc.finalize(), expected);
    }
}
