use std::fs;
use std::path::Path;
use tracing::{info, error};

pub struct SafeDelete;

impl SafeDelete {
    pub fn delete_file(path: &Path) -> Result<(), String> {
        if !path.exists() {
            return Ok(());
        }

        if path.is_file() {
            match fs::remove_file(path) {
                Ok(_) => {
                    info!("Deleted file: {:?}", path);
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to delete file {:?}: {}", path, e);
                    Err(e.to_string())
                }
            }
        } else {
            Err(format!("Path is not a file: {:?}", path))
        }
    }
}
