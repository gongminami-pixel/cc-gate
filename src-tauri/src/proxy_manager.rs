//! Manage 3 Node.js proxy child processes.
//!
//! Lifecycle: proxies live and die with the GUI app.
//! - mimo2codex (:8688) — Responses API → Chat Completions
//! - claude-proxy (:8689) — Anthropic Messages → Chat Completions
//! - chat-proxy (:8690) — Chat Completions passthrough

use std::collections::HashMap;
use std::net::TcpStream;
use std::path::PathBuf;
use std::time::Duration;
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

use crate::config_store;
use crate::config_writer;
use crate::error::{AppError, Result};
use crate::types::ProxyStatus;

/// Resolve `node` at runtime because macOS GUI apps don't inherit the
/// user's shell PATH (nvm / Homebrew node won't be found by bare `node`).
///
/// Strategy: prefer an nvm version that also has `mimo2codex` installed,
/// since that's the proxy that requires a globally-installed Node script.
fn find_node() -> (PathBuf, PathBuf) {
    let home = dirs::home_dir();

    // Helper: scan nvm versions, return (node_path, bin_dir) for the best match
    let nvm_node_bin = home.as_ref().and_then(|h| {
        let nvm_root = h.join(".nvm").join("versions").join("node");
        if !nvm_root.exists() { return None; }

        let mut versions: Vec<PathBuf> = std::fs::read_dir(&nvm_root)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
        versions.sort();

        // 1st pass: pick a version that has BOTH node AND mimo2codex
        for v in versions.iter().rev() {
            let bin = v.join("bin");
            let node = bin.join("node");
            let mimo = bin.join("mimo2codex");
            if node.exists() && mimo.exists() {
                return Some((node, bin));
            }
        }
        // 2nd pass: any version with node (mimo2codex is at least a node script, will try to run it)
        for v in versions.iter().rev() {
            let bin = v.join("bin");
            let node = bin.join("node");
            if node.exists() {
                return Some((node, bin));
            }
        }
        None
    });

    if let Some(pair) = nvm_node_bin {
        return pair;
    }

    // Homebrew / system fallbacks (unlikely to have mimo2codex, but better than nothing)
    let candidates: &[(&str, &str)] = &[
        ("/usr/local/bin", "/usr/local/bin"),
        ("/opt/homebrew/bin", "/opt/homebrew/bin"),
        ("/usr/bin", "/usr/bin"),
    ];
    for (node_p, bin_p) in candidates {
        let node = PathBuf::from(node_p).join("node");
        if node.exists() {
            return (node, PathBuf::from(bin_p));
        }
    }

    // fnm
    if let Some(h) = &home {
        let fnm_root = h.join(".fnm").join("node-versions");
        if fnm_root.is_dir() {
            let mut vers: Vec<PathBuf> = std::fs::read_dir(&fnm_root)
                .into_iter()
                .flatten()
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.is_dir())
                .collect();
            vers.sort();
            if let Some(v) = vers.last() {
                let bin = v.join("bin");
                return (bin.join("node"), bin);
            }
        }
        let volta = h.join(".volta").join("bin");
        let node = volta.join("node");
        if node.exists() {
            return (node, volta);
        }
    }

    (PathBuf::from("node"), PathBuf::from("/usr/local/bin"))
}

/// Kill whatever is listening on a TCP port (if anything).
fn kill_port_occupant(port: u16) {
    let port_s = port.to_string();
    if let Ok(out) = std::process::Command::new("lsof")
        .args(["-ti", &format!(":{}", port_s)])
        .output()
    {
        let pids = String::from_utf8_lossy(&out.stdout);
        for pid in pids.split_whitespace() {
            tracing::info!("Killing PID {} occupying port {}", pid, port);
            let _ = std::process::Command::new("kill")
                .arg("-9")
                .arg(pid)
                .output();
        }
    }
}

/// Check if something is listening on localhost:port (non-blocking TCP connect).
fn port_is_listening(port: u16) -> bool {
    TcpStream::connect_timeout(
        &format!("127.0.0.1:{}", port).parse().unwrap(),
        Duration::from_millis(200),
    )
    .is_ok()
}

pub struct ProxyManager {
    children: Mutex<HashMap<String, Child>>,
    node_path: PathBuf,
    bin_dir: PathBuf,
}

impl ProxyManager {
    pub fn new() -> Self {
        let (node, bin) = find_node();
        tracing::info!("ProxyManager: node={}, bin_dir={}", node.display(), bin.display());
        Self {
            children: Mutex::new(HashMap::new()),
            node_path: node,
            bin_dir: bin,
        }
    }

