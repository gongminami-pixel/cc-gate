# cc-x-llm — 多模型 AI 工具配置管理器

## 项目定位

替代 CC Switch，提供一个更轻量、更透明的 GUI 工具，统一管理本地 AI 编程工具的模型配置。核心能力：

1. **CLI 一键配置** — 自动生成 codex-ds/glm/qwen/mimo 和 claude-ds/glm/qwen/mimo 等终端 alias
2. **桌面端对话内切模型** — ChatGPT/Codex 桌面端和 Claude Desktop 都能在对话中用 `/model` 切换不同厂商的大模型
3. **全工具覆盖** — Codex / Claude Code / Hermes / OpenCode / OpenClaw 及各自桌面端，一套配置全管
4. **用量统计与预算管控** — 按模型/厂商/日期统计 token 消耗和费用，超限自动拦截
5. **代理管理** — 管理 3 个本地代理的启停和健康检查

---

## 一、现有架构分析

### 1.1 整体数据流

```
┌──────────────────────────────────────────────────────────────────────┐
│                          用户交互层                                    │
│                                                                       │
│  Codex CLI    ChatGPT桌面端   Claude CLI   Claude Desktop             │
│  Hermes       OpenCode        OpenClaw     Aider                      │
└──┬──────────────┬─────────────────┬──────────────────┬───────────────┘
   │              │                 │                  │
   │ Responses    │ Chat Completions│ Anthropic        │ Chat Completions
   │ API          │ (标准)          │ Messages API     │ (标准)
   ▼              ▼                 ▼                  ▼
┌──────────┐ ┌──────────┐ ┌──────────────┐ ┌──────────────────┐
│mimo2codex│ │chat-proxy│ │claude-proxy  │ │  chat-proxy       │
│:8688     │ │:8690     │ │.js :8689     │ │  :8690 (同上)     │
│Responses │ │Chat 透传 │ │Anthropic→Chat│ │                   │
│→Chat     │ │+ 用量管控│ │+ 用量管控    │ │                   │
└────┬─────┘ └────┬─────┘ └──────┬───────┘ └──────┬───────────┘
     │            │              │                │
     └────────────┼──────────────┼────────────────┘
                  │              │
          Chat Completions API   │
                  ▼              ▼
     ┌──────────────────────────────────────────┐
     │              上游模型厂商                   │
     │   DeepSeek │ 智谱GLM │ 阿里Qwen │ 小米MiMo │
     └──────────────────────────────────────────┘
```

**三个代理，按协议分组：**

| 端口 | 代理 | 协议翻译 | 服务对象 |
|------|------|----------|----------|
| 8688 | mimo2codex | Responses API → Chat Completions | Codex CLI / ChatGPT桌面端 / Reasonix |
| 8689 | claude-proxy.js | Anthropic Messages → Chat Completions | Claude Code CLI / Claude Desktop |
| 8690 | chat-proxy（自建） | Chat Completions 透传 + 用量管控 | Hermes / OpenCode / OpenClaw / Aider |

核心原则：**按协议分组，不是按工具分组**。同一种 API 协议的工具共用一个代理。不存在 10 个端口——3 个覆盖全部。

### 1.2 关键配置文件

| 文件 | 用途 | 谁写谁读 |
|------|------|----------|
| `~/.mimo2codex/providers.json` | 供应商/模型路由配置 | cc-x-llm 写，3 个代理都读 |
| `~/.mimo2codex/.env` | 各厂商 API 密钥 | cc-x-llm 写，3 个代理都读 |
| `~/.codex/config.toml` | Codex CLI/桌面端配置（base_url→8688） | cc-x-llm 写，Codex 读 |
| `~/.codex/cc-switch-model-catalog.json` | 模型目录（所有可用模型的定义） | cc-x-llm 写，Codex 桌面端 `/model` 读 |
| `~/.claude/settings.json` | Claude Code 配置（base_url→8689） | cc-x-llm 写，Claude 读 |
| `~/.zshrc` | Shell alias（codex-ds/claude-ds 等） | cc-x-llm 写 |
| `~/.hermes/config.yaml` | Hermes Agent 配置（base_url→8690） | cc-x-llm 写，Hermes 读 |
| `~/.config/opencode/config.toml` | OpenCode 配置 | cc-x-llm 写，OpenCode 读 |
| OpenClaw 配置 | 按 OpenClaw 自身格式 | cc-x-llm 写 |
| `~/.cc-x-llm/usage.db` | 用量 + 预算限额数据库 | 代理写，GUI 读 |

