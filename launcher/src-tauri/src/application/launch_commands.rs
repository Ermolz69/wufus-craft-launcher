use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use tracing::info;

use crate::application::error::LauncherError;
use crate::core::java::{self, MIN_JAVA_VERSION};
use crate::core::launcher::{BuildReadiness, GameLaunchParams, LaunchConfig, LaunchReadiness};
use crate::core::settings::{get_settings_path, save_to_disk, SettingsState};
use crate::core::state::update_status::UpdateStatus;
use crate::infrastructure::cache::cache_paths::CachePaths;
use crate::infrastructure::cache::manifest_cache::ManifestCache;
use crate::infrastructure::fs::PathResolver;
use crate::infrastructure::local_state::local_state_store::LocalStateStore;
use crate::infrastructure::local_state::state_paths::StatePaths;
use crate::infrastructure::processes::GameRunner;

/// Check whether the current game installation is ready to launch.
///
/// Performs fast local checks only (no network):
/// - game directory exists
/// - local state records a completed installation
/// - state is not corrupted or mid-update
/// - required directories are present for the detected loader type
///
/// Returns [`BuildReadiness`] so the frontend can route to launch,
/// update, or repair without any additional round-trip.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn prepare_launch(
    app: AppHandle,
    settings: State<'_, SettingsState>,
) -> Result<BuildReadiness, LauncherError> {
    let settings = settings.0.lock().unwrap().clone();

    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| LauncherError::SystemError(format!("Cannot get app data dir: {e}")))?;

    let resolver = PathResolver::new(app_data_dir.clone(), settings.game_path);
    let game_dir = &resolver.game;

    info!("Checking build readiness at {}", game_dir.display());

    // 1. Game directory must exist
    if !game_dir.exists() {
        return Ok(BuildReadiness {
            status: LaunchReadiness::NeedsUpdate,
            minecraft_version: None,
            loader: None,
            loader_version: None,
            reason: Some("Game directory does not exist.".into()),
        });
    }

    // 2. Load local state (always succeeds — returns default when missing)
    let state_paths = StatePaths::new(game_dir);
    let local_state = LocalStateStore::load(&state_paths);

    // 3. No recorded installation → first run
    let Some(installed_version) = &local_state.current_version else {
        return Ok(BuildReadiness {
            status: LaunchReadiness::NeedsUpdate,
            minecraft_version: None,
            loader: None,
            loader_version: None,
            reason: Some("Game has not been installed yet.".into()),
        });
    };

    // 4. State explicitly marked corrupted by a failed update
    if local_state.status == UpdateStatus::Corrupted {
        return Ok(BuildReadiness {
            status: LaunchReadiness::NeedsRepair,
            minecraft_version: None,
            loader: None,
            loader_version: None,
            reason: Some("Installation is marked as corrupted.".into()),
        });
    }

    // 5. State still InProgress → previous update did not finish
    if local_state.is_interrupted() {
        return Ok(BuildReadiness {
            status: LaunchReadiness::NeedsRepair,
            minecraft_version: None,
            loader: None,
            loader_version: None,
            reason: Some("A previous update did not complete cleanly.".into()),
        });
    }

    // 6. Load cached manifest to extract loader / version metadata (best-effort)
    let cache_paths = CachePaths::new(&app_data_dir);
    let cached_manifest = ManifestCache::load(&cache_paths, installed_version).ok();

    let (mc_ver, loader, loader_ver) = cached_manifest.as_ref().map_or((None, None, None), |m| {
        (
            Some(m.build.minecraft_version.clone()),
            Some(m.build.loader.clone()),
            Some(m.build.loader_version.clone()),
        )
    });

    // 7. Modded builds require mods/
    let is_modded = loader
        .as_deref()
        .is_some_and(|l| !l.is_empty() && l != "vanilla");

    if is_modded && !game_dir.join("mods").exists() {
        return Ok(BuildReadiness {
            status: LaunchReadiness::NeedsUpdate,
            minecraft_version: mc_ver,
            loader,
            loader_version: loader_ver,
            reason: Some("mods/ directory is missing.".into()),
        });
    }

    // 8. Sanity: config/ must exist when files have been installed
    if !local_state.installed_files.is_empty() && !game_dir.join("config").exists() {
        return Ok(BuildReadiness {
            status: LaunchReadiness::NeedsRepair,
            minecraft_version: mc_ver,
            loader,
            loader_version: loader_ver,
            reason: Some("config/ directory is missing.".into()),
        });
    }

    info!("Build ready — version={installed_version}");
    Ok(BuildReadiness {
        status: LaunchReadiness::Ready,
        minecraft_version: mc_ver,
        loader,
        loader_version: loader_ver,
        reason: None,
    })
}

