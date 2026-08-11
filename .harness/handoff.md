# Harness Handoff

_Last updated: 2026-08-11T09:12:49+07:00_

## Goal

构建 CC-Gate — Tauri 2 + Vue 3 桌面应用，统一管理 10 AI Agent 的模型/路由/Key/Shell。开源在 https://github.com/gongminami-pixel/cc-gate。

## State snapshot

- **Branch**: main
- **Commit**: 4a33e73（docs: TODO 同步 — 去非线 + 原生 alias 完成；ahead 4 未 push）
- **Version**: 0.1.16（tauri.conf.json 已 bump，未提交）
- **Released**: 0.1.15 为上一发版；0.1.16 正在双端构建 + 发布中
- **Uncommitted**: src-tauri/tauri.conf.json（version 0.1.15 → 0.1.16）

## Context you must load (JIT)

- `src-tauri/src/config_writer.rs` — 配置写入 + 别名生成（无后缀=官方原生，带后缀=走代理）
- `claude-proxy.js` / `chat-proxy.js` — 代理路由（~/.mimo2codex/ 运行时与源码同步）
- `src-tauri/src/proxy_manager.rs` — 端口管理 + lsof 绝对路径
- `src/components/PageShell.vue` — 别名列表（含原生条目）
- `src/components/PageRelayKeys.vue` — 中转 Key（预设只留 OpenRouter）
- `scripts/release.sh` — GitHub Release 上传脚本（版本号硬编码，发版前需改 TAG/VERSION/DMG/EXE 路径）
- `windows-vm-build-guide.md` — Windows 虚拟机构建 Runbook
- `~/.config/gh/hosts.yml` — GitHub CLI 认证（gongminami-pixel）

## What works

- ✅ 0.1.15 已发版（macOS DMG + Windows NSIS setup + SHA256，GitHub Release）
- ✅ 去非线智能：代码/注释/测试全清，providers.json 只剩 4 个官方直连 provider（DeepSeek/GLM/Qwen/MiMo）
- ✅ 无后缀 alias = 官方原生：codex→openai provider + gpt-5.5；claude→api.anthropic.com 显式覆盖 settings.json；aider→裸命令；带后缀 alias 仍走本地代理
- ✅ 中转预设只留 OpenRouter（PageRelayKeys.vue）
- ✅ 新增回归测试 bare_aliases_are_native_suffixed_stay_proxy + 门控测试 manual_apply_when_requested（CCGATE_MANUAL_APPLY=1 可无 GUI 应用配置，替代已删除的 apply_sim）
- ✅ Rust 4 测试过、proxy 16 项测试过、前端 vue-tsc build 过
- ✅ 三代理（8688/8689/8690）已重启加载新配置，实测响应正常
- ✅ macOS 0.1.16 DMG 构建成功（src-tauri/target/release/bundle/dmg/CC-Gate_0.1.16_x64.dmg，3.7M）
- ✅ Windows 构建已启动（VM 10.211.55.8，nsis target 已 patch）

## What's broken

- ⚠️ 原生 codex 官方 token 过期（需 `codex login`）；原生 claude 需官方登录/key
- ⚠️ claude-opus-5 / gpt-5.6 等官方模型在 opencode 里会 400（非线移除后无 provider 路由；需官方凭据或从首页取消勾选）
- ⚠️ Parallels Standard 版 `prlctl start` 被禁（需 Pro/Business）；VM 启动用 `open ~/Parallels/Windows\ 10.pvm` 或 GUI

## Next actions (ordered)

1. Windows 构建完成（后台 ssh），scp 回 `*_x64-setup.exe` 到 /tmp
2. 计算双包 SHA256（dmg + setup.exe）
3. 更新 scripts/release.sh：TAG=v0.1.16、VERSION=0.1.16、DMG 路径、EXE=/tmp/CC-Gate_0.1.16_x64-setup.exe、Changes 更新
4. 跑 release.sh 创建 v0.1.16 Release + 上传两包 + 更新哈希（Release body 的 SHA256 表）
5. 同步记忆提交 + git push origin main（本项目是开源项目，必须 push）
6. 更新 cc-gate skill（如发布流程有变化）

## Beware

- **Windows 构建**：`win-vm-build` skill；源码 tar 包必须含 `claude-proxy.js chat-proxy.js scripts/`（include_str! 依赖）；排除 .xwin-cache/gen/target/node_modules
- **VM nsis patch**：tauri.conf.json targets 在 VM 上用 patch_nsis.ps1 改为 ["app","nsis"]，本地保持 ["app","dmg"]
- **release.sh 版本号硬编码**：每次发版必须改 TAG/VERSION/DMG/EXE 四处，否则传错版本
- **构建上传后**：必须更新 GitHub Release 的 SHA256 值（release.sh 创建时 body 自动带哈希）
- **★本项目是开源项目**（github.com/gongminami-pixel/cc-gate）：每次"同步加提交"后必须额外 `git push origin main`，与全局记忆 "只本地不 push" 不同
- **★0.1.16 是全新 release tag**（非覆盖旧 tag）
