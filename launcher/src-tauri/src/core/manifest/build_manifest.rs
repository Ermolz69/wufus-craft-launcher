use serde::{Deserialize, Serialize};
use super::version_info::VersionInfo;
use super::file_entry::FileEntry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildManifest {
    pub build: VersionInfo,
    pub files: Vec<FileEntry>,
    pub protected_paths: Vec<String>,
}
