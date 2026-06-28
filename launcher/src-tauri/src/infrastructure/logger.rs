use chrono::Local;
use std::fs::OpenOptions;
use tauri::{AppHandle, Manager};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn setup_logger(app_handle: &AppHandle) -> Result<(), String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {e}"))?;

    let logs_dir = app_data_dir.join("logs");

    if !logs_dir.exists() {
        std::fs::create_dir_all(&logs_dir)
            .map_err(|e| format!("Failed to create logs dir: {e}"))?;
    }

    // File named launcher.YYYY-MM-DD.log, created fresh each app start.
    // For a game launcher (short sessions) this is simpler and cleaner than
    // tracing-appender's daily rotation which forces the date as the file suffix.
    let date_str = Local::now().format("%Y-%m-%d");
    let log_path = logs_dir.join(format!("launcher.{date_str}.log"));

    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|e| format!("Failed to open log file '{}': {e}", log_path.display()))?;

    let (non_blocking_appender, guard) = tracing_appender::non_blocking(log_file);

    // Keep the guard alive for the process lifetime so the writer thread stays running.
    Box::leak(Box::new(guard));

    let console_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_level(true)
        .pretty();

    let file_layer = fmt::layer()
        .with_writer(non_blocking_appender)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(true);

    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer)
        .init();

    tracing::info!("Logger initialized → {}", log_path.display());
    Ok(())
}
