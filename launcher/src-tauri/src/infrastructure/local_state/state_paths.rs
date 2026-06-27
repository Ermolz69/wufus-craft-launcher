use std::path::{Path, PathBuf};

pub struct StatePaths {
    pub base_dir: PathBuf,
}

impl StatePaths {
    pub fn new(game_path: &Path) -> Self {
        Self {
            base_dir: game_path.join(".wufus"),
        }
    }

    pub fn state_file(&self) -> PathBuf {
        self.base_dir.join("state.json")
    }

    pub fn lock_file(&self) -> PathBuf {
        self.base_dir.join("lock")
    }

    pub fn backups_dir(&self) -> PathBuf {
        self.base_dir.join("backups")
    }

    pub fn tmp_dir(&self) -> PathBuf {
        self.base_dir.join("tmp")
    }
}
