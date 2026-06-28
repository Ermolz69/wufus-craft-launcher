use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsItem {
    pub id: String,
    pub title: String,
    pub body: String,
    pub date: String,
    #[serde(default)]
    pub image_url: Option<String>,
    #[serde(default)]
    pub link_url: Option<String>,
}

const CACHE_FILE: &str = "news.json";
/// Maximum items to keep in cache (prevents unbounded growth).
const MAX_CACHED: usize = 20;

pub fn load_cache(cache_dir: &Path) -> Vec<NewsItem> {
    let path = cache_dir.join(CACHE_FILE);
    fs::read_to_string(&path)
        .ok()
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default()
}

pub fn save_cache(cache_dir: &Path, news: &[NewsItem]) {
    let path = cache_dir.join(CACHE_FILE);
    let items = &news[..news.len().min(MAX_CACHED)];
    if let Ok(json) = serde_json::to_string_pretty(items) {
        let _ = fs::create_dir_all(cache_dir);
        let _ = fs::write(&path, json);
    }
}

/// Derive the news feed URL from the manifest URL by replacing the last
/// path segment with "news.json".
///
/// Example: `https://cdn.example.com/v2/manifest.json`
///       →  `https://cdn.example.com/v2/news.json`
pub fn news_url(manifest_url: &str) -> String {
    match manifest_url.rfind('/') {
        Some(pos) => format!("{}/news.json", &manifest_url[..pos]),
        None => format!("{manifest_url}/news.json"),
    }
}
