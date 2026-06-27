use std::path::{Path, PathBuf};
use tracing::warn;
use walkdir::WalkDir;

pub struct FileScanner;

impl FileScanner {
    pub fn scan_directory(base_dir: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        if !base_dir.exists() {
            return files;
        }

        for entry in WalkDir::new(base_dir).into_iter().filter_map(|e| match e {
            Ok(entry) => Some(entry),
            Err(err) => {
                warn!("Scan error in {}: {err}", base_dir.display());
                None
            },
        }) {
            if entry.file_type().is_file() {
                if let Ok(relative_path) = entry.path().strip_prefix(base_dir) {
                    files.push(relative_path.to_path_buf());
                }
            }
        }

        files
    }
}
