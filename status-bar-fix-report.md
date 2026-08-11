# Claude Code / Codex CLI 状态栏信息丢失 — 原因分析与修复方案

## 问题现象

使用 Claude Code CLI 或 Codex CLI（通过 cc-gate 代理连接 DeepSeek）时，
对话界面底部状态栏本应显示的信息丢失了：

- 模型名（如 `deepseek-v4-pro`）
- Token 用量（如 `12.3K tokens`）
- 项目目录名
- 累计花费（如 `$0.05`）

现在状态栏只剩右侧的 `100% context used`，前面全是空白。

---

## 一、架构概览

cc-gate 启动 3 个 Node.js 代理进程：

| 代理 | 端口 | 协议翻译 | 主要使用者 |
|------|------|----------|------------|
| mimo2codex | 8688 | Responses API → Chat Completions | Codex CLI |
| claude-proxy | 8689 | Anthropic Messages → Chat Completions | Claude Code CLI |
| chat-proxy | 8690 | Chat Completions 直通 | Hermes, OpenCode, Aider 等 |

### Claude Code 链路

```
Claude Code CLI → claude-proxy (:8689) → DeepSeek API
       │                  │
   发送 Anthropic    翻译 Anthropic ↔ OpenAI
   Messages 格式     Chat Completions
```

### Codex CLI 链路

```
Codex CLI → mimo2codex (:8688) → DeepSeek API
       │                  │
   发送 OpenAI        翻译 OpenAI Responses
   Responses API      ↔ Chat Completions
```

---

## 二、Claude Code 问题分析（claude-proxy.js）

### 根因

`~/.mimo2codex/claude-proxy.js` 第 160 行：

```javascript
// anthropicToOpenAI() 函数返回的请求体
const openaiReq = {
    model: anthropicReq.model,
    messages,
    max_tokens: anthropicReq.max_tokens || 4096,
    temperature: anthropicReq.temperature,
    top_p: anthropicReq.top_p,
    stop: anthropicReq.stop_sequences,
    stream: false,   // ← 硬编码关闭流式传输
};
```

Claude Code 默认发送 `stream: true` 的 Anthropic Messages 请求，
期望收到 Anthropic 格式的 SSE（Server-Sent Events）流。

但 claude-proxy 在翻译 Anthropic → OpenAI 请求时，
**强制把 `stream` 设为 `false`**，导致：

1. DeepSeek 返回的是**非流式 JSON**（一次完整响应）
2. claude-proxy 的非流式处理路径（`httpRequest`，第 79-99 行）
   收集整个响应后再调用 `openAIToAnthropic` 翻译成 Anthropic 格式的 JSON
3. Claude Code 收到的是**非流式 Anthropic 响应**

### 为什么状态栏信息丢失

Claude Code 的 UI 更新逻辑绑定在流式事件上：

| 信息 | 数据来源 | 状态栏依赖 |
|------|----------|------------|
| 模型名 | `message_start` 事件中的 `message.model` | 流式事件 |
| Token 数 | `message_start`（input_tokens）+ `message_delta`（output_tokens） | 流式事件 |
| 花费 | 用模型名查内部定价表 × token 数 | 依赖上面两项 |
| context 使用率 | 本地计算（不依赖 API 响应） | **唯一正常显示的** |

当收到非流式 JSON 响应时，这些流式事件不会触发，
Claude Code 不更新状态栏对应字段，只剩本地算的 `100% context used` 能显示。

### 附带问题：模型名不匹配

claude-proxy 的 `/v1/models` 端点（第 287-297 行）返回的模型 ID
带有 `claude-` 前缀（如 `claude-deepseek-v4-pro`），
Claude Code 的内部定价表中没有这个条目，即使有 token 数据也算不出花费。

### 修复方案

**方案一：真正的流式翻译（推荐，彻底解决）**

将 Anthropic SSE 事件和 OpenAI SSE chunk 做双向实时翻译：

```
Claude Code → claude-proxy → DeepSeek
   SSE |                      | SSE
   Anthropic                 OpenAI
   format                    format
```

实现要点：
1. 检测 `stream: true` 的请求
2. 把 Anthropic 请求体翻译成 OpenAI 格式，保留 `stream: true`
3. 用流式方式发送给 DeepSeek
4. 收到 OpenAI SSE chunk 后，实时翻译成 Anthropic SSE 事件：
   - `choices[0].delta.content` → `content_block_delta` 的 `text_delta`
   - `choices[0].delta.tool_calls` → `content_block_start/delta` 的 `tool_use`
   - 最终 chunk 的 `finish_reason` → `message_delta` 的 `stop_reason`
   - 最终 chunk 的 `usage` → `message_delta` 的 `usage`
5. 设置正确的 SSE Content-Type: `text/event-stream`

参考实现模式（mimo2codex 已有类似的翻译逻辑）：
- `iterChatStreamChunks` — 解析上游 SSE
- `pipeChatStreamToResponses` — 翻译并写回客户端 SSE

**方案二：保留 `usage` 但用非流式（简单，但有副作用）**

不改造流式管道，只在现有的非流式响应中确保 `model` 字段使用
去掉 `claude-` 前缀的模型名（如 `deepseek-v4-pro`），以便 Claude Code
能在其定价表中匹配。

但这个方案**不确定能解决状态栏问题**——Claude Code 可能根本不会
从非流式 JSON 响应中更新状态栏。不建议。

**修改文件**

| 文件 | 位置 | 改动 |
|------|------|------|
| `~/.mimo2codex/claude-proxy.js` | cc-gate 的代理脚本 | 第 160 行 + 新增流式翻译函数 |

