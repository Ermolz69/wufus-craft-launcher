use crate::core::file_policy::file_category::FileCategory;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledFile {
    pub sha256: String,
    pub size: u64,
    pub category: FileCategory,
    pub managed: bool,
    pub source_manifest_version: String,
    pub installed_at: u64,
}
