use std::fs;
use crate::core::manifest::build_manifest::BuildManifest;
use super::cache_paths::CachePaths;
use tracing::{info, warn};

pub struct ManifestCache;

impl ManifestCache {
    pub fn save(paths: &CachePaths, manifest: &BuildManifest) -> Result<(), String> {
        let file_path = paths.manifest_file(&manifest.build.id);
        
        let json = serde_json::to_string_pretty(manifest)
            .map_err(|e| format!("Failed to serialize manifest: {}", e))?;
            
        fs::write(&file_path, json)
            .map_err(|e| format!("Failed to write manifest to cache: {}", e))?;
            
        info!("Saved manifest to cache: {:?}", file_path);
        Ok(())
    }

    pub fn load(paths: &CachePaths, version_id: &str) -> Result<BuildManifest, String> {
        let file_path = paths.manifest_file(version_id);
        
        if !file_path.exists() {
            return Err("Manifest cache not found".into());
        }

        let json = fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to read manifest cache: {}", e))?;
            
        let manifest: BuildManifest = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to deserialize manifest cache: {}", e))?;
            
        Ok(manifest)
    }
}