---

## 三、Codex CLI 问题分析（mimo2codex）

### 根因

mimo2codex **已经正确支持流式转发**。问题在于**发给 DeepSeek 的
请求中缺少一个参数**。

`streamToSse.js`（编译后路径 `dist/translate/streamToSse.js`）第 300-304 行：

```javascript
if (chunk.usage) {
    state.usage = {
        input_tokens: chunk.usage.prompt_tokens,
        output_tokens: chunk.usage.completion_tokens,
        total_tokens: chunk.usage.total_tokens,
    };
}
```

这段代码**等待** DeepSeek 返回的流式 chunk 中出现 `usage` 字段，
用来填充 token 统计。这个 `usage` 最终会放入 `response.completed`
SSE 事件（`buildResponseSnapshot` 函数，第 52 行）。

但 DeepSeek（遵循 OpenAI 兼容 API 规范）在流式模式下
**默认不返回 `usage`**。必须显式传入：

```json
{
  "stream_options": {
    "include_usage": true
  }
}
```

才会在最后一个 SSE chunk 中带上 token 统计。

**整个 mimo2codex 代码中没有任何地方发送 `stream_options`。**

搜索验证：
- `server.js` — 无 `stream_options` 引用
- `reqToChat.js` — 无 `stream_options` 引用
- `providers/deepseek.js` — 无 `stream_options` 引用
- `upstream/openaiCompatClient.js` — 无 `stream_options` 引用

### 为什么状态栏信息丢失

链路分析：

```
1. Codex 发送流式 Responses API 请求 → mimo2codex (:8688)
2. mimo2codex 翻译成 Chat Completions 发给 DeepSeek
   ❌ 缺少 stream_options: { include_usage: true }
3. DeepSeek 返回流式 SSE chunks（无 usage）
4. streamToSse.js 中 chunk.usage 始终为 undefined
5. state.usage 始终为 null
6. response.completed 事件中 usage: null
7. Codex CLI 拿不到 token 数据 → 状态栏显示异常
```

模型名同样：`state.model = req.model`（streamToSse.js 第 28 行），
来自 Codex 传入的原始模型名。如果 Codex 配置文件里 `model = "claude-opus-4-5"`，
Codex 可能识别这个模型但因缺少 usage 而整行渲染异常。

### 修复方案

**方案一：在 mimo2codex 源码中添加 `stream_options`（推荐）**

在 `providers/deepseek.js` 的 `preprocessResponses` 中，
`normalizeDeepseekBody(chat)` 之后添加：

```javascript
// DeepSeek requires explicit opt-in to return usage in streaming
chat.stream_options = { include_usage: true };
```

或者在 `reqToChat.js` 的请求构建函数中统一添加。

**方案二：在 `normalizeDeepseekBody` 中统一处理**

```javascript
function normalizeDeepseekBody(chat) {
    // ... 现有逻辑 ...

    // 确保流式响应包含 token 用量
    if (chat.stream === true || chat.stream === undefined) {
        chat.stream_options = { include_usage: true };
    }
}
```

**方案三：在 `upstream/openaiCompatClient.js` 中全局添加**

在每个 Chat Completions 流式请求中统一注入 `stream_options`。
影响范围更广，会影响所有 Provider（MiMo、GLM、Qwen），
但这些 Provider 通常也遵循同样的规则。

**修改文件**

| 文件 | 路径 | 改动 | 行数 |
|------|------|------|------|
| `deepseek.js` | `dist/providers/deepseek.js` | 添加 `stream_options` | +1 行 |
| 或 `normalizeDeepseekBody` | 同上 | 同上 | +3-8 行 |
| 或 `openaiCompatClient.js` | `dist/upstream/openaiCompatClient.js` | 全局注入 | +3 行 |

**⚠️ 维护注意事项**

mimo2codex 是 npm 全局包，代码在 `~/.nvm/versions/node/v20.20.2/lib/node_modules/mimo2codex/dist/`。
修改的是编译后的 JS 文件。执行 `npm update -g mimo2codex` 会**覆盖**修改，
需要重新应用补丁。

建议策略：
1. 直接向 mimo2codex 上游仓库提交 PR（这是标准的 OpenAI 兼容需求）
2. 在补丁合入前，cc-gate 每次更新时手动重新打补丁
3. 或者 fork mimo2codex 并锁定版本

---

## 四、对比总结

| | Claude Code | Codex CLI |
|---|---|---|
| **代理** | claude-proxy (:8689) | mimo2codex (:8688) |
| **代理文件** | `~/.mimo2codex/claude-proxy.js` | `~/.nvm/…/mimo2codex/dist/` |
| **自维护** | ✅ 是（cc-gate 项目内） | ❌ 否（npm 包，上游维护） |
| **流式可用** | ❌ `stream: false` 硬编码 | ✅ 流式管道正常 |
| **根因** | 强制关闭流式传输 | 缺 `stream_options: {include_usage: true}` |
| **修复难度** | 中等（需实现流式翻译） | 极简单（+1 行） |
| **修复后维护** | 无（自己控制） | npm update 会覆盖 |

---

## 五、修复顺序建议

1. **先修 Codex**（改动最小，1 行代码，立即见效）
2. **再修 Claude Code**（改动较大，需要实现流式翻译管道）
3. **向 mimo2codex 上游提 PR**（一劳永逸解决 npm update 覆盖问题）

---

*文档生成时间：2026-07-27*
*关联项目：cc-gate（`/Users/ami/pro/python/py3/破卷相关/cc-x-llm`）*
