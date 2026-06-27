use thiserror::Error;

#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("File system error: {0}")]
    FileSystem(String),
    #[error("Invalid data: {0}")]
    InvalidData(String),
}
