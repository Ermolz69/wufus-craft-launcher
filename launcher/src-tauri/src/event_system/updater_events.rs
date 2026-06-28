use crate::core::updater::repair_report::ActionReport;
use crate::infrastructure::network::download_queue::ProgressSnapshot;
use serde::Serialize;

/// Tauri event name emitted to the frontend during update / repair.
pub const UPDATER_EVENT: &str = "updater_event";

#[derive(Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateStage {
    CheckingFiles,
    Downloading,
    Finalizing,
}

/// Category that lets the frontend display an appropriate user-facing message.
#[derive(Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    /// Internet unavailable or download failed.
    Network,
    /// Not enough free disk space.
    DiskSpace,
    /// Filesystem permission denied.
    FileAccess,
    /// Everything else.
    Internal,
}

/// All events the frontend can receive from the updater on the `updater_event` channel.
#[derive(Clone, Serialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum UpdaterEvent {
    /// The updater moved to a new phase.
    Stage { stage: UpdateStage },
    /// Byte-level progress snapshot, emitted after every downloaded chunk.
    Progress(ProgressSnapshot),
    /// The operation completed successfully.
    Done(ActionReport),
    /// The operation failed. `kind` drives the UI message; `message` goes to the log.
    Error { kind: ErrorKind, message: String },
    /// The user cancelled the operation; the build is untouched.
    Cancelled,
}

impl UpdaterEvent {
    pub const fn stage(s: UpdateStage) -> Self {
        Self::Stage { stage: s }
    }

    pub fn error(kind: ErrorKind, msg: impl Into<String>) -> Self {
        Self::Error {
            kind,
            message: msg.into(),
        }
    }

    pub const fn done(report: ActionReport) -> Self {
        Self::Done(report)
    }
}
