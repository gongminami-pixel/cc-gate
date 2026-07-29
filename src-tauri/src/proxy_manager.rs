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

    // Windows: hermes node (CC-Gate's bundled runtime on Windows)
    #[cfg(windows)]
    {
        if let Some(h) = &home {
            let hermes_node = h.join("AppData").join("Local").join("hermes").join("node").join("node.exe");
            if hermes_node.exists() {
                let bin = hermes_node.parent().unwrap().to_path_buf();
                return (hermes_node, bin);
            }
            // Also check Local\node\node.exe (nvm-windows style)
            let nvmw_node = h.join("AppData").join("Local").join("node").join("node.exe");
            if nvmw_node.exists() {
                let bin = nvmw_node.parent().unwrap().to_path_buf();
                return (nvmw_node, bin);
            }
        }
        // %ProgramFiles%\nodejs\node.exe
        let pf = PathBuf::from(r"C:\Program Files\nodejs\node.exe");
        if pf.exists() {
            return (pf, PathBuf::from(r"C:\Program Files\nodejs"));
        }
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

    // Last resort: ask the OS where node is (cmd /c where on Windows, which on Unix)
    #[cfg(windows)]
    {
        if let Ok(out) = std::process::Command::new("cmd").args(["/c", "where node"]).output() {
            let first = String::from_utf8_lossy(&out.stdout)
                .lines().next().map(|l| l.trim().to_string()).unwrap_or_default();
            let p = PathBuf::from(&first);
            if p.exists() {
                let bin = p.parent().unwrap_or(&p).to_path_buf();
                tracing::info!("find_node: found via cmd /c where: {}", p.display());
                return (p, bin);
            }
        }
    }
    #[cfg(not(windows))]
    {
        if let Ok(out) = std::process::Command::new("which").arg("node").output() {
            let first = String::from_utf8_lossy(&out.stdout)
                .lines().next().map(|l| l.trim().to_string()).unwrap_or_default();
            let p = PathBuf::from(&first);
            if p.exists() {
                let bin = p.parent().unwrap_or(&p).to_path_buf();
                return (p, bin);
            }
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
        // Write providers.json + .env before starting the proxies.
        // Both must be rewritten together: providers.json names the env var holding
        // each relay key, so refreshing one alone leaves the proxy looking up a key
        // name that .env doesn't define (empty apiKey → upstream 401).
        if let Ok(cfg) = config_store::load() {
            if let Err(e) = config_writer::write_providers(&cfg) {
                tracing::warn!("write_providers before proxy start failed: {e}");
            }
            if let Err(e) = config_writer::write_user_api_keys(&cfg) {
                tracing::warn!("write_user_api_keys before proxy start failed: {e}");
            }
        }

        for name in &["mimo2codex", "claude-proxy", "chat-proxy"] {
            let (port, script) = self.proxy_script_for(name);
            if port_is_listening(port) {
                tracing::info!("Proxy {} port {} already listening — skipping startup", name, port);
                continue;
            }

            // One attempt with 30s timeout — npx may download packages on first run
            match tokio::time::timeout(Duration::from_secs(30), self.start(name, port, &script)).await {
                Ok(Ok(s)) => {
                    tracing::info!("Proxy {} started on port {}", name, s.port);
                }
                Ok(Err(e)) => {
                    tracing::error!("Proxy {} failed to start: {e}", name);
                }
                Err(_) => {
                    tracing::error!("Proxy {} start timed out after 15s", name);
                }
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
                (8689, p.to_string_lossy().to_string())
            }
            "chat-proxy" => {
                let p = crate::paths::mimo2codex_dir().join("chat-proxy.js");
                (8690, p.to_string_lossy().to_string())
            }
            _ => (0, String::new()),
        }
    }

    pub async fn start(&self, name: &str, port: u16, script: &str) -> Result<ProxyStatus> {
        self.stop(name).await.ok();
        kill_port_occupant(port);
        tokio::time::sleep(Duration::from_millis(300)).await;

        let is_mimo = name == "mimo2codex";
        tracing::info!(
            "Starting proxy {} on port {} (script={})",
            name, port, script,
        );

        let mut child = if is_mimo {
            // mimo2codex: use our resolved node binary to run the script.
            // Never run the script directly (#!/usr/bin/env node fails when
            // the GUI app doesn't inherit the user's shell PATH).
            let global = self.bin_dir.join("mimo2codex");
            #[cfg(windows)] let global = global.with_extension("cmd");
            if global.exists() {
                let mut c = Command::new(&self.node_path);
                c.arg(&global).args(["--port", &port.to_string()]).kill_on_drop(true);
                c.stderr(std::process::Stdio::piped());
                crate::win_console::hide_console_async(&mut c);
                c.spawn().map_err(|e| AppError::Proxy(format!("{name}: {e}")))?
            } else {
                #[cfg(windows)] let npx = "npx.cmd";
                #[cfg(not(windows))] let npx = "npx";
                let mut c = Command::new(npx);
                c.args(["-y", "mimo2codex", "--port", &port.to_string()]).kill_on_drop(true);
                c.stderr(std::process::Stdio::piped());
                crate::win_console::hide_console_async(&mut c);
                c.spawn().map_err(|e| AppError::Proxy(format!("{name}: {e}")))?
            }
        } else {
            // claude-proxy / chat-proxy: node <script> --port <port>
            let mut c = Command::new(&self.node_path);
            c.arg(script).args(["--port", &port.to_string()]).kill_on_drop(true);
            c.stderr(std::process::Stdio::piped());
            crate::win_console::hide_console_async(&mut c);
            c.spawn().map_err(|e| AppError::Proxy(format!("{name}: {e}")))?
        };

        // Forward the proxy's stderr into the app log. Without this the proxies'
        // routing diagnostics ("→ model → provider", upstream error bodies) go
        // nowhere the user can reach, so a misroute can only be guessed at.
        if let Some(err) = child.stderr.take() {
            let tag = name.to_string();
            tokio::spawn(async move {
                use tokio::io::{AsyncBufReadExt as _, BufReader};
                let mut lines = BufReader::new(err).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    tracing::info!(target: "proxy", "[{tag}] {line}");
                }
            });
        }

        let pid = child.id();
        tokio::time::sleep(Duration::from_millis(500)).await;

        match child.try_wait() {
            Ok(Some(status)) => {
                tracing::error!("Proxy {} exited immediately with {status}", name);
                return Err(AppError::Proxy(format!(
                    "{name} exited immediately (status {status}). Check dependencies."
                )));
            }
            Ok(None) => {
                if !port_is_listening(port) {
                    tracing::warn!("Proxy {} PID {:?} port {} not yet listening", name, pid, port);
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
