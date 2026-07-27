//! Writes configuration to external tool config files.
//! Phase 4: per-model routing (direct | relay:<name>).

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs;
use crate::error::AppError;
use std::io::Write;

use crate::error::Result;
use crate::paths;
use crate::types::{AppConfig, ModelDef, agent_list};

// ── Provider metadata (native direct endpoints) ─────────────

struct ProviderMeta {
    id: &'static str,
    name: &'static str,
    base_url: &'static str,
    /// Primary env key name. We also check aliases when writing providers.json.
    env_key: &'static str,
    /// Alternative env key names to fall back to (checked in .env). Order = preference.
    env_key_aliases: &'static [&'static str],
    feature: Option<&'static str>,
}

const PROVIDER_META: &[ProviderMeta] = &[
    ProviderMeta { id: "deepseek",  name: "DeepSeek",       base_url: "https://api.deepseek.com/v1",                                                  env_key: "DEEPSEEK_API_KEY", env_key_aliases: &["DS_API_KEY"],     feature: None },
    ProviderMeta { id: "glm",       name: "智谱GLM",        base_url: "https://open.bigmodel.cn/api/paas/v4",                                         env_key: "GLM_API_KEY",      env_key_aliases: &["ZHIPU_API_KEY"],  feature: Some("forceParallelToolCalls") },
    ProviderMeta { id: "qwen",      name: "阿里Qwen-Max",   base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1",                             env_key: "QWEN_API_KEY",     env_key_aliases: &[],                 feature: None },
    ProviderMeta { id: "qwen38",    name: "阿里Qwen3.8",    base_url: "https://token-plan.cn-beijing.maas.aliyuncs.com/compatible-mode/v1",          env_key: "QWEN38_API_KEY",   env_key_aliases: &[],                 feature: None },
    ProviderMeta { id: "xiaomi",    name: "小米MiMo",       base_url: "https://api.xiaomimimo.com/v1",                                                  env_key: "MIMO_API_KEY",     env_key_aliases: &["MINIMAX_API_KEY"], feature: None },
    ProviderMeta { id: "anthropic", name: "Anthropic Opus", base_url: "https://api.anthropic.com",                                                      env_key: "",                 env_key_aliases: &[],                 feature: None },
    ProviderMeta { id: "openai",    name: "OpenAI GPT",     base_url: "https://api.openai.com/v1",                                                      env_key: "",                 env_key_aliases: &[],                 feature: None },
];

fn meta_by_id(id: &str) -> Option<&'static ProviderMeta> {
    PROVIDER_META.iter().find(|m| m.id == id)
}

/// Pick the best env key name — the one that actually exists in .env wins.
/// If the primary key or any alias is present in .env, use that name.
/// Otherwise default to the primary key (so the user knows what to create).
fn resolve_env_key(meta: &ProviderMeta) -> String {
    if meta.env_key.is_empty() { return String::new(); }

    let env_path = paths::mimo_env();
    if let Ok(content) = std::fs::read_to_string(&env_path) {
        // Build a set of present key names
        let present: std::collections::HashSet<&str> = content.lines()
            .filter_map(|l| l.split_once('=').map(|(k, _)| k.trim()))
            .collect();

        // Prefer the primary key if it exists
        if present.contains(meta.env_key) {
            return meta.env_key.to_string();
        }
        // Try each alias
        for alias in meta.env_key_aliases {
            if present.contains(alias) {
                return alias.to_string();
            }
        }
    }

    // Not in .env — return the primary (canonical) name
    meta.env_key.to_string()
}

// ── providers.json ────────────────��──────────────────────────

