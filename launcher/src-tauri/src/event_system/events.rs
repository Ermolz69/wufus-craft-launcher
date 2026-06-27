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

// In the future we will use tauri::Emitter to emit these events to React.
