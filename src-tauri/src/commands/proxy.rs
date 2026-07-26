use tauri::State;
use std::sync::Arc;

use crate::error::{AppError, Result};
use crate::proxy_manager::ProxyManager;
use crate::types::ProxyStatus;

#[tauri::command]
pub async fn start_proxy(
    proxy_mgr: State<'_, Arc<ProxyManager>>,
    name: String,
) -> Result<ProxyStatus> {
    let (port, script) = proxy_mgr.proxy_script_for(&name);
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
    let (port, script) = proxy_mgr.proxy_script_for(&name);
    if script.is_empty() {
        return Err(AppError::Proxy(format!("Unknown proxy: {}", name)));
    }
    proxy_mgr.restart(&name, port, &script).await
}
