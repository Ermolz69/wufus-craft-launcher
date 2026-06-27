use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum UpdateStatus {
    #[default]
    UpToDate,
    InProgress,
    Corrupted,
}
