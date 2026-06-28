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
    #[error("Checksum mismatch for '{path}': expected {expected}, got {actual}")]
    ChecksumMismatch {
        path: String,
        expected: String,
        actual: String,
    },
    #[error("Size mismatch for '{path}': expected {expected} bytes, got {actual} bytes")]
    SizeMismatch {
        path: String,
        expected: u64,
        actual: u64,
    },
    #[error("Download was cancelled")]
    Cancelled,
}