### 1.3 CC Switch 做了什么

CC Switch 是一个菜单栏 GUI 应用，核心功能：

1. **供应商管理** — 在 SQLite 数据库中存储多个供应商配置（API URL、密钥、模型目录）
2. **配置同步** — 把当前激活的供应商配置写入 `config.toml` 和模型目录文件
3. **模型目录维护** — 维护 `cc-switch-model-catalog.json`，控制 `/model` 菜单显示哪些模型
4. **用量统计** — 内置代理（端口 15721）拦截请求，记录 token 和费用
5. **健康检查** — stream check、provider health 监控

**在我们的代理架构下，CC Switch 其实只做了第 2、3 两件事**。因为路由和鉴权已经由 mimo2codex/claude-proxy 处理，CC Switch 的内置代理（15721）和用量统计是冗余的。

### 1.4 已验证的踩坑清单

以下是从实际配置中踩出来的技术细节，必须在新工具中正确处理：

#### (A) Codex 桌面端模型列表为空
- 原因：CC Switch 当前供应商的 modelCatalog 没有模型，或 config.toml 与 CC Switch 数据库不一致
- 解决：config.toml 中 `requires_openai_auth = true`（CC Switch 要求），`name` 字段与供应商匹配

#### (B) 推理强度缺少 xhigh（显示只有 3 级而非 5 级）
- 原因：`supported_reasoning_levels` 中 `effort` 用了 `"max"`
- 解决：OpenAI 桌面端只认 `"xhigh"`（参考 `~/.codex/models_cache.json` OpenAI 官方缓存）
- 需同时更新 `cc-switch-model-catalog.json` 文件和 CC Switch DB 中的 modelCatalog
- 5 级应为：`none`, `low`, `medium`, `high`, `xhigh`

#### (C) Claude Code 只接受 `claude-` 前缀的模型 ID
- Claude Code 的 gateway model discovery 只认 `id` 以 `claude-` 或 `anthropic-` 开头的模型
- claude-proxy.js 的 `/v1/models` 必须返回 `claude-deepseek-v4-pro` 这类 ID
- 消息路由时自动 strip `claude-` 前缀

#### (D) Claude Code 安全分类器跨厂商串扰
- Claude Code 的安全分类器使用 SONNET 层级模型
- 如果 claude-ds 的 SONNET 指向 GLM，GLM 挂了 → 所有 alias 都无法 Edit
- 解决：每个 alias 的 DEFAULT_* 全部用同一厂商模型

#### (E) Claude Code 需要 `ANTHROPIC_DEFAULT_*_MODEL` 全套变量
- 只设 `ANTHROPIC_MODEL` 没用——Claude Code 按 tier（opus/sonnet/haiku/fable）读取对应变量
- 必须设齐：OPUS、SONNET、HAIKU、FABLE

#### (F) `requires_openai_auth` 的陷阱
- `false`：CLI 正常工作（不校验 API key）
- `true`：桌面端配合 CC Switch 的 auth 配置工作
- 如果桌面端用了 `false` 而 CC Switch 期望 `true`，模型验证可能失败

#### (G) CC Switch 数据库字段名 vs 文件字段名
- CC Switch DB 的 modelCatalog：`model`（不是 `slug`）、`displayName`（不是 `display_name`）
- `cc-switch-model-catalog.json` 文件：`slug`、`display_name`
- 桌面端可能读任意一个来源

#### (H) mimo2codex provider ID 冲突
- `mimo` 是内置保留 ID，不能用 → 改用 `xiaomi`
- 如果 providers.json 中多个 provider 有同名 model → 先匹配到的优先

---

## 二、技术方案设计

### 2.1 核心思路：绕过"供应商"概念，纯模型名路由

现有架构中，mimo2codex 和 claude-proxy 已经实现了**按模型名自动路由**——看到 `deepseek-v4-pro` 走 DeepSeek，看到 `glm-5.2` 走智谱。所以不需要"切换供应商"这个概念。

cc-x-llm 只需要做一件事：**把全部模型写进模型目录文件**，让桌面端和 CLI 都能看到所有模型。切换就是选模型名，代理自动路由。

