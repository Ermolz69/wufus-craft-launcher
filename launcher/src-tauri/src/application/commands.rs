use crate::application::error::LauncherError;
use crate::core::settings::{get_settings_path, save_to_disk, LauncherSettings, SettingsState};
use crate::infrastructure::fs::{FsManager, PathResolver};
use tauri::{AppHandle, Manager, State};
use tracing::{info, info_span};

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn initialize_fs(
    app_handle: AppHandle,
    state: State<SettingsState>,
) -> Result<(), LauncherError> {
    let settings = state.0.lock().unwrap().clone();

    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| LauncherError::SystemError(format!("Failed to get app data dir: {e}")))?;

    let resolver = PathResolver::new(app_data_dir, settings.game_path);
    FsManager::initialize(&resolver)?;

    Ok(())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn get_settings(state: State<SettingsState>) -> Result<LauncherSettings, LauncherError> {
    let _span = info_span!("settings", task = "get").entered();
    let settings = state.0.lock().unwrap().clone();
    Ok(settings)
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn save_settings(
    app_handle: AppHandle,
    state: State<SettingsState>,
    settings: LauncherSettings,
) -> Result<(), LauncherError> {
    let _span = info_span!("settings", task = "save").entered();
    let path = get_settings_path(&app_handle).map_err(LauncherError::SystemError)?;
    save_to_disk(&path, &settings).map_err(LauncherError::SettingsWriteError)?;

    *state.0.lock().unwrap() = settings;
    info!("Settings saved successfully.");
    Ok(())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn reset_settings(
    app_handle: AppHandle,
    state: State<SettingsState>,
) -> Result<LauncherSettings, LauncherError> {
    let _span = info_span!("settings", task = "reset").entered();
    let default_settings = LauncherSettings::default();
    let path = get_settings_path(&app_handle).map_err(LauncherError::SystemError)?;
    save_to_disk(&path, &default_settings).map_err(LauncherError::SettingsWriteError)?;

    *state.0.lock().unwrap() = default_settings.clone();
    info!("Settings reset to default.");
    Ok(default_settings)
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn frontend_log_info(message: String) {
    tracing::info!(target: "frontend", "{}", message);
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn frontend_log_error(message: String) {
    tracing::error!(target: "frontend", "{}", message);
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn open_logs_folder(app_handle: tauri::AppHandle) -> Result<(), LauncherError> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| LauncherError::SystemError(format!("Failed to get app data dir: {e}")))?;
    let logs_dir = app_data_dir.join("logs");

    if !logs_dir.exists() {
        std::fs::create_dir_all(&logs_dir)
            .map_err(|e| LauncherError::SystemError(format!("Failed to create logs dir: {e}")))?;
    }

    open::that(logs_dir)
        .map_err(|e| LauncherError::SystemError(format!("Failed to open logs folder: {e}")))?;
    Ok(())
}
