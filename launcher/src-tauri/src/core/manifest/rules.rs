use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdatePolicy {
    ReplaceIfHashDiffers,
    InstallIfMissing,
    PreserveExisting,
    BackupThenReplace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeletePolicy {
    DeleteIfRemovedFromManifest,
    KeepIfUnknown,
    NeverDelete,
    AskUser,
}
