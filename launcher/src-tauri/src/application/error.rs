use thiserror::Error;

#[derive(Error, Debug)]
pub enum LauncherError {
    #[error("Failed to read settings: {0}")]
    SettingsReadError(String),
    #[error("Failed to write settings: {0}")]
    SettingsWriteError(String),
    #[error("System error: {0}")]
    SystemError(String),
    // other errors will go here
}

impl serde::Serialize for LauncherError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
