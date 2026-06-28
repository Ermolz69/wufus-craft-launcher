use super::update_plan::{UpdateAction, UpdatePlan};
use crate::core::file_policy::policy::FilePolicy;
use crate::core::manifest::build_manifest::BuildManifest;
use crate::core::state::local_state::LocalState;
use crate::infrastructure::fs::sha256_file;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tracing::warn;

pub struct ManifestComparator;

impl ManifestComparator {
    pub fn compare(
        manifest: &BuildManifest,
        local_state: &LocalState,
        local_files: &[PathBuf],
        game_dir: &Path,
    ) -> UpdatePlan {
        let mut plan = UpdatePlan::new();

        let manifest_paths: HashSet<&str> =
            manifest.files.iter().map(|e| e.path.as_str()).collect();

        for entry in &manifest.files {
            let local_abs = game_dir.join(&entry.path);
            let local_exists = local_abs.exists();
            let is_protected = manifest
                .protected_paths
                .iter()
                .any(|p| entry.path.starts_with(p.as_str()));

            let local_hash = if local_exists {
                match sha256_file(&local_abs) {
                    Ok(h) => Some(h),
                    Err(e) => {
                        let path = local_abs.display();
                        warn!("Cannot hash {path}: {e}, treating as corrupted");
                        None
                    },
                }
            } else {
                None
            };

            let decision = FilePolicy::decide(
                Path::new(&entry.path),
                local_exists,
                local_hash.as_deref(),
                Some(entry),
                is_protected,
            );

            plan.add(UpdateAction {
                file_entry: Some(entry.clone()),
                local_path: PathBuf::from(&entry.path),
                decision,
            });
        }

        for local_rel in local_files {
            let path_str = local_rel.to_string_lossy().replace('\\', "/");
            if manifest_paths.contains(path_str.as_str()) {
                continue;
            }

            let is_protected = manifest
                .protected_paths
                .iter()
                .any(|p| path_str.starts_with(p.as_str()));

            let was_managed = local_state
                .installed_files
                .get(&path_str)
                .is_some_and(|f| f.managed);

            let decision = FilePolicy::decide_deletion(local_rel, was_managed, None, is_protected);

            plan.add(UpdateAction {
                file_entry: None,
                local_path: local_rel.clone(),
                decision,
            });
        }

        plan
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::file_policy::decision::Decision;
    use crate::core::file_policy::file_category::FileCategory;
    use crate::core::manifest::file_entry::FileEntry;
    use crate::core::manifest::rules::{DeletePolicy, UpdatePolicy};
    use crate::core::manifest::version_info::VersionInfo;
    use crate::core::state::installed_file::InstalledFile;
    use sha2::{Digest, Sha256};
    use tempfile::TempDir;

    fn make_manifest(files: Vec<FileEntry>) -> BuildManifest {
        BuildManifest {
            build: VersionInfo {
                id: "1.0".into(),
                version: "1.0.0".into(),
                minecraft_version: "1.21".into(),
                loader: "fabric".into(),
                loader_version: "0.16".into(),
            },
            files,
            protected_paths: vec![],
        }
    }

    fn make_entry(path: &str, sha256: &str, size: u64) -> FileEntry {
        FileEntry {
            path: path.into(),
            url: "http://example.com/file".into(),
            sha256: sha256.into(),
            size,
            category: FileCategory::Asset,
            managed: true,
            update_policy: UpdatePolicy::ReplaceIfHashDiffers,
            delete_policy: DeletePolicy::DeleteIfRemovedFromManifest,
        }
    }

    fn sha256_of(content: &[u8]) -> String {
        hex::encode(Sha256::digest(content))
    }

    fn write_file(dir: &TempDir, rel: &str, content: &[u8]) {
        let path = dir.path().join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, content).unwrap();
    }