**用户可见的模型由用户自己选择**：cc-x-llm 维护一个完整的模型定义库（所有支持的厂商模型），但最终写入模型目录的只有用户勾选的那些。用户不用 Glm 就不勾选，模型列表就不出现它。不影响代理路由——代理还是认识所有模型，只是不在 UI 里显示。

### 2.2 需要写入的文件

#### (a) `~/.codex/config.toml`（Codex 全局配置）

```toml
model_provider = "custom"
model = "deepseek-v4-pro"          # 默认模型
model_reasoning_effort = "high"
model_context_window = 1000000
model_max_output_tokens = 393216
model_catalog_json = "cc-switch-model-catalog.json"

[model_providers.custom]
name = "cc-x-llm"                   # 供应商名字，随便
base_url = "http://127.0.0.1:8688/v1"   # 指向 mimo2codex
wire_api = "responses"
requires_openai_auth = true
```

关键点：
- `base_url` 指向 mimo2codex 而非 CC Switch 的 :15721
- `model_catalog_json` 指向包含全部模型的目录文件
- `requires_openai_auth = true` 桌面端需要

#### (b) `~/.codex/cc-switch-model-catalog.json`（模型目录）

需要包含全部 4 个厂商的模型，每个模型完整字段：

```json
{
  "models": [
    {
      "slug": "deepseek-v4-pro",
      "display_name": "DeepSeek V4 Pro",
      "context_window": 1000000,
      "max_context_window": 1000000,
      "effective_context_window_percent": 95,
      "default_reasoning_level": "high",
      "default_reasoning_summary": "none",
      "input_modalities": ["text"],
      "supported_reasoning_levels": [
        {"effort": "none", "description": "Disable Thinking"},
        {"effort": "low", "description": "Low"},
        {"effort": "medium", "description": "Medium"},
        {"effort": "high", "description": "High"},
        {"effort": "xhigh", "description": "Extra high"}
      ],
      "supports_reasoning_summaries": true,
      "supports_parallel_tool_calls": false,
      "supports_search_tool": false,
      "support_verbosity": false,
      "supported_in_api": true,
      "shell_type": "shell_command",
      "apply_patch_tool_type": "freeform",
      "visibility": "list",
      "priority": 100,
      "additional_speed_tiers": [],
      "service_tiers": [],
      "experimental_supported_tools": [],
      "truncation_policy": {"mode": "bytes", "limit": 10000}
    }
    // ... 其他模型
  ]
}
```

**当前模型列表：**

| slug | display_name | context | output | priority |
|------|-------------|---------|--------|----------|
| deepseek-v4-pro | DeepSeek V4 Pro | 1M | 384K | 100 |
| glm-5.2 | GLM-5.2 | 128K | 16K | 200 |
| qwen3.8-max-preview | Qwen3.8 Max Preview | 1M | 64K | 300 |
| mimo-v2.5-pro | MiMo V2.5 Pro | 128K | 16K | 1000 |
| mimo-v2.5 | MiMo V2.5 | 1M | - | 1001 |

#### (c) `~/.claude/settings.json`（Claude Code 配置）

```json
{
  "env": {
    "ANTHROPIC_BASE_URL": "http://127.0.0.1:8689"
  },
  "model": "opus",
  "effortLevel": "xhigh"
}
```

注意：settings.json 只设 `ANTHROPIC_BASE_URL` 指向 claude-proxy，不设具体模型变量。具体模型通过 alias 的环境变量注入。

#### (d) `~/.zshrc` Shell Aliases（CLI）

**Codex CLI：**

```bash
alias codex-ds='codex --dangerously-bypass-approvals-and-sandbox -c model="deepseek-v4-pro" -c model_context_window=1000000 -c model_max_output_tokens=393216'
alias codex-glm='codex --dangerously-bypass-approvals-and-sandbox -c model="glm-5.2" -c model_context_window=131072 -c model_max_output_tokens=16384'
alias codex-qwen='codex --dangerously-bypass-approvals-and-sandbox -c model="qwen3.8-max-preview" -c model_context_window=1048576 -c model_max_output_tokens=65536'
alias codex-mimo='codex --dangerously-bypass-approvals-and-sandbox -c model="mimo-v2.5-pro" -c model_context_window=131072 -c model_max_output_tokens=16384'
```

**Claude Code CLI：**

