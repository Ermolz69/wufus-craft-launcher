use super::local_state::LocalState;
use crate::application::error::LauncherError;

pub struct StateValidator;

impl StateValidator {
    pub fn validate(state: &LocalState) -> Result<(), LauncherError> {
        if state.schema_version != 1 {
            return Err(LauncherError::SystemError(format!("Unsupported schema version: {}", state.schema_version)));
        }
        Ok(())
    }
}
