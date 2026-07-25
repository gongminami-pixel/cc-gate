//! Manage 3 Node.js proxy child processes.
//!
//! Lifecycle: proxies live and die with the GUI app.
//! - mimo2codex (:8688) — Responses API → Chat Completions
//! - claude-proxy (:8689) — Anthropic Messages → Chat Completions
//! - chat-proxy (:8690) — Chat Completions passthrough

use std::collections::HashMap;
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

use crate::config_store;
use crate::error::{AppError, Result};
use crate::types::ProxyStatus;

pub struct ProxyManager {
    children: Mutex<HashMap<String, Child>>,
}

impl ProxyManager {
    pub fn new() -> Self {
        Self {
            children: Mutex::new(HashMap::new()),
        }
    }

    /// Start proxies that are enabled in the config.
    pub async fn start_enabled(&self) -> Result<()> {
        let cfg = config_store::load()?;
        if cfg.autostart_mimo2codex {
            let _ = self.start("mimo2codex", cfg.proxy_ports.mimo2codex, "mimo2codex").await;
        }
        if cfg.autostart_claude_proxy {
            let claude_proxy_path = crate::paths::mimo2codex_dir().join("claude-proxy.js");
            let script = if claude_proxy_path.exists() {
                claude_proxy_path.to_string_lossy().to_string()
            } else {
                // Fall back to the version bundled with CC-Gate
                "node".to_string()
            };
            let _ = self.start("claude-proxy", cfg.proxy_ports.claude_proxy, &script).await;
        }
        if cfg.autostart_chat_proxy {
            let chat_proxy_path = crate::paths::mimo2codex_dir().join("chat-proxy.js");
            let script = if chat_proxy_path.exists() {
                chat_proxy_path.to_string_lossy().to_string()
            } else {
                "node".to_string()
            };
            let _ = self.start("chat-proxy", cfg.proxy_ports.chat_proxy, &script).await;
        }
        Ok(())
    }

    pub async fn start(&self, name: &str, port: u16, script: &str) -> Result<ProxyStatus> {
        // Kill existing process with same name first
        self.stop(name).await.ok();

        tracing::info!("Starting proxy {} on port {} (script: {})", name, port, script);

        let child = Command::new("node")
            .arg(script)
            .arg("--port")
            .arg(port.to_string())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| AppError::Proxy(format!("failed to start {}: {}", name, e)))?;

        let pid = child.id();
        let mut children = self.children.lock().await;
        children.insert(name.to_string(), child);

        Ok(ProxyStatus {
            name: name.to_string(),
            port,
            running: true,
            pid,
            script: script.to_string(),
        })
    }

    pub async fn stop(&self, name: &str) -> Result<ProxyStatus> {
        let mut children = self.children.lock().await;
        if let Some(mut child) = children.remove(name) {
            tracing::info!("Stopping proxy {}", name);
            let _ = child.kill().await;
            let _ = child.wait().await;
        }

        Ok(ProxyStatus {
            name: name.to_string(),
            port: 0,
            running: false,
            pid: None,
            script: String::new(),
        })
    }

    pub async fn restart(&self, name: &str, port: u16, script: &str) -> Result<ProxyStatus> {
        self.stop(name).await.ok();
        self.start(name, port, script).await
    }

    pub async fn status(&self, name: &str, port: u16, script: &str) -> ProxyStatus {
        let children = self.children.lock().await;
        if let Some(child) = children.get(name) {
            ProxyStatus {
                name: name.to_string(),
                port,
                running: true,
                pid: child.id(),
                script: script.to_string(),
            }
        } else {
            ProxyStatus {
                name: name.to_string(),
                port,
                running: false,
                pid: None,
                script: script.to_string(),
            }
        }
    }

    pub async fn status_all(&self) -> Vec<ProxyStatus> {
        let cfg = match config_store::load() {
            Ok(c) => c,
            Err(_) => return vec![],
        };
        let claude_proxy_path = crate::paths::mimo2codex_dir().join("claude-proxy.js");
        let claude_script = if claude_proxy_path.exists() {
            claude_proxy_path.to_string_lossy().to_string()
        } else {
            "node".to_string()
        };
        let chat_proxy_path = crate::paths::mimo2codex_dir().join("chat-proxy.js");
        let chat_script = if chat_proxy_path.exists() {
            chat_proxy_path.to_string_lossy().to_string()
        } else {
            "node".to_string()
        };

        vec![
            self.status("mimo2codex", cfg.proxy_ports.mimo2codex, "mimo2codex").await,
            self.status("claude-proxy", cfg.proxy_ports.claude_proxy, &claude_script).await,
            self.status("chat-proxy", cfg.proxy_ports.chat_proxy, &chat_script).await,
        ]
    }

    pub async fn shutdown_all(&self) {
        let mut children = self.children.lock().await;
        for (name, mut child) in children.drain() {
            tracing::info!("Shutting down proxy {}", name);
            let _ = child.kill().await;
            let _ = child.wait().await;
        }
    }
}
