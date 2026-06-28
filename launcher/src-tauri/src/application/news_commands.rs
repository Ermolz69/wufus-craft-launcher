use std::time::Duration;

use reqwest::Client;
use tauri::{AppHandle, Manager, State};
use tracing::{info, warn};

use crate::core::news::{self, NewsItem};
use crate::core::settings::SettingsState;

/// Fetch the news feed.
///
/// Strategy:
/// 1. Load cached news from `{app_data}/cache/news.json` as an instant fallback.
/// 2. Try to fetch fresh news from the network (5-second timeout).
/// 3. On success: update the cache and return fresh items.
/// 4. On failure: return cached items silently (empty list if no cache exists).
///
/// Returns `Ok(Vec<NewsItem>)` — always succeeds; worst case is an empty list.
///
/// Tauri v2 requires async commands with `State<'_>` to return `Result`.
#[tauri::command]
pub async fn fetch_news(
    app: AppHandle,
    settings: State<'_, SettingsState>,
) -> Result<Vec<NewsItem>, String> {
    let manifest_url = settings.0.lock().unwrap().manifest_url.clone();

    let cache_dir = app
        .path()
        .app_data_dir()
        .map(|d| d.join("cache"))
        .unwrap_or_default();

    let cached = news::load_cache(&cache_dir);
    let url = news::news_url(&manifest_url);

    match fetch_from_network(&url).await {
        Ok(fresh) => {
            info!("News fetched: {} items from {url}", fresh.len());
            news::save_cache(&cache_dir, &fresh);
            Ok(fresh)
        }
        Err(e) => {
            warn!("News fetch failed ({e}); serving {} cached items", cached.len());
            Ok(cached)
        }
    }
}

async fn fetch_from_network(url: &str) -> Result<Vec<NewsItem>, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }

    resp.json::<Vec<NewsItem>>().await.map_err(|e| e.to_string())
}
