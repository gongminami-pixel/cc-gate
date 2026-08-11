# Harness Handoff

_Last updated: 2026-08-11T11:35:00+07:00_

## Goal

构建 CC-Gate — Tauri 2 + Vue 3 桌面应用，统一管理 10 AI Agent 的模型/路由/Key/Shell。开源在 https://github.com/gongminami-pixel/cc-gate。当前 0.1.18 已发布：OpenCode 接入 + 检测修复 + 配置保留。

## Current Status

- v0.1.18 双端发布完成（Release: https://github.com/gongminami-pixel/cc-gate/releases/tag/v0.1.18）
  - macOS DMG / Windows setup.exe 已上传，SHA256 表 + Changes 三段在 Release body
  - /Applications/CC-Gate.app = 0.1.18（旧版备份 .bak-20260811-102433/-111351/-113107）
  - main 最新 = 0.1.18 发布提交 + 同步记忆提交（待本轮提交）
- 9 agent 全部可代理且检测正确（full_apply_all_agents_proxied 回归测试常驻）
- 本机配置：codex config.toml（含保留的 [projects]）、opencode.jsonc（ccgate provider）、openclaw.json（ccgate provider）、claude settings.json、hermes、zshrc 别名块 —— 全部为 cc-gate 写入状态

## Next Actions

1. 用户启动 app 后验证：应用按钮 → 9 agent 状态全绿（含 OpenCode/OpenClaw）
2. 若用户需要：把 10:32 被旧版抹掉的 MCP 配置（node_repl / code-review-graph）补回 config.toml（现在有 preserve 逻辑不会再丢）
3. 无（0.1.18 三项已全部完成并自动化测试）

## Key Decisions (latest)

- 0.1.18 三个候选一次性实现 + 自动化测试 + 发布（用户要求"全部一次性搞好"）
- JSONC 解析换官方 json5 crate（手写行级剥离无法区分尾逗号/分隔逗号）
- OpenCode 路径修正为 ~/.config/opencode/opencode.jsonc（原指向 config.toml 是错的）
- write_codex_config 用 toml crate 解析保留 [projects.*]/[mcp_servers.*]
- 发布顺序教训已应用：先 commit+push 再 release.sh（v0.1.18 tag 指向 bed4dd5 正确）

## Agents

- 10 agents: codex_cli/codex_desktop/codex_reasonix/claude_cli/claude_desktop/hermes/openclaw/opencode/aider/cursor
- 代理端口：8688 codex / 8689 claude / 8690 chat(opencode/openclaw)，由 app 拉起
- 合并模型目录：~/.codex/cc-gate-model-catalog.json（7 模型）
- 配置管理：备份 ~/.mimo2codex/backups/*.orig（幂等）→ 写全部 agent 配置 → 恢复按钮还原
