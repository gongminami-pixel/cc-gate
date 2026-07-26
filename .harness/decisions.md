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

## 2026-07-26T01:30:00+08:00 — 远程模型目录自动更新（解决厂商出新模型需重编 CC-Gate 的问题）
**Why**: 用户问"厂商出新模型咱们的程序还起作用么"——builtin_models() 硬编码 9 个模型，新模型必须改代码 + 重编 + 出包。用户要求不依赖 CC-Gate 发版就能跟上厂商更新
**What**: 
  - models-catalog.json 放仓库根目录 → GitHub raw URL 可访问
  - 新增 model_catalog.rs：fetch_remote_catalog (reqwest HTTPS) + 本地缓存 (~/.mimo2codex/models-cache.json) + merge_remote_models
  - merge 策略：远程参数覆盖本地（context_window/pricing 等），但保留用户 enabled 状态；远程新模型默认 enabled=false
  - 启动时后台静默拉取（不阻塞 UI）
  - 首页"检查模型更新"按钮供用户主动刷新
  - 离线兜底链：缓存 → builtin_models()
**Alternatives**: 
  - 不从远程拉，纯依赖定���发版更新 builtin_models() ——太重
  - 从代理 /v1/models 动态发现——代理端 /v1/models 返回不完整，且 Rust 侧没有消费代码
  - JSON 放独立仓库——当前放主仓库根目录，简单够用
**Evidence**: models-catalog.json, src-tauri/src/model_catalog.rs, src-tauri/src/config_store.rs:13-31
**Supersedes**: builtin_models() 作为唯一模型源（现降级为兜底）
**Impact**: 厂商出新模型只需改 models-catalog.json + git push；所有 CC-Gate 实例自动获取。builtin_models() 保留不动作终极兜底

## 2026-07-26T01:30:00+08:00 — 侧边栏隐藏用量统计和模型参数
**Why**: 用户说模型参数不准确（context_window 等没校准），用量统计逻辑也未启用
**What**: 注释掉 Sidebar.vue 中 usage 和 models 两个菜单项；两个 proxy .js 中 recordUsage() 调用注释掉。代码保留不删，以便将来校准后恢复
**Alternatives**: 删除代码——将来恢复需从 git history 找回
**Evidence**: src/components/Sidebar.vue:9-10, claude-proxy.js:364,390, chat-proxy.js:291
**Supersedes**: -
**Impact**: 用户看不到这两个菜单项；用量 jsonl 不再写入（节省磁盘 I/O+隐私）