// ── Game launch ───────────────────────────────────────────────────────────────

/// Spawn the Minecraft game process.
///
/// Requires:
/// - Java path saved in settings (populated automatically by `check_java`)
/// - `{game_dir}/.wufus/launch.json` present (downloaded by the updater)
///
/// Redirects game stdout/stderr to `{game_dir}/logs/latest.log`.
/// If `close_after_launch` is set in settings, the launcher exits immediately
/// after the game process has been confirmed started.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn launch_minecraft(
    app: AppHandle,
    settings: State<'_, SettingsState>,
) -> Result<(), LauncherError> {
    let current = settings.0.lock().unwrap().clone();

    let java_exe = current.java_path.as_deref().ok_or_else(|| {
        LauncherError::SystemError(
            "Java path is not set. Use 'Check Java' to detect it, \
             or configure it in Settings → Java Path."
                .into(),
        )
    })?;

    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| LauncherError::SystemError(format!("App data dir: {e}")))?;

    let resolver = PathResolver::new(app_data, current.game_path.clone());
    let game_dir = &resolver.game;

    info!("Launching Minecraft from {}", game_dir.display());

    let config = LaunchConfig::load(game_dir).map_err(LauncherError::SystemError)?;
    let params = GameLaunchParams::build(java_exe, game_dir, &config, &current);

    GameRunner::launch(params)?;

    info!(
        "Game spawned successfully. close_after_launch={}",
        current.close_after_launch
    );

    if current.close_after_launch {
        app.exit(0);
    }

    Ok(())
}

// ── Java check ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JavaStatus {
    /// Java ≥ `MIN_JAVA_VERSION` found and ready.
    Found,
    /// Java found but major version is below the minimum.
    TooOld,
    /// No Java executable detected at all.
    NotFound,
}

/// Result returned to the frontend by `check_java`.
#[derive(Debug, Clone, Serialize)]
pub struct JavaCheckResult {
    pub status: JavaStatus,
    /// Absolute path to the `java` executable (if any was found).
    pub java_path: Option<String>,
    /// Major version of the found Java (e.g. 21).
    pub version: Option<u32>,
    /// Vendor string (e.g. `"OpenJDK"`, "Oracle").
    pub vendor: Option<String>,
    /// Minimum version required.
    pub minimum_required: u32,
}

/// Locate and verify a suitable Java installation.
///
/// Search order:
/// 1. Explicit path from settings (`java_path` field)
/// 2. `JAVA_HOME` env var
/// 3. `java` on PATH
/// 4. Well-known vendor directories
///
/// If a suitable Java is found and the settings `java_path` was previously empty,
/// the path is saved automatically so future launches skip the directory scan.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn check_java(
    app: AppHandle,
    settings: State<'_, crate::core::settings::SettingsState>,
) -> JavaCheckResult {
    let mut current = settings.0.lock().unwrap().clone();
    let explicit = current.java_path.clone();

    // Try to find a suitable (>= MIN_JAVA_VERSION) Java
    if let Some(found) = java::find_suitable_java(explicit.as_deref()) {
        info!(
            "Java found: {} v{} at {}",
            found.vendor, found.version, found.path
        );

        // Auto-save the detected path if it wasn't set explicitly, so next launch is instant.
        if current.java_path.as_deref() != Some(found.path.as_str()) {
            current.java_path = Some(found.path.clone());
            *settings.0.lock().unwrap() = current.clone();
            if let Ok(path) = get_settings_path(&app) {
                let _ = save_to_disk(&path, &current);
            }
        }

        return JavaCheckResult {
            status: JavaStatus::Found,
            java_path: Some(found.path),
            version: Some(found.version),
            vendor: Some(found.vendor),
            minimum_required: MIN_JAVA_VERSION,
        };
    }

    // No suitable Java — check whether *any* Java exists (to distinguish TooOld vs NotFound)
    if let Some(old) = java::find_any_java(explicit.as_deref()) {
        info!(
            "Java too old: {} v{} at {} (need >= {})",
            old.vendor, old.version, old.path, MIN_JAVA_VERSION
        );
        return JavaCheckResult {
            status: JavaStatus::TooOld,
            java_path: Some(old.path),
            version: Some(old.version),
            vendor: Some(old.vendor),
            minimum_required: MIN_JAVA_VERSION,
        };
    }

    info!("No Java installation found.");
    JavaCheckResult {
        status: JavaStatus::NotFound,
        java_path: None,
        version: None,
        vendor: None,
        minimum_required: MIN_JAVA_VERSION,
    }
}
