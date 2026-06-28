use crate::application::error::LauncherError;
use crate::core::settings::SettingsState;
use crate::core::updater::repair_report::ActionReport;
use crate::core::updater::update_plan::UpdatePlan;
use crate::core::updater::update_service::{UpdateService, DEFAULT_CONCURRENCY};
use crate::event_system::updater_events::{UpdateStage, UpdaterEvent, UPDATER_EVENT};
use crate::infrastructure::cache::cache_paths::CachePaths;
use crate::infrastructure::fs::PathResolver;
use crate::infrastructure::local_state::local_state_store::LocalStateStore;
use crate::infrastructure::local_state::state_paths::StatePaths;
use crate::infrastructure::network::cancel_token::CancelToken;
use crate::infrastructure::network::download_queue::ProgressSnapshot;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};
use tracing::{error, info};

// ── State ──────────────────────────────────────────────────────────────────

/// Shared state that persists for the lifetime of the app.
/// Holds the active cancel token so `cancel_update` can stop a running task.
pub struct UpdaterState {
    pub cancel: Mutex<Option<CancelToken>>,
    pub is_running: AtomicBool,
}

impl Default for UpdaterState {
    fn default() -> Self {
        Self {
            cancel: Mutex::new(None),
            is_running: AtomicBool::new(false),
        }
    }
}

// ── Helpers ────────────────────────────────────────────────────────────────

fn emit(app: &AppHandle, event: UpdaterEvent) {
    if let Err(e) = app.emit(UPDATER_EVENT, event) {
        error!("Failed to emit updater event: {e}");
    }
}

fn plan_to_report(plan: &UpdatePlan) -> ActionReport {
    ActionReport {
        files_checked: (plan.to_download.len()
            + plan.to_delete.len()
            + plan.skipped.len()) as u64,
        files_restored: plan.to_download.len() as u64,
        files_deleted: plan.to_delete.len() as u64,
        files_ok: plan.skipped.len() as u64,
    }
}

// ── Core logic ─────────────────────────────────────────────────────────────

/// Inner async logic shared by both update and repair commands.
/// Returns an `ActionReport` on success, or a `LauncherError` on failure / cancellation.
async fn try_run(
    app: AppHandle,
    manifest_url: String,
    game_path: String,
    cancel: CancelToken,
) -> Result<ActionReport, LauncherError> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| LauncherError::SystemError(format!("App data dir: {e}")))?;

    let resolver = PathResolver::new(app_data_dir.clone(), game_path);
    let cache_paths = CachePaths::new(&app_data_dir);
    let state_paths = StatePaths::new(&resolver.game);
    let local_state = LocalStateStore::load(&state_paths);

    emit(&app, UpdaterEvent::stage(UpdateStage::CheckingFiles));

    let (_, plan) = UpdateService::prepare_update_plan(
        &manifest_url,
        &resolver.game,
        &cache_paths,
        &local_state,
    )
    .await?;

    let report = plan_to_report(&plan);

    if plan.is_up_to_date() {
        return Ok(report);
    }

    emit(&app, UpdaterEvent::stage(UpdateStage::Downloading));

    let app_clone = app.clone();
    let on_progress = move |snapshot: ProgressSnapshot| {
        emit(&app_clone, UpdaterEvent::Progress(snapshot));
    };

    UpdateService::execute_plan(
        &plan,
        &resolver.game,
        &resolver.temp,
        DEFAULT_CONCURRENCY,
        cancel,
        on_progress,
    )
    .await?;

    emit(&app, UpdaterEvent::stage(UpdateStage::Finalizing));

    Ok(report)
}

/// Spawned background task that drives the full update/repair flow
/// and emits events for every transition.
async fn run_updater(
    app: AppHandle,
    manifest_url: String,
    game_path: String,
    cancel: CancelToken,
    label: &'static str,
) {
    info!("{label}: starting");

    let result = try_run(app.clone(), manifest_url, game_path, cancel).await;

    // Release the cancel slot and running flag regardless of outcome.
    let updater_state = app.state::<UpdaterState>();
    *updater_state.cancel.lock().unwrap() = None;
    updater_state.is_running.store(false, Ordering::Release);

    match result {
        Ok(report) => {
            info!(
                "{label}: done — restored={}, deleted={}, ok={}",
                report.files_restored, report.files_deleted, report.files_ok
            );
            emit(&app, UpdaterEvent::done(report));
        },
        Err(LauncherError::UpdateCancelled) => {
            info!("{label}: cancelled by user");
            emit(&app, UpdaterEvent::Cancelled);
        },
        Err(e) => {
            error!("{label}: failed — {e}");
            emit(&app, UpdaterEvent::error(e.to_string()));
        },
    }
}

/// Common entry point used by both `start_update` and `start_repair`.
fn begin_run(
    app: AppHandle,
    updater: &UpdaterState,
    manifest_url: String,
    game_path: String,
    label: &'static str,
) -> Result<(), LauncherError> {
    if updater.is_running.swap(true, Ordering::Acquire) {
        return Err(LauncherError::SystemError(
            "Update already in progress".into(),
        ));
    }

    let cancel = CancelToken::new();
    *updater.cancel.lock().unwrap() = Some(cancel.clone());

    tauri::async_runtime::spawn(run_updater(
        app,
        manifest_url,
        game_path,
        cancel,
        label,
    ));

    Ok(())
}

// ── Commands ───────────────────────────────────────────────────────────────

/// Starts a normal update: checks the manifest and downloads any changed or missing files.
/// Returns immediately; progress is reported via `updater_event` Tauri events.
#[tauri::command]
pub fn start_update(
    app: AppHandle,
    updater: State<'_, UpdaterState>,
    settings: State<'_, SettingsState>,
) -> Result<(), LauncherError> {
    let s = settings.0.lock().unwrap().clone();
    begin_run(app, &updater, s.manifest_url, s.game_path, "Update")
}

/// Starts a full integrity check and restores any corrupted or missing files.
/// Functionally identical to `start_update` — both use the same manifest comparison
/// pipeline. The distinction is semantic: Repair is user-triggered.
#[tauri::command]
pub fn start_repair(
    app: AppHandle,
    updater: State<'_, UpdaterState>,
    settings: State<'_, SettingsState>,
) -> Result<(), LauncherError> {
    let s = settings.0.lock().unwrap().clone();
    begin_run(app, &updater, s.manifest_url, s.game_path, "Repair")
}

/// Signals the active update or repair to stop after the current chunk.
/// The installed build is left untouched; the operation can be resumed later.
#[tauri::command]
pub fn cancel_update(updater: State<'_, UpdaterState>) {
    if let Some(token) = updater.cancel.lock().unwrap().as_ref() {
        token.cancel();
        info!("Cancel requested by user");
    }
}
