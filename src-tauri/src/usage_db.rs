//! Usage statistics database (SQLite).
//!
//! Two data sources:
//! 1. usage.jsonl — proxies append one JSON line per request
//! 2. usage.db — Rust syncs JSONL into SQLite, provides query API
//!
//! WAL mode enabled for concurrent access between Rust and potential
//! future direct-write from proxies.

use std::fs;
use std::path::PathBuf;

use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::paths;

// ── JSONL record (written by proxies) ────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct UsageRecord {
    pub request_id: String,
    pub model: String,
    pub provider: String,       // deepseek, glm, qwen, qwen38, xiaomi
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
    pub proxy: String,          // mimo2codex | claude-proxy | chat-proxy
    pub timestamp: String,      // ISO 8601
}

// ── Aggregated query results ─────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct DailyUsage {
    pub date: String,
    pub total_tokens: u64,
    pub total_cost_usd: f64,
    pub request_count: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelUsage {
    pub model: String,
    pub display_name: String,
    pub total_tokens: u64,
    pub total_cost_usd: f64,
    pub request_count: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct UsageSummary {
    pub today_cost_usd: f64,
    pub month_cost_usd: f64,
    pub total_requests: u64,
    pub today_tokens: u64,
    pub models: Vec<ModelUsage>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BudgetLimit {
    pub model_id: String,
    pub daily_limit_usd: Option<f64>,
    pub monthly_limit_usd: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    pub id: u64,
    pub request_id: String,
    pub model: String,
    pub provider: String,
    pub proxy: String,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
    pub cost_usd: f64,
    pub created_at: String,
}

// ── Database path ────────────────────────────────────────────

fn db_path() -> PathBuf {
    paths::mimo2codex_dir().join("usage.db")
}

fn jsonl_path() -> PathBuf {
    paths::mimo2codex_dir().join("usage.jsonl")
}

// ── Schema init ──────────────────────────────────────────────

fn get_or_create_db() -> Result<Connection> {
    let db = Connection::open(db_path())?;
    db.execute_batch("PRAGMA journal_mode=WAL")?;

    db.execute_batch(r#"
        CREATE TABLE IF NOT EXISTS request_logs (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            request_id      TEXT NOT NULL,
            model           TEXT NOT NULL,
            provider        TEXT NOT NULL,
            proxy           TEXT NOT NULL DEFAULT '',
            prompt_tokens   INTEGER NOT NULL,
            completion_tokens INTEGER NOT NULL,
            total_tokens    INTEGER NOT NULL,
            cost_usd        REAL NOT NULL DEFAULT 0,
            created_at      TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS idx_logs_model ON request_logs(model);
        CREATE INDEX IF NOT EXISTS idx_logs_provider ON request_logs(provider);
        CREATE INDEX IF NOT EXISTS idx_logs_created ON request_logs(created_at);
        CREATE INDEX IF NOT EXISTS idx_logs_request_id ON request_logs(request_id);

        CREATE TABLE IF NOT EXISTS pricing (
            model_id              TEXT PRIMARY KEY,
            display_name          TEXT NOT NULL,
            input_price_per_1k    REAL NOT NULL,
            output_price_per_1k   REAL NOT NULL
        );

        CREATE TABLE IF NOT EXISTS budget_limits (
            model_id          TEXT PRIMARY KEY,
            daily_limit_usd   REAL,
            monthly_limit_usd REAL
        );
    "#)?;

    // Ensure pricing data is populated from built-in catalog
    let pricing = crate::types::builtin_models();
    for m in &pricing {
        db.execute(
            "INSERT OR IGNORE INTO pricing (model_id, display_name, input_price_per_1k, output_price_per_1k) VALUES (?1, ?2, ?3, ?4)",
            params![m.slug, m.display_name, m.input_price_per_1k, m.output_price_per_1k],
        )?;
    }

    Ok(db)
}

// ── Import JSONL into SQLite ─────────────────────────────────

pub fn import_jsonl() -> Result<usize> {
    let jl_path = jsonl_path();
    if !jl_path.exists() { return Ok(0); }

    let content = fs::read_to_string(&jl_path)?;
    let db = get_or_create_db()?;

    // Get pricing for cost calculation
    let mut pricing: std::collections::HashMap<String, (f64, f64)> = std::collections::HashMap::new();
    let mut stmt = db.prepare("SELECT model_id, input_price_per_1k, output_price_per_1k FROM pricing")?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?, row.get::<_, f64>(2)?))
    })?;
    for row in rows {
        if let Ok((id, inp, out)) = row {
            pricing.insert(id, (inp, out));
        }
    }

    let mut imported = 0;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }

        let record: UsageRecord = match serde_json::from_str(line) {
            Ok(r) => r,
            Err(_) => continue,
        };

        // Skip duplicates
        let exists: bool = db.query_row(
            "SELECT COUNT(*) > 0 FROM request_logs WHERE request_id = ?1",
            params![record.request_id],
            |row| row.get(0),
        ).unwrap_or(false);

        if exists { continue; }

        // Calculate cost
        let cost_usd = if let Some((inp_price, out_price)) = pricing.get(&record.model) {
            (record.prompt_tokens as f64 / 1000.0 * inp_price)
                + (record.completion_tokens as f64 / 1000.0 * out_price)
        } else {
            0.0
        };

        db.execute(
            "INSERT INTO request_logs (request_id, model, provider, proxy, prompt_tokens, completion_tokens, total_tokens, cost_usd, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                record.request_id,
                record.model,
                record.provider,
                record.proxy,
                record.prompt_tokens,
                record.completion_tokens,
                record.total_tokens,
                cost_usd,
                record.timestamp,
            ],
        )?;
        imported += 1;
    }

    Ok(imported)
}

