pub mod application;
pub mod core;
pub mod event_system;
pub mod infrastructure;

use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let _ = infrastructure::logger::setup_logger(app.handle());

            // Initialize settings state
            let initial_settings = core::settings::load_settings(app.handle());
            app.manage(core::settings::SettingsState(Mutex::new(initial_settings)));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            application::commands::initialize_fs,
            application::commands::frontend_log_info,
            application::commands::frontend_log_error,
            application::commands::open_logs_folder,
            application::commands::get_settings,
            application::commands::save_settings,
            application::commands::reset_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
