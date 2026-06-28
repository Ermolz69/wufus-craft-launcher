use super::cancel_token::CancelToken;
use super::download_error::DownloadError;
use crate::core::manifest::file_entry::FileEntry;
use crate::infrastructure::fs::sha256_file;
use reqwest::Client;
use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

pub struct FileDownloader;

impl FileDownloader {
    /// Verifies that a temp file on disk matches the expected size and SHA-256.
    /// Size is checked first (cheap) before the full hash pass (expensive).
    pub(crate) fn verify_existing_temp(
        path: &Path,
        entry: &FileEntry,
    ) -> Result<(), DownloadError> {
        let size = std::fs::metadata(path)
            .map_err(|e| DownloadError::FileSystem(e.to_string()))?
            .len();

        if size != entry.size {
            return Err(DownloadError::SizeMismatch {
                path: entry.path.clone(),
                expected: entry.size,
                actual: size,
            });
        }

        let actual = sha256_file(path)
            .map_err(|e| DownloadError::FileSystem(e.to_string()))?;

        if actual != entry.sha256 {
            return Err(DownloadError::ChecksumMismatch {
                path: entry.path.clone(),
                expected: entry.sha256.clone(),
                actual,
            });
        }

        Ok(())
    }

    /// Downloads a file to a temporary location and verifies its size and checksum.
    /// If a valid temp file already exists from a previous run, reuses it without
    /// issuing a network request. Invalid temp files are deleted and re-downloaded.
    /// Checks `cancel` between every chunk; cleans up the temp file on any error.
    /// Calls `on_bytes(n)` after each chunk with the number of bytes just written.
    pub async fn download_to_temp(
        client: &Client,
        entry: &FileEntry,
        temp_dir: &Path,
        cancel: &CancelToken,
        on_bytes: impl Fn(u64) + Send,
    ) -> Result<PathBuf, DownloadError> {
        let temp_path = temp_dir.join(format!("{}.tmp", entry.sha256));

        // Reuse a valid temp file from an interrupted previous session.
        if temp_path.exists() {
            match Self::verify_existing_temp(&temp_path, entry) {
                Ok(()) => {
                    info!(
                        path = %entry.path,
                        bytes = entry.size,
                        "Resuming: reusing verified temp file"
                    );
                    on_bytes(entry.size);
                    return Ok(temp_path);
                },
                Err(e) => {
                    warn!(path = %entry.path, reason = %e, "Removing invalid temp file");
                    let _ = std::fs::remove_file(&temp_path);
                },
            }
        }

        debug!(
            path = %entry.path,
            url = %entry.url,
            dest = %temp_path.display(),
            "Starting download"
        );

        let mut res = client
            .get(&entry.url)
            .send()
            .await
            .map_err(|e| DownloadError::Network(e.to_string()))?;

        if !res.status().is_success() {
            return Err(DownloadError::Network(format!(
                "HTTP {} downloading '{}'",
                res.status(),
                entry.path
            )));
        }

        let mut file = std::fs::File::create(&temp_path)
            .map_err(|e| DownloadError::FileSystem(format!("Cannot create temp file: {e}")))?;

        let mut hasher = Sha256::new();
        let mut downloaded: u64 = 0;

        loop {
            if cancel.is_cancelled() {
                drop(file);
                let _ = std::fs::remove_file(&temp_path);
                return Err(DownloadError::Cancelled);
            }

            match res.chunk().await {
                Ok(Some(chunk)) => {
                    file.write_all(&chunk).map_err(|e| {
                        let _ = std::fs::remove_file(&temp_path);
                        DownloadError::FileSystem(format!(
                            "Write error for '{}': {e}",
                            entry.path
                        ))
                    })?;
                    hasher.update(&chunk);
                    let n = chunk.len() as u64;
                    downloaded += n;
                    on_bytes(n);
                },
                Ok(None) => break,
                Err(e) => {
                    drop(file);
                    let _ = std::fs::remove_file(&temp_path);
                    return Err(DownloadError::Network(format!(
                        "Stream interrupted for '{}': {e}",
                        entry.path
                    )));
                },
            }
        }

        file.flush().map_err(|e| {
            let _ = std::fs::remove_file(&temp_path);
            DownloadError::FileSystem(format!("Flush error for '{}': {e}", entry.path))
        })?;
        drop(file);

        if downloaded != entry.size {
            let _ = std::fs::remove_file(&temp_path);
            return Err(DownloadError::SizeMismatch {
                path: entry.path.clone(),
                expected: entry.size,
                actual: downloaded,
            });
        }

        let actual_hash = hex::encode(hasher.finalize());
        if actual_hash != entry.sha256 {
            let _ = std::fs::remove_file(&temp_path);
            return Err(DownloadError::ChecksumMismatch {
                path: entry.path.clone(),
                expected: entry.sha256.clone(),
                actual: actual_hash,
            });
        }

        info!(
            path = %entry.path,
            bytes = downloaded,
            "Download verified"
        );

        Ok(temp_path)
    }

