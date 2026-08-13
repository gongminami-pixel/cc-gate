# Harness Handoff

_Last updated: 2026-08-13T13:25:00+08:00_

## Goal

构建 CC-Gate — Tauri 2 + Vue 3 桌面应用，统一管理 10 AI Agent 的模型/路由/Key/Shell。开源在 https://github.com/gongminami-pixel/cc-gate。当前 0.1.19 发布：deepseek-v4-pro 原生 Responses 支持 + 版本号显示修复。

## Context you must load

- `CLAUDE.md` (project root) + `DESIGN.md` (project root)
- `AGENTS.md` (project root)
- `.harness/handoff.md` (this file)
- `.harness/decisions.md` (latest 10 entries)
- `.harness/waypoints/` (latest)

## State snapshot

- **Branch**: `main` @ `53fc5d1`（同步前 HEAD）
- **Uncommitted**: `types.rs`、`config_writer.rs`、`tauri.conf.json`、`Cargo.toml`、`Sidebar.vue`、`PageStartup.vue`、`scripts/release.sh`、`.harness/`
- **Version**: 0.1.19（tauri.conf.json + Cargo.toml 已同步）
- **macOS DMG**: `CC-Gate_0.1.19_x64.dmg` SHA256 `a8a875bfc1396e96a38025e51cfb2f4913a561c9040a9ce6d3b0e03ee6ff4ebf`
- **Windows exe**: `CC-Gate_0.1.19_x64-setup.exe` SHA256 `67be4d552f8d3f433df664d91e46405608262b2eaff8b3981cd970befcb36d3e`

## What works / what's broken

- ✅ deepseek-v4-pro 原生 Responses API（curl /v1/responses + codex exec 直连均验证通过）
- ✅ cargo check 0 errors / cargo test 8 passed
- ✅ 双端构建 0.1.19 完成（含版本号修复）
- ✅ Sidebar 左下角版本号动态读取（getAppVersion），不再硬编码 v0.1.0
- ⚠️ mimo2codex(8688) 仍保留 —— GLM/Qwen/MiMo 无 Responses 接口，经 Codex 时仍需翻译

## Next actions

1. commit 所有改动 + `git push` 推送到 origin/main
2. `bash scripts/release.sh` 发布 v0.1.19

## Open questions

- 无

## Key Decisions (latest)

- deepseek-v4-pro 设 native_responses=true（2026-08-13 实测原生支持 Responses API）
- mimo2codex 不删 —— GLM/Qwen/MiMo 仍走 8688（GLM /responses 实测 404）
- Sidebar 版本号硬编码 v0.1.0 → 动态 getAppVersion()；Cargo.toml version 同步 0.1.19（消除与 tauri.conf.json 脱节）
- 启动项 mimo2codex 说明文字修正为「仅非原生模型」

## Beware

- 开源项目 — 同步提交后必须 `git push` 推送到 origin/main
- 双端构建：Windows 包在 Mac Parallels VM 里用 `cmd /c` 构建，绝不在 Mac 上跑 cargo-xwin
- macOS tauri build 后台跑会卡在文件锁（cargo 被孤立），前台跑才稳；Windows 后台 SSH 构建同样会孤立（rustc 继续在 VM 上编译，需轮询 tasklist 等完成）
- 版本号三处：tauri.conf.json（bundle 命名 + package_info.version，UI getAppVersion 读它）、Cargo.toml（crate version）、package.json（前端 npm 版本，与 app 显示无关）