// ── Query APIs ───────────────────────────────────────────────

pub fn get_summary() -> Result<UsageSummary> {
    // Ensure latest JSONL is imported
    import_jsonl()?;

    let db = get_or_create_db()?;
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let this_month = chrono::Local::now().format("%Y-%m").to_string();

    let today_cost: f64 = db.query_row(
        "SELECT COALESCE(SUM(cost_usd), 0) FROM request_logs WHERE date(created_at) = ?1",
        params![today],
        |row| row.get(0),
    ).unwrap_or(0.0);

    let month_cost: f64 = db.query_row(
        "SELECT COALESCE(SUM(cost_usd), 0) FROM request_logs WHERE strftime('%Y-%m', created_at) = ?1",
        params![this_month],
        |row| row.get(0),
    ).unwrap_or(0.0);

    let total_requests: u64 = db.query_row(
        "SELECT COUNT(*) FROM request_logs",
        [],
        |row| row.get(0),
    ).unwrap_or(0);

    let today_tokens: u64 = db.query_row(
        "SELECT COALESCE(SUM(total_tokens), 0) FROM request_logs WHERE date(created_at) = ?1",
        params![today],
        |row| row.get(0),
    ).unwrap_or(0);

    // Per-model breakdown
    let mut model_stmt = db.prepare(
        "SELECT l.model, COALESCE(p.display_name, l.model), SUM(l.total_tokens), SUM(l.cost_usd), COUNT(*)
         FROM request_logs l LEFT JOIN pricing p ON l.model = p.model_id
         GROUP BY l.model ORDER BY SUM(l.cost_usd) DESC"
    )?;
    let models: Vec<ModelUsage> = model_stmt.query_map([], |row| {
        Ok(ModelUsage {
            model: row.get(0)?,
            display_name: row.get(1)?,
            total_tokens: row.get(2)?,
            total_cost_usd: row.get(3)?,
            request_count: row.get(4)?,
        })
    })?.filter_map(|r| r.ok()).collect();

    Ok(UsageSummary {
        today_cost_usd: today_cost,
        month_cost_usd: month_cost,
        total_requests,
        today_tokens,
        models,
    })
}

pub fn get_daily_usage(days: u32) -> Result<Vec<DailyUsage>> {
    import_jsonl()?;
    let db = get_or_create_db()?;

    let mut stmt = db.prepare(
        "SELECT date(created_at), SUM(total_tokens), SUM(cost_usd), COUNT(*)
         FROM request_logs
         WHERE created_at >= datetime('now', ?1)
         GROUP BY date(created_at)
         ORDER BY date(created_at) DESC"
    )?;

    let rows = stmt.query_map(params![format!("-{} days", days)], |row| {
        Ok(DailyUsage {
            date: row.get::<_, String>(0)?,
            total_tokens: row.get::<_, u64>(1)?,
            total_cost_usd: row.get::<_, f64>(2)?,
            request_count: row.get::<_, u64>(3)?,
        })
    })?;

    Ok(rows.filter_map(|r| r.ok()).collect())
}