    /// Moves a verified temp file to its final destination.
    /// Creates parent directories as needed.
    /// Falls back to copy+delete if rename fails (e.g. cross-device move).
    pub fn install_temp_file(temp_path: &Path, dest: &Path) -> Result<(), DownloadError> {
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                DownloadError::FileSystem(format!(
                    "Cannot create directory '{}': {e}",
                    parent.display()
                ))
            })?;
        }

        if std::fs::rename(temp_path, dest).is_ok() {
            return Ok(());
        }

        // Cross-device fallback: copy then remove
        warn!(
            src = %temp_path.display(),
            dest = %dest.display(),
            "Rename failed, falling back to copy"
        );

        std::fs::copy(temp_path, dest).map_err(|e| {
            DownloadError::FileSystem(format!("Copy error to '{}': {e}", dest.display()))
        })?;

        if let Err(e) = std::fs::remove_file(temp_path) {
            warn!(
                path = %temp_path.display(),
                "Failed to remove temp file after copy: {e}"
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::file_policy::file_category::FileCategory;
    use crate::core::manifest::rules::{DeletePolicy, UpdatePolicy};
    use sha2::{Digest, Sha256};
    use tempfile::TempDir;

    fn sha256_of(data: &[u8]) -> String {
        hex::encode(Sha256::digest(data))
    }

    fn make_entry(path: &str, content: &[u8]) -> FileEntry {
        FileEntry {
            path: path.into(),
            url: "http://example.com/file".into(),
            sha256: sha256_of(content),
            size: content.len() as u64,
            category: FileCategory::Asset,
            managed: true,
            update_policy: UpdatePolicy::ReplaceIfHashDiffers,
            delete_policy: DeletePolicy::DeleteIfRemovedFromManifest,
        }
    }

    fn write_temp(dir: &TempDir, name: &str, content: &[u8]) -> PathBuf {
        let path = dir.path().join(name);
        std::fs::write(&path, content).unwrap();
        path
    }

    // ── verify_existing_temp ──────────────────────────────────────────────────

    #[test]
    fn verify_accepts_matching_temp_file() {
        let dir = TempDir::new().unwrap();
        let content = b"valid mod content";
        let entry = make_entry("mods/mod.jar", content);
        let path = dir.path().join(format!("{}.tmp", entry.sha256));
        std::fs::write(&path, content).unwrap();

        assert!(
            FileDownloader::verify_existing_temp(&path, &entry).is_ok(),
            "valid temp file must pass verification"
        );
    }

    #[test]
    fn verify_rejects_wrong_size() {
        let dir = TempDir::new().unwrap();
        let content = b"short";
        let entry = make_entry("mods/mod.jar", content);
        // Write different (longer) content so size doesn't match.
        let path = dir.path().join(format!("{}.tmp", entry.sha256));
        std::fs::write(&path, b"much longer content than expected").unwrap();

        assert!(
            matches!(
                FileDownloader::verify_existing_temp(&path, &entry),
                Err(DownloadError::SizeMismatch { .. })
            ),
            "size mismatch must be detected"
        );
    }

    #[test]
    fn verify_rejects_wrong_hash() {
        let dir = TempDir::new().unwrap();
        let content = b"original content";
        let mut entry = make_entry("mods/mod.jar", content);
        // Corrupt the expected hash so size matches but hash doesn't.
        let tampered = b"tampered content"; // same length as "original content"
        entry.size = tampered.len() as u64;
        let path = dir.path().join(format!("{}.tmp", entry.sha256));
        std::fs::write(&path, tampered).unwrap();

        assert!(
            matches!(
                FileDownloader::verify_existing_temp(&path, &entry),
                Err(DownloadError::ChecksumMismatch { .. })
            ),
            "checksum mismatch must be detected"
        );
    }

    // ── install_temp_file ─────────────────────────────────────────────────────

    #[test]
    fn install_creates_parent_dirs_and_moves_file() {
        let src_dir = TempDir::new().unwrap();
        let dest_dir = TempDir::new().unwrap();

        let content = b"hello verified file";
        let temp_path = write_temp(&src_dir, "abc123.tmp", content);
        let dest = dest_dir.path().join("mods/subdir/mod.jar");

        FileDownloader::install_temp_file(&temp_path, &dest).unwrap();

        assert!(dest.exists(), "destination file must exist");
        assert_eq!(std::fs::read(&dest).unwrap(), content);
        assert!(!temp_path.exists(), "temp file must be removed after install");
    }

    #[test]
    fn install_overwrites_existing_file() {
        let src_dir = TempDir::new().unwrap();
        let dest_dir = TempDir::new().unwrap();

        let old_content = b"old data";
        let new_content = b"new verified data";
        let dest = dest_dir.path().join("mod.jar");
        std::fs::write(&dest, old_content).unwrap();

        let temp_path = write_temp(&src_dir, "new.tmp", new_content);
        FileDownloader::install_temp_file(&temp_path, &dest).unwrap();

        assert_eq!(std::fs::read(&dest).unwrap(), new_content);
    }
}
