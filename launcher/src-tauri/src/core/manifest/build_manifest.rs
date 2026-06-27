use super::file_entry::FileEntry;
use super::version_info::VersionInfo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildManifest {
    pub build: VersionInfo,
    pub files: Vec<FileEntry>,
    pub protected_paths: Vec<String>,
}