```bash
alias claude-ds='ANTHROPIC_BASE_URL="http://127.0.0.1:8689" \
  ANTHROPIC_AUTH_TOKEN=proxy \
  ANTHROPIC_MODEL="claude-deepseek-v4-pro" \
  ANTHROPIC_DEFAULT_OPUS_MODEL="claude-deepseek-v4-pro" \
  ANTHROPIC_DEFAULT_SONNET_MODEL="claude-deepseek-v4-pro" \
  ANTHROPIC_DEFAULT_HAIKU_MODEL="claude-deepseek-v4-flash" \
  ANTHROPIC_DEFAULT_FABLE_MODEL="claude-deepseek-v4-pro" \
  CLAUDE_CODE_ENABLE_GATEWAY_MODEL_DISCOVERY=1 \
  claude --dangerously-skip-permissions'

alias claude-glm='ANTHROPIC_BASE_URL="http://127.0.0.1:8689" \
  ANTHROPIC_AUTH_TOKEN=proxy \
  ANTHROPIC_MODEL="claude-glm-5.2" \
  ANTHROPIC_DEFAULT_OPUS_MODEL="claude-glm-5.2" \
  ANTHROPIC_DEFAULT_SONNET_MODEL="claude-glm-5.2" \
  ANTHROPIC_DEFAULT_HAIKU_MODEL="claude-glm-5.2" \
  ANTHROPIC_DEFAULT_FABLE_MODEL="claude-glm-5.2" \
  CLAUDE_CODE_ENABLE_GATEWAY_MODEL_DISCOVERY=1 \
  claude --dangerously-skip-permissions'

alias claude-qwen='ANTHROPIC_BASE_URL="http://127.0.0.1:8689" \
  ANTHROPIC_AUTH_TOKEN=proxy \
  ANTHROPIC_MODEL="claude-qwen3.8-max-preview" \
  ANTHROPIC_DEFAULT_OPUS_MODEL="claude-qwen3.8-max-preview" \
  ANTHROPIC_DEFAULT_SONNET_MODEL="claude-qwen-max" \
  ANTHROPIC_DEFAULT_HAIKU_MODEL="claude-qwen-max" \
  ANTHROPIC_DEFAULT_FABLE_MODEL="claude-qwen3.8-max-preview" \
  CLAUDE_CODE_ENABLE_GATEWAY_MODEL_DISCOVERY=1 \
  claude --dangerously-skip-permissions'

alias claude-mimo='ANTHROPIC_BASE_URL="http://127.0.0.1:8689" \
  ANTHROPIC_AUTH_TOKEN=proxy \
  ANTHROPIC_MODEL="claude-mimo-v2.5-pro" \
  ANTHROPIC_DEFAULT_OPUS_MODEL="claude-mimo-v2.5-pro" \
  ANTHROPIC_DEFAULT_SONNET_MODEL="claude-mimo-v2.5-pro" \
  ANTHROPIC_DEFAULT_HAIKU_MODEL="claude-mimo-v2.5-pro" \
  ANTHROPIC_DEFAULT_FABLE_MODEL="claude-mimo-v2.5-pro" \
  CLAUDE_CODE_ENABLE_GATEWAY_MODEL_DISCOVERY=1 \
  claude --dangerously-skip-permissions'
```

**Aider CLI：**

```bash
alias aider-ds='OPENAI_API_BASE=http://127.0.0.1:8688/v1 OPENAI_API_KEY=proxy aider --model openai/deepseek-v4-pro'
alias aider-glm='OPENAI_API_BASE=http://127.0.0.1:8688/v1 OPENAI_API_KEY=proxy aider --model openai/glm-5.2'
alias aider-qwen='OPENAI_API_BASE=http://127.0.0.1:8688/v1 OPENAI_API_KEY=proxy aider --model openai/qwen-max'
```

**关键注意事项：**
- Claude Code alias 中每个厂商必须自包含全部 DEFAULT_* 变量（见踩坑 D）
- Claude Code 模型名前面加 `claude-` 前缀才能被 gateway discovery 识别（见踩坑 C）
- Claude Code 不在 alias 里用 `--model` 标志——会被校验为非法模型名，必须用环境变量
- Codex 用 `-c` 覆盖 config.toml（-c 优先级更高，config.toml 的默认值作为 fallback）

#### (e) Hermes / OpenCode / OpenClaw（Chat Completions 透传）

这三个工具都使用标准 OpenAI Chat Completions API，不需要协议翻译。统一走 chat-proxy（:8690）做用量统计和预算管控。

