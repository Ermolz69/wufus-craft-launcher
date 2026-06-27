use crate::infrastructure::network::http_client::HttpClient;
use crate::core::manifest::version_info::VersionIndex;
use crate::application::error::LauncherError;

pub struct UpdateChecker;

impl UpdateChecker {
    pub async fn check_for_updates(index_url: &str) -> Result<VersionIndex, LauncherError> {
        let index = HttpClient::fetch_version_index(index_url).await
            .map_err(|e| LauncherError::SystemError(format!("Failed to check for updates: {}", e)))?;
            
        Ok(index)
    }
}
