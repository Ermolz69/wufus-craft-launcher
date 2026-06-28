use tauri::State;
use tracing::{info, warn};

use crate::core::server_status::{self, ServerStatus};
use crate::core::settings::SettingsState;

/// Ping the Minecraft server and return its current status.
///
/// Uses the 1.7+ Server List Ping (SLP) protocol over a direct TCP connection
/// to `settings.server_host:settings.server_port`. Connection and read timeouts
/// are both 3 seconds.
///
/// Always returns a valid `ServerStatus` — any failure produces `Offline`.
/// Never blocks the UI because the frontend calls this asynchronously via `invoke`.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn check_server_status(settings: State<'_, SettingsState>) -> ServerStatus {
    let (host, port) = {
        let s = settings.0.lock().unwrap();
        (s.server_host.clone(), s.server_port)
    };

    info!("Pinging {host}:{port}");

    match server_status::ping(&host, port) {
        Ok(status) => {
            info!(
                "Server online — {}/{} players, {} ms",
                status.players.unwrap_or(0),
                status.max_players.unwrap_or(0),
                status.ping_ms.unwrap_or(0),
            );
            status
        }
        Err(e) => {
            warn!("Server ping failed: {e}");
            ServerStatus::offline()
        }
    }
}
