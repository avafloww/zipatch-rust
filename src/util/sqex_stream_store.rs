use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::SqexFileStream;
use crate::error::Result;

/// A cache/store for Square Enix file streams
///
/// Keeps file streams open and returns them from cache on subsequent requests,
/// avoiding repeated file open operations.
#[derive(Debug)]
pub struct SqexFileStreamStore {
    streams: HashMap<PathBuf, SqexFileStream>,
}

impl SqexFileStreamStore {
    /// Creates a new empty stream store
    pub fn new() -> Self {
        Self {
            streams: HashMap::new(),
        }
    }

    /// Gets a stream for the given path, opening it if not already cached
    ///
    /// # Arguments
    /// * `path` - Path to the file
    /// * `write` - If true, open for writing; if false, open for reading
    /// * `tries` - Number of retry attempts (default: 5)
    /// * `sleeptime` - Sleep time in seconds between retries (default: 1)
    pub fn get_stream<P: AsRef<Path>>(
        &mut self,
        path: P,
        write: bool,
        tries: u32,
        sleeptime: u64,
    ) -> Result<&mut SqexFileStream> {
        // Normalize the path
        let path = path.as_ref();
        let normalized_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

        // Check if we already have this stream
        if !self.streams.contains_key(&normalized_path) {
            // Open new stream with retry logic
            let stream =
                SqexFileStream::wait_for_stream(&normalized_path, write, tries, sleeptime)?;
            self.streams.insert(normalized_path.clone(), stream);
        }

        // Return mutable reference to the stream
        Ok(self.streams.get_mut(&normalized_path).unwrap())
    }

    /// Checks if a stream for the given path is already cached
    pub fn contains<P: AsRef<Path>>(&self, path: P) -> bool {
        let path = path.as_ref();
        let normalized_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        self.streams.contains_key(&normalized_path)
    }

    /// Gets the number of cached streams
    pub fn len(&self) -> usize {
        self.streams.len()
    }

    /// Checks if the store is empty
    pub fn is_empty(&self) -> bool {
        self.streams.is_empty()
    }

    /// Closes and removes a specific stream from the cache
    pub fn remove<P: AsRef<Path>>(&mut self, path: P) -> Option<SqexFileStream> {
        let path = path.as_ref();
        let normalized_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        self.streams.remove(&normalized_path)
    }

    /// Closes all cached streams
    pub fn clear(&mut self) {
        self.streams.clear();
    }
}

impl Default for SqexFileStreamStore {
    fn default() -> Self {
        Self::new()
    }
}

// Drop trait automatically closes all streams when the store is dropped
impl Drop for SqexFileStreamStore {
    fn drop(&mut self) {
        self.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_store_new() {
        let store = SqexFileStreamStore::new();
        assert_eq!(store.len(), 0);
        assert!(store.is_empty());
    }

    #[test]
    fn test_stream_store_default() {
        let store = SqexFileStreamStore::default();
        assert_eq!(store.len(), 0);
    }
}
