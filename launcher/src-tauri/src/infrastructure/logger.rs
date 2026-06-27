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

    // Console output layer
    let console_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_level(true)
        .pretty(); // Use pretty formatting for the console

    // File output layer with rotation (daily)
    let file_appender = tracing_appender::rolling::daily(logs_dir, "launcher.log");
    let (non_blocking_appender, guard) = tracing_appender::non_blocking(file_appender);

    // Leak the guard so the background logging thread stays alive for the app's lifetime.
    Box::leak(Box::new(guard));

    let file_layer = fmt::layer()
        .with_writer(non_blocking_appender)
        .with_ansi(false) // Don't write ansi color codes to file
        .with_target(true)
        .with_thread_ids(true);

    // Environment filter defaults to INFO if RUST_LOG is not set
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer)
        .init();

    tracing::info!("Tracing logger initialized.");
    Ok(())
}