**Hermes Agent** (`~/.hermes/config.yaml` 或环境变量)：

```yaml
# 在 providers 段配置自定义 provider
providers:
  cc-x-llm:
    base_url: "http://127.0.0.1:8690/v1"
    api_key: "proxy"  # chat-proxy 自己处理上游鉴权
```

Hermes 通过 `/model` 命令或 `-m` 参数切换模型名，chat-proxy 按模型名路由。

**OpenCode** (`~/.config/opencode/config.toml`)：

```toml
[model_providers.cc-x-llm]
base_url = "http://127.0.0.1:8690/v1"
api_key = "proxy"
```

**OpenClaw**：按 OpenClaw 自身配置格式，将 API endpoint 指向 `http://127.0.0.1:8690/v1`。

**Aider** 也走这个代理（虽然 aider 用了 `openai/` 前缀，chat-proxy 对模型名做 normalize）：

```bash
alias aider-ds='OPENAI_API_BASE=http://127.0.0.1:8690/v1 OPENAI_API_KEY=proxy aider --model openai/deepseek-v4-pro'
alias aider-glm='OPENAI_API_BASE=http://127.0.0.1:8690/v1 OPENAI_API_KEY=proxy aider --model openai/glm-5.2'
alias aider-qwen='OPENAI_API_BASE=http://127.0.0.1:8690/v1 OPENAI_API_KEY=proxy aider --model openai/qwen-max'
```

**chat-proxy 实现要点**：

chat-proxy 是最简单的代理——不做协议翻译，纯透传。功能：
1. 接收请求 → 按模型名查 providers.json 路由到上游
2. 返回响应 → 顺手提取 `usage` 写入 usage.db
3. 请求前查 budget_limits，超限返回 429
4. 提供 `/v1/models` 端点列出全部模型

约 200 行 Node.js 代码，几乎就是 mimo2codex 去掉 Responses API 翻译逻辑的简化版。

### 2.3 桌面端对话内切模型原理

#### ChatGPT/Codex 桌面端
- 读取 `config.toml` → 连接 `base_url`（mimo2codex :8688）
- `/model` 命令读取 `model_catalog_json` 指向的文件（`cc-switch-model-catalog.json`）
- 文件包含全部厂商模型 → 用户可选任意模型
- 切换后，Codex 发新模型名给 mimo2codex → 代理按模型名路由到对应厂商
- **不需要 CC Switch！** 只要模型目录文件写对了就行

#### Claude Code 桌面端
- 读取 `settings.json` → `ANTHROPIC_BASE_URL` 指向 claude-proxy (:8689)
- `/model` 命令通过 gateway model discovery 协议查询 proxy 的 `GET /v1/models`
- claude-proxy.js 需要返回所有厂商模型（带 `claude-` 前缀）
- 切换后 Claude Code 发送新模型名 → proxy strip `claude-` 前缀 → 路由到对应厂商
- **不需要 CC Switch！** 只要 proxy 的 `/v1/models` 端点返回全部模型

### 2.4 Claude Code 桌面端的特殊处理

Claude Code 桌面端和 CLI 共享 `~/.claude/settings.json`。但桌面端不能设 alias 环境变量。需要在 settings.json 中配置好所有 tier 的默认模型，使 /model picker 能正常工作。

方案：settings.json 中保持不变（只设 BASE_URL），proxy 的 `/v1/models` 返回全部模型。用户启动桌面端时默认走 settings.json 中 environment 指定的模型（或上次选择的模型），然后通过 /model 切换。

**关键**：claude-proxy.js 的 `/v1/models` 端点必须返回以下格式：

```json
{
  "data": [
    {"id": "claude-deepseek-v4-pro", "type": "model", "display_name": "DeepSeek V4 Pro"},
    {"id": "claude-glm-5.2", "type": "model", "display_name": "GLM-5.2"},
    {"id": "claude-qwen3.8-max-preview", "type": "model", "display_name": "Qwen3.8 Max"},
    {"id": "claude-mimo-v2.5-pro", "type": "model", "display_name": "MiMo V2.5 Pro"}
  ]
}
```

### 2.5 用量统计与预算管控

这是 cc-x-llm 的核心差异化功能：既能看用量，也能控用量。

#### 数据采集方式

**不要在代理外部解析日志**——不可靠、时延大、格式不稳定。正确做法：

