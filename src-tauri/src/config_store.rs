use std::collections::HashSet;
use std::fs;

use crate::error::Result;
use crate::paths;
use crate::types::{AppConfig, agent_list, builtin_models, agent_id_key, all_api_key_names};

pub fn load() -> Result<AppConfig> {
    let p = paths::app_config_path();
    let mut cfg = if p.exists() {
        let s = fs::read_to_string(&p)?;
        serde_json::from_str::<AppConfig>(&s)?
    } else {
        AppConfig::default()
    };
    merge_builtin_models(&mut cfg);
    ensure_agent_models(&mut cfg);
    ensure_model_routing(&mut cfg);
    auto_fill_keys_from_env(&mut cfg);
    Ok(cfg)
}

/// Read existing values from .env and auto-fill api_keys for known env var names.
fn auto_fill_keys_from_env(cfg: &mut AppConfig) {
    let env_path = paths::mimo_env();
    if !env_path.exists() { return; }
    let content = match fs::read_to_string(&env_path) {
        Ok(s) => s,
        Err(_) => return,
    };

    let known: HashSet<&str> = all_api_key_names().iter().map(|(env, _, _)| *env).collect();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') { continue; }
        if let Some((key, val)) = trimmed.split_once('=') {
            let key = key.trim();
            let val = val.trim().trim_matches('"').trim_matches('\'');
            if known.contains(key) && !val.is_empty() {
                // Only auto-fill if user hasn't already set a value
                cfg.api_keys.entry(key.to_string()).or_insert_with(|| val.to_string());
            }
        }
    }
}

pub fn save(cfg: &AppConfig) -> Result<()> {
    let p = paths::app_config_path();
    if let Some(parent) = p.parent() { fs::create_dir_all(parent)?; }
    fs::write(&p, serde_json::to_string_pretty(cfg)?)?;
    Ok(())
}

fn merge_builtin_models(cfg: &mut AppConfig) {
    for builtin in builtin_models() {
        if !cfg.models.iter().any(|m| m.slug == builtin.slug) {
            cfg.models.push(builtin);
        }
    }
}

fn ensure_agent_models(cfg: &mut AppConfig) {
    let all_slugs: Vec<String> = cfg.models.iter().filter(|m| m.enabled).map(|m| m.slug.clone()).collect();
    if cfg.agent_models.is_empty() {
        for agent in agent_list() {
            cfg.agent_models.insert(agent_id_key(&agent.id), all_slugs.clone());
        }
    } else {
        for agent in agent_list() {
            let key = agent_id_key(&agent.id);
            cfg.agent_models.entry(key).or_insert_with(|| all_slugs.clone());
        }
    }
    cfg.version = 3;
}

/// Ensure model_routing has entries for all models (default: "direct")
fn ensure_model_routing(cfg: &mut AppConfig) {
    for m in &cfg.models {
        cfg.model_routing.entry(m.slug.clone()).or_insert_with(|| "direct".into());
    }
}