pub fn write_providers(cfg: &AppConfig) -> Result<()> {
    // Collect enabled model slugs from all agents that write_providers
    let enabled_slugs: BTreeSet<String> = agent_list().iter()
        .filter(|a| a.writes_providers)
        .flat_map(|a| cfg.agent_models.get(&crate::types::agent_id_key(&a.id)).cloned().unwrap_or_default())
        .collect();

    // Group models: key = (provider_id, routing)
    type ProviderKey = (String, String);
    let mut groups: BTreeMap<ProviderKey, Vec<&ModelDef>> = BTreeMap::new();

    for m in &cfg.models {
        if !enabled_slugs.contains(&m.slug) { continue; }

        let routing = cfg.model_routing.get(&m.slug)
            .map(|s| s.as_str())
            .unwrap_or("direct");

        let key = (m.provider.clone(), routing.to_string());
        groups.entry(key).or_default().push(m);
    }

    let relay_by_name: BTreeMap<&str, &crate::types::RelayConfig> = cfg.relays.iter()
        .map(|r| (r.name.as_str(), r))
        .collect();

    let mut entries: Vec<serde_json::Value> = Vec::new();

    for ((provider_id, routing), models) in &groups {
        if models.is_empty() { continue; }

        let (base_url, env_key, display_suffix) = if routing == "direct" {
            let meta = meta_by_id(provider_id);
            if meta.is_none() { continue; }
            let meta = meta.unwrap();
            if meta.env_key.is_empty() {
                // Provider with no env key (Anthropic/OpenAI) — skip from providers.json
                // (claude-proxy.js handles anthropic passthrough as a built-in)
                continue;
            }
            (meta.base_url.to_string(), resolve_env_key(meta), String::new())
        } else if routing.starts_with("relay:") {
            let relay_name = &routing[6..];
            let relay = relay_by_name.get(relay_name);
            if relay.is_none() { continue; }
            let relay = relay.unwrap();
            let env_key = format!("RELAY_{}_API_KEY", relay_name.to_uppercase().replace(' ', "_").replace('-', "_"));
            (relay.url.clone(), env_key, format!(" via {}", relay_name))
        } else {
            continue;
        };

        let display_name = meta_by_id(provider_id)
            .map(|m| format!("{}{}", m.name, display_suffix))
            .unwrap_or_else(|| format!("{}{}", provider_id, display_suffix));

        let feature = meta_by_id(provider_id).and_then(|m| m.feature);

        let provider_entry = serde_json::json!({
            "id": format!("{}-{}", provider_id, routing.replace(':', "-")),
            "name": display_name,
            "baseUrl": base_url,
            "envKey": env_key,
            "defaultModel": models[0].slug,
            "models": models.iter().map(|m| serde_json::json!({
                "id": m.slug,
                "displayName": m.display_name,
                "contextWindow": m.context_window,
                "maxOutputTokens": m.max_output_tokens,
            })).collect::<Vec<_>>(),
        });

        let mut entry = provider_entry;
        if feature.is_some() {
            entry["features"] = serde_json::json!({"forceParallelToolCalls": true});
        }
        // Mark Anthropic-native endpoints so claude-proxy.js does native passthrough
        if provider_id == "anthropic" && routing == "direct" {
            entry["anthropicEndpoint"] = serde_json::json!(true);
        }
        entries.push(entry);
    }

    let content = serde_json::to_string_pretty(&serde_json::json!({ "providers": entries }))?;
    write_if_changed(&paths::providers_json(), &content)
}

// ── .env relay keys ─────────────────────────────────────────

pub fn write_env_relay_keys(cfg: &AppConfig) -> Result<()> {
    let env_path = paths::mimo_env();
    let existing = if env_path.exists() { fs::read_to_string(&env_path).unwrap_or_default() } else { String::new() };

    let mut lines: Vec<String> = existing.lines()
        .filter(|l| !l.trim().starts_with("RELAY_") || !l.contains("_API_KEY"))
        .map(|l| l.to_string())
        .collect();

    // Append relay keys
    for relay in &cfg.relays {
        let env_key = format!("RELAY_{}_API_KEY", relay.name.to_uppercase().replace(' ', "_").replace('-', "_"));
        lines.push(format!("{env_key}={}", relay.key));
    }

    let content = lines.join("\n").trim_end().to_string() + "\n";
    write_if_changed(&env_path, &content)
}

// ── Codex config.toml ────────────────────────────────────────

