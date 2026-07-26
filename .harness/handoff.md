# Harness Handoff

_Last updated: 2026-07-26T02:15:00+08:00_
_Harness version: harness-framework v1_

## Goal

构建 CC-Gate，一个 Tauri 2 + Vue 3 桌面应用，替代 CC Switch，统一管理 10 个 AI Agent 的模型配置、中转站、API Key、Shell Alias、用量统计和代理进程。已开源在 https://github.com/gongminami-pixel/cc-gate。

## State snapshot

- **Branch**: main
- **Commit**: 547a42e feat: 远程模型目录自动更新 + 侧边栏隐藏用量/模型参数
- **Uncommitted**: src-tauri/gen/schemas/windows-schema.json (untracked, auto-generated)
- **In-progress files**: 无

## Context you must load (JIT)

- `models-catalog.json` — 远程模型目录的源（9 个模型，GitHub raw 可访问）
- `src-tauri/src/model_catalog.rs` — 远程拉取+缓存+合并逻辑
- `src-tauri/src/config_writer.rs` — 配置写入核心
- `src-tauri/src/types.rs` — 数据模型（10 Agent + ModelDef + RelayConfig）
- `DESIGN.md` — 设计文档

## What works

- ✅ 10 Agent 全部有配置写入（Cursor 除外）
- ✅ 远程模型目录自动更新：GitHub raw → 本地缓存 → 合并到用户配置
- ✅ 启动时后台静默拉取，有新模型自动合并入本地
- ✅ 首页"检查模型更新"按钮（model header 右侧），新模型"新"badge
- ✅ 侧边栏隐藏用量统计/模型参数（代码保留未删）
- ✅ 用量记录逻辑已禁用（两个 proxy .js 中 recordUsage 注释掉）
- ✅ providers.json 分组（provider_id, routing），direct/relay 两模式
- ✅ Shell alias 自动生成，跨平台（bash/zsh/PS5/PS7）
- ✅ Relay 中转站 + API Key 管理（22 提供商）
- ✅ 启动项管理（3 代理 + App 自启动，macOS launchd）
- ✅ 工具检测独立页 + OnceLock 缓存 + 刷新按钮
- ✅ 首页应用按钮 dirty-aware
- ✅ Hermes / OpenClaw / Reasonix 配置写入

## What's broken

- ⚠️ Cursor：专有 API，不兼容标准接口
- ⚠️ OpenCode：项目级配置，无全局文件可注入
- ⚠️ 当前对话走 claude-proxy，点"应用"会断连

## Next actions (ordered)

1. 用户测试"检查模型更新"确认 404 已修复
2. 双端构建（Mac + Windows）
3. 用户测试各 Agent 模型切换
4. OpenCode 配置策略调研
5. 下载页部署（三端构建）

## Open questions

- OpenCode 项目级配置如何自动注��？
- OpenClaw 配置兼容性需用户验证
- 下载页部署目标服务器？

## Beware

- 代理重启 = 杀死本对话（claude-proxy 是当前通道）
- JSON5 解析简化版（仅剥 // 注释和尾逗号）
- 远程 catalog URL: `https://raw.githubusercontent.com/gongminami-pixel/cc-gate/main/models-catalog.json`
- builtin_models() 保留作终极兜底（缓存没有 + 远程失败时用）
- 新模型从远程合并时：参数以远程为准，但保留用户本地 enabled 状态
- `models-catalog.json` 更新后只需 git push，所有 CC-Gate 实例下次启动/手动刷新就能拿到
