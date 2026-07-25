# Harness Handoff

_Last updated: 2026-07-25T23:00:00+08:00_
_Harness version: harness-framework v1_

## Goal

构建 CC-Gate（原名 cc-x-llm），一个 Tauri 2 + Vue 3 桌面应用，替代 CC Switch，统一管理 10 个 AI Agent 的模型配置、中转站、API Key、Shell Alias、用量统计和代理进程。

## State snapshot

- **Branch**: main（首次初始化）
- **Commit**: 待首次提交
- **Uncommitted**: 全部文件（首次 git init）
- **In-progress files**:
  - `src-tauri/src/config_writer.rs` — 新增 Hermes/OpenClaw 配置写入，修正 Reasonix/Codex CLI/Aider 的 writes_providers
  - `src-tauri/src/types.rs` — 修正 codex_cli/claude_cli/aider/reasonix 的 writes_providers/writes_catalog
  - `src-tauri/src/tool_check.rs` — 工��检测缓存（OnceLock），新增 refresh()
  - `src-tauri/src/paths.rs` — 新增 openclaw_config_json() 路径
  - `src-tauri/src/error.rs` — 新增 Yaml(AppError) variant
  - `src-tauri/Cargo.toml` — 新增 serde_yaml 依赖
  - `src/components/PageTools.vue` — 新建独立工具检测页
  - `src/components/PageHome.vue` — 移除工具检测卡片，应用按钮 dirty-aware

## Context you must load (JIT)

- `当前工作_TODO.md` — 项目开发 TODO
- `DESIGN.md` — 设计文档
- `src-tauri/src/config_writer.rs` — 配置写入核心逻辑
- `src-tauri/src/types.rs` — 数据模型（10 Agent + ModelDef + RelayConfig）

## What works

- ✅ 10 Agent 全部有配置写入（Cursor 除外）
- ✅ providers.json 分组（provider_id, routing），支持 direct/relay
- ✅ Shell alias 自动生成，跨平台（macOS/Linux/PowerShell）
- ✅ Relay 中转站管理 + API Key 管理（22 提供商）
- ✅ 用量统计（8 时段分桶）
- ✅ 启动项管理（3 代理 + App 自启动）
- ✅ 工具检测独立页 + 缓存 + 刷新
- ✅ 首页应用按钮 dirty-aware
- ✅ Hermes config.yaml 写入（serde_yaml 解析合并）
- ✅ OpenClaw openclaw.json 写入（JSON5 兼容合并）
- ✅ Reasonix 共享 Codex 配置

## What's broken / blocked

- ⚠️ Cursor：专有 API，不兼容标准接口
- ⚠️ OpenCode：项目级配置，无全局文件
- ⚠️ 当前对话走 claude-proxy，不能点"应用"（会断连）

## Next actions (ordered)

1. 双端构建（Mac + Windows）
2. 首次 git 提交
3. 用户测试：退出对话 → CC-Gate 点应用 → 测各 Agent
4. OpenCode 配置策略调研
5. 三端构建（需下载页方案）

## Open questions

- OpenCode 项目级配置如何自动注入？
- OpenClaw 配置兼容性需用户验证
- 下载页部署目标服务器？

## Beware

- 代理重启 = 杀死本对话（claude-proxy 是当前通道）
- config_writer.rs JSON5 解析是简化版（仅剥 // 注释和尾逗号）
- Hermes YAML 完整解析合并，不破坏用户手写配置
- 工具检测缓存 Rust 侧 OnceLock，进程存活期一次
- Model catalog 合并 Codex CLI + Desktop + Reasonix 三方模型
