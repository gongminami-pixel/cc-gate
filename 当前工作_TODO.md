# CC-Gate 当前工作 TODO

> 目标：搭建 Tauri + Vue 3 GUI 项目，替代 CC Switch，统一管理 AI 工具模型配置
> 开始时间：2026-07-25

## Phase 进度

- [x] Phase 1: 核心配置引擎 + GUI 骨架 ✅ (2026-07-25)
- [x] Phase 2: chat-proxy + claude-proxy /v1/models 增强 ✅ (2026-07-25)
- [x] Phase 3: 用量统计 SQLite + 代理内嵌记录 ✅ (2026-07-25)
- [x] Phase 4: Agent 分配大模型 + Apply 流程 ✅ (2026-07-25)
  - 首页两列布局：左 Agent 列表 × 右模型复选框
  - 10 个 Agent：Codex CLI/桌面端、Claude CLI/桌面端、Hermes、OpenCode、OpenClaw、Aider、Cursor、Reasonix
  - "应用"按钮：统一写入配置文件 + 自动重启代理
  - 删掉仪表盘，侧边栏：首页 / 模型定义 / Shell / 用量 / 设置
- [x] Phase 5: 打包发布 ✅ (2026-08,已发 0.1.10 → 0.1.15 多个 GitHub Release,macOS DMG + Windows NSIS 双端)
- [x] 代理层 Bug 收尾 ✅ (count_tokens 端点、isAnthropicNative 检测、anthropicEndpoint 路由,commit 6c3cfb6)
- [x] 去非线智能 + 无后缀 alias 官方原生 ✅ (commit 20e1e89:codex/claude/aider 原生,providers.json 仅 4 官方直连,中转预设只留 OpenRouter)
- [ ] 正式 1.0:补自动化测试覆盖 config_writer/UI 层、发布流程脚本化收尾、文档同步

## 关键决策

- 前端：Vue 3（复用 frp-pilot 的 CSS 变量系统）
- 代���生命周期：跟 GUI app 同生共死
- Agent 分配模型：每个 Agent 独立勾选，providers.json 取所有 Agent 的并集
- 重启安全：应用时自动重启所有代理，用户点"应用"即确认