pub fn write_codex_config(cfg: &AppConfig) -> Result<()> {
    // Merge Codex Desktop + Reasonix models for default model (they share config.toml)
    let mut all_slugs: Vec<String> = cfg.agent_models
        .get("codex_desktop").cloned().unwrap_or_default();
    all_slugs.extend(
        cfg.agent_models.get("reasonix").cloned().unwrap_or_default()
    );
    all_slugs.sort();
    all_slugs.dedup();

    let default_model = all_slugs.first().cloned()
        .unwrap_or_else(|| "deepseek-v4-pro".into());

    let default_model_def = cfg.models.iter().find(|m| m.slug == default_model);
    let default_ctxt = default_model_def.map(|m| m.context_window).unwrap_or(1_000_000);
    let default_max_out = default_model_def.map(|m| m.max_output_tokens).unwrap_or(393_216);
    let base_url = format!("http://127.0.0.1:{}/v1", cfg.proxy_ports.mimo2codex);

    let content = format!(r#"model_provider = "custom"
model = "{default_model}"
model_reasoning_effort = "high"
model_context_window = {default_ctxt}
model_max_output_tokens = {default_max_out}
model_catalog_json = "cc-switch-model-catalog.json"

[model_providers.custom]
name = "CC-Gate"
base_url = "{base_url}"
wire_api = "responses"
requires_openai_auth = true
"#);

    write_if_changed(&paths::codex_config_toml(), &content)
}

// ── Model catalog ────────────────────────────────────────────

pub fn write_model_catalog(cfg: &AppConfig) -> Result<()> {
    // Merge models from both Codex CLI and Codex Desktop — both share the same proxy (mimo2codex)
    let mut codex_slugs: BTreeSet<String> = cfg.agent_models
        .get("codex_desktop").cloned().unwrap_or_default()
        .into_iter().collect();
    codex_slugs.extend(
        cfg.agent_models.get("codex_cli").cloned().unwrap_or_default()
    );
    let codex_set: BTreeSet<&str> = codex_slugs.iter().map(|s| s.as_str()).collect();

    let models: Vec<serde_json::Value> = cfg.models.iter()
        .filter(|m| codex_set.contains(m.slug.as_str()))
        .map(|m| serde_json::json!({
            "slug": m.slug, "display_name": m.display_name,
            "context_window": m.context_window, "max_context_window": m.context_window,
            "effective_context_window_percent": 95,
            "default_reasoning_level": m.default_reasoning_level,
            "default_reasoning_summary": "none", "input_modalities": ["text"],
            "supported_reasoning_levels": [
                {"effort":"none","description":"Disable Thinking"},
                {"effort":"low","description":"Low"},
                {"effort":"medium","description":"Medium"},
                {"effort":"high","description":"High"},
                {"effort":"xhigh","description":"Extra high"}
            ],
            "supports_reasoning_summaries": m.supports_reasoning_summaries,
            "supports_parallel_tool_calls": false, "supports_search_tool": false,
            "support_verbosity": false, "supported_in_api": true,
            "shell_type": "shell_command", "apply_patch_tool_type": "freeform",
            "visibility": "list", "priority": m.priority,
            "additional_speed_tiers": [], "service_tiers": [],
            "experimental_supported_tools": [],
            "truncation_policy": {"mode":"bytes","limit":10000},
            "base_instructions": format!("You are Codex, a coding agent powered by {}. You help the user with programming tasks. Read the codebase first, ask questions when needed, and implement solutions directly. Prefer existing patterns and keep changes minimal.", m.display_name),
            "description": format!("{} model via CC-Gate proxy", m.display_name),
            "default_verbosity": "low",
            "supports_image_detail_original": false,
            "upgrade": null
        }))
        .collect();

    let content = serde_json::to_string_pretty(&serde_json::json!({ "models": models }))?;
    write_if_changed(&paths::codex_model_catalog_json(), &content)
}

// ── Claude settings ──────────────────────────────────────────

pub fn write_claude_settings(cfg: &AppConfig) -> Result<()> {
    // Merge models from both Claude CLI and Claude Desktop
    let mut claude_slugs: BTreeSet<String> = cfg.agent_models
        .get("claude_desktop").cloned().unwrap_or_default()
        .into_iter().collect();
    claude_slugs.extend(
        cfg.agent_models.get("claude_cli").cloned().unwrap_or_default()
    );

    // Default model: first assigned (BTreeSet iterates sorted for stability), fallback to deepseek
    let default_model = claude_slugs.iter().next()
        .map(|s| if s.starts_with("claude-") { s.clone() } else { format!("claude-{}", s) })
        .unwrap_or_else(|| "claude-deepseek-v4-pro".into());

    let base_url = format!("http://127.0.0.1:{}", cfg.proxy_ports.claude_proxy);

    // ── Deploy status-line script ──
    let status_line_script = paths::mimo2codex_dir().join("status-line.sh");
    let _ = fs::write(&status_line_script, include_str!("../../scripts/status-line.sh"));

    // StatusLine command: "bash ~/.mimo2codex/status-line.sh" — Claude Code pipes JSON via stdin
    let status_line_cmd = format!("bash {}", status_line_script.display());

    let settings = serde_json::json!({
        "env": {"ANTHROPIC_BASE_URL": base_url},
        "model": default_model,
        "effortLevel": "xhigh",
        "statusLine": {
            "type": "command",
            "command": status_line_cmd,
        },
    });
    write_if_changed(&paths::claude_settings_json(), &serde_json::to_string_pretty(&settings)?)
}

// ── Deploy proxy scripts to ~/.mimo2codex/ ──────────────────

/// Copy built-in proxy scripts to ~/.mimo2codex/ so the proxy manager can launch them.
/// Scripts are embedded at compile time via `include_str!` — no runtime file dependency.
///
/// `write_if_changed` is not used here because we WANT to overwrite on every apply
/// to ensure the scripts are always up-to-date with the shipped CC-Gate version.
pub fn deploy_proxy_scripts() -> Result<()> {
    let dest = paths::mimo2codex_dir();
    fs::create_dir_all(&dest)?;

    // claude-proxy.js — only write if missing (avoid disk I/O on every launch)
    let cp = dest.join("claude-proxy.js");
    if !cp.exists() {
        fs::write(&cp, include_str!("../../claude-proxy.js"))
            .map_err(|e| AppError::Io(e))?;
        tracing::info!("Deployed claude-proxy.js → {}", cp.display());
    }

    // chat-proxy.js
    let chat = dest.join("chat-proxy.js");
    if !chat.exists() {
        fs::write(&chat, include_str!("../../chat-proxy.js"))
            .map_err(|e| AppError::Io(e))?;
        tracing::info!("Deployed chat-proxy.js → {}", chat.display());
    }

    // status-line.sh
    let sl = dest.join("status-line.sh");
    if !sl.exists() {
        fs::write(&sl, include_str!("../../scripts/status-line.sh"))
            .map_err(|e| AppError::Io(e))?;
        tracing::info!("Deployed status-line.sh → {}", sl.display());
    }

    Ok(())
}

/// Run the platform installer script to ensure Node.js + npm + mimo2codex are available.
/// Called once on startup. Non-blocking — fires and forgets.
pub async fn ensure_environment() {
    #[cfg(target_os = "windows")]
    let (script, label) = (
        include_str!("../../scripts/setup-windows.ps1"),
        "setup-windows.ps1",
    );
    #[cfg(target_os = "macos")]
    let (script, label) = (
        include_str!("../../scripts/setup-mac.sh"),
        "setup-mac.sh",
    );
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    return;

    let dir = paths::mimo2codex_dir();
    let _ = fs::create_dir_all(&dir);
    let path = dir.join(label);
    // Write script to disk so user can inspect it
    if let Err(e) = fs::write(&path, script) {
        tracing::warn!("Failed to write {}: {e}", label);
        return;
    }

    let result = {
        #[cfg(target_os = "windows")]
        {
            // Run via powershell
            let mut cmd = tokio::process::Command::new("powershell");
            cmd.args(["-ExecutionPolicy", "Bypass", "-File"]);
            cmd.arg(path.to_string_lossy().to_string());
            cmd.kill_on_drop(true);
            crate::win_console::hide_console_async(&mut cmd);
            cmd.output().await
        }
        #[cfg(target_os = "macos")]
        {
            let mut cmd = tokio::process::Command::new("bash");
            cmd.arg(path.to_string_lossy().to_string());
            cmd.kill_on_drop(true);
            cmd.output().await
        }
    };

    match result {
        Ok(o) if o.status.success() => {
            tracing::info!("Environment setup OK");
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            tracing::warn!("Environment setup exited non-zero: {stderr}");
        }
        Err(e) => {
            tracing::warn!("Environment setup failed: {e}");
        }
    }
}

// ── Shell aliases ────────────────────────────────────────────

const CCGATE_BEGIN: &str = "# >>> CC-Gate aliases >>>";
const CCGATE_END:   &str = "# <<< CC-Gate aliases <<<";
const CCGATE_BEGIN_PS: &str = "# >>> CC-Gate functions >>>";
const CCGATE_END_PS:   &str = "# <<< CC-Gate functions <<<";

pub fn write_shell_aliases(cfg: &AppConfig) -> Result<()> {
    let bash = generate_bash_aliases(cfg);
    let ps   = generate_powershell_functions(cfg);

    for path in paths::shell_configs() {
        let is_ps = path.to_string_lossy().ends_with(".ps1");
        let (content, begin, end) = if is_ps {
            (&ps, CCGATE_BEGIN_PS, CCGATE_END_PS)
        } else {
            (&bash, CCGATE_BEGIN, CCGATE_END)
        };

        let existing = if path.exists() { fs::read_to_string(&path).unwrap_or_default() } else { String::new() };
        let new_content = if existing.contains(begin) {
            let before = &existing[..existing.find(begin).unwrap()].trim_end();
            let after_start = existing.find(end).unwrap() + end.len();
            let after = existing[after_start..].trim_start();
            format!("{}\n{}\n{}", before, content.trim_end(), after).trim_end().to_string() + "\n"
        } else {
            let t = existing.trim_end();
            if t.is_empty() { content.to_string() } else { format!("{}\n\n{}", t, content) }
        };

        let mut f = fs::File::create(&path)?;
        f.write_all(new_content.as_bytes())?;
        tracing::info!("Shell aliases written to {}", path.display());
    }
    Ok(())
}

fn generate_bash_aliases(cfg: &AppConfig) -> String {
    let mut out = String::from(CCGATE_BEGIN) + "\n";
    gen_aliases_impl(cfg, &mut out, false);
    out.push_str(CCGATE_END); out.push('\n');
    out
}

fn generate_powershell_functions(cfg: &AppConfig) -> String {
    let mut out = String::from(CCGATE_BEGIN_PS) + "\n";
    gen_aliases_impl(cfg, &mut out, true);
    out.push_str(CCGATE_END_PS); out.push('\n');
    out
}

fn gen_aliases_impl(cfg: &AppConfig, out: &mut String, powershell: bool) {
    // ── Codex ──────────────────────────────────────────────
    let codex_slugs = cfg.agent_models.get("codex_cli").cloned().unwrap_or_default();
    if !codex_slugs.is_empty() {
        // Native alias: "codex" (uses first assigned model, no -suffix)
        if let Some(m) = codex_slugs.first().and_then(|s| cfg.models.iter().find(|m| &m.slug == s)) {
            if powershell {
                out.push_str(&format!(
                    "function codex {{ $env:CC_GATE_MODEL='{}'; codex --dangerously-bypass-approvals-and-sandbox -c model='{}' -c model_context_window={} -c model_max_output_tokens={} }}\n",
                    m.slug, m.slug, m.context_window, m.max_output_tokens
                ));
            } else {
                out.push_str(&format!(
                    "alias codex='CC_GATE_MODEL=\"{}\" codex --dangerously-bypass-approvals-and-sandbox -c model=\"{}\" -c model_context_window={} -c model_max_output_tokens={}'\n",
                    m.slug, m.slug, m.context_window, m.max_output_tokens
                ));
            }
        }
        // Per-model aliases: codex-{short}
        for slug in &codex_slugs {
            if let Some(m) = cfg.models.iter().find(|m| &m.slug == slug) {
                let aname = codex_alias(slug);
                if powershell {
                    out.push_str(&format!(
                        "function {} {{ $env:CC_GATE_MODEL='{}'; codex --dangerously-bypass-approvals-and-sandbox -c model='{}' -c model_context_window={} -c model_max_output_tokens={} }}\n",
                        aname, slug, m.slug, m.context_window, m.max_output_tokens
                    ));
                } else {
                    out.push_str(&format!(
                        "alias {}='CC_GATE_MODEL=\"{}\" codex --dangerously-bypass-approvals-and-sandbox -c model=\"{}\" -c model_context_window={} -c model_max_output_tokens={}'\n",
                        aname, slug, m.slug, m.context_window, m.max_output_tokens
                    ));
                }
            }
        }
    }

    // ── Claude CLI ──────────────────────────────────────────
    let claude_slugs = cfg.agent_models.get("claude_cli").cloned().unwrap_or_default();
    if !claude_slugs.is_empty() {
        // Native alias: "claude" (uses first assigned model)
        if let Some(slug) = claude_slugs.first() {
            let cm = format!("claude-{}", slug);
            let port = cfg.proxy_ports.claude_proxy;
            if powershell {
                out.push_str(&format!(
                    "function claude {{ $env:CC_GATE_MODEL='{slug}'; $env:ANTHROPIC_BASE_URL='http://127.0.0.1:{port}'; $env:ANTHROPIC_AUTH_TOKEN='proxy'; $env:ANTHROPIC_MODEL='{cm}'; $env:ANTHROPIC_DEFAULT_OPUS_MODEL='{cm}'; $env:ANTHROPIC_DEFAULT_SONNET_MODEL='{cm}'; $env:ANTHROPIC_DEFAULT_HAIKU_MODEL='{cm}'; $env:ANTHROPIC_DEFAULT_FABLE_MODEL='{cm}'; $env:CLAUDE_CODE_ENABLE_GATEWAY_MODEL_DISCOVERY='1'; claude --dangerously-skip-permissions }}\n",
                    slug=slug, port=port, cm=cm,
                ));
            } else {
                out.push_str(&format!(
                    "alias claude='CC_GATE_MODEL=\"{slug}\" \\\n  ANTHROPIC_BASE_URL=\"http://127.0.0.1:{port}\" \\\n  ANTHROPIC_AUTH_TOKEN=proxy \\\n  ANTHROPIC_MODEL=\"{cm}\" \\\n  ANTHROPIC_DEFAULT_OPUS_MODEL=\"{cm}\" \\\n  ANTHROPIC_DEFAULT_SONNET_MODEL=\"{cm}\" \\\n  ANTHROPIC_DEFAULT_HAIKU_MODEL=\"{cm}\" \\\n  ANTHROPIC_DEFAULT_FABLE_MODEL=\"{cm}\" \\\n  CLAUDE_CODE_ENABLE_GATEWAY_MODEL_DISCOVERY=1 \\\n  claude --dangerously-skip-permissions'\n",
                    slug=slug, port=port, cm=cm,
                ));
            }
        }
        // Per-model aliases: claude-{short}
        for slug in &claude_slugs {
            let aname = claude_alias(slug);
            let cm = format!("claude-{}", slug);
            let port = cfg.proxy_ports.claude_proxy;
            if powershell {
                out.push_str(&format!(
                    "function {} {{ $env:CC_GATE_MODEL='{slug}'; $env:ANTHROPIC_BASE_URL='http://127.0.0.1:{port}'; $env:ANTHROPIC_AUTH_TOKEN='proxy'; $env:ANTHROPIC_MODEL='{cm}'; $env:ANTHROPIC_DEFAULT_OPUS_MODEL='{cm}'; $env:ANTHROPIC_DEFAULT_SONNET_MODEL='{cm}'; $env:ANTHROPIC_DEFAULT_HAIKU_MODEL='{cm}'; $env:ANTHROPIC_DEFAULT_FABLE_MODEL='{cm}'; $env:CLAUDE_CODE_ENABLE_GATEWAY_MODEL_DISCOVERY='1'; claude --dangerously-skip-permissions }}\n",
                    aname, slug=slug, port=port, cm=cm,
                ));
            } else {
                out.push_str(&format!(
                    "alias {aname}='CC_GATE_MODEL=\"{slug}\" \\\n  ANTHROPIC_BASE_URL=\"http://127.0.0.1:{port}\" \\\n  ANTHROPIC_AUTH_TOKEN=proxy \\\n  ANTHROPIC_MODEL=\"{cm}\" \\\n  ANTHROPIC_DEFAULT_OPUS_MODEL=\"{cm}\" \\\n  ANTHROPIC_DEFAULT_SONNET_MODEL=\"{cm}\" \\\n  ANTHROPIC_DEFAULT_HAIKU_MODEL=\"{cm}\" \\\n  ANTHROPIC_DEFAULT_FABLE_MODEL=\"{cm}\" \\\n  CLAUDE_CODE_ENABLE_GATEWAY_MODEL_DISCOVERY=1 \\\n  claude --dangerously-skip-permissions'\n",
                    aname=aname, slug=slug, port=port, cm=cm,
                ));
            }
        }
    }

    // ── Aider ──────────────────────────────────────────────
    let aider_slugs = cfg.agent_models.get("aider").cloned().unwrap_or_default();
    if !aider_slugs.is_empty() {
        // Native alias: "aider"
        if let Some(slug) = aider_slugs.first() {
            let port = cfg.proxy_ports.chat_proxy;
            if powershell {
                out.push_str(&format!(
                    "function aider {{ $env:CC_GATE_MODEL='{}'; $env:OPENAI_API_BASE='http://127.0.0.1:{port}/v1'; $env:OPENAI_API_KEY='proxy'; aider --model openai/{} }}\n",
                    slug, slug
                ));
            } else {
                out.push_str(&format!(
                    "alias aider='CC_GATE_MODEL=\"{}\" OPENAI_API_BASE=http://127.0.0.1:{}/v1 OPENAI_API_KEY=proxy aider --model openai/{}'\n",
                    slug, port, slug
                ));
            }
        }
        // Per-model: aider-{short}
        for slug in &aider_slugs {
            let aname = aider_alias(slug);
            let port = cfg.proxy_ports.chat_proxy;
            if powershell {
                out.push_str(&format!(
                    "function {} {{ $env:CC_GATE_MODEL='{}'; $env:OPENAI_API_BASE='http://127.0.0.1:{port}/v1'; $env:OPENAI_API_KEY='proxy'; aider --model openai/{} }}\n",
                    aname, slug, slug
                ));
            } else {
                out.push_str(&format!(
                    "alias {}='CC_GATE_MODEL=\"{}\" OPENAI_API_BASE=http://127.0.0.1:{}/v1 OPENAI_API_KEY=proxy aider --model openai/{}'\n",
                    aname, slug, port, slug
                ));
            }
        }
    }
}

pub fn remove_shell_aliases() -> Result<()> {
    let begins = [CCGATE_BEGIN, CCGATE_BEGIN_PS];
    let ends   = [CCGATE_END,   CCGATE_END_PS];

    for path in paths::shell_configs() {
        if !path.exists() { continue; }
        let existing = fs::read_to_string(&path).unwrap_or_default();

        for (&begin, &end) in begins.iter().zip(ends.iter()) {
            if let (Some(b), Some(e)) = (existing.find(begin), existing.find(end)) {
                let content = format!("{}\n{}\n",
                    existing[..b].trim_end(),
                    existing[e + end.len()..].trim_start()
                ).trim_end().to_string() + "\n";
                fs::write(&path, &content)?;
                break;
            }
        }
    }
    Ok(())
}

// ── User API keys → .env ────────────────────────────────────

pub fn write_user_api_keys(cfg: &AppConfig) -> Result<()> {
    let env_path = paths::mimo_env();
    let existing = if env_path.exists() { fs::read_to_string(&env_path).unwrap_or_default() } else { String::new() };

    // Collect env var names that this function manages (user-entered keys + relay keys)
    let managed_keys: HashSet<&str> = cfg.api_keys.keys().map(|s| s.as_str()).collect();
    let _relay_keys: HashSet<String> = cfg.relays.iter()
        .map(|r| format!("RELAY_{}_API_KEY", r.name.to_uppercase().replace(' ', "_").replace('-', "_")))
        .collect();

    // Keep lines whose env var is NOT in managed_keys and NOT a RELAY_ key
    let other_env_keys: HashSet<&str> = ["DS_API_KEY", "GLM_API_KEY", "QWEN_API_KEY",
        "QWEN38_API_KEY", "MIMO_API_KEY", "DEEPSEEK_API_KEY"].iter().copied().collect();

    let mut preserved: Vec<String> = Vec::new();
    for line in existing.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            preserved.push(line.to_string());
            continue;
        }
        if let Some((key, _)) = trimmed.split_once('=') {
            let key = key.trim();
            // Skip lines whose key is in managed_keys or is a RELAY_ key
            if managed_keys.contains(key) { continue; }
            if key.starts_with("RELAY_") && key.ends_with("_API_KEY") { continue; }
            if other_env_keys.contains(key) { continue; }
        }
        preserved.push(line.to_string());
    }

    // Append user-entered API keys
    for (key, val) in &cfg.api_keys {
        if !val.is_empty() {
            preserved.push(format!("{key}={val}"));
        }
    }

    // Append relay keys
    for relay in &cfg.relays {
        let env_key = format!("RELAY_{}_API_KEY", relay.name.to_uppercase().replace(' ', "_").replace('-', "_"));
        preserved.push(format!("{env_key}={}", relay.key));
    }

    let content = preserved.join("\n").trim_end().to_string() + "\n";
    write_if_changed(&env_path, &content)
}

