use super::state_paths::StatePaths;
use crate::application::error::LauncherError;
use crate::core::state::local_state::LocalState;
use crate::core::state::validation::StateValidator;
use std::fs;
use tracing::{error, info, warn};

pub struct LocalStateStore;

impl LocalStateStore {
    pub fn load(paths: &StatePaths) -> LocalState {
        let file_path = paths.state_file();

        if !file_path.exists() {
            return LocalState::default();
        }

        let json = match fs::read_to_string(&file_path) {
            Ok(j) => j,
            Err(e) => {
                error!("Failed to read state file {}: {e}", file_path.display());
                return LocalState::default();
            },
        };

        let state: LocalState = match serde_json::from_str(&json) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to parse state file {}: {e}", file_path.display());
                return LocalState::default();
            },
        };

        if let Err(e) = StateValidator::validate(&state) {
            warn!("State validation failed: {e}. Re-creating state.");
            return LocalState::default();
        }

        if state.is_interrupted() {
            warn!("Previous update was interrupted.");
        }

        state
    }

    pub fn save(paths: &StatePaths, state: &LocalState) -> Result<(), LauncherError> {
        if !paths.base_dir.exists() {
            fs::create_dir_all(&paths.base_dir).map_err(|e| {
                LauncherError::SystemError(format!("Failed to create .wufus directory: {e}"))
            })?;
        }

        let file_path = paths.state_file();
        let json = serde_json::to_string_pretty(state)
            .map_err(|e| LauncherError::SystemError(format!("Failed to serialize state: {e}")))?;

        fs::write(&file_path, json)
            .map_err(|e| LauncherError::SystemError(format!("Failed to write state file: {e}")))?;

        info!("Local state saved to {}", file_path.display());
        Ok(())
    }
}
