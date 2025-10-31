use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Error type for ZiPatch operations
#[derive(Error, Debug)]
pub enum ZiPatchError {
    /// I/O error occurred
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Invalid magic number in patch file header
    #[error("Invalid magic number: expected valid ZiPatch signature, got {0:08X?}")]
    InvalidMagic([u32; 3]),

    /// Checksum mismatch in chunk
    #[error("Checksum mismatch at offset {offset}: expected {expected:08X}, got {actual:08X}")]
    ChecksumMismatch {
        offset: u64,
        expected: u32,
        actual: u32,
    },

    /// Unknown chunk type encountered
    #[error("Unknown chunk type '{0}' at offset {1}")]
    UnknownChunkType(String, u64),

    /// Unknown SQPK command encountered
    #[error("Unknown SQPK command '{0}' at offset {1}")]
    UnknownSqpkCommand(char, u64),

    /// Invalid chunk data
    #[error("Invalid chunk data at offset {offset}: {reason}")]
    InvalidChunkData { offset: u64, reason: String },

    /// Unexpected end of file
    #[error("Unexpected end of file at offset {0}")]
    UnexpectedEof(u64),

    /// Invalid UTF-8 string
    #[error("Invalid UTF-8 string: {0}")]
    InvalidString(#[from] std::string::FromUtf8Error),

    /// File not found
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    /// Directory creation failed
    #[error("Failed to create directory {path}: {source}")]
    DirectoryCreationFailed { path: PathBuf, source: io::Error },

    /// File operation failed
    #[error("File operation failed on {path}: {source}")]
    FileOperationFailed { path: PathBuf, source: io::Error },

    /// Missing old file (when IgnoreMissing is false)
    #[error("Old file missing: {0}")]
    OldFileMissing(PathBuf),

    /// Old file mismatch (when IgnoreOldMismatch is false)
    #[error("Old file mismatch: {0}")]
    OldFileMismatch(PathBuf),

    /// Decompression failed
    #[error("Decompression failed: {0}")]
    DecompressionFailed(String),

    /// Invalid expansion ID
    #[error("Invalid expansion ID: {0}")]
    InvalidExpansionId(u16),

    /// Invalid platform
    #[error("Invalid platform value: {0}")]
    InvalidPlatform(u8),

    /// SQPK size mismatch
    #[error("SQPK inner size mismatch: outer={outer}, inner={inner}")]
    SqpkSizeMismatch { outer: u32, inner: i32 },

    /// File stream retry exhausted
    #[error("Failed to open file {path} after {tries} attempts")]
    FileStreamRetryExhausted { path: PathBuf, tries: u32 },

    /// Invalid file header version
    #[error("Invalid file header version: {0}")]
    InvalidFileHeaderVersion(u8),

    /// Generic error with custom message
    #[error("{0}")]
    Custom(String),
}

/// Result type alias for ZiPatch operations
pub type Result<T> = std::result::Result<T, ZiPatchError>;
