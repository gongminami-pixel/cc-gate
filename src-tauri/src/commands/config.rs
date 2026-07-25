use tauri::State;
use tauri::Manager;
use std::sync::Arc;

use crate::config_store;
use crate::config_writer;
use crate::error::{AppError, Result};
use crate::launchd;
use crate::proxy_manager::ProxyManager;
use crate::types::{AppConfig, AgentMeta, RelayConfig, ProxyStatus, agent_list, agent_id_key};

#[tauri::command] pub fn get_config() -> Result<AppConfig> { config_store::load() }
#[tauri::command] pub fn save_config(cfg: AppConfig) -> Result<()> { config_store::save(&cfg) }
#[tauri::command] pub fn get_agent_list() -> Vec<AgentMeta> { agent_list() }

#[derive(serde::Serialize)]
pub struct ShellInfo {
    pub config_file: String,
    pub reload_cmd: String,
    pub platform_os: String,
}
#[tauri::command] pub fn get_shell_info() -> ShellInfo {
    ShellInfo {
        config_file: crate::paths::shell_description().to_string(),
        reload_cmd: crate::paths::shell_reload_cmd().to_string(),
        platform_os: std::env::consts::OS.to_string(),
    }
}

#[tauri::command]
pub async fn apply_agent_config(
    proxy_mgr: State<'_, Arc<ProxyManager>>,
    cfg: AppConfig,
) -> Result<ApplyResult> {
    config_store::save(&cfg)?;
    config_writer::write_all_tool_configs(&cfg)?;

    let mut restarted: Vec<String> = vec![];
    let needs_mimo = agent_list().iter().filter(|a| a.proxy == "mimo2codex").any(|a| cfg.agent_models.contains_key(&agent_id_key(&a.id)));
    let needs_claude = agent_list().iter().filter(|a| a.proxy == "claude-proxy").any(|a| cfg.agent_models.contains_key(&agent_id_key(&a.id)));
    let needs_chat = agent_list().iter().filter(|a| a.proxy == "chat-proxy").any(|a| cfg.agent_models.contains_key(&agent_id_key(&a.id)));

    if needs_mimo && cfg.autostart_mimo2codex {
        if let Ok(s) = proxy_mgr.restart("mimo2codex", 8688, "mimo2codex").await { if s.running { restarted.push("mimo2codex".into()); } }
    }
    if needs_claude && cfg.autostart_claude_proxy {
        let script = crate::paths::mimo2codex_dir().join("claude-proxy.js").to_string_lossy().to_string();
        if let Ok(s) = proxy_mgr.restart("claude-proxy", 8689, &script).await { if s.running { restarted.push("claude-proxy".into()); } }
    }
    if needs_chat && cfg.autostart_chat_proxy {
        let script = crate::paths::mimo2codex_dir().join("chat-proxy.js").to_string_lossy().to_string();
        if let Ok(s) = proxy_mgr.restart("chat-proxy", 8690, &script).await { if s.running { restarted.push("chat-proxy".into()); } }
    }

    Ok(ApplyResult { success: true, message: "配置已应用".into(), restarted_proxies: restarted })
}

#[derive(serde::Serialize, Clone)]
pub struct ApplyResult { pub success: bool, pub message: String, pub restarted_proxies: Vec<String> }

// ── Relay CRUD ─────────────────────────────────────────────

#[tauri::command]
pub fn add_relay(mut cfg: AppConfig, name: String, url: String, key: String) -> Result<AppConfig> {
    if cfg.relays.iter().any(|r| r.name == name) {
        return Err(AppError::Config(format!("中转站 '{}' 已存在", name)));
    }
    cfg.relays.push(RelayConfig { name, url, key });
    config_store::save(&cfg)?;
    Ok(cfg)
}

#[tauri::command]
pub fn update_relay(mut cfg: AppConfig, old_name: String, name: String, url: String, key: String) -> Result<AppConfig> {
    if let Some(r) = cfg.relays.iter_mut().find(|r| r.name == old_name) {
        r.name = name; r.url = url; r.key = key;
    }
    config_store::save(&cfg)?;
    Ok(cfg)
}

#[tauri::command]
pub fn delete_relay(mut cfg: AppConfig, name: String) -> Result<AppConfig> {
    cfg.relays.retain(|r| r.name != name);
    // Also clean up model_routing entries pointing to this relay
    let target = format!("relay:{}", name);
    for (_, routing) in cfg.model_routing.iter_mut() {
        if *routing == target { *routing = "direct".into(); }
    }
    config_store::save(&cfg)?;
    Ok(cfg)
}

// ── Legacy / proxy ─────────────────────────────────────────

#[tauri::command] pub fn write_tool_configs(cfg: AppConfig) -> Result<String> {
    config_store::save(&cfg)?; config_writer::write_all_tool_configs(&cfg)?;
    Ok("All configs written".into())
}
#[tauri::command] pub fn get_proxy_status(proxy_mgr: State<'_, Arc<ProxyManager>>) -> Vec<ProxyStatus> {
    let mgr = proxy_mgr.inner().clone();
    let (tx, rx) = std::sync::mpsc::channel();
    tauri::async_runtime::spawn(async move { let _ = tx.send(mgr.status_all().await); });
    rx.recv().unwrap_or_default()
}
#[tauri::command] pub fn get_app_autostart_status() -> serde_json::Value { serde_json::json!({ "enabled": launchd::autostart_status() }) }
#[tauri::command] pub fn set_app_autostart(enabled: bool) -> Result<serde_json::Value> {
    if enabled { launchd::enable_autostart()?; } else { launchd::disable_autostart()?; }
    Ok(serde_json::json!({ "enabled": launchd::autostart_status() }))
}
#[tauri::command] pub fn quit_app(app: tauri::AppHandle) { app.exit(0); }
#[tauri::command] pub fn hide_main_window(app: tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("main") { let _ = w.hide(); }
}
