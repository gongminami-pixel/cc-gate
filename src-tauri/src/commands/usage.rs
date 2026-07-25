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
