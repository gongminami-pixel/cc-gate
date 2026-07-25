//! Manage the CC-Gate LaunchAgent that auto-starts the app at login.

use std::fs;
use std::process::Command;

use crate::error::{AppError, Result};
use crate::paths;

const LABEL: &str = "com.CC-Gate.app";

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

fn current_uid() -> u32 {
    unsafe { libc::getuid() as u32 }
}

fn domain() -> String { format!("gui/{}", current_uid()) }

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

pub fn disable_autostart() -> Result<()> {
    let _ = Command::new("launchctl")
        .args(["bootout", &format!("{}/{LABEL}", domain())])
        .output();
    let plist = paths::app_launchagent_plist()?;
    if plist.exists() { fs::remove_file(&plist)?; }
    Ok(())
}

pub fn autostart_status() -> bool {
    let plist = match paths::app_launchagent_plist() { Ok(p) => p, Err(_) => return false };
    if !plist.exists() { return false; }
    let out = Command::new("launchctl")
        .args(["print", &format!("{}/{LABEL}", domain())])
        .output();
    matches!(out, Ok(o) if o.status.success())
}
