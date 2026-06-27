use super::build_manifest::BuildManifest;
use crate::application::error::LauncherError;

pub fn validate_manifest(manifest: &BuildManifest) -> Result<(), LauncherError> {
    for file in &manifest.files {
        if is_unsafe_path(&file.path) {
            return Err(LauncherError::SystemError(format!(
                "Unsafe path detected in manifest: {}",
                file.path
            )));
        }
    }

    for path in &manifest.protected_paths {
        if is_unsafe_path(path) {
            return Err(LauncherError::SystemError(format!(
                "Unsafe protected path detected in manifest: {path}",
            )));
        }
    }

    Ok(())
}

fn is_unsafe_path(path: &str) -> bool {
    path.contains("..") || path.starts_with('/') || path.starts_with('\\') || path.contains(':')
}