    /// Start all three proxies unconditionally on app launch.
    /// Retries up to 5 times with exponential backoff for each proxy.
    /// For mimo2codex, rewrites providers.json before each retry to self-heal.
    /// Does NOT kill existing proxies — if port is already listening, skip.
    pub async fn start_enabled(&self) -> Result<()> {
        // Write providers.json before starting mimo2codex
        if let Ok(cfg) = config_store::load() {
            if let Err(e) = config_writer::write_providers(&cfg) {
                tracing::warn!("write_providers before proxy start failed: {e}");
            }
        }

        for name in &["mimo2codex", "claude-proxy", "chat-proxy"] {
            let (port, script) = self.proxy_script_for(name);
            if port_is_listening(port) {
                tracing::info!("Proxy {} port {} already listening — skipping startup", name, port);
                continue;
            }

            // Retry loop: up to 5 attempts, 1s → 2s → 4s → 8s → 16s backoff
            let mut started = false;
            for attempt in 0..5 {
                match self.start(name, port, &script).await {
                    Ok(s) => {
                        tracing::info!("Proxy {} started on port {} (attempt {})", name, s.port, attempt + 1);
                        started = true;
                        break;
                    }
                    Err(e) => {
                        tracing::error!("{} start attempt {} failed: {e}", name, attempt + 1);
                        if attempt < 4 {
                            let delay_ms = (1 << attempt) * 1000; // 1s, 2s, 4s, 8s
                            tracing::info!("Retrying {} in {}ms...", name, delay_ms);
                            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;

                            // Rewrite providers.json before each mimo2codex retry (self-heal)
                            if *name == "mimo2codex" {
                                if let Ok(cfg) = config_store::load() {
                                    let _ = config_writer::write_providers(&cfg);
                                }
                            }
                        }
                    }
                }
            }

            if !started {
                tracing::error!("FATAL: Proxy {} failed to start after 5 attempts", name);
            }
        }
        Ok(())
    }

    /// Resolve the script path for a named proxy. Public so Tauri commands can use it.
    pub fn proxy_script_for(&self, name: &str) -> (u16, String) {
        match name {
            "mimo2codex" => {
                // Use bin_dir (same dir as node) for the mimo2codex global script
                let p = self.bin_dir.join("mimo2codex");
                (8688, p.to_string_lossy().to_string())
            }
            "claude-proxy" => {
                let p = crate::paths::mimo2codex_dir().join("claude-proxy.js");
                (8689, if p.exists() { p.to_string_lossy().to_string() } else { "claude-proxy.js".into() })
            }
            "chat-proxy" => {
                let p = crate::paths::mimo2codex_dir().join("chat-proxy.js");
                (8690, if p.exists() { p.to_string_lossy().to_string() } else { "chat-proxy.js".into() })
            }
            _ => (0, String::new()),
        }
    }

    pub async fn start(&self, name: &str, port: u16, script: &str) -> Result<ProxyStatus> {
        // Kill existing process with same name first
        self.stop(name).await.ok();

        // Also kill anything squatting on this port (e.g. leftover from previous CC-Gate crash)
        kill_port_occupant(port);
        // Brief pause to let the OS release the port
        tokio::time::sleep(Duration::from_millis(300)).await;

        tracing::info!(
            "Starting proxy {} on port {} (node={}, script={})",
            name, port,
            self.node_path.display(),
            script,
        );

        let mut child = Command::new(&self.node_path)
            .arg(script)
            .arg("--port")
            .arg(port.to_string())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| AppError::Proxy(format!("failed to start {name}: {e}")))?;

        let pid = child.id();

        // Give the process a moment to start listening
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Verify the child hasn't already crashed
        match child.try_wait() {
            Ok(Some(status)) => {
                tracing::error!(
                    "Proxy {} exited immediately with {status} — script path may be wrong or missing deps",
                    name
                );
                return Err(AppError::Proxy(format!(
                    "{name} exited immediately (status {status}). Check that the script exists and its dependencies are installed."
                )));
            }
            Ok(None) => {
                // Still running — good
                if !port_is_listening(port) {
                    tracing::warn!("Proxy {} spawned (PID {:?}) but port {} not yet listening", name, pid, port);
                }
            }
            Err(e) => {
                tracing::warn!("try_wait failed for {}: {e}", name);
            }
        }

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

    /// Check real liveness:
    /// 1. try_wait the Child handle → clean up dead ones
    /// 2. If not in our HashMap, check if port is still listening (could be orphan)
    pub async fn status(&self, name: &str, port: u16, script: &str) -> ProxyStatus {
        let mut children = self.children.lock().await;
        if let Some(child) = children.get_mut(name) {
            match child.try_wait() {
                Ok(Some(status)) => {
                    // Dead — clean up
                    tracing::warn!("Proxy {} was dead (exit {status}), removing from tracker", name);
                    children.remove(name);
                    // fall through to port check below
                }
                Ok(None) => {
                    // Still alive according to OS
                    return ProxyStatus {
                        name: name.to_string(),
                        port,
                        running: true,
                        pid: child.id(),
                        script: script.to_string(),
                    };
                }
                Err(_) => {
                    // Can't check — assume alive
                    return ProxyStatus {
                        name: name.to_string(),
                        port,
                        running: true,
                        pid: child.id(),
                        script: script.to_string(),
                    };
                }
            }
        }

        // Not in our HashMap (or was just removed). Check if port is still alive.
        if port_is_listening(port) {
            return ProxyStatus {
                name: name.to_string(),
                port,
                running: true,
                pid: None,
                script: script.to_string(),
            };
        }

        ProxyStatus {
            name: name.to_string(),
            port,
            running: false,
            pid: None,
            script: script.to_string(),
        }
    }

    pub async fn status_all(&self) -> Vec<ProxyStatus> {
        let mut out = Vec::with_capacity(3);
        for name in &["mimo2codex", "claude-proxy", "chat-proxy"] {
            let (port, script) = self.proxy_script_for(name);
            out.push(self.status(name, port, &script).await);
        }
        out
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
