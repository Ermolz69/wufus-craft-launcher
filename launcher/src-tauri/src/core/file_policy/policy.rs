use super::decision::Decision;
use crate::core::manifest::file_entry::FileEntry;
use crate::core::manifest::rules::UpdatePolicy;
use std::path::Path;

pub struct FilePolicy;

impl FilePolicy {
    pub fn decide(
        local_path: &Path, // relative path inside game area
        local_exists: bool,
        local_hash: Option<&str>,
        manifest_entry: Option<&FileEntry>,
        is_protected_path: bool,
    ) -> Decision {
        let path_str = local_path.to_string_lossy().to_string();

        // 1. Protected paths (saves, screenshots, etc.)
        if is_protected_path {
            if !local_exists {
                // If it's in manifest but protected, we can install it if missing
                if let Some(entry) = manifest_entry {
                    if entry.update_policy == UpdatePolicy::InstallIfMissing {
                        return Decision::Install;
                    }
                }
            }
            return Decision::Skip("Protected path".into());
        }

        // 2. Known managed file
        if let Some(entry) = manifest_entry {
            if !local_exists {
                return Decision::Install;
            }

            // File exists, check update policy
            match entry.update_policy {
                UpdatePolicy::ReplaceIfHashDiffers => {
                    if let Some(l_hash) = local_hash {
                        if l_hash != entry.sha256 {
                            return Decision::Update;
                        }
                        return Decision::Skip("Hash matches".into());
                    }
                    return Decision::Update; // No local hash means we update
                },
                UpdatePolicy::InstallIfMissing => {
                    return Decision::Skip("Already installed".into());
                },
                UpdatePolicy::PreserveExisting => {
                    return Decision::Skip("Preserve existing policy".into());
                },
                UpdatePolicy::BackupThenReplace => {
                    if let Some(l_hash) = local_hash {
                        if l_hash != entry.sha256 {
                            return Decision::BackupThenReplace;
                        }
                        return Decision::Skip("Hash matches".into());
                    }
                    return Decision::BackupThenReplace;
                },
            }
        }

        // 3. Unknown local file (not in manifest)
        // Hardcoded protection for common paths if they are not explicitly protected by manifest
        if path_str.starts_with("saves") || path_str.starts_with("screenshots") {
            return Decision::Skip("Default protected folder".into());
        }

        // Apply fallback policies based on inferred categories or general rules
        if path_str.starts_with("mods") {
            return Decision::Skip("Unknown mod (preserving user mod)".into());
        }

        if path_str.starts_with("resourcepacks") || path_str.starts_with("shaderpacks") {
            return Decision::Skip("Unknown user pack".into());
        }

        // General unknown file
        Decision::Skip("Unknown file, default skip".into())
    }

    pub fn decide_deletion(
        _local_path: &Path, // relative path
        was_managed: bool,  // Did it use to be in a previous manifest?
        manifest_entry: Option<&FileEntry>,
        is_protected_path: bool,
    ) -> Decision {
        // If it's protected, NEVER delete
        if is_protected_path {
            return Decision::Skip("Protected path".into());
        }

        // If it's currently in the manifest, we don't delete it (we might update it though)
        if manifest_entry.is_some() {
            return Decision::Skip("Exists in current manifest".into());
        }

        // If it used to be managed but is no longer in manifest
        if was_managed {
            return Decision::Delete;
        }

        Decision::Skip("Unknown file".into())
    }
}
