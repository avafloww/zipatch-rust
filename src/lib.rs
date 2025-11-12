//! # ZiPatch Library
//!
//! A 100% safe Rust library for reading and applying Final Fantasy XIV patch files (ZiPatch format).
//!
//! This library provides functionality to:
//! - Read ZiPatch (.patch) files
//! - Parse chunk-based patch file format
//! - Apply patches to game installations
//! - Inspect patch contents and changes
//!
//! ## Example
//!
//! ```no_run
//! use zipatch::{ZiPatchFile, ZiPatchConfig, Platform};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Open a patch file
//! let mut patch = ZiPatchFile::from_path("game.patch")?;
//!
//! // Get patch header information
//! let header = patch.header();
//! println!("Patch version: {}", header.version);
//!
//! // Create configuration
//! let mut config = ZiPatchConfig::builder("/path/to/game")
//!     .platform(Platform::Win32)
//!     .ignore_missing(true)
//!     .build();
//!
//! // Calculate changed files
//! let changes = patch.calculate_changed_files(&config)?;
//! println!("Added files: {:?}", changes.added);
//! println!("Modified files: {:?}", changes.modified);
//! println!("Deleted files: {:?}", changes.deleted);
//!
//! // Iterate through chunks and apply them
//! for chunk_result in patch.chunks() {
//!     let mut chunk = chunk_result?;
//!     chunk.apply(&mut config)?;
//! }
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]
#![warn(clippy::all, clippy::cargo, clippy::incompatible_msrv)]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::module_name_repetitions,
    clippy::doc_markdown,
    missing_docs
)]
pub mod chunk;
pub mod config;
pub mod error;
pub mod file;
pub mod inspection;
pub mod util;

// Re-export commonly used types
pub use chunk::{SqpkCommand, ZiPatchChunk};
pub use config::{Platform, ZiPatchConfig, ZiPatchConfigBuilder};
pub use error::{Result, ZiPatchError};
pub use file::ZiPatchFile;
pub use inspection::{ZiPatchChangeSet, ZiPatchCommandCounts};