// ── All-in-one ───────────────────────────────────────────────

pub fn write_all_tool_configs(cfg: &AppConfig) -> Result<()> {
    // Back up original configs before first modification (idempotent)
    crate::backup::ensure_all_backups();

    // Deploy proxy scripts to ~/.mimo2codex/ so CC-Gate can launch them
    deploy_proxy_scripts()?;

    write_codex_config(cfg)?;
    write_model_catalog(cfg)?;
    write_claude_settings(cfg)?;
    write_providers(cfg)?;
    write_user_api_keys(cfg)?;
    write_shell_aliases(cfg)?;
    write_hermes_config(cfg)?;
    write_openclaw_config(cfg)?;
    tracing::info!("All tool configs written");
    Ok(())
}

// ── Hermes config.yaml ─────────────────────────────────────

pub fn write_hermes_config(cfg: &AppConfig) -> Result<()> {
    let slugs: Vec<String> = cfg.agent_models
        .get("hermes").cloned().unwrap_or_default();
    if slugs.is_empty() { return Ok(()); }

    let path = paths::hermes_config_yaml();
    let src = if path.exists() { fs::read_to_string(&path).unwrap_or_default() } else { String::new() };

    // Parse as generic YAML value (preserve non-CC-Gate keys)
    let mut doc: serde_yaml::Value = if src.trim().is_empty() {
        serde_yaml::Value::Mapping(serde_yaml::Mapping::new())
    } else {
        serde_yaml::from_str(&src).unwrap_or_else(|_| serde_yaml::Value::Mapping(serde_yaml::Mapping::new()))
    };

    let port = cfg.proxy_ports.chat_proxy;
    let base_url = format!("http://127.0.0.1:{port}/v1");
    let default_model = slugs.first().cloned().unwrap();

    // Build CC-Gate provider entry
    let mut models_map = serde_yaml::Mapping::new();
    for slug in &slugs {
        if let Some(m) = cfg.models.iter().find(|d| &d.slug == slug) {
            let mut mm = serde_yaml::Mapping::new();
            mm.insert("context_length".into(), serde_yaml::Value::Number((m.context_window as i64).into()));
            mm.insert("name".into(), serde_yaml::Value::String(format!("{} (CC-Gate)", m.display_name)));
            models_map.insert(serde_yaml::Value::String(slug.clone()), serde_yaml::Value::Mapping(mm));
        }
    }

    let mut provider = serde_yaml::Mapping::new();
    provider.insert("name".into(), "ccgate".into());
    provider.insert("base_url".into(), base_url.into());
    provider.insert("api_key".into(), "proxy".into());
    provider.insert("api_mode".into(), "chat_completions".into());
    provider.insert("models".into(), serde_yaml::Value::Mapping(models_map));
    provider.insert("model".into(), default_model.into());

    // Filter existing custom_providers to keep non-CC-Gate ones
    let mut new_providers: Vec<serde_yaml::Value> = Vec::new();
    if let serde_yaml::Value::Mapping(ref map) = doc {
        if let Some(serde_yaml::Value::Sequence(existing)) = map.get("custom_providers") {
            for entry in existing {
                if let Some(name) = entry.get("name").and_then(|v| v.as_str()) {
                    if name == "ccgate" { continue; } // remove old CC-Gate entry
                }
                new_providers.push(entry.clone());
            }
        }
    }
    new_providers.push(serde_yaml::Value::Mapping(provider));

    if let serde_yaml::Value::Mapping(ref mut map) = doc {
        map.insert("custom_providers".into(), serde_yaml::Value::Sequence(new_providers));
    }

    let out = serde_yaml::to_string(&doc)?;
    write_if_changed(&path, &out)
}

