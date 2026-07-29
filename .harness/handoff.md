# Harness Handoff

_Last updated: 2026-07-29T16:35:00+08:00_

## Goal

构建 CC-Gate — Tauri 2 + Vue 3 桌面应用，统一管理 10 AI Agent 的模型/路由/Key/Shell。开源在 https://github.com/gongminami-pixel/cc-gate。

## State snapshot

- **Branch**: main
- **Commit**: 45099e7 chore: bump version to 0.1.11 for release
- **Tag**: v0.1.11
- **Uncommitted**: tauri.conf.json 0.1.11→0.1.12, config_writer.rs alias 展开修复

## Context you must load (JIT)

- `src-tauri/src/config_writer.rs` — 配置写入（alias 生成含 `\codex`/`\claude`/`\aider` 防展开）
- `claude-proxy.js` — 代理路由核心
- `src-tauri/src/commands/usage.rs` — 日志诊断出口
- `src/components/PageAbout.vue` — 诊断信息页
- `windows-vm-build-guide.md` — Windows 虚拟机构建 Runbook
- `cc-gate-alias-展开bug.md` — zsh alias 递归展开的详细分析

## What works

- ✅ Mac DMG + Windows exe 双端包 v0.1.11 已上传 GitHub Release（SHA256 已更新）
- ✅ claude-proxy.js 路由修复、config_writer.rs 配置加固、proxy_manager.rs 启动修复
- ✅ 诊断信息页（版本号 + 日志尾部 + 一键复制）
- ✅ zsh alias 递归展开 bug 已修复（`\codex`/`\claude`/`\aider`）

## What's broken

- ⚠️ 镜像站点加载卡顿（网络问题，非 CC-Gate bug）

## Next actions (ordered)

1. 双端构建 v0.1.12 + GitHub Release + SHA256
2. 用户反馈测试

## Beware

- **Windows 构建**：用 `win-vm-build` skill；每次用全新 `cc-gate-build` 目录，`rmdir /s /q` 清理
- **tar 打包**必须含 `claude-proxy.js chat-proxy.js scripts/`（`include_str!` 依赖）
- **relay_env_key() 是单真源**，所有写 API key 的地方必须调用它
- **per-model alias 末尾命令名**必须用 `\codex`/`\claude`/`\aider` 防 zsh 递归展开