#[derive(Debug, Clone, Serialize)]
pub struct PerModelSlot {
    pub model: String,
    pub display_name: String,
    pub tokens: u64,
    pub cost_usd: f64,
    pub requests: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PerModelUsage {
    pub label: String,
    pub models: Vec<PerModelSlot>,
}

pub fn get_per_model_usage() -> Result<Vec<PerModelUsage>> {
    import_jsonl()?;
    let db = get_or_create_db()?;

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let now = chrono::Local::now();
    let weekday = now.format("%u").to_string().parse::<i64>().unwrap_or(1); // 1=Mon
    let monday_this = now - chrono::Duration::days(weekday - 1);
    let monday_last = monday_this - chrono::Duration::days(7);
    let monday_prev = monday_last - chrono::Duration::days(7);
    let first_this_month = now.format("%Y-%m-01").to_string();
    let first_last_month = if now.format("%m").to_string() == "01" {
        format!("{}-12-01", now.format("%Y").to_string().parse::<i32>().unwrap_or(2025) - 1)
    } else {
        let y = now.format("%Y").to_string();
        let m = now.format("%m").to_string().parse::<u32>().unwrap_or(1) - 1;
        format!("{y}-{m:02}-01")
    };

    let sql_today        = format!("date(created_at) = '{today}'");
    let sql_yesterday    = format!("date(created_at) = date('{today}', '-1 day')");
    let sql_day_before   = format!("date(created_at) = date('{today}', '-2 days')");
    let sql_this_week    = format!("date(created_at) >= '{}'", monday_this.format("%Y-%m-%d"));
    let sql_last_week    = format!("date(created_at) >= '{}' AND date(created_at) < '{}'", monday_last.format("%Y-%m-%d"), monday_this.format("%Y-%m-%d"));
    let sql_prev_week    = format!("date(created_at) >= '{}' AND date(created_at) < '{}'", monday_prev.format("%Y-%m-%d"), monday_last.format("%Y-%m-%d"));
    let sql_this_month   = format!("date(created_at) >= '{first_this_month}'");
    let sql_last_month   = format!("date(created_at) >= '{first_last_month}' AND date(created_at) < '{first_this_month}'");

    let buckets: Vec<(&str, &str)> = vec![
        ("今天",   &sql_today),
        ("昨天",   &sql_yesterday),
        ("前天",   &sql_day_before),
        ("本周",   &sql_this_week),
        ("上周",   &sql_last_week),
        ("上上周", &sql_prev_week),
        ("本月",   &sql_this_month),
        ("上月",   &sql_last_month),
    ];

    let mut result = Vec::new();
    for (label, where_clause) in &buckets {
        let sql = format!(
            "SELECT l.model, COALESCE(p.display_name, l.model), SUM(l.total_tokens), SUM(l.cost_usd), COUNT(*)
             FROM request_logs l LEFT JOIN pricing p ON l.model = p.model_id
             WHERE {}
             GROUP BY l.model ORDER BY SUM(l.cost_usd) DESC",
            where_clause
        );

        let rows: Vec<PerModelSlot> = db.prepare(&sql)?
            .query_map([], |row| {
                Ok(PerModelSlot {
                    model: row.get(0)?,
                    display_name: row.get::<_, String>(1).unwrap_or_default(),
                    tokens: row.get::<_, i64>(2).unwrap_or(0) as u64,
                    cost_usd: row.get::<_, f64>(3).unwrap_or(0.0),
                    requests: row.get::<_, i64>(4).unwrap_or(0) as u64,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        result.push(PerModelUsage { label: (*label).to_string(), models: rows });
    }

    Ok(result)
}

pub fn get_recent_logs(limit: u32) -> Result<Vec<LogEntry>> {
    import_jsonl()?;
    let db = get_or_create_db()?;

    let mut stmt = db.prepare(
        "SELECT id, request_id, model, provider, proxy, prompt_tokens, completion_tokens, total_tokens, cost_usd, created_at
         FROM request_logs ORDER BY id DESC LIMIT ?1"
    )?;

    let rows = stmt.query_map(params![limit], |row| {
        Ok(LogEntry {
            id: row.get(0)?,
            request_id: row.get(1)?,
            model: row.get(2)?,
            provider: row.get(3)?,
            proxy: row.get(4)?,
            prompt_tokens: row.get(5)?,
            completion_tokens: row.get(6)?,
            total_tokens: row.get(7)?,
            cost_usd: row.get(8)?,
            created_at: row.get(9)?,
        })
    })?;

    Ok(rows.filter_map(|r| r.ok()).collect())
}