// ── OpenClaw openclaw.json ──────────────────────────────────

pub fn write_openclaw_config(cfg: &AppConfig) -> Result<()> {
    let slugs: Vec<String> = cfg.agent_models
        .get("openclaw").cloned().unwrap_or_default();
    if slugs.is_empty() { return Ok(()); }

    let path = paths::openclaw_config_json();
    let src = if path.exists() { fs::read_to_string(&path).unwrap_or_default() } else { String::new() };

    // Parse existing config as JSON5-lenient: try serde_json first
    let doc: serde_json::Value = if src.trim().is_empty() {
        serde_json::Value::Object(serde_json::Map::new())
    } else {
        serde_json::from_str(&src).unwrap_or_else(|_| {
            // Fallback: strip // comments and trailing commas for lenient parse
            let cleaned = src.lines()
                .map(|l| {
                    let trimmed = l.trim_start();
                    if trimmed.starts_with("//") { return String::new(); }
                    let l2 = if let Some(pos) = l.find("//") {
                        let before = &l[..pos];
                        if before.matches('"').count() % 2 == 0 { before.to_string() } else { l.to_string() }
                    } else { l.to_string() };
                    let l3 = l2.trim_end();
                    if l3.ends_with(',') { l3[..l3.len()-1].to_string() } else { l3.to_string() }
                })
                .filter(|l| !l.is_empty())
                .collect::<Vec<_>>()
                .join("\n");
            serde_json::from_str(&cleaned).unwrap_or(serde_json::Value::Object(serde_json::Map::new()))
        })
    };

    let port = cfg.proxy_ports.chat_proxy;
    let default_model = slugs.first().cloned().unwrap();

    let models_json: Vec<serde_json::Value> = slugs.iter()
        .filter_map(|slug| cfg.models.iter().find(|d| &d.slug == slug))
        .map(|m| serde_json::json!({
            "id": m.slug,
            "name": m.display_name,
            "reasoning": false,
            "input": ["text"],
            "cost": {"input": m.input_price_per_1k / 1000.0, "output": m.output_price_per_1k / 1000.0, "cacheRead": 0, "cacheWrite": 0},
            "contextWindow": m.context_window,
            "maxTokens": m.max_output_tokens,
        }))
        .collect();

    // Merge into existing config
    let mut map = if let serde_json::Value::Object(m) = doc { m } else { serde_json::Map::new() };

    // agents.defaults.model.primary
    let primary = serde_json::json!({"primary": format!("ccgate/{default_model}")});
    if let serde_json::Value::Object(ref mut agents) = map.entry("agents".to_string()).or_insert(serde_json::json!({})) {
        if let serde_json::Value::Object(ref mut defaults) = agents.entry("defaults".to_string()).or_insert(serde_json::json!({})) {
            defaults.insert("model".to_string(), primary);
        }
    }

    // models.providers.ccgate
    let ccgate_provider = serde_json::json!({
        "baseUrl": format!("http://127.0.0.1:{port}/v1"),
        "apiKey": "proxy",
        "api": "openai-completions",
        "models": models_json,
    });
    if let serde_json::Value::Object(ref mut models) = map.entry("models".to_string()).or_insert(serde_json::json!({})) {
        if let serde_json::Value::Object(ref mut providers) = models.entry("providers".to_string()).or_insert(serde_json::json!({})) {
            providers.insert("ccgate".to_string(), ccgate_provider);
        }
    }

    let out = serde_json::to_string_pretty(&map)? + "\n";
    write_if_changed(&path, &out)
}

// ── Helpers ─────────────────────────────────────────────────

fn write_if_changed(path: &std::path::Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() { fs::create_dir_all(parent)?; }
    let current = if path.exists() { fs::read_to_string(path).unwrap_or_default() } else { String::new() };
    if current == content { tracing::info!("{} unchanged, skip", path.display()); return Ok(()); }
    let mut f = fs::File::create(path)?;
    f.write_all(content.as_bytes())?;
    tracing::info!("{} written", path.display());
    Ok(())
}

fn short(slug: &str) -> &str { match slug {
    "deepseek-v4-pro" => "ds", "deepseek-v4-flash" => "ds-flash",
    "glm-5.2" => "glm", "qwen3.8-max-preview" => "qwen", "qwen-max" => "qwen-max",
    "mimo-v2.5-pro" => "mimo", "mimo-v2.5" => "mimo-v2.5",
    "claude-opus-5" => "opus", "gpt-5.6" => "gpt",
    _ => slug,
}}
fn codex_alias(s: &str) -> String { format!("codex-{}", short(s)) }
fn claude_alias(s: &str) -> String { format!("claude-{}", short(s)) }
fn aider_alias(s: &str) -> String { format!("aider-{}", short(s)) }

