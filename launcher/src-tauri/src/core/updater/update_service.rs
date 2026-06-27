use super::update_plan::{UpdateAction, UpdatePlan};
use crate::application::error::LauncherError;
use crate::core::file_policy::policy::FilePolicy;
use crate::core::manifest::build_manifest::BuildManifest;
use crate::core::manifest::validation::validate_manifest;
use crate::infrastructure::cache::{cache_paths::CachePaths, manifest_cache::ManifestCache};
use crate::infrastructure::fs::file_scanner::FileScanner;
use crate::infrastructure::network::http_client::HttpClient;
use std::path::{Path, PathBuf};
use tracing::warn;

pub struct UpdateService;

impl UpdateService {
    pub async fn prepare_update_plan(
        manifest_url: &str,
        game_dir: &Path,
        cache_paths: &CachePaths,
    ) -> Result<(BuildManifest, UpdatePlan), LauncherError> {
        // 1. Fetch manifest
        let manifest = match HttpClient::fetch_build_manifest(manifest_url).await {
            Ok(m) => {
                validate_manifest(&m)?;
                let _ = ManifestCache::save(cache_paths, &m);
                m
            },
            Err(e) => {
                warn!("Network error: {}. Falling back to cache.", e);
                // Note: Realistically we need the version ID from local state, assuming "latest" for now
                if let Ok(m) = ManifestCache::load(cache_paths, "latest") {
                    m
                } else {
                    return Err(LauncherError::SystemError(
                        "Failed to fetch manifest and no cache available.".into(),
                    ));
                }
            },
        };

        // 2. Scan local directory
        let local_files = FileScanner::scan_directory(game_dir);
        let mut plan = UpdatePlan::new();

        // 3. Evaluate manifest files
        for entry in &manifest.files {
            let local_path = game_dir.join(&entry.path);
            let local_exists = local_path.exists();
            let is_protected = manifest
                .protected_paths
                .iter()
                .any(|p| entry.path.starts_with(p));

            let local_hash = if local_exists {
                Some("dummy_hash") // MVP
            } else {
                None
            };

            let decision = FilePolicy::decide(
                Path::new(&entry.path),
                local_exists,
                local_hash,
                Some(entry),
                is_protected,
            );

            plan.add(UpdateAction {
                file_entry: Some(entry.clone()),
                local_path: PathBuf::from(&entry.path),
                decision,
            });
        }

        // 4. Evaluate unknown local files
        for local_rel_path in local_files {
            let path_str = local_rel_path.to_string_lossy().replace('\\', "/");
            let in_manifest = manifest.files.iter().any(|e| e.path == path_str);
            let is_protected = manifest
                .protected_paths
                .iter()
                .any(|p| path_str.starts_with(p));

            if !in_manifest {
                let was_managed = false;

                let decision =
                    FilePolicy::decide_deletion(&local_rel_path, was_managed, None, is_protected);

                plan.add(UpdateAction {
                    file_entry: None,
                    local_path: local_rel_path.clone(),
                    decision,
                });
            }
        }

        Ok((manifest, plan))
    }
}
