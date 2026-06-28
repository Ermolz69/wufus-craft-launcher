use std::fs;
use std::process::Stdio;

use tracing::{error, info};

use crate::application::error::LauncherError;
use crate::core::launcher::GameLaunchParams;

pub struct GameRunner;

impl GameRunner {
    /// Spawn the game process and return immediately.
    ///
    /// Both stdout and stderr of the JVM are redirected to
    /// `{game_dir}/logs/latest.log` (appended). A background thread waits for
    /// the child to prevent zombies on POSIX; on Windows this is a no-op.
    pub fn launch(params: GameLaunchParams) -> Result<(), LauncherError> {
        let logs_dir = params.game_dir.join("logs");
        fs::create_dir_all(&logs_dir)
            .map_err(|e| LauncherError::SystemError(format!("Cannot create logs dir: {e}")))?;

        let log_path = logs_dir.join("latest.log");

        let log_out = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .map_err(|e| LauncherError::SystemError(format!("Cannot open game log: {e}")))?;

        let log_err = log_out
            .try_clone()
            .map_err(|e| LauncherError::SystemError(format!("Cannot clone log handle: {e}")))?;

        let mut cmd = params.into_command();
        cmd.stdout(Stdio::from(log_out));
        cmd.stderr(Stdio::from(log_err));

        let mut child = cmd
            .spawn()
            .map_err(|e| LauncherError::SystemError(format!("Failed to start game: {e}")))?;

        let pid = child.id();
        let log_display = log_path.display().to_string();
        info!("Game process started (pid {pid}). Logs → {log_display}");

        std::thread::spawn(move || match child.wait() {
            Ok(status) => info!("Game process (pid {pid}) exited: {status}"),
            Err(e) => error!("Game process wait error (pid {pid}): {e}"),
        });

        Ok(())
    }
}
