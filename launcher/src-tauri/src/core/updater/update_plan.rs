use crate::core::file_policy::decision::Decision;
use crate::core::manifest::file_entry::FileEntry;
use std::path::PathBuf;

pub struct UpdateAction {
    pub file_entry: Option<FileEntry>,
    pub local_path: PathBuf,
    pub decision: Decision,
}

#[derive(Default)]
pub struct UpdatePlan {
    pub to_download: Vec<UpdateAction>,
    pub to_delete: Vec<UpdateAction>,
    pub to_backup: Vec<UpdateAction>,
    pub skipped: Vec<UpdateAction>,
}

impl UpdatePlan {
    pub const fn new() -> Self {
        Self {
            to_download: Vec::new(),
            to_delete: Vec::new(),
            to_backup: Vec::new(),
            skipped: Vec::new(),
        }
    }

    pub fn download_size_bytes(&self) -> u64 {
        self.to_download
            .iter()
            .filter_map(|a| a.file_entry.as_ref())
            .map(|e| e.size)
            .sum()
    }

    pub const fn is_up_to_date(&self) -> bool {
        self.to_download.is_empty() && self.to_delete.is_empty() && self.to_backup.is_empty()
    }

    pub fn add(&mut self, action: UpdateAction) {
        match action.decision {
            Decision::Install | Decision::Update => self.to_download.push(action),
            Decision::Delete => self.to_delete.push(action),
            Decision::BackupThenReplace => self.to_backup.push(action),
            Decision::Skip(_) => self.skipped.push(action),
            Decision::AskUser => {
                // For MVP, we treat AskUser as Skip
                self.skipped.push(action);
            },
        }
    }
}