**mimo2codex 端**（:8688）：mimo2codex 是 npm 包，可以在它的配置或插件机制中插入一个 hook。如果没有原生 hook，fork mimo2codex 加一个中间件，在响应返回给客户端之前，提取 `usage` 字段写入 SQLite。

**claude-proxy.js 端**（:8689）：这是自己写的 Node.js 脚本，直接在 `proxyRequest()` 函数的响应处理位置加几行代码，提取 token 信息写入同一个 SQLite。

**chat-proxy 端**（:8690）：自建透传代理，直接在请求处理 pipeline 中加记录逻辑，最简单。

**采集时机**：上游返回响应 → 提取 `usage` 对象 → 写入数据库 → 返回给客户端。对用户完全透明，不影响延迟。

#### 上游 API 的 usage 格式

所有厂商的 Chat Completions 响应都遵循 OpenAI 格式：

```json
{
  "id": "chatcmpl-xxx",
  "model": "deepseek-v4-pro",
  "usage": {
    "prompt_tokens": 1234,
    "completion_tokens": 567,
    "total_tokens": 1801
  }
}
```

流式响应（SSE）的最后一个 chunk 也包含 `usage`。代理需要缓存所有 SSE chunk，在最后一个 chunk 到达时提取 usage。

#### 数据库设计

```sql
-- 请求明细表
CREATE TABLE request_logs (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    request_id  TEXT NOT NULL,
    model       TEXT NOT NULL,          -- 模型名（如 deepseek-v4-pro）
    provider    TEXT NOT NULL,          -- 厂商（deepseek/glm/qwen/xiaomi）
    prompt_tokens    INTEGER NOT NULL,
    completion_tokens INTEGER NOT NULL,
    total_tokens     INTEGER NOT NULL,
    latency_ms  INTEGER,               -- 请求延迟
    status_code INTEGER,               -- HTTP 状态码
    error_msg   TEXT,                  -- 错误信息
    cost_usd    REAL NOT NULL DEFAULT 0, -- 费用（美元）
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_logs_model ON request_logs(model);
CREATE INDEX idx_logs_provider ON request_logs(provider);
CREATE INDEX idx_logs_created ON request_logs(created_at);

-- 定价表
CREATE TABLE pricing (
    model_id              TEXT PRIMARY KEY,
    display_name          TEXT NOT NULL,
    input_price_per_1k    REAL NOT NULL,   -- 输入价格（美元/1K tokens）
    output_price_per_1k   REAL NOT NULL    -- 输出价格（美元/1K tokens）
);

-- 预算限额表
CREATE TABLE budget_limits (
    model_id    TEXT PRIMARY KEY,
    daily_limit_usd   REAL,          -- 日限额（美元），NULL=不限制
    monthly_limit_usd REAL           -- 月限额（美元），NULL=不限制
);
```

**费用计算**：`cost_usd = (prompt_tokens/1000 × input_price) + (completion_tokens/1000 × output_price)`

#### 初始定价数据

| model_id | display_name | input/1K | output/1K |
|----------|-------------|----------|-----------|
| deepseek-v4-pro | DeepSeek V4 Pro | $0.0003 | $0.002 |
| deepseek-v4-flash | DeepSeek V4 Flash | $0.0001 | $0.0005 |
| glm-5.2 | GLM-5.2 | $0.0014 | $0.0014 |
| qwen3.8-max-preview | Qwen3.8 Max | $0.0013 | $0.0052 |
| qwen-max | Qwen-Max | $0.003 | $0.012 |
| mimo-v2.5-pro | MiMo V2.5 Pro | $0.0005 | $0.001 |

> 注：价格会变动，GUI 应提供编辑入口。

#### 预算管控实现

**拦截逻辑（伪代码）**：

