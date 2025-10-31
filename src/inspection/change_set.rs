/// Represents the set of file changes from applying a patch
///
/// Contains lists of files that were added, deleted, or modified.
#[derive(Debug, Clone, Default)]
pub struct ZiPatchChangeSet {
    /// Files that were added by the patch
    pub added: Vec<String>,
    /// Files that were deleted by the patch
    pub deleted: Vec<String>,
    /// Files that were modified by the patch
    pub modified: Vec<String>,
}

impl ZiPatchChangeSet {
    /// Creates a new empty change set
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new change set with the given lists
    pub fn with_changes(added: Vec<String>, deleted: Vec<String>, modified: Vec<String>) -> Self {
        Self {
            added,
            deleted,
            modified,
        }
    }

    /// Gets the total number of changes
    pub fn total_changes(&self) -> usize {
        self.added.len() + self.deleted.len() + self.modified.len()
    }

    /// Checks if the change set is empty
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.deleted.is_empty() && self.modified.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_changeset() {
        let cs = ZiPatchChangeSet::new();
        assert!(cs.is_empty());
        assert_eq!(cs.total_changes(), 0);
    }

    #[test]
    fn test_with_changes() {
        let cs = ZiPatchChangeSet::with_changes(
            vec!["file1.dat".to_string()],
            vec!["file2.dat".to_string()],
            vec!["file3.dat".to_string()],
        );

        assert_eq!(cs.total_changes(), 3);
        assert!(!cs.is_empty());
    }
}
