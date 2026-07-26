# Harness Handoff

_Last updated: 2026-07-26T12:15:00+08:00_

## Goal

构建 CC-Gate — Tauri 2 + Vue 3 桌面应用，统一管理 10 AI Agent 的模型/路由/Key/Shell。开源在 https://github.com/gongminami-pixel/cc-gate。

## State snapshot

- **Branch**: main
- **Commit**: 1588cb6 chore: 发布脚本
- **Uncommitted**: 无
- **Tag**: v0.1.0

## Context you must load (JIT)

- `models-catalog.json` — 远程模型目录源（9 个模型，GitHub raw）
- `src-tauri/src/model_catalog.rs` — 远程拉取+缓存+合并逻辑
- `src-tauri/src/tool_check.rs` — check_one() 逐个检测 + save_to_cache()
- `src-tauri/src/types.rs` — 数据模型（10 Agent + 9 ModelDef + RelayConfig）
- `src-tauri/src/config_writer.rs` — 配置写入核心
- `README.md` — 开源首页说明文档
- `scripts/release.sh` — GitHub Release 发布脚本

## What works

- ✅ 10 Agent 全配置���入（Cursor 除外）
- ✅ **工具检测渐进式加载**——前端逐个调用 checkOneTool()，6 条逐一亮起
- ✅ **远程模型目录自动更新**——GitHub raw → 本地缓存 → 合并
- ✅ 首页"检查模型更新"按钮 + 新模型"新"badge
- ✅ providers.json 分组路由（direct/relay）
- ✅ Shell alias 自动注入（bash/zsh/PS5/PS7）
- ✅ Relay 中转站 + API Key 管理（22 提供商）
- ✅ 启动项管理（3 代理 + App，macOS launchd）
- ✅ Hermes / OpenClaw / Reasonix 配置写入
- ✅ macOS + Windows 双端构建
- ✅ **模型参数已更新**：Opus 5 (1M), GPT-5.6, GLM 1M
- ✅ v0.1.0 tag 已推送 GitHub

## What's broken

- ⚠️ Cursor：专有 API
- ⚠️ OpenCode：项目级配置
- ⚠️ GitHub Release 安装包尚未上传（需跑 release.sh）
- ⚠️ 当前对话走 claude-proxy，点"应用"会断连

## Next actions (ordered)

1. 用户跑 `bash scripts/release.sh` 上传双端安装包到 GitHub Releases
2. 模型参数进一步校准（qwen3.8 上下文、mimo 参数）
3. OpenCode 配置策略调研
4. 用户测试各 Agent 模型切换

## Beware

- 代理重启 = 杀死本对话（claude-proxy 是当前通道）
- 远程 catalog URL: `https://raw.githubusercontent.com/gongminami-pixel/cc-gate/main/models-catalog.json`
- `check_one_tool()` command 按名匹配：`"node"`, `"python3"`, `"codex"`, `"claude"`, `"aider"`, `"bash"`
- release.sh 需要 GitHub token 在 macOS 钥匙串（`security find-internet-password -s github.com`）
- `short()` 别名映射已更新：opus → `claude-opus-5`, gpt → `gpt-5.6`
