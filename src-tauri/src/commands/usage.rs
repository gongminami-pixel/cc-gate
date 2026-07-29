use crate::error::Result;
use crate::usage_db;

#[tauri::command]
pub fn get_usage_summary() -> Result<usage_db::UsageSummary> {
    usage_db::get_summary()
}

#[tauri::command]
pub fn get_daily_usage(days: u32) -> Result<Vec<usage_db::DailyUsage>> {
    usage_db::get_daily_usage(days)
}

#[tauri::command]
pub fn get_recent_logs(limit: u32) -> Result<Vec<usage_db::LogEntry>> {
    usage_db::get_recent_logs(limit)
}

#[tauri::command]
pub fn get_per_model_usage() -> Result<Vec<usage_db::PerModelUsage>> {
    usage_db::get_per_model_usage()
}

#[tauri::command]
pub fn import_usage_data() -> Result<u32> {
    let n = usage_db::import_jsonl()?;
    Ok(n as u32)
}

/// Tail today's app log so users can copy proxy routing diagnostics out of the UI.
/// The proxies' stderr is forwarded here (see proxy_manager::start).
#[tauri::command]
pub fn get_app_log_tail(lines: u32) -> Result<String> {
    let dir = crate::paths::logs_dir()?;
    let mut newest: Option<(std::time::SystemTime, std::path::PathBuf)> = None;
    for entry in std::fs::read_dir(&dir)?.flatten() {
        let p = entry.path();
        if p.extension().and_then(|e| e.to_str()) != Some("log") { continue; }
        let mtime = entry.metadata().and_then(|m| m.modified()).ok();
        if let Some(t) = mtime {
            if newest.as_ref().map_or(true, |(bt, _)| t > *bt) {
                newest = Some((t, p));
            }
        }
    }
    let Some((_, path)) = newest else {
        return Ok(format!("(no log file yet in {})", dir.display()));
    };
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    let all: Vec<&str> = content.lines().collect();
    let start = all.len().saturating_sub(lines as usize);
    Ok(format!("{}\n\n{}", path.display(), all[start..].join("\n")))
}

/// Version from tauri.conf.json — the value the bundle actually shipped with.
/// Returned from Rust so the UI can never drift from the built artifact.
#[tauri::command]
pub fn get_app_version(app: tauri::AppHandle) -> String {
    app.package_info().version.to_string()
}

/// Copy text to the clipboard. Done in Rust because the frontend clipboard API
/// needs an ACL capability this app doesn't define; the Rust side is unrestricted.
#[tauri::command]
pub fn copy_to_clipboard(app: tauri::AppHandle, text: String) -> Result<()> {
    use tauri_plugin_clipboard_manager::ClipboardExt as _;
    app.clipboard()
        .write_text(text)
        .map_err(|e| crate::error::AppError::Config(format!("clipboard: {e}")))
}
