use crate::core::manifest::version_info::VersionIndex;
use crate::core::manifest::build_manifest::BuildManifest;
use super::download_error::DownloadError;
use reqwest::Client;

pub struct HttpClient;

impl HttpClient {
    pub async fn fetch_version_index(url: &str) -> Result<VersionIndex, DownloadError> {
        let client = Client::new();
        let res = client.get(url).send().await.map_err(|e| DownloadError::Network(e.to_string()))?;
        
        if !res.status().is_success() {
            return Err(DownloadError::Network(format!("Failed to fetch version index: HTTP {}", res.status())));
        }
        
        let index = res.json::<VersionIndex>().await.map_err(|e| DownloadError::InvalidData(e.to_string()))?;
        Ok(index)
    }

    pub async fn fetch_build_manifest(url: &str) -> Result<BuildManifest, DownloadError> {
        let client = Client::new();
        let res = client.get(url).send().await.map_err(|e| DownloadError::Network(e.to_string()))?;
        
        if !res.status().is_success() {
            return Err(DownloadError::Network(format!("Failed to fetch build manifest: HTTP {}", res.status())));
        }
        
        let manifest = res.json::<BuildManifest>().await.map_err(|e| DownloadError::InvalidData(e.to_string()))?;
        Ok(manifest)
    }
}