    #[test]
    fn matching_file_is_skipped() {
        let tmp = TempDir::new().unwrap();
        let content = b"hello world";
        let hash = sha256_of(content);
        write_file(&tmp, "assets/file.jar", content);

        let manifest = make_manifest(vec![make_entry("assets/file.jar", &hash, 11)]);
        let local_files = vec![PathBuf::from("assets/file.jar")];

        let plan = ManifestComparator::compare(
            &manifest,
            &LocalState::default(),
            &local_files,
            tmp.path(),
        );

        assert!(
            plan.to_download.is_empty(),
            "matching file must not be downloaded"
        );
        assert!(plan.to_delete.is_empty());
        assert_eq!(plan.skipped.len(), 1);
        assert!(plan.is_up_to_date());
    }

    #[test]
    fn missing_file_is_queued_for_install() {
        let tmp = TempDir::new().unwrap();
        let manifest = make_manifest(vec![make_entry("mods/mod.jar", "deadbeef", 500)]);

        let plan = ManifestComparator::compare(&manifest, &LocalState::default(), &[], tmp.path());

        assert_eq!(plan.to_download.len(), 1);
        assert!(matches!(plan.to_download[0].decision, Decision::Install));
        assert_eq!(plan.download_size_bytes(), 500);
    }

    #[test]
    fn corrupted_file_is_queued_for_update() {
        let tmp = TempDir::new().unwrap();
        write_file(&tmp, "mods/mod.jar", b"corrupted content");

        let manifest = make_manifest(vec![make_entry("mods/mod.jar", "expected_hash_1234", 100)]);
        let local_files = vec![PathBuf::from("mods/mod.jar")];

        let plan = ManifestComparator::compare(
            &manifest,
            &LocalState::default(),
            &local_files,
            tmp.path(),
        );

        assert_eq!(plan.to_download.len(), 1);
        assert!(matches!(plan.to_download[0].decision, Decision::Update));
    }

    #[test]
    fn managed_file_removed_from_manifest_is_deleted() {
        let tmp = TempDir::new().unwrap();
        write_file(&tmp, "mods/old_mod.jar", b"old mod");

        let manifest = make_manifest(vec![]);
        let mut local_state = LocalState::default();
        local_state.installed_files.insert(
            "mods/old_mod.jar".into(),
            InstalledFile {
                sha256: "whatever".into(),
                size: 100,
                category: FileCategory::Mod,
                managed: true,
                source_manifest_version: "0.9".into(),
                installed_at: 0,
            },
        );
        let local_files = vec![PathBuf::from("mods/old_mod.jar")];

        let plan = ManifestComparator::compare(&manifest, &local_state, &local_files, tmp.path());

        assert_eq!(
            plan.to_delete.len(),
            1,
            "managed file no longer in manifest must be deleted"
        );
    }

    #[test]
    fn user_file_not_in_manifest_is_preserved() {
        let tmp = TempDir::new().unwrap();
        write_file(&tmp, "saves/my_world/level.dat", b"world data");

        let manifest = make_manifest(vec![]);
        let local_files = vec![PathBuf::from("saves/my_world/level.dat")];

        let plan = ManifestComparator::compare(
            &manifest,
            &LocalState::default(),
            &local_files,
            tmp.path(),
        );

        assert!(plan.to_delete.is_empty(), "user save must never be deleted");
    }

    #[test]
    fn download_size_is_sum_of_missing_file_sizes() {
        let tmp = TempDir::new().unwrap();
        let manifest = make_manifest(vec![
            make_entry("a.jar", "hash1", 500),
            make_entry("b.jar", "hash2", 300),
        ]);

        let plan = ManifestComparator::compare(&manifest, &LocalState::default(), &[], tmp.path());

        assert_eq!(plan.download_size_bytes(), 800);
    }

    #[test]
    fn empty_manifest_with_no_local_files_is_up_to_date() {
        let tmp = TempDir::new().unwrap();
        let manifest = make_manifest(vec![]);

        let plan = ManifestComparator::compare(&manifest, &LocalState::default(), &[], tmp.path());

        assert!(plan.is_up_to_date());
    }
}
