/// Statistics about commands in a patch file
///
/// Contains counts of different command types found in the patch.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ZiPatchCommandCounts {
    /// Number of AddDirectory commands
    pub add_directories: u32,
    /// Number of DeleteDirectory commands
    pub delete_directories: u32,
    /// Total number of commands
    pub total_commands: u32,
    /// Number of SQPK Add commands
    pub sqpk_add_commands: u32,
    /// Number of SQPK Delete commands
    pub sqpk_delete_commands: u32,
    /// Number of SQPK Expand commands
    pub sqpk_expand_commands: u32,
    /// Number of SQPK Header commands
    pub sqpk_header_commands: u32,
    /// Number of SQPK File commands
    pub sqpk_file_commands: u32,
}

impl ZiPatchCommandCounts {
    /// Creates a new empty command counts structure
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new command counts structure with the given values
    #[allow(clippy::too_many_arguments)]
    pub fn with_counts(
        add_directories: u32,
        delete_directories: u32,
        total_commands: u32,
        sqpk_add_commands: u32,
        sqpk_delete_commands: u32,
        sqpk_expand_commands: u32,
        sqpk_header_commands: u32,
        sqpk_file_commands: u32,
    ) -> Self {
        Self {
            add_directories,
            delete_directories,
            total_commands,
            sqpk_add_commands,
            sqpk_delete_commands,
            sqpk_expand_commands,
            sqpk_header_commands,
            sqpk_file_commands,
        }
    }
}
