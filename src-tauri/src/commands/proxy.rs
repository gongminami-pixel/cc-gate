use tauri::State;
use std::sync::Arc;

use crate::error::{AppError, Result};
use crate::proxy_manager::ProxyManager;
use crate::types::ProxyStatus;

fn proxy_script(name: &str) -> (u16, String) {
    match name {
        "mimo2codex" => (8688, "mimo2codex".into()),
        "claude-proxy" => {
            let p = crate::paths::mimo2codex_dir().join("claude-proxy.js");
            (8689, p.to_string_lossy().to_string())
        }
        "chat-proxy" => {
            let p = crate::paths::mimo2codex_dir().join("chat-proxy.js");
            (8690, p.to_string_lossy().to_string())
        }
        _ => (0, String::new()),
    }
}

#[tauri::command]
pub async fn start_proxy(
    proxy_mgr: State<'_, Arc<ProxyManager>>,
    name: String,
) -> Result<ProxyStatus> {
    let (port, script) = proxy_script(&name);
    if script.is_empty() {
        return Err(AppError::Proxy(format!("Unknown proxy: {}", name)));
    }
    proxy_mgr.start(&name, port, &script).await
}

#[tauri::command]
pub async fn stop_proxy(
    proxy_mgr: State<'_, Arc<ProxyManager>>,
    name: String,
) -> Result<ProxyStatus> {
    proxy_mgr.stop(&name).await
}

#[tauri::command]
pub async fn restart_proxy(
    proxy_mgr: State<'_, Arc<ProxyManager>>,
    name: String,
) -> Result<ProxyStatus> {
    let (port, script) = proxy_script(&name);
    if script.is_empty() {
        return Err(AppError::Proxy(format!("Unknown proxy: {}", name)));
    }
    proxy_mgr.restart(&name, port, &script).await
}
