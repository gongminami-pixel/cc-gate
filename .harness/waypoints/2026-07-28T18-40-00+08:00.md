# Harness Handoff

_Last updated: 2026-07-27T16:50:00+08:00_

## Goal

构建 CC-Gate — Tauri 2 + Vue 3 桌面应用，统一管理 10 AI Agent 的模型/路由/Key/Shell。开源在 https://github.com/gongminami-pixel/cc-gate。

## State snapshot

- **Branch**: main
- **Commit**: 712c8d1 chore: bump version to 0.1.1
- **Tag**: v0.1.1（即将 bump 到 v0.1.2）
- **Uncommitted**:
  - `scripts/status-line.sh` — 新增 StatusLine 脚本
  - `src-tauri/src/config_writer.rs` — write_claude_settings: model 加 `claude-` 前缀 + statusLine 嵌入
  - `claude-proxy-sse-fix.md` / `status-bar-fix-report.md` — 临时文档（不提交）

## Context you must load (JIT)

- `src-tauri/src/proxy_manager.rs` — 代理管理核心
- `src-tauri/src/config_writer.rs` — 配置写入（所有 Agent）
- `claude-proxy.js` — SSE 流代理脚本（Anthropic→OpenAI 协议转换，已修复 4 bug）
- `scripts/status-line.sh` — Claude Code 状态栏脚本

## What works

- ✅ SSE 流代理 4 bug 全修
- ✅ /v1/models 返回 context_window + max_output_tokens
- ✅ Claude Code model 字段带 `claude-` 前缀匹配 gateway 模型 ID
- ✅ CC-Gate 自动部署 statusLine 脚本到 `~/.mimo2codex/status-line.sh`
- ✅ 所有 Agent 上下文窗口/价格正确写入各自配置文件
- ✅ 3 代理无条件下启动 + 状态栏实时显示
- ✅ v0.1.1 Release: Mac DMG + Windows exe + SHA256

## What's broken

- ⚠️ DMG 打包偶尔因旧 app 未退出失败
- ⚠️ 上面那条 Claude Code context 提示条（`XX% context used`）无法关闭，但 model 名正确

## Next actions (ordered)

1. 本次：git 提交 + bump 0.1.2 + push + 双端构建 + Release 上传 + SHA256
2. 下次：claude-proxy.js 嵌入 Tauri bundle 防被 cc-gate backup 还原

## Beware

- **Windows 构建**：用 Parallels VM，cmd.exe（非 PowerShell），手动 set PATH=cargo;node 路径
- **claude-proxy.js** 三处同步：项目根 + ~/.mimo2codex/ + /tmp/claude-proxy-fixed.js
- **DeepSeek stream**：必须 disabled thinking
- **model slug 前缀**：settings.json 用 `claude-` 前缀（匹配 gateway），providers.json 用裸 slug（匹配代理路由）
