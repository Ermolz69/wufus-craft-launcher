use crate::infrastructure::network::download_error::DownloadError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LauncherError {
    #[error("Failed to read settings: {0}")]
    SettingsReadError(String),
    #[error("Failed to write settings: {0}")]
    SettingsWriteError(String),
    #[error("System error: {0}")]
    SystemError(String),
    #[error("Download failed: {0}")]
    DownloadError(#[from] DownloadError),
    #[error("Update was cancelled by user")]
    UpdateCancelled,
}

impl serde::Serialize for LauncherError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
