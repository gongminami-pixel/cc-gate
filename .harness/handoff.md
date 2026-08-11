# Harness Handoff

_Last updated: 2026-08-11T11:20:00+07:00_

## Goal

构建 CC-Gate — Tauri 2 + Vue 3 桌面应用，统一管理 10 AI Agent 的模型/路由/Key/Shell。开源在 https://github.com/gongminami-pixel/cc-gate。当前任务已全部完成：0.1.17 双端构建发布 + 全量更新（push + 同步记忆 + 提交）。

## State snapshot

- **Branch**: main @ a1640a3（release: v0.1.17 catalog 文件名根治 + 双端构建发布 + 同步记忆）—— 已 push origin main
- **Version**: 0.1.17（已发布）
- **Released**: v0.1.17 GitHub Release 已创建，双包已上传（DMG 118b2891... / setup.exe 430b3c34...，SHA256 表已更新）
- **Installed**: /Applications/CC-Gate.app = 0.1.17（Info.plist 0.1.17 + 二进制含 catalog 修复 2 处）；旧版备份 .bak-20260811-102433 / -111351
- **tag**: 本地 v0.1.17 → a1640a3；远端 v0.1.17 → 0fad240（旧 commit，用户拒绝 force push，接受现状）

## Context you must load (JIT)

- `src-tauri/src/paths.rs` — codex_model_catalog_json() = ~/.codex/cc-gate-model-catalog.json（0.1.17 根治核心）
- `src-tauri/src/config_writer.rs` — write_all_tool_configs 全链 + codex 模板 model_catalog_json = "cc-gate-model-catalog.json"
- `src-tauri/src/backup.rs` — 备份/恢复（.orig/.absent）+ is_agent_proxied 检测
- `scripts/release.sh` — GitHub Release 脚本（版本号硬编码，0.1.17；Changes 段反引号会被 heredoc 吞，用纯文本或 gh release edit 修正）

## What works

- ✅ 0.1.17 双端构建 + 发布完成（macOS DMG 3.7M + Windows NSIS setup 2.9M，SHA256 双端校验一致）
- ✅ main 已 push（a1640a3），含源码修复（paths.rs/config_writer.rs/DESIGN.md）+ 版本 bump + .harness 同步
- ✅ catalog 文件名根治落地：本机 config.toml → cc-gate-model-catalog.json（7 模型合并目录），codex-ds 切换器全模型
- ✅ 全链 headless 测试（apply → restore → re-apply）：Aider/Cursor 恢复生效、OpenClaw provider 写入正确
- ✅ cc-gate skill 已更新：发版流程教训（先 push 再发版）+ 3 个测试发现 + manual_apply 范围说明

## What's broken

- ⚠️ **发现 1（环境）**：本机 .orig 备份被早期 cc-gate 版本污染 → 恢复按钮恢复的是"旧 cc-gate 状态"；干净机器验证
- ⚠️ **发现 2（代码 bug）**：`is_agent_proxied(OpenClaw)` 查 `"id":"ccgate"`，实际写 `models.providers.ccgate`（map key）→ UI 恒显示 OpenClaw 未代理。修复方向：检测改为查 models.providers 含 "ccgate"
- ⚠️ **发现 3（未完成功能）**：OpenCode 半成品 —— 无 write_opencode_config；paths 指向 config.toml 而实际是 opencode.jsonc；检测查 TOML 语法不符。需补写入函数 + 修路径/格式/检测
- ⚠️ 远端 v0.1.17 tag 指向旧 commit（0fad240），Release 页 Source code 快照是 0.1.16；不影响双包下载
- ⚠️ 本机 config.toml 用 write_codex_config 整文件模板会抹掉 [projects]/[mcp_servers]（待产品决策）

## Next actions (ordered)

1. （待用户决策）修复发现 2：is_agent_proxied(OpenClaw) 检测格式 → 0.1.18
2. （待用户决策）实现发现 3：OpenCode 配置写入（write_opencode_config + 路径 opencode.jsonc + JSONC 检测）→ 0.1.18
3. （待用户决策）write_codex_config 模板合并保留 [projects]/[mcp_servers]
4. 下次发版顺序：commit → push main → 再跑 release.sh（避免 tag 指旧 commit）

## Beware

- **本机 config.toml 不可用 write_codex_config 整文件模板**（会抹掉 [projects]/[mcp_servers]）—— 已记入 skill
- **.orig 备份被污染**：恢复按钮测试必须用干净机器
- **release.sh 版本号硬编码**：发版必须改 TAG/VERSION/DMG/EXE 四处；Changes 段反引号被 heredoc 吞
- **★本项目是开源项目**：每次"同步加提交"后必须额外 `git push origin main`
- **★0.1.18 是新 release tag**（非覆盖）
- VM：prlctl exec 非 Pro 不可用；SSH 免密可达（ping 不通是防火墙挡 ICMP）；Windows 源码包必须含 tsconfig.node.json（曾漏导致 vue-tsc 失败）
