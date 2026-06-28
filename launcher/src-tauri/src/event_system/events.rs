use crate::infrastructure::network::ProgressSnapshot;
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct ProgressEvent {
    pub task: String,
    pub percent: f64,
}

#[derive(Clone, Serialize)]
pub struct LogEvent {
    pub level: String,
    pub message: String,
}

/// Emitted on the `update://progress` channel during an active update.
/// Mirrors [`ProgressSnapshot`] but is the canonical frontend event type.
pub type UpdateProgressEvent = ProgressSnapshot;
