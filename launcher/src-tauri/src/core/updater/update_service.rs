use super::manifest_comparator::ManifestComparator;
use super::update_plan::UpdatePlan;
use crate::application::error::LauncherError;
use crate::core::manifest::build_manifest::BuildManifest;
use crate::core::manifest::validation::validate_manifest;
use crate::core::state::local_state::LocalState;
use crate::infrastructure::cache::{cache_paths::CachePaths, manifest_cache::ManifestCache};
use crate::infrastructure::fs::file_scanner::FileScanner;
use crate::infrastructure::network::cancel_token::CancelToken;
use crate::infrastructure::network::download_queue::{DownloadQueue, ProgressSnapshot};
use crate::infrastructure::network::http_client::HttpClient;
use std::path::Path;
use tracing::warn;

/// Default number of simultaneous downloads during an update.
pub const DEFAULT_CONCURRENCY: usize = 3;

pub struct UpdateService;

impl UpdateService {
    pub async fn prepare_update_plan(
        manifest_url: &str,
        game_dir: &Path,
        cache_paths: &CachePaths,
        local_state: &LocalState,
    ) -> Result<(BuildManifest, UpdatePlan), LauncherError> {
        let manifest = match HttpClient::fetch_build_manifest(manifest_url).await {
            Ok(m) => {
                validate_manifest(&m)?;
                let _ = ManifestCache::save(cache_paths, &m);
                m
            },
            Err(e) => {
                warn!("Network error: {e}. Falling back to cache.");
                ManifestCache::load(cache_paths, "latest").map_err(|_| {
                    LauncherError::SystemError(
                        "Failed to fetch manifest and no cache available.".into(),
                    )
                })?
            },
        };

        let local_files = FileScanner::scan_directory(game_dir);
        let plan = ManifestComparator::compare(&manifest, local_state, &local_files, game_dir);

        Ok((manifest, plan))
    }

    /// Executes an update plan with bounded parallel downloads.
    ///
    /// Files are staged in `temp_dir`, verified, then atomically moved into `game_dir`.
    /// The installed build is never modified until a file passes both size and checksum checks.
    ///
    /// Returns `Err(UpdateCancelled)` if `cancel` was triggered without a download error.
    /// Returns `Err(DownloadError(_))` on the first hard failure.
    pub async fn execute_plan<F>(
        plan: &UpdatePlan,
        game_dir: &Path,
        temp_dir: &Path,
        concurrency: usize,
        cancel: CancelToken,
        on_progress: F,
    ) -> Result<(), LauncherError>
    where
        F: Fn(ProgressSnapshot) + Send + Sync + 'static,
    {
        DownloadQueue::new(concurrency)
            .run(plan, game_dir, temp_dir, cancel, on_progress)
            .await
    }
}
