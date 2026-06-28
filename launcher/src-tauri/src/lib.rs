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

            let initial_settings = core::settings::load_settings(app.handle());
            app.manage(core::settings::SettingsState(Mutex::new(initial_settings)));
            app.manage(application::updater_commands::UpdaterState::default());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            application::commands::initialize_fs,
            application::commands::frontend_log_info,
            application::commands::frontend_log_error,
            application::commands::open_logs_folder,
            application::commands::get_settings,
            application::commands::save_settings,
            application::commands::reset_settings,
            application::updater_commands::start_update,
            application::updater_commands::start_repair,
            application::updater_commands::cancel_update,
            application::launch_commands::prepare_launch,
            application::launch_commands::check_java,
            application::launch_commands::launch_minecraft,
            application::news_commands::fetch_news,
            application::server_commands::check_server_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
