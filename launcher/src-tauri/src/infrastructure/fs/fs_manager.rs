use std::fs;
use std::path::Path;
use tracing::{error, info, info_span, warn};
use crate::application::error::LauncherError;
use super::path_resolver::PathResolver;

pub struct FsManager;

impl FsManager {
    pub fn initialize(paths: &PathResolver) -> Result<(), LauncherError> {
        let _span = info_span!("fs_manager", task="init").entered();
        info!("Initializing File System Structure...");

        Self::validate_path(&paths.game)?;

        if paths.temp.exists() {
            info!("Cleaning up temp directory: {:?}", paths.temp);
            if let Err(e) = fs::remove_dir_all(&paths.temp) {
                warn!("Failed to completely clean temp directory: {}", e);
            }
        }

        Self::ensure_dir_rw(&paths.cache)?;
        Self::ensure_dir_rw(&paths.temp)?;
        Self::ensure_dir_rw(&paths.logs)?;
        Self::ensure_dir_rw(&paths.game)?;

        info!("File System Structure initialized successfully.");
        Ok(())
    }

    fn validate_path(path: &Path) -> Result<(), LauncherError> {
        let path_str = path.to_string_lossy().to_lowercase();
        
        if path_str.is_empty() {
            return Err(LauncherError::SystemError("Game path cannot be empty.".into()));
        }

        if path_str == "c:\\" || path_str == "c:/" || path.parent().is_none() {
            return Err(LauncherError::SystemError(format!(
                "Cannot use root directory '{}' as game path. Please use a subfolder.",
                path_str
            )));
        }

        if path_str.contains("c:\\windows") || path_str.contains("c:\\program files") {
            return Err(LauncherError::SystemError(format!(
                "Cannot use system directory '{}' as game path. Please use a user folder.",
                path_str
            )));
        }

        Ok(())
    }

    fn ensure_dir_rw(dir: &Path) -> Result<(), LauncherError> {
        if !dir.exists() {
            if let Err(e) = fs::create_dir_all(dir) {
                error!("Failed to create directory {:?}: {}", dir, e);
                return Err(LauncherError::SystemError(format!(
                    "Нет прав на запись в папку {}. Выберите другую папку в настройках или проверьте права доступа. Рекомендуем использовать папку внутри профиля пользователя.",
                    dir.to_string_lossy()
                )));
            }
        }

        let test_file = dir.join(".write_test");
        if let Err(e) = fs::write(&test_file, b"test") {
            error!("Write permission test failed for {:?}: {}", dir, e);
            return Err(LauncherError::SystemError(format!(
                "Нет прав на запись в папку {}. Выберите другую папку в настройках или проверьте права доступа. Рекомендуем использовать папку внутри профиля пользователя.",
                dir.to_string_lossy()
            )));
        }

        if let Err(e) = fs::remove_file(&test_file) {
            warn!("Failed to remove test file {:?}: {}", test_file, e);
        }

        Ok(())
    }
}
