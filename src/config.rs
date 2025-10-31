use std::path::PathBuf;

use crate::error::{Result, ZiPatchError};
use crate::util::SqexFileStreamStore;

/// Platform identifier for FFXIV installation
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Platform {
    /// Windows (PC)
    Win32 = 0,
    /// PlayStation 3
    Ps3 = 1,
    /// PlayStation 4
    Ps4 = 2,
    /// Unknown platform
    #[default]
    Unknown = 3,
}

impl Platform {
    /// Creates a Platform from a u8 value
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Platform::Win32),
            1 => Ok(Platform::Ps3),
            2 => Ok(Platform::Ps4),
            3 => Ok(Platform::Unknown),
            _ => Err(ZiPatchError::InvalidPlatform(value)),
        }
    }

    /// Creates a Platform from a u16 value
    pub fn from_u16(value: u16) -> Result<Self> {
        match value {
            0 => Ok(Platform::Win32),
            1 => Ok(Platform::Ps3),
            2 => Ok(Platform::Ps4),
            3 => Ok(Platform::Unknown),
            _ => Err(ZiPatchError::InvalidPlatform(value as u8)),
        }
    }

    /// Converts the platform to a u16 value
    pub fn as_u16(self) -> u16 {
        self as u16
    }
}

/// Configuration for applying ZiPatch files
#[derive(Debug)]
pub struct ZiPatchConfig {
    /// Path to the game installation directory
    game_path: PathBuf,

    /// Target platform
    pub platform: Platform,

    /// If true, ignore missing old files when applying patches
    pub ignore_missing: bool,

    /// If true, ignore mismatches in old file content
    pub ignore_old_mismatch: bool,

    /// Optional file stream cache for performance
    pub store: Option<SqexFileStreamStore>,
}

impl ZiPatchConfig {
    /// Creates a new ZiPatchConfig with the given game path
    pub fn new<P: Into<PathBuf>>(game_path: P) -> Self {
        Self {
            game_path: game_path.into(),
            platform: Platform::default(),
            ignore_missing: false,
            ignore_old_mismatch: false,
            store: None,
        }
    }

    /// Gets the game path
    pub fn game_path(&self) -> &PathBuf {
        &self.game_path
    }

    /// Creates a builder for ZiPatchConfig
    pub fn builder<P: Into<PathBuf>>(game_path: P) -> ZiPatchConfigBuilder {
        ZiPatchConfigBuilder::new(game_path)
    }
}

/// Builder for ZiPatchConfig
#[derive(Debug)]
pub struct ZiPatchConfigBuilder {
    game_path: PathBuf,
    platform: Platform,
    ignore_missing: bool,
    ignore_old_mismatch: bool,
    store: Option<SqexFileStreamStore>,
}

impl ZiPatchConfigBuilder {
    /// Creates a new builder with the given game path
    pub fn new<P: Into<PathBuf>>(game_path: P) -> Self {
        Self {
            game_path: game_path.into(),
            platform: Platform::default(),
            ignore_missing: false,
            ignore_old_mismatch: false,
            store: None,
        }
    }

    /// Sets the platform
    pub fn platform(mut self, platform: Platform) -> Self {
        self.platform = platform;
        self
    }

    /// Sets whether to ignore missing files
    pub fn ignore_missing(mut self, ignore: bool) -> Self {
        self.ignore_missing = ignore;
        self
    }

    /// Sets whether to ignore old file mismatches
    pub fn ignore_old_mismatch(mut self, ignore: bool) -> Self {
        self.ignore_old_mismatch = ignore;
        self
    }

    /// Sets the file stream store
    pub fn store(mut self, store: SqexFileStreamStore) -> Self {
        self.store = Some(store);
        self
    }

    /// Builds the ZiPatchConfig
    pub fn build(self) -> ZiPatchConfig {
        ZiPatchConfig {
            game_path: self.game_path,
            platform: self.platform,
            ignore_missing: self.ignore_missing,
            ignore_old_mismatch: self.ignore_old_mismatch,
            store: self.store,
        }
    }
}
