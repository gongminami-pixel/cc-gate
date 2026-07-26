//! Manage the CC-Gate LaunchAgent that auto-starts the app at login.
//! All functions are macOS-only; non-macOS targets get stubs.

#[cfg(target_os = "macos")]
use std::fs;
#[cfg(target_os = "macos")]
use std::process::Command;

#[cfg(target_os = "macos")]
use crate::error::{AppError, Result};
#[cfg(target_os = "macos")]
use crate::paths;

#[cfg(not(target_os = "macos"))]
use crate::error::Result;

#[cfg(target_os = "macos")]
const LABEL: &str = "com.CC-Gate.app";

#[cfg(target_os = "macos")]
fn render_plist(executable: &str) -> String {
    format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyLists-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>{LABEL}</string>
  <key>ProgramArguments</key>
  <array>
    <string>{executable}</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>ProcessType</key>
  <string>Interactive</string>
  <key>KeepAlive</key>
  <false/>
</dict>
</plist>
"#)
}

#[cfg(target_os = "macos")]
fn current_uid() -> u32 {
    unsafe { libc::getuid() as u32 }
}

#[cfg(target_os = "macos")]
fn domain() -> String { format!("gui/{}", current_uid()) }

#[cfg(target_os = "macos")]
pub fn enable_autostart() -> Result<()> {
    let exe = std::env::current_exe()
        .map_err(|e| AppError::other(format!("current_exe: {e}")))?;
    let exe_str = exe.to_string_lossy().to_string();
    let plist = paths::app_launchagent_plist()?;
    if let Some(parent) = plist.parent() { fs::create_dir_all(parent)?; }
    fs::write(&plist, render_plist(&exe_str))?;

    let _ = Command::new("launchctl")
        .args(["bootout", &format!("{}/{LABEL}", domain())])
        .output();
    let out = Command::new("launchctl")
        .args(["bootstrap", &domain(), plist.to_string_lossy().as_ref()])
        .output()
        .map_err(|e| AppError::Launchctl(format!("spawn: {e}")))?;
    if !out.status.success() {
        return Err(AppError::Launchctl(format!(
            "bootstrap failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn disable_autostart() -> Result<()> {
    let _ = Command::new("launchctl")
        .args(["bootout", &format!("{}/{LABEL}", domain())])
        .output();
    let plist = paths::app_launchagent_plist()?;
    if plist.exists() { fs::remove_file(&plist)?; }
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn autostart_status() -> bool {
    let plist = match paths::app_launchagent_plist() { Ok(p) => p, Err(_) => return false };
    if !plist.exists() { return false; }
    let out = Command::new("launchctl")
        .args(["print", &format!("{}/{LABEL}", domain())])
        .output();
    matches!(out, Ok(o) if o.status.success())
}

// ── Non-macOS stubs ────────────────────────────────────

#[cfg(not(target_os = "macos"))]
pub fn enable_autostart() -> Result<()> {
    tracing::info!("autostart: not supported on this platform");
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn disable_autostart() -> Result<()> {
    tracing::info!("autostart: not supported on this platform");
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn autostart_status() -> bool { false }
