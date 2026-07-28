# Harness Handoff

_Last updated: 2026-07-28T18:40:00+08:00_

## Goal

构建 CC-Gate — Tauri 2 + Vue 3 桌面应用，统一管理 10 AI Agent 的模型/路由/Key/Shell。开源在 https://github.com/gongminami-pixel/cc-gate。

## State snapshot

- **Branch**: main
- **Commit**: c9f4600 feat: 中转站弹窗改造——四个输入框改为Modal弹窗，页面更清爽
- **Tag**: v0.1.10
- **Uncommitted**: none（已全部提交并 push）

## Context you must load (JIT)

- `src-tauri/src/proxy_manager.rs` — 代理管理核心
- `src-tauri/src/config_writer.rs` — 配置写入（所有 Agent）
- `src/components/PageRelayKeys.vue` — 中转站 UI（Modal 弹窗模式）
- `scripts/release.sh` — GitHub Release 发布脚本
- `windows-vm-build-guide.md` — Windows 虚拟机构建完整 Runbook

## What works

- ✅ Mac DMG (3.9MB) + Windows exe (2.94MB) 双端包 v0.1.10 已上传 GitHub Release
- ✅ 中转站弹窗改造：四个输入框改为 Modal 弹窗，页面更清爽
- ✅ 旧版本 Release 资产已清理，标注废弃
- ✅ `win-vm-build` skill 已固化到 `~/.claude/skills/win-vm-build/`，说"双端构建"即可触发

## What's broken

- ⚠️ `codex-cli` 配置写入后用户称无变化，待排查（未确认是 CC-Gate 问题还是 codex 问题）
- ⚠️ 镜像站点加载卡顿（确认是网络问题，非 CC-Gate bug）

## Next actions (ordered)

1. codex-cli 配置写入后生效问题排查（用户报）
2. 其他用户反馈的问题
3. 下次改动后：git 提交 + bump + push + 双端构建 + Release + SHA256

## Beware

- **Windows 构建**：用 `win-vm-build` skill 或 `windows-vm-build-guide.md` runbook。核心：每次构建用**全新 `cc-gate-build` 目录**（`rmdir /s /q` 清理），否则残留的 Tauri 自动生成 `Cargo.toml` 导致 `include_str!` 路径错误
- **build 前必须确认 VM 的 `cc-x-llm` 目录无残留 `Cargo.toml`**（Tauri CLI 自动生成在项目根的伪造文件）
- **`beforeBuildCommand`** 在 Windows 上必须是 `"npm run build"`（不是 pnpm），用 Python UTF-8 patch 后 scp 到 VM，不用 PowerShell 改
- **分块回传**：exe 用 256KB chunks + Python 二进制拼接，不用 cat（RTK 代理会当 UTF-8 处理）
- **claude-proxy.js** 三处同步：项目根 + ~/.mimo2codex/ + /tmp/claude-proxy-fixed.js
- **DeepSeek stream**：必须 disabled thinking
- **model slug 前缀**：settings.json 用 `claude-` 前缀（匹配 gateway），providers.json 用裸 slug（匹配代理路由）
