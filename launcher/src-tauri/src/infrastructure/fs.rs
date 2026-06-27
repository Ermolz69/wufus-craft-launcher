use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, error, info_span, warn};
use crate::application::error::LauncherError;

pub struct PathResolver {
    pub app_data: PathBuf,
    pub cache: PathBuf,
    pub temp: PathBuf,
    pub logs: PathBuf,
    pub game: PathBuf,
}

impl PathResolver {
    pub fn new(app_data_path: PathBuf, game_path: String) -> Self {
        Self {
            cache: app_data_path.join("cache"),
            temp: app_data_path.join("temp"),
            logs: app_data_path.join("logs"),
            app_data: app_data_path,
            game: PathBuf::from(game_path),
        }
    }
}

pub struct FsManager;

impl FsManager {
    pub fn initialize(paths: &PathResolver) -> Result<(), LauncherError> {
        let _span = info_span!("fs_manager", task="init").entered();
        info!("Initializing File System Structure...");

        // 1. Basic validation of the game path
        Self::validate_path(&paths.game)?;

        // 2. Clean up temp folder from previous sessions
        if paths.temp.exists() {
            info!("Cleaning up temp directory: {:?}", paths.temp);
            if let Err(e) = fs::remove_dir_all(&paths.temp) {
                warn!("Failed to completely clean temp directory: {}", e);
            }
        }

        // 3. Create necessary directories
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

        // Extremely basic checks to prevent system dir overriding
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
        // Create directory if not exists
        if !dir.exists() {
            if let Err(e) = fs::create_dir_all(dir) {
                error!("Failed to create directory {:?}: {}", dir, e);
                return Err(LauncherError::SystemError(format!(
                    "Нет прав на запись в папку {}. Выберите другую папку в настройках или проверьте права доступа. Рекомендуем использовать папку внутри профиля пользователя.",
                    dir.to_string_lossy()
                )));
            }
        }

        // Test read/write permissions
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
