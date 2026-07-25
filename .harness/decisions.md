# Harness Decisions Ledger

_Append-only. Each entry captures the "why" behind a choice._

---

## 2026-07-25T22:45:00+08:00 — 启用 harness-framework
**Why**: 跨对话/跨模型/跨上下文压缩的零漂移长记忆
**What**: 建立 .harness/ 通过 harness-framework skill 管理会话状态
**Alternatives**: 纯 CLAUDE.md + auto-memory — 缺结构化会话级状态
**Evidence**: -
**Supersedes**: -
**Impact**: 后续会话通过"读取记忆"等触发词 boot

## 2026-07-25T23:00:00+08:00 — 工具检测挪到左侧独立菜单项
**Why**: 用户认为放在首页底部不合适，应该像"启动项"一样独立
**What**: 新建 PageTools.vue，左侧菜单加"🔧 工具检测"项（启动项上面）
**Alternatives**: 保留在首页底部卡片、做成弹窗
**Evidence**: src/components/PageTools.vue, src/components/Sidebar.vue:12
**Supersedes**: -
**Impact**: 首页不再堵塞等待工具检测；用户点左侧菜单主动查看

## 2026-07-25T23:00:00+08:00 — 工具检测加 OnceLock 缓存
**Why**: 每次进首页都跑 6 个外部命令卡 2 秒，工具不会频繁装卸
**What**: Rust 侧 OnceLock<Mutex<Vec<ToolStatus>>>，首次跑后缓存
**Alternatives**: 前端 localStorage 缓存 -- 不准确（前端不知道真实状态）
**Evidence**: src-tauri/src/tool_check.rs:19-35
**Supersedes**: -
**Impact**: 后续调用无 IO 开销；提供 refresh() 手动清缓存

## 2026-07-25T23:00:00+08:00 — 首页应用按钮 dirty-aware
**Why**: 用户要求"只有改了设置才能点，没改就灰着"
**What**: computed dirty 对比 workingModels/modelRouting 与原始 config
**Alternatives**: 手动 watch 各个字段 -- 太碎
**Evidence**: src/components/PageHome.vue:22-38
**Supersedes**: -
**Impact**: 用户视觉上知道是否需要点应用

## 2026-07-25T23:00:00+08:00 — Codex CLI / Claude CLI / Aider writes_providers=true
**Why**: CLI Agent 的模型必须进 providers.json，否则代理不认识、切换模型时报 stream disconnected
**What**: 三个 CLI Agent 的 writes_providers 改为 true
**Alternatives**: 只靠桌面端的 writes_providers -- CLI 模型不完整
**Evidence**: src-tauri/src/types.rs:36,38,43
**Supersedes**: -
**Impact**: /model 命令能看到完整模型列表，切换不断流

## 2026-07-25T23:00:00+08:00 — 新增 Hermes + OpenClaw 配置写入
**Why**: 用户确认这两个工具可以写全局配置
**What**: write_hermes_config (serde_yaml 合并) + write_openclaw_config (JSON5 兼容)
**Alternatives**: 不做 -- 用户需手动配
**Evidence**: src-tauri/src/config_writer.rs:437-575
**Supersedes**: -
**Impact**: Hermes custom_providers 和 OpenClaw models.providers 自动管理
