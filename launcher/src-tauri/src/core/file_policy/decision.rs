#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    Install,
    Update,
    Delete,
    Skip(String), // Reason for skipping
    BackupThenReplace,
    AskUser,
}
