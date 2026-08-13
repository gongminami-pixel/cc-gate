//! Remote model catalog — fetches latest model definitions from GitHub.
//! Local cache → remote fetch → merge into user config.

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::paths;
use crate::types::ModelDef;

/// Remote catalog JSON URL (raw GitHub — updated by maintainers when vendors release new models).
const CATALOG_URL: &str =
    "https://raw.githubusercontent.com/gongminami/cc-gate/main/models-catalog.json";

/// Deserialized from `models-catalog.json` hosted on GitHub.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteCatalog {
    pub version: u32,
    pub updated_at: String,
    pub models: Vec<ModelDef>,
}

/// Path to local cache file: `~/.mimo2codex/models-cache.json`
pub fn catalog_cache_path() -> PathBuf {
    paths::mimo2codex_dir().join("models-cache.json")
}

/// Read the cached remote catalog (if it exists).
pub fn read_catalog_cache() -> Option<RemoteCatalog> {
    let path = catalog_cache_path();
    if !path.exists() {
        return None;
    }
    let data = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&data)
        .inspect_err(|e| tracing::warn!("catalog cache parse failed: {e}"))
        .ok()
}

/// Fetch the remote catalog from GitHub.
pub async fn fetch_remote_catalog() -> Result<RemoteCatalog, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {e}"))?;

    let resp = client
        .get(CATALOG_URL)
        .send()
        .await
        .map_err(|e| format!("网络请求失败: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("服务器返回 {}", resp.status()));
    }

    let text = resp
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {e}"))?;

    serde_json::from_str(&text).map_err(|e| format!("模型 JSON 解析失败: {e}"))
}

/// Write catalog to local cache.
pub fn save_catalog_cache(catalog: &RemoteCatalog) {
    let path = catalog_cache_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(catalog) {
        if let Err(e) = fs::write(&path, json) {
            tracing::warn!("catalog cache write failed: {e}");
        } else {
            tracing::info!("catalog cache saved (version {})", catalog.version);
        }
    }
}

/// Merge remote catalog models into the existing user config model list.
///
/// | Scenario | Handling |
/// |---|---|
/// | Remote has new models (not in existing) | Append with remote's `enabled` value |
/// | Remote has existing models | Update params (context_window, pricing, etc.); **keep** user's `enabled` state |
/// | Models only in local | Preserve as-is (no deletion) |
///
/// Returns (new_model_count, slugs_of_new_models).
pub fn merge_remote_models(existing: &mut Vec<ModelDef>, remote: &[ModelDef]) -> (u32, Vec<String>) {
    let mut new_slugs: Vec<String> = Vec::new();

    for r in remote {
        if let Some(emu) = existing.iter_mut().find(|e| e.slug == r.slug) {
            // Refresh parametric fields from remote (keep user's enabled + priority)
            emu.display_name = r.display_name.clone();
            emu.provider = r.provider.clone();
            emu.context_window = r.context_window;
            emu.max_output_tokens = r.max_output_tokens;
            emu.default_reasoning_level = r.default_reasoning_level.clone();
            emu.supports_reasoning_summaries = r.supports_reasoning_summaries;
            emu.input_price_per_1k = r.input_price_per_1k;
            emu.output_price_per_1k = r.output_price_per_1k;
        } else {
            // New model — add with remote's default enabled state
            new_slugs.push(r.slug.clone());
            existing.push(r.clone());
        }
    }

    let count = new_slugs.len() as u32;
    if count > 0 {
        tracing::info!("catalog merge: {} new models ({})", count, new_slugs.join(", "));
    }
    (count, new_slugs)
}

/// Result returned by the `check_model_updates` Tauri command.
#[derive(Debug, Clone, Serialize)]
pub struct CheckUpdateResult {
    /// How many brand-new models were discovered.
    pub new_models: u32,
    /// Slugs of those new models (for frontend badge display).
    pub new_slugs: Vec<String>,
    /// Remote catalog version number.
    pub version: u32,
    /// ISO8601 timestamp from the remote catalog.
    pub updated_at: String,
}
