use std::fs;
use std::path::{Path, PathBuf};

use super::{SqexFileStream, SqexFileStreamStore};
use crate::error::{Result, ZiPatchError};

/// Represents a Square Enix game file with a relative path
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SqexFile {
    /// Relative path to the file within the game directory
    pub relative_path: String,
}

impl SqexFile {
    /// Creates a new SqexFile with the given relative path
    pub fn new<S: Into<String>>(relative_path: S) -> Self {
        Self {
            relative_path: relative_path.into(),
        }
    }

    /// Opens a stream to the file with retry logic
    ///
    /// # Arguments
    /// * `base_path` - The base game directory path
    /// * `write` - Whether to open for writing (true) or reading (false)
    /// * `tries` - Number of retry attempts (default: 5)
    /// * `sleeptime` - Sleep time in seconds between retries (default: 1)
    pub fn open_stream<P: AsRef<Path>>(
        &self,
        base_path: P,
        write: bool,
        tries: u32,
        sleeptime: u64,
    ) -> Result<SqexFileStream> {
        let full_path = self.resolve_full_path(base_path);
        SqexFileStream::wait_for_stream(&full_path, write, tries, sleeptime)
    }

    /// Opens a stream using a file stream store (cache)
    ///
    /// # Arguments
    /// * `store` - The file stream store to use
    /// * `base_path` - The base game directory path
    /// * `write` - Whether to open for writing (true) or reading (false)
    /// * `tries` - Number of retry attempts (default: 5)
    /// * `sleeptime` - Sleep time in seconds between retries (default: 1)
    pub fn open_stream_with_store<'a, P: AsRef<Path>>(
        &self,
        store: &'a mut SqexFileStreamStore,
        base_path: P,
        write: bool,
        tries: u32,
        sleeptime: u64,
    ) -> Result<&'a mut SqexFileStream> {
        let full_path = self.resolve_full_path(base_path);
        store.get_stream(&full_path, write, tries, sleeptime)
    }

    /// Creates the directory tree for this file
    ///
    /// # Arguments
    /// * `base_path` - The base game directory path
    pub fn create_directory_tree<P: AsRef<Path>>(&self, base_path: P) -> Result<()> {
        let full_path = self.resolve_full_path(base_path);

        if let Some(parent) = Path::new(&full_path).parent() {
            fs::create_dir_all(parent).map_err(|e| ZiPatchError::DirectoryCreationFailed {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }

        Ok(())
    }

    /// Resolves the full path by combining base path and relative path
    fn resolve_full_path<P: AsRef<Path>>(&self, base_path: P) -> PathBuf {
        let base = base_path.as_ref();
        base.join(&self.relative_path)
    }

    /// Gets the expansion folder name for a given expansion ID
    ///
    /// # Arguments
    /// * `expansion_id` - The expansion ID (0 for base game, 1+ for expansions)
    ///
    /// # Returns
    /// * "ffxiv" for base game (expansion_id == 0)
    /// * "ex{N}" for expansions (e.g., "ex1", "ex2", etc.)
    pub fn get_expansion_folder(expansion_id: u8) -> String {
        if expansion_id == 0 {
            "ffxiv".to_string()
        } else {
            format!("ex{}", expansion_id)
        }
    }

    /// Gets all files for a given expansion
    ///
    /// # Arguments
    /// * `full_path` - The full game installation path
    /// * `expansion_id` - The expansion ID
    ///
    /// # Returns
    /// A vector of all file paths in the sqpack and movie directories for this expansion
    pub fn get_all_expansion_files<P: AsRef<Path>>(
        full_path: P,
        expansion_id: u16,
    ) -> Result<Vec<PathBuf>> {
        let full_path = full_path.as_ref();
        let xpac_folder = Self::get_expansion_folder(expansion_id as u8);

        let sqpack_path = full_path.join("sqpack").join(&xpac_folder);
        let movie_path = full_path.join("movie").join(&xpac_folder);

        let mut files = Vec::new();

        // Add files from sqpack directory
        if sqpack_path.exists() && sqpack_path.is_dir() {
            for entry in fs::read_dir(&sqpack_path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    files.push(path);
                }
            }
        }

        // Add files from movie directory
        if movie_path.exists() && movie_path.is_dir() {
            for entry in fs::read_dir(&movie_path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    files.push(path);
                }
            }
        }

        Ok(files)
    }
}

impl std::fmt::Display for SqexFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.relative_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_expansion_folder() {
        assert_eq!(SqexFile::get_expansion_folder(0), "ffxiv");
        assert_eq!(SqexFile::get_expansion_folder(1), "ex1");
        assert_eq!(SqexFile::get_expansion_folder(2), "ex2");
        assert_eq!(SqexFile::get_expansion_folder(5), "ex5");
    }

    #[test]
    fn test_resolve_full_path() {
        let file = SqexFile::new("sqpack/ffxiv/000000.win32.dat0");
        let full_path = file.resolve_full_path("/game");

        assert_eq!(
            full_path,
            PathBuf::from("/game/sqpack/ffxiv/000000.win32.dat0")
        );
    }

    #[test]
    fn test_display() {
        let file = SqexFile::new("test/path.dat");
        assert_eq!(format!("{}", file), "test/path.dat");
    }
}
