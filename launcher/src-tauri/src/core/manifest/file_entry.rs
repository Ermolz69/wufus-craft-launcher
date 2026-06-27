use crate::core::file_policy::file_category::FileCategory;
use crate::core::manifest::rules::{DeletePolicy, UpdatePolicy};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String, // Relative to Game Area
    pub url: String,
    pub sha256: String,
    pub size: u64,
    pub category: FileCategory,
    pub managed: bool,
    pub update_policy: UpdatePolicy,
    pub delete_policy: DeletePolicy,
}
