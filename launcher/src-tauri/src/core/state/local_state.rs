use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::installed_file::InstalledFile;
use super::update_status::UpdateStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalState {
    pub schema_version: u32,
    pub current_version: Option<String>,
    pub manifest_hash: Option<String>,
    pub last_update_time: u64,
    pub installed_files: HashMap<String, InstalledFile>,
    pub status: UpdateStatus,
}

impl Default for LocalState {
    fn default() -> Self {
        Self {
            schema_version: 1,
            current_version: None,
            manifest_hash: None,
            last_update_time: 0,
            installed_files: HashMap::new(),
            status: UpdateStatus::default(),
        }
    }
}

impl LocalState {
    pub fn is_interrupted(&self) -> bool {
        self.status == UpdateStatus::InProgress
    }
}
