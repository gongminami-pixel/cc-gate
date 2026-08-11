# Bug: providers.json 缺少 `anthropicEndpoint: true`，导致 Claude Opus 5 路由到 OpenAI endpoint (404)

## 症状

- `claude-opus` 别名启动 Claude Code 后，发送消息时报：
  ```
  There's an issue with the selected model (claude-claude-opus-5).
  It may not exist or you may not have access to it.
  ```
- claude-proxy.js 日志显示请求被路由到了 OpenAI 格式：
  ```
  → claude-claude-opus-5 → Claude Opus 5 (translate: https://api.nonelinear.com/anthropic/chat/completions)
  ← claude-claude-opus-5 ERROR 404
  ```
- 但直接 curl 相同 URL 的 `/v1/messages`（Anthropic 格式）返回 200，说明上游 API 正常。

## 根因

`config_writer.rs` 的 `write_providers()` 函数中，判断是否设置 `anthropicEndpoint: true` 的条件有 bug。

### 问题代码

`src-tauri/src/config_writer.rs` 第 190-197 行：

```rust
let is_anthropic_native = provider_id == "anthropic" && (
    routing == "direct" ||
    (routing.starts_with("relay:") && relay_by_name.get(&routing[6..])
        .and_then(|r| r.anthropic_url.as_ref()).is_some())  // ← BUG: 要求 anthropic_url 非空
);
if is_anthropic_native {
    entry["anthropicEndpoint"] = serde_json::json!(true);
}
```

### 为什么错

1. 对于 `provider_id == "anthropic"` 且 `routing == "relay:非线anthropic"` 的情况：
   - 第 154-155 行：base URL 正确使用了 `relay.url`（因为 `anthropic_url` 为空时的 fallback）
   - 但第 193 行：`is_anthropic_native` 判断要求 `relay.anthropic_url` 非空
   - 结果：base URL 正确指向 Anthropic endpoint，但 `anthropicEndpoint` 未设为 `true`

2. 没有 `anthropicEndpoint: true`，claude-proxy.js 将该 provider 当作 OpenAI 格式处理：
   - 请求发送到 `{baseUrl}/chat/completions` 而不是 `{baseUrl}/v1/messages`
   - 上游返回 404

3. 当前 relay 配置中 `anthropic_url` 为 `null`（因为同一个 URL 同时服务 Anthropic 和 OpenAI），触发此 bug。

### 触发条件

- 用 relay 路由 Anthropic 模型
- 该 relay 的 `anthropic_url` 字段为 `null`（只在 `url` 字段填了 Anthropic endpoint）

## 修复

### 临时修复（已应用，会被 cc-gate 重启覆盖）

手动在 `~/.mimo2codex/providers.json` 中给 `claude-opus-5` 的 provider 条目加上：
```json
"anthropicEndpoint": true
```

然后重启 claude-proxy（kill + 重启，或重启 cc-gate）。

### 永久修复（需改 cc-gate 源码）

`config_writer.rs` 第 191-194 行，去掉对 `anthropic_url` 非空的依赖：

**改前：**
```rust
let is_anthropic_native = provider_id == "anthropic" && (
    routing == "direct" ||
    (routing.starts_with("relay:") && relay_by_name.get(&routing[6..])
        .and_then(|r| r.anthropic_url.as_ref()).is_some())
);
```

**改后：**
```rust
let is_anthropic_native = provider_id == "anthropic" && (
    routing == "direct" ||
    routing.starts_with("relay:")
);
```

逻辑：任何 Anthropic provider 通过 relay 路由时，始终是 Anthropic-native passthrough，不管 relay 是否有独立的 `anthropic_url`。base URL 已经在第 154-155 行正确 fallback 到 `relay.url` 了。

### 影响范围

只影响 `config_writer.rs` 中的 `write_providers()`，不影响其他函数。修改后需要重新构建 cc-gate。

## 验证

修复后，providers.json 中 Anthropic relay 条目应包含 `"anthropicEndpoint": true`：

```json
{
  "id": "anthropic-relay-非线anthropic",
  "name": "Anthropic Opus via 非线anthropic",
  "baseUrl": "https://api.nonelinear.com/anthropic",
  "envKey": "RELAY_X975EX7EBFANTHROPIC_API_KEY",
  "anthropicEndpoint": true,   // ← 这一行
  "defaultModel": "claude-opus-5",
  "models": [...]
}
```

claude-proxy.js 日志应显示 `passthrough` 而非 `translate`：
```
→ claude-claude-opus-5 → Claude Opus 5 (passthrough: https://api.nonelinear.com/anthropic)
```
