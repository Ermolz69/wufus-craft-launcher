use serde::Serialize;

/// Summary produced at the end of an update or repair operation.
#[derive(Clone, Serialize)]
pub struct ActionReport {
    /// Total files checked against the manifest.
    pub files_checked: u64,
    /// Files that were missing or corrupted and have been restored.
    pub files_restored: u64,
    /// Managed files no longer in the manifest that have been removed.
    pub files_deleted: u64,
    /// Files that were already correct and required no action.
    pub files_ok: u64,
}
