use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};
use tracing::{error, info, warn};

fn default_manifest_url() -> String {
    "https://update.example.com/manifest.json".to_string()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LauncherSettings {
    pub game_path: String,
    pub ram_gb: u32,
    pub close_after_launch: bool,
    pub minimize_on_close: bool,
    /// URL of the remote build manifest.
    /// Uses a placeholder default so existing settings files without this field
    /// still deserialise correctly.
    #[serde(default = "default_manifest_url")]
    pub manifest_url: String,
}

impl Default for LauncherSettings {
    fn default() -> Self {
        let game_path = dirs::home_dir().map_or_else(
            || "C:\\Games\\WufusCraft".to_string(),
            |p| {
                p.join("Games")
                    .join("WufusCraft")
                    .to_string_lossy()
                    .to_string()
            },
        );

        Self {
            game_path,
            ram_gb: 4,
            close_after_launch: true,
            minimize_on_close: false,
            manifest_url: default_manifest_url(),
        }
    }
}

pub struct SettingsState(pub Mutex<LauncherSettings>);

pub fn get_settings_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {e}"))?;

    if !app_data_dir.exists() {
        fs::create_dir_all(&app_data_dir)
            .map_err(|e| format!("Failed to create app data dir: {e}"))?;
    }

    Ok(app_data_dir.join("settings.json"))
}

pub fn load_settings(app_handle: &AppHandle) -> LauncherSettings {
    let settings_path = match get_settings_path(app_handle) {
        Ok(path) => path,
        Err(e) => {
            error!("Cannot determine settings path: {e}");
            return LauncherSettings::default();
        },
    };

    if !settings_path.exists() {
        info!("Settings file not found, creating default.");
        let default_settings = LauncherSettings::default();
        let _ = save_to_disk(&settings_path, &default_settings);
        return default_settings;
    }

    let file_content = match fs::read_to_string(&settings_path) {
        Ok(content) => content,
        Err(e) => {
            error!("Failed to read settings file: {e}");
            return LauncherSettings::default();
        },
    };

    match serde_json::from_str::<LauncherSettings>(&file_content) {
        Ok(settings) => settings,
        Err(e) => {
            warn!("Settings file is corrupted or outdated: {e}. Resetting to defaults.",);
            let default_settings = LauncherSettings::default();
            let _ = save_to_disk(&settings_path, &default_settings);
            default_settings
        },
    }
}

pub fn save_to_disk(path: &PathBuf, settings: &LauncherSettings) -> Result<(), String> {
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(())
}
