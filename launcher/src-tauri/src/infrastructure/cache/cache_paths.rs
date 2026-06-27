use std::path::PathBuf;

pub struct CachePaths {
    pub base_dir: PathBuf,
}

impl CachePaths {
    pub fn new(app_data_path: PathBuf) -> Self {
        Self {
            base_dir: app_data_path.join("cache"),
        }
    }

    pub fn manifest_file(&self, version_id: &str) -> PathBuf {
        self.base_dir.join(format!("{}_manifest.json", version_id))
    }
}
