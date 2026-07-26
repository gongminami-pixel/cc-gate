//! Detect installed CLI tools and offer to install missing ones.

use std::process::Command;
use std::sync::OnceLock;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolStatus {
    pub name: String,       // display name
    pub command: String,    // binary to check, e.g. "codex", "node"
    pub installed: bool,
    pub version: Option<String>,
    pub install_cmd: String, // one-liner to install
    pub link: String,       // documentation link
    pub category: String,   // "runtime" | "tool"
}

static CACHE: OnceLock<std::sync::Mutex<Vec<ToolStatus>>> = OnceLock::new();

fn cache() -> &'static std::sync::Mutex<Vec<ToolStatus>> {
    CACHE.get_or_init(|| std::sync::Mutex::new(check_now()))
}

/// 首次调用时缓存结果；后续返回缓存。进程退出前工具不会变。
pub fn check_all() -> Vec<ToolStatus> {
    cache().lock().unwrap().clone()
}

/// 强制重新检测（用户点了刷新按钮）
pub fn refresh() -> Vec<ToolStatus> {
    let mut guard = cache().lock().unwrap();
    *guard = check_now();
    guard.clone()
}

/// 渐进式检测：逐条执行检测并通过回调即时返回结果，最后更新缓存。
/// 供前端 streaming command 使用——每个工具检测完就 emit 事件，用户不卡等。
/// 将检测结果存入缓存（渐进式检测完成后调用）
pub fn save_to_cache(results: Vec<ToolStatus>) {
    let mut guard = cache().lock().unwrap();
    *guard = results;
}

pub fn check_one(name: &str) -> Option<ToolStatus> {
    match name {
        "node" => Some(check_node_npm()),
        "python3" => Some(check_python()),
        "codex" => Some(check_codex()),
        "claude" => Some(check_claude_code()),
        "aider" => Some(check_aider()),
        "bash" => Some(check_git_bash()),
        _ => None,
    }
}

fn check_now() -> Vec<ToolStatus> {
    vec![
        check_node_npm(),
        check_python(),
        check_codex(),
        check_claude_code(),
        check_aider(),
        check_git_bash(),
    ]
}

#[cfg(not(target_os = "windows"))]
fn run_version(cmd: &str, args: &[&str]) -> Option<String> {
    // GUI apps don't inherit user PATH (set in .zshrc). Source it explicitly,
    // then run the command. Check exit code so "command not found"
    // error messages aren't treated as version strings.
    let arg_str = std::iter::once(cmd).chain(args.iter().copied()).collect::<Vec<_>>().join(" ");
    Command::new("/bin/zsh")
        .args(["-c", &format!("source ~/.zshrc 2>/dev/null; {arg_str} 2>&1")])
        .output()
        .ok()
        .and_then(|o| {
            if !o.status.success() { return None; }
            let raw = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if raw.is_empty() { return None; }
            raw.lines().find(|l| !l.trim().is_empty()).map(|s| s.to_string())
        })
}

#[cfg(target_os = "windows")]
fn run_version(cmd: &str, args: &[&str]) -> Option<String> {
    Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .and_then(|o| {
            let mut s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if s.is_empty() {
                s = String::from_utf8_lossy(&o.stderr).trim().to_string();
            }
            if s.is_empty() { return None; }
            s.lines().find(|l| !l.trim().is_empty()).map(|l| l.to_string())
        })
}

fn check_node_npm() -> ToolStatus {
    let node = run_version("node", &["--version"]);
    let npm  = run_version("npm", &["--version"]);
    ToolStatus {
        name: "Node.js & npm".into(),
        command: "node".into(),
        installed: node.is_some() && npm.is_some(),
        version: node.map(|n| {
            let nv = npm.map(|p| format!(", npm {p}")).unwrap_or_default();
            // node --version already includes "v" prefix
            format!("{n}{nv}")
        }),
        install_cmd: "请从 https://nodejs.org 下载安装".into(),
        link: "https://nodejs.org".into(),
        category: "runtime".into(),
    }
}

fn check_python() -> ToolStatus {
    let ver = run_version("python3", &["--version"])
        .or_else(|| run_version("python", &["--version"]));
    ToolStatus {
        name: "Python 3".into(),
        command: "python3".into(),
        installed: ver.is_some(),
        version: ver,
        install_cmd: "请从 https://python.org 下载安装".into(),
        link: "https://python.org".into(),
        category: "runtime".into(),
    }
}

fn check_codex() -> ToolStatus {
    let ver = run_version("codex", &["--version"]);
    ToolStatus {
        name: "Codex CLI".into(),
        command: "codex".into(),
        installed: ver.is_some(),
        version: ver,
        install_cmd: "npm i -g @anthropic/codex".into(),
        link: "https://docs.anthropic.com/codex".into(),
        category: "tool".into(),
    }
}

fn check_claude_code() -> ToolStatus {
    let ver = run_version("claude", &["--version"]);
    ToolStatus {
        name: "Claude Code CLI".into(),
        command: "claude".into(),
        installed: ver.is_some(),
        version: ver,
        install_cmd: "npm i -g @anthropic/claude-code".into(),
        link: "https://docs.anthropic.com/claude-code".into(),
        category: "tool".into(),
    }
}

fn check_aider() -> ToolStatus {
    let ver = run_version("aider", &["--version"]);
    ToolStatus {
        name: "Aider".into(),
        command: "aider".into(),
        installed: ver.is_some(),
        version: ver,
        install_cmd: "pip install aider-chat".into(),
        link: "https://aider.chat".into(),
        category: "tool".into(),
    }
}

#[cfg(target_os = "windows")]
fn check_git_bash() -> ToolStatus {
    let found = which_git_bash();
    ToolStatus {
        name: "Git Bash".into(),
        command: "bash".into(),
        installed: found,
        version: None,
        install_cmd: "winget install Git.Git".into(),
        link: "https://git-scm.com/download/win".into(),
        category: "runtime".into(),
    }
}

#[cfg(not(target_os = "windows"))]
fn check_git_bash() -> ToolStatus {
    ToolStatus {
        name: "Shell".into(),
        command: "bash".into(),
        installed: true,
        version: Some("已内置".into()),
        install_cmd: "".into(),
        link: "".into(),
        category: "runtime".into(),
    }
}

#[cfg(target_os = "windows")]
fn which_git_bash() -> bool {
    let paths = [
        r"C:\Program Files\Git\bin\bash.exe",
        r"C:\Program Files (x86)\Git\bin\bash.exe",
    ];
    for p in &paths {
        if std::path::PathBuf::from(p).exists() { return true; }
    }
    false
}
