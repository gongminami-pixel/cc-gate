# Harness Handoff

_Last updated: 2026-08-11T10:50:00+07:00_

## Goal

构建 CC-Gate — Tauri 2 + Vue 3 桌面应用，统一管理 10 AI Agent 的模型/路由/Key/Shell。开源在 https://github.com/gongminami-pixel/cc-gate。当前任务：0.1.17 双端构建发布 + 全量更新（push + 同步记忆 + 提交）。

## State snapshot

- **Branch**: main
- **Commit**: 0fad240（release: v0.1.16）+ 未提交 5 个文件：DESIGN.md、src-tauri/src/config_writer.rs、src-tauri/src/paths.rs（catalog 文件名根治）、src-tauri/tauri.conf.json（0.1.16→0.1.17）、scripts/release.sh（TAG/VERSION/DMG/EXE → 0.1.17）
- **Version**: 0.1.17（构建中）
- **Released**: 0.1.16 已发布（macOS DMG + Windows NSIS setup + SHA256，GitHub Release）

## Context you must load (JIT)

- `src-tauri/src/paths.rs` — codex_model_catalog_json() = ~/.codex/cc-gate-model-catalog.json（本次根治核心）
- `src-tauri/src/config_writer.rs` — write_all_tool_configs 全链 + codex 模板 model_catalog_json = "cc-gate-model-catalog.json"
- `src-tauri/src/backup.rs` — 备份/恢复（.orig/.absent）+ is_agent_proxied 检测
- `scripts/release.sh` — GitHub Release 上传脚本（版本号硬编码，已改 0.1.17）
- `claude-proxy.js` / `chat-proxy.js` — 代理路由（~/.mimo2codex/ 运行时与源码同步）

## What works

- ✅ catalog 文件名根治：cc-switch-model-catalog.json → cc-gate-model-catalog.json（paths.rs + config_writer.rs + DESIGN.md 11 处），与 CC Switch 彻底隔离
- ✅ 本机落地：~/.codex/config.toml 单行指向新目录（[projects]/[mcp_servers] 外科手术保留）；7 模型合并目录生成；.zshrc 别名规范形态；新二进制已装 /Applications（旧版备份 .bak-20260811-102433）
- ✅ 全链 headless 测试（apply → restore → re-apply 循环）：全部 agent 配置写入正常、Aider/Cursor 恢复生效、OpenClaw ccgate provider 正确写入（8690）、codex exec 走 8688 返回 OK
- ✅ cargo test 5 项全过；npm run build + cargo build 零错误；release 二进制 strings 验证新文件名 2 处/旧文件名 0 处
- ✅ 0.1.17 macOS DMG 构建中（后台 proc_078f61504238）、Windows 构建中（后台 proc_8dffd47e5fb4，VM 10.211.55.8 SSH 免密，nsis 已 patch）

## What's broken

- ⚠️ **发现 1（环境）**：本机 codex/claude/hermes 的 .orig 备份被早期 cc-gate 版本污染（本身含代理内容）→ 恢复按钮恢复的是"旧 cc-gate 状态"而非用户原始。干净机器不受影响；测试恢复功能要用干净机器
- ⚠️ **发现 2（代码 bug）**：`is_agent_proxied(OpenClaw)` 查 `"id":"ccgate"`，而 write_openclaw_config 实际写 `models.providers.ccgate`（map key）→ UI 恒显示 OpenClaw 未代理（假阴性）。修复方向：检测改为查 `models.providers` 含 `"ccgate"`
- ⚠️ **发现 3（未完成功能）**：OpenCode 半成品 —— agent 列表/备份(.absent)/检测都有，但 write_all_tool_configs 无 write_opencode_config；paths::opencode_config_toml() 指向 ~/.config/opencode/config.toml 而 opencode 实际读 opencode.jsonc；检测查 TOML 语法与 JSONC 不符。需补写入函数 + 修正路径/格式/检测
- ⚠️ 原生 codex 官方 token 过期；claude-opus-5/gpt-5.6 在 opencode 400（需官方凭据）
- ⚠️ 本机 config.toml 曾含用户手工痕迹（medium/200000/o-ds），应用按钮整文件替换会清掉 projects/MCP —— write_codex_config 模板不含 [projects]/[mcp_servers]，将来装机可能覆盖用户配置（待产品决策）

## Next actions (ordered)

1. macOS DMG + Windows setup.exe 构建完成（两个后台进程，notify_on_complete）
2. Windows exe 从 VM scp 回 /tmp + 双包 SHA256 校验（VM certutil 核对）
3. 跑 scripts/release.sh 创建 v0.1.17 Release + 上传双包（Changes 更新：catalog 文件名根治 + 全链测试 + 3 个发现）
4. git add + commit（"release: v0.1.17 catalog 文件名根治 + 双端构建发布"）+ tag v0.1.17 + push origin main
5. 更新 cc-gate skill（0.1.17 发布 + 3 个发现已在 skill 记录，补发布结果）
6. 3 个发现的修复方案与用户确认后开新任务

## Beware

- **本机 config.toml 不可用 write_codex_config 整文件模板**（会抹掉 [projects]/[mcp_servers]）—— 已记入 skill
- **.orig 备份被污染**：恢复按钮测试必须用干净机器；本机恢复=旧 cc-gate 状态
- **release.sh 版本号硬编码**：已改 0.1.17（TAG/VERSION/DMG/EXE 四处）
- **Windows 构建**：源码 tar 必须含 claude-proxy.js chat-proxy.js scripts/（include_str! 依赖）；nsis patch 在解压后传；`cmd /c` + PATH 手动加 cargo/nodejs
- **★本项目是开源项目**：每次"同步加提交"后必须额外 `git push origin main`
- **★0.1.17 是新 release tag**（非覆盖）
- VM prlctl exec 非 Pro 不可用；SSH 免密可达（ping 不通是 Windows 防火墙挡 ICMP，正常）