```javascript
// 在代理的请求处理函数中，转发到上游之前
async function checkBudget(model, estimatedInputTokens) {
  const limit = await db.get(
    'SELECT daily_limit_usd, monthly_limit_usd FROM budget_limits WHERE model_id=?',
    [model]
  );
  if (!limit) return; // 没设限额，放行

  const today = new Date().toISOString().slice(0, 10);
  const thisMonth = today.slice(0, 7);

  // 查今日已用量
  const dailyUsed = await db.get(
    'SELECT COALESCE(SUM(cost_usd), 0) as total FROM request_logs WHERE model=? AND date(created_at)=?',
    [model, today]
  );

  // 查本月已用量
  const monthlyUsed = await db.get(
    'SELECT COALESCE(SUM(cost_usd), 0) as total FROM request_logs WHERE model=? AND strftime("%Y-%m", created_at)=?',
    [model, thisMonth]
  );

  if (limit.daily_limit_usd && dailyUsed.total >= limit.daily_limit_usd) {
    return { blocked: true, reason: `日限额已用完 (${limit.daily_limit_usd})` };
  }
  if (limit.monthly_limit_usd && monthlyUsed.total >= limit.monthly_limit_usd) {
    return { blocked: true, reason: `月限额已用完 (${limit.monthly_limit_usd})` };
  }
  // 放行
}
```

**拦截后的行为**：
- 代理直接返回 HTTP 429 Too Many Requests
- 响应体带上错误信息，告诉用户哪个模型超了、超了多少
- 不转发到上游，不产生费用

#### GUI 展示

**用量统计页面**：

1. **概览卡片** — 今日总费用 / 本月总费用 / 总请求数
2. **按模型分组柱状图** — 横轴模型名，纵轴费用或 token 数
3. **趋势图** — 近 30 天/12 个月费用曲线
4. **详细列表** — 每次请求的时间、模型、token 数、费用、延迟
5. **导出** — CSV 下载

**预算管控页面**：

1. 每个模型一行：模型名、已用/限额（进度条）、日限额输入框、月限额输入框
2. 超额时进度条变红
3. 修改限额实时生效（写 SQLite，代理下次请求就读到新值）

#### 关键设计决策

- **数据库放在代理可写的位置**：建议 `~/.mimo2codex/usage.db`，三个代理进程都能访问。如果用 Electron/Tauri 的 app data 目录，代理进程可能没有写入权限。
- **并发安全**：SQLite 默认串行写。如果三个代理同时写入，需要启用 WAL 模式（`PRAGMA journal_mode=WAL`），支持一写多读。
- **定期清理**：提供"保留最近 N 天数据"设置，防止数据库无限增长。
- **费用估算标记**：费用基于定价表计算，如果定价表过期或不准确，GUI 应标黄提示"费用为估算值"。

---

## 三、GUI 设计

### 3.1 技术选型建议

| 选项 | 优点 | 缺点 |
|------|------|------|
| **Electron + React** | 生态丰富、UI 好看 | 体积大（~150MB） |
| **Tauri + React** | 体积小（~10MB）、Rust 后端 | 生态不如 Electron |
| **SwiftUI 原生 macOS** | 原生体验最好 | 只支持 macOS |
| **Python + tkinter/PyQt** | 开发快 | UI 不好看、打包麻烦 |

**推荐**：Tauri + React（跨平台、体积小、前端生态好）。如果要快速出原型，Electron 也行。

### 3.2 功能模块

1. **仪表盘** — 当前状态概览
   - mimo2codex (:8688) 运行状态 ✅/❌
   - claude-proxy (:8689) 运行状态 ✅/❌
   - chat-proxy (:8690) 运行状态 ✅/❌
   - 当前可用模型列表（5 个模型 × 4 厂商）

2. **模型配置** — 管理模型目录
   - 完整模型库（内置所有厂商模型定义）
   - 勾选启用的模型才写入 `cc-switch-model-catalog.json`
   - 未勾选的模型不在桌面端 `/model` 列表中显示
   - 调整优先级、context window 等参数
   - 一键写入配置文件

3. **Shell 集成** — 管理 CLI alias
   - 一键注入/移除 alias
   - 预览 alias 内容
   - 支持自定义别名

4. **用量统计** — Token 和费用
   - 今日/本周/本月用量
   - 按模型分组柱状图
   - 费用估算

5. **代理管理** — 启停代理
   - 启动/停止 mimo2codex (:8688)
   - 启动/停止 claude-proxy (:8689)
   - 启动/停止 chat-proxy (:8690)
   - 查看日志
   - 开机自启开关（launchd plist × 3）

6. **设置** — API 密钥、偏好
   - 管理各厂商 API key（写入 `~/.mimo2codex/.env`）
   - 代理端口配置
   - 主题/语言

### 3.3 与现有组件的交互

