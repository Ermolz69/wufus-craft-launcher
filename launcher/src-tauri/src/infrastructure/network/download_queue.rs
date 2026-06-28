use super::cancel_token::CancelToken;
use super::download_error::DownloadError;
use super::file_downloader::FileDownloader;
use crate::application::error::LauncherError;
use crate::core::manifest::file_entry::FileEntry;
use crate::core::updater::update_plan::UpdatePlan;
use crate::infrastructure::fs::SafeDelete;
use reqwest::Client;
use serde::Serialize;
use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tracing::{debug, error, info, info_span, warn};

/// Point-in-time view of queue progress, suitable for serialisation to the frontend.
#[derive(Clone, Serialize)]
pub struct ProgressSnapshot {
    pub total_files: u64,
    pub completed_files: u64,
    pub failed_files: u64,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    /// Bytes per second averaged over the entire session so far.
    pub bytes_per_sec: f64,
    /// Bytes remaining to download.
    pub remaining_bytes: u64,
}

struct SharedProgress {
    total_files: u64,
    completed_files: AtomicU64,
    failed_files: AtomicU64,
    total_bytes: u64,
    downloaded_bytes: AtomicU64,
    start: Instant,
}

impl SharedProgress {
    fn new(total_files: u64, total_bytes: u64) -> Arc<Self> {
        Arc::new(Self {
            total_files,
            completed_files: AtomicU64::new(0),
            failed_files: AtomicU64::new(0),
            total_bytes,
            downloaded_bytes: AtomicU64::new(0),
            start: Instant::now(),
        })
    }

    fn snapshot(&self) -> ProgressSnapshot {
        let downloaded = self.downloaded_bytes.load(Ordering::Relaxed);
        let elapsed = self.start.elapsed().as_secs_f64();
        let bytes_per_sec = if elapsed > 0.0 {
            downloaded as f64 / elapsed
        } else {
            0.0
        };
        let remaining_bytes = self.total_bytes.saturating_sub(downloaded);
        ProgressSnapshot {
            total_files: self.total_files,
            completed_files: self.completed_files.load(Ordering::Relaxed),
            failed_files: self.failed_files.load(Ordering::Relaxed),
            total_bytes: self.total_bytes,
            downloaded_bytes: downloaded,
            bytes_per_sec,
            remaining_bytes,
        }
    }
}

/// Executes an [`UpdatePlan`] with bounded parallelism.
///
/// At most `concurrency` files are downloaded simultaneously.
/// Progress is reported via `on_progress` after every received chunk.
/// Cancellation is cooperative: check happens between chunks.
pub struct DownloadQueue {
    concurrency: usize,
}

impl DownloadQueue {
    pub fn new(concurrency: usize) -> Self {
        Self {
            concurrency: concurrency.max(1),
        }
    }

