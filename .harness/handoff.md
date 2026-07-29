# Harness Handoff

_Last updated: 2026-07-29T15:00:00+08:00_

## Goal

构建 CC-Gate — Tauri 2 + Vue 3 桌面应用，统一管理 10 AI Agent 的模型/路由/Key/Shell。开源在 https://github.com/gongminami-pixel/cc-gate。

## State snapshot

- **Branch**: main
- **Commit**: 41f6267 docs(harness): 同步记忆 v0.1.10 弹窗改造+双端构建+Release
- **Tag**: v0.1.10
- **Uncommitted** (8 files, +270/-55): claude-proxy.js 多项修复, config_writer.rs relay_env_key 单真源, proxy_manager.rs 启动时完整配置写入 + stderr pipe, commands/usage.rs 日志诊断出口, PageAbout.vue 诊断信息区, ipc/api.ts IPC 接口

## Context you must load (JIT)

- `claude-proxy.js` — 代理路由核心（模型解析、Anthropic 直通、超时、non-ASCII env var）
- `src-tauri/src/config_writer.rs` — 配置写入（relay_env_key 单真源、deploy_proxy_scripts）
- `src-tauri/src/proxy_manager.rs` — 代理管理（启动时 providers.json + .env 同时写、stderr pipe）
- `src-tauri/src/commands/usage.rs` — 新增 get_app_log_tail / get_app_version / copy_to_clipboard
- `src/components/PageAbout.vue` — 关于页（诊断信息区）
- `src/ipc/api.ts` — IPC 接口声明
- `windows-vm-build-guide.md` — Windows 虚拟机构建完整 Runbook
- `src-tauri/src/lib.rs` — Tauri command 注册

## What works

- ✅ Mac DMG (3.9MB) + Windows exe (2.94MB) 双端包 v0.1.10 已上传 GitHub Release
- ✅ 中转站弹窗改造：四个输入框改为 Modal 弹窗
- ✅ `win-vm-build` skill 已固化，说"双端构建"即可触发
- ✅ `relay_env_key()` 单真源：中文 relay 名稳定转译，不再互相覆盖 key
- ✅ `deploy_proxy_scripts()` 用 `write_if_changed`：升级 CC-Gate 后代理脚本自动更新
- ✅ `start_enabled()` 同时写 providers.json + .env：解决代理 401 问题
- ✅ 代理 stderr → app log：诊断信息页可查看代理路由日志
- ✅ 诊断信息页：版本号（Rust 端读取）+ 日志尾部 + 一键复制

## What's broken

- ⚠️ 镜像站点加载卡顿（网络问题，非 CC-Gate bug）

## What was fixed (this session)

- 🔧 claude-proxy.js: 默认端口 8689, 非 ASCII env var 解析, per-provider 超时/版本, 模型名解析健壮, Anthropic 直通优先 providers.json, passthrough 用 provider.apiKey
- 🔧 config_writer.rs: relay_env_key 单真源 + 单测, deploy_proxy_scripts write_if_changed, write_user_api_keys 同名 key
- 🔧 proxy_manager.rs: 启动同时写两文件, stderr piped
- 🔧 日志诊断出口: get_app_log_tail + get_app_version + copy_to_clipboard

## Next actions (ordered)

1. 本地提交当前 8 个文件的改动 + push
2. 下次出包前 bump 版本号（0.1.10 → 0.1.11）
3. 出包后双端/三端构建 + Release + SHA256 更新

## Beware

- **Windows 构建**：用 `win-vm-build` skill 或 `windows-vm-build-guide.md` runbook。核心：每次构建用**全新 `cc-gate-build` 目录**（`rmdir /s /q` 清理），否则残留的 Tauri 自动生成 `Cargo.toml` 导致 `include_str!` 路径错误
- **build 前必须确认 VM 的 `cc-x-llm` 目录无残留 `Cargo.toml`**
- **`beforeBuildCommand`** 在 Windows 上必须是 `"npm run build"`（不是 pnpm），用 Python UTF-8 patch 后 scp 到 VM，不用 PowerShell 改
- **分块回传**：exe 用 256KB chunks + Python 二进制拼接
- **claude-proxy.js** 三处同步：项目根 + ~/.mimo2codex/ + /tmp/claude-proxy-fixed.js
- **DeepSeek stream**：必须 disabled thinking
- **relay_env_key() 是单真源**：所有写 `RELAY_*_API_KEY` 的地方必须调用它，禁止手写