```
cc-x-llm GUI
    │
    ├─ 写文件 ──→ ~/.codex/config.toml
    │             ~/.codex/cc-switch-model-catalog.json
    │             ~/.claude/settings.json
    │             ~/.zshrc
    │             ~/.hermes/config.yaml
    │             ~/.config/opencode/config.toml
    │             ~/.mimo2codex/providers.json
    │             ~/.mimo2codex/.env
    │
    ├─ 管理进程 ──→ mimo2codex    (launchd :8688)
    │               claude-proxy   (launchd :8689)
    │               chat-proxy     (launchd :8690)
    │
    └─ 用量统计 ──→ ~/.mimo2codex/usage.db (SQLite, 3 代理共享写入)
```

---

## 四、实施路线图

### Phase 1：核心配置引擎（无 GUI）
- 模型定义数据结构（JSON Schema）
- 写入 `config.toml`、`cc-switch-model-catalog.json`、`settings.json` 的模块
- Hermes / OpenCode / OpenClaw 配置写入模块
- Shell alias 生成/注入模块（Codex Claude Aider 三套）
- 验证：配置写入后各种工具 CLI 和桌面端正常工作

### Phase 2：代理增强
- 新增 chat-proxy（:8690）—— Chat Completions 透传 + 模型名路由
- claude-proxy.js（:8689）的 `/v1/models` 端返回全部模型（带 `claude-` 前缀）
- mimo2codex（:8688）端口迁移 + provider 配置更新

### Phase 3：用量统计与预算管控
- SQLite 数据库初始化（request_logs / pricing / budget_limits 三张表）
- mimo2codex fork 或 hook：在代理响应阶段提取 usage 写入数据库
- claude-proxy.js 改造：在响应处理中内嵌同样的记录逻辑
- chat-proxy 内嵌同样的记录逻辑（天然支持，自己写的）
- 预算拦截逻辑：请求前查限额，超限返回 429
- 聚合查询 API（按模型/日期/厂商汇总）
- 验证：发几个请求后用 sqlite3 查表确认数据落库正确

### Phase 4：GUI
- Tauri/Electron 项目搭建
- 仪表盘页面
- 模型配置页面
- Shell 集成页面
- 用量统计页面
- 设置页面

### Phase 5：打包发布
- macOS 签名公证
- Windows 安装包
- 自动更新

---

## 五、关键参考文件

| 文件 | 说明 |
|------|------|
| `~/.mimo2codex/providers.json` | mimo2codex 供应商配置参考 |
| `~/.mimo2codex/.env` | API 密钥存储格式 |
| `~/.mimo2codex/claude-proxy.js` | Claude Code 代理实现参考 |
| `~/.codex/config.toml` | Codex 配置文件格式 |
| `~/.codex/cc-switch-model-catalog.json` | 模型目录完整格式 |
| `~/.codex/models_cache.json` | OpenAI 官方模型格式参考（推理等级用 xhigh） |
| `~/.claude/settings.json` | Claude Code 配置文件格式 |
| `~/.cc-switch/cc-switch.db` | CC Switch 数据库 schema 参考（用量表设计） |
| `~/.zshrc` | 现有 alias 格式 |
| `~/Library/LaunchAgents/com.user.mimo2codex.plist` | launchd plist 格式参考 |
| `~/Library/LaunchAgents/com.claude-proxy.plist` | launchd plist 格式参考 |

---

## 六、与 CC Switch 的差异

| | CC Switch | cc-x-llm |
|------|-----------|----------|
| 供应商概念 | 有（每次只能激活一个） | 无（所有模型平等地在一个目录里） |
| 代理端口 | 1 个（15721，内置） | 3 个（8688/8689/8690，按协议分组） |
| 支持工具 | Codex / Claude（桌面端为主） | Codex / Claude / Hermes / OpenCode / OpenClaw（CLI+桌面） |
| 桌面端切模型 | 同供应商内可切，跨供应商需手动切换 | 任意模型随时切换 |
| 用量统计 | 通过内置代理记录 | 代理内嵌记录，直写 SQLite |
| 预算管控 | 无（只有用量查看） | 有（日/月限额，超限返回 429） |
| 配置存储 | SQLite 数据库（CC Switch 独占） | 直接写文件（config.toml 等）+ SQLite 做用量 |
| CLI 支持 | 无（只管桌面端） | 有（自动注入 alias） |
| 复杂度 | 高（多层代理、供应商切换逻辑） | 中（3 代理 + 配置引擎 + 用量） |