    /// Removes `.tmp` files in `temp_dir` whose SHA-256 stem is not in the current plan.
    /// This prevents temp files from old or superseded updates from accumulating on disk.
    pub(crate) fn clean_stale_temp_files(plan: &UpdatePlan, temp_dir: &Path) {
        let expected: HashSet<String> = plan
            .to_download
            .iter()
            .filter_map(|a| a.file_entry.as_ref())
            .map(|e| format!("{}.tmp", e.sha256))
            .collect();

        let Ok(dir_iter) = std::fs::read_dir(temp_dir) else {
            return;
        };

        for entry in dir_iter.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.ends_with(".tmp") && !expected.contains(name_str.as_ref()) {
                match std::fs::remove_file(entry.path()) {
                    Ok(()) => debug!("Removed stale temp file: {}", entry.path().display()),
                    Err(e) => warn!(
                        "Failed to remove stale temp file '{}': {e}",
                        entry.path().display()
                    ),
                }
            }
        }
    }

    pub async fn run<F>(
        &self,
        plan: &UpdatePlan,
        game_dir: &Path,
        temp_dir: &Path,
        cancel: CancelToken,
        on_progress: F,
    ) -> Result<(), LauncherError>
    where
        F: Fn(ProgressSnapshot) + Send + Sync + 'static,
    {
        Self::clean_stale_temp_files(plan, temp_dir);

        let entries: Vec<FileEntry> = plan
            .to_download
            .iter()
            .filter_map(|a| a.file_entry.clone())
            .collect();

        let total_files = entries.len() as u64;
        let total_bytes: u64 = entries.iter().map(|e| e.size).sum();

        // Drop the span guard before any .await to keep the future Send.
        {
            let _span = info_span!("download_queue", task = "run").entered();
            info!(total_files, total_bytes, concurrency = self.concurrency, "Queue started");
        }

        let progress = SharedProgress::new(total_files, total_bytes);
        let on_progress = Arc::new(on_progress);
        let semaphore = Arc::new(Semaphore::new(self.concurrency));
        let game_dir = Arc::new(game_dir.to_path_buf());
        let temp_dir = Arc::new(temp_dir.to_path_buf());
        let client = Client::new();

        let mut join_set: JoinSet<Result<(), DownloadError>> = JoinSet::new();

        for entry in entries {
            if cancel.is_cancelled() {
                break;
            }

            // Acquire before spawning — blocks when all slots are busy.
            let permit = semaphore
                .clone()
                .acquire_owned()
                .await
                .expect("semaphore closed");

            let client = client.clone();
            let game_dir = Arc::clone(&game_dir);
            let temp_dir = Arc::clone(&temp_dir);
            let cancel = cancel.clone();
            let progress = Arc::clone(&progress);
            let on_progress = Arc::clone(&on_progress);

            join_set.spawn(async move {
                let _permit = permit;

                if cancel.is_cancelled() {
                    return Err(DownloadError::Cancelled);
                }

                let on_bytes = {
                    let progress = Arc::clone(&progress);
                    let on_progress = Arc::clone(&on_progress);
                    move |n: u64| {
                        progress.downloaded_bytes.fetch_add(n, Ordering::Relaxed);
                        on_progress(progress.snapshot());
                    }
                };

                let temp_path = FileDownloader::download_to_temp(
                    &client,
                    &entry,
                    &temp_dir,
                    &cancel,
                    on_bytes,
                )
                .await?;

                let dest = game_dir.join(&entry.path);
                FileDownloader::install_temp_file(&temp_path, &dest)?;

                progress.completed_files.fetch_add(1, Ordering::Relaxed);
                on_progress(progress.snapshot());

                Ok(())
            });
        }

        // Drain the join set and collect the first non-cancellation error.
        let mut first_error: Option<DownloadError> = None;
        while let Some(outcome) = join_set.join_next().await {
            match outcome {
                Ok(Ok(())) => {},
                Ok(Err(DownloadError::Cancelled)) => {
                    // Expected when cancel was requested — do not propagate as a hard error.
                },
                Ok(Err(e)) => {
                    warn!("Download task failed: {e}");
                    progress.failed_files.fetch_add(1, Ordering::Relaxed);
                    if first_error.is_none() {
                        first_error = Some(e);
                    }
                    cancel.cancel();
                },
                Err(join_err) => {
                    error!("Download task panicked: {join_err}");
                    cancel.cancel();
                    if first_error.is_none() {
                        first_error = Some(DownloadError::Network(
                            "Internal task error".into(),
                        ));
                    }
                },
            }
        }

        if let Some(e) = first_error {
            return Err(LauncherError::DownloadError(e));
        }

        if cancel.is_cancelled() {
            return Err(LauncherError::UpdateCancelled);
        }

        // Apply deletions only after all downloads succeed.
        for action in &plan.to_delete {
            let path = game_dir.join(&action.local_path);
            SafeDelete::delete_file(&path).map_err(LauncherError::SystemError)?;
        }

        let snapshot = progress.snapshot();
        info!(
            files = snapshot.completed_files,
            bytes = snapshot.downloaded_bytes,
            "Queue completed"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::file_policy::decision::Decision;
    use crate::core::file_policy::file_category::FileCategory;
    use crate::core::manifest::file_entry::FileEntry;
    use crate::core::manifest::rules::{DeletePolicy, UpdatePolicy};
    use crate::core::updater::update_plan::{UpdateAction, UpdatePlan};
    use tempfile::TempDir;

    fn make_download_plan(sha256: &str) -> UpdatePlan {
        let mut plan = UpdatePlan::new();
        plan.add(UpdateAction {
            file_entry: Some(FileEntry {
                path: "mods/mod.jar".into(),
                url: "http://example.com/mod.jar".into(),
                sha256: sha256.into(),
                size: 100,
                category: FileCategory::Mod,
                managed: true,
                update_policy: UpdatePolicy::ReplaceIfHashDiffers,
                delete_policy: DeletePolicy::DeleteIfRemovedFromManifest,
            }),
            local_path: "mods/mod.jar".into(),
            decision: Decision::Install,
        });
        plan
    }

    #[test]
    fn stale_temp_files_are_removed() {
        let dir = TempDir::new().unwrap();
        let stale = dir.path().join("deadbeef.tmp");
        let current = dir.path().join("abc123.tmp");
        std::fs::write(&stale, b"old").unwrap();
        std::fs::write(&current, b"current").unwrap();

        let plan = make_download_plan("abc123");
        DownloadQueue::clean_stale_temp_files(&plan, dir.path());

        assert!(!stale.exists(), "stale temp file must be removed");
        assert!(current.exists(), "current temp file must be preserved");
    }

    #[test]
    fn non_tmp_files_are_not_touched() {
        let dir = TempDir::new().unwrap();
        let other = dir.path().join("manifest.json");
        std::fs::write(&other, b"{}").unwrap();

        let plan = UpdatePlan::new();
        DownloadQueue::clean_stale_temp_files(&plan, dir.path());

        assert!(other.exists(), "non-.tmp files must not be touched");
    }

    #[test]
    fn empty_temp_dir_does_not_panic() {
        let dir = TempDir::new().unwrap();
        let plan = UpdatePlan::new();
        // Must not panic even when temp_dir is empty.
        DownloadQueue::clean_stale_temp_files(&plan, dir.path());
    }
}
