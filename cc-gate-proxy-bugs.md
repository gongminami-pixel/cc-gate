# cc-gate 代理层已知 Bug 及修复方案

> 2026-07-28，接入非线智能中转站时发现
> **源码位置**：项目根 `claude-proxy.js`（唯一真源，编译期由 `src-tauri/src/config_writer.rs:338` 的 `include_str!` 嵌入二进制）
> **Runtime 副本**：`~/.mimo2codex/claude-proxy.js`（由 `deploy_proxy_scripts()` 首次部署，见下方"六"的重要说明）
> **本文行号基准**：`claude-proxy.js` @ sha256 `f3cb83ee…d0d8f1`（= 项目根 = runtime = `/tmp/claude-proxy-fixed.js`，三份完全一致）

---

## 一、架构回顾

cc-gate 通过 3 个本地 Node.js 代理管理多 provider 路由（端口见 `src-tauri/src/proxy_manager.rs:241-249`）：

- mimo2codex `:8688` — Responses API → Chat Completions
- claude-proxy `:8689` — Anthropic Messages → Chat Completions
- chat-proxy `:8690` — Chat Completions 直通

> **注**：`claude-proxy.js:12` 的内置默认端口是 `8789`，仅在**不带 `--port` 裸跑**时生效。cc-gate 启动时总是显式传 `--port 8689`（`proxy_manager.rs:245`），所以正常路径永远是 8689。手动起修补版时若忘了 `--port`，会监听 8789 而 Claude Code 仍打 8689 → 表现为"改了没用"。

```
Claude Code ──→ 8689 (claude-proxy.js)
                  │
                  ├── 模型名命中硬编码 Anthropic 列表? → 直通 api.anthropic.com (Bug 1)
                  ├── provider.anthropicEndpoint === true? → 直通 provider 端点 (Bug 3)
                  └── 默认 → Anthropic→OpenAI 格式转换 → provider 端点
```

模型名解析路径：

```
model="claude-claude-opus-5" (Claude Code 经 gateway discovery 发现的 ID)
  → modelId.startsWith("claude-") ? slice(7) → "claude-opus-5"   (Bug 2 影响)
  → 硬编码 Anthropic 列表检查                                     (Bug 1)
  → TOKEN_MAP[token] || realModelId → PROVIDERS 查路由
```

---

## 二、Bug 1：硬编码模型直通拦截

**位置**：`claude-proxy.js:617-619`

```javascript
// 当前代码（有问题）
const ANTHROPIC_MODELS = ['claude-opus-5', 'claude-opus-4-5', 'claude-sonnet-5', 'claude-haiku-4-5'];
const isAnthropicNative = ANTHROPIC_MODELS.includes(modelId) || ANTHROPIC_MODELS.includes(realModelId) ||
  realModelId.startsWith('claude-opus-') || realModelId.startsWith('claude-sonnet-') || realModelId.startsWith('claude-haiku-');
if (isAnthropicNative) {
  const clientKey = authHeader || 'no-key';
  // ★ 直接发到 api.anthropic.com，完全跳过 providers.json！
  await streamPassthrough('https://api.anthropic.com/v1/messages', {
    headers: { 'x-api-key': clientKey, 'anthropic-version': '2023-06-01', ... }
  }, reqBody, res);
  return;   // ← 提前 return，后面的 provider 路由永不执行
}
```

**现象**：中转站的 Opus 5 API key 被发给 Anthropic 官方 API，返回 401。

**影响范围**：任何 provider 只要配了 `claude-opus-*` / `claude-sonnet-*` / `claude-haiku-*` 命名的模型，都会被这段硬编码拦截，无法走 providers.json 路由。**注意 `startsWith` 分支比列表更宽**——即便把某个型号从 `ANTHROPIC_MODELS` 数组里删掉，只要前缀仍匹配就照样被拦截，所以两处必须一起改。

**临时缓解**（缩小拦截范围，非根治）：

```javascript
// 1) 从数组中移除会被第三方 provider 使用的模型名
const ANTHROPIC_MODELS = ['claude-opus-4-5', 'claude-sonnet-5', 'claude-haiku-4-5'];

// 2) startsWith 同步收窄，否则第 1 步无效
realModelId.startsWith('claude-opus-4') ||  // 仅匹配 opus-4.x，不匹配 opus-5
realModelId.startsWith('claude-sonnet-') ||
realModelId.startsWith('claude-haiku-');
```

> 这只是把问题从 opus-5 挪走：下次有人给 `claude-sonnet-5` 配中转站，同样的坑会重现。

**根治方案**：反转优先级——**先查 providers.json，查不到才走硬编码直通**。即把 `isAnthropicNative` 判断移到 `PROVIDERS[resolvedModel]` 查找**之后**，仅当 `!provider` 且模型名像 Claude 原生模型时才直通官方。

---

## 三、Bug 2：模型名前缀剥离错误

**位置**：`claude-proxy.js:612`

```javascript
// 当前代码（有问题）
const realModelId = modelId.startsWith('claude-') ? modelId.slice(7) : modelId;
```

**问题分析**：

`/v1/models` 的 gateway discovery（`claude-proxy.js:572`）给**每个** provider 模型都加了 `claude-` 前缀：

```javascript
id: 'claude-' + p.defaultModel,       // claude- prefix required by CC
```

于是同一个前缀承载了两种语义：

| providers.json 中的模型 ID | Gateway 返回 | slice(7) 结果 | 是否正确 |
|------------|-------------|--------------|---------|
| `deepseek-v4-pro` | `claude-deepseek-v4-pro` | `deepseek-v4-pro` | ✅ 正确 |
| `glm-5.2` | `claude-glm-5.2` | `glm-5.2` | ✅ 正确 |
| `claude-opus-5` | `claude-claude-opus-5` | `claude-opus-5` | ✅ 正确 |

**真正出错的场景是绕过 gateway discovery**——直接发送 providers.json 里的原始模型名（curl 直连代理、旧配置、或手写 `ANTHROPIC_MODEL`）：

```javascript
"claude-opus-5".slice(7)  // → "opus-5"  ← 破坏了真正的模型名！
```

**影响**：`realModelId` 变成 `opus-5`，`PROVIDERS` 中找不到 → 落到 `claude-proxy.js:658-662` 的兜底分支，报 `Unknown token: <token>. Use: ds, qwen, glm, mimo`。

> **注意这条错误信息本身有误导性**：它说的是 token 无效，但真实原因往往是 **model 名解析错误**导致 `PROVIDERS[resolvedModel]` 未命中。排查时不要被"Unknown token"带偏去查 key。

**修复**（剥离后回退检查）：

```javascript
let realModelId = modelId.startsWith('claude-') ? modelId.slice(7) : modelId;
// 剥离后在 PROVIDERS 中找不到，但原始 modelId 能找到 → 说明它本身就是真实模型名，撤销剥离
if (!PROVIDERS[realModelId] && PROVIDERS[modelId]) realModelId = modelId;
```

**根本原因**：`claude-` 前缀最初是为 DeepSeek/GLM 等**非 Claude** 模型设计的约定（`claude-deepseek-v4-pro` → `deepseek-v4-pro`），但真正的 Claude 模型名本身就以 `claude-` 开头，产生歧义。

**根治方案**：gateway 改用不与真实模型名冲突的前缀机制（如 `x-` 前缀或 metadata 标记），或代理端统一改为"精确匹配优先 + 前缀剥离兜底"的二级查找。

---

## 四、Bug 3：Anthropic 直通路径 Key 传递错误

**位置**：`claude-proxy.js:680` 和 `:684`（流式 + 非流式两处都要改）

```javascript
// 当前代码（有问题）
if (provider.anthropicEndpoint) {
  await streamPassthrough(provider.baseUrl + '/v1/messages', {
    headers: { 'x-api-key': authHeader || 'no-key', ... }  // ← 用的是客户端 key
  }, reqBody, res);
}
```

vs OpenAI 转换路径（`claude-proxy.js:716` / `:720`，正确的做法）：

```javascript
// OpenAI 转换路径 — 正确使用 provider key
headers: { 'Authorization': `Bearer ${provider.apiKey}` }
```

**现象**：第三方 provider 的 Anthropic 直通路径忽略 providers.json 中 `envKey` 对应的真实 key，转发客户端传来的任意 token → 401。

> 讽刺的是 `claude-proxy.js:665-670` 刚刚检查过 `if (!provider.apiKey) → 500`，确认 key 存在，紧接着却不用它。

**修复**：

```javascript
headers: { 'x-api-key': provider.apiKey || authHeader || 'no-key', ... }
```

**根治方案**：Anthropic 直通与 OpenAI 转换两条路径统一优先使用 `provider.apiKey`。

---

## 五、附加问题：env key 编码限制（真实存在，但触发条件与原记录不同）

**位置**：`claude-proxy.js:22`（`loadEnv()`）

```javascript
const m = line.match(/^(\w+)=(.+)$/);
```

`\w` = `[A-Za-z0-9_]`，不匹配中文。**已实测确认**：

```
RELAY_NL_API_KEY=…                 → MATCH
RELAY_非线ANTHROPIC_API_KEY=…      → NO MATCH   ← 该行被静默丢弃
```

`~/.mimo2codex/.env` 中当前确实存在两个读不到的键：`RELAY_非线OPENAI_API_KEY`、`RELAY_非线ANTHROPIC_API_KEY`。

**但真正的病根在 Rust 侧，且是两处不一致的生成逻辑**：

| 位置 | 生成方式 | 中转站名"非线"的结果 |
|---|---|---|
| `config_writer.rs:195`（写 `.env`）| `relay.name.to_uppercase().replace(' ',"_").replace('-',"_")` — **保留中文** | `RELAY_非线_API_KEY` ← proxy 读不到 |
| `config_writer.rs:128`（写 providers.json `envKey`）| `sanitize_provider_id()` — **过滤掉所有非 ASCII**（`config_writer.rs:75-80`）| `RELAY__API_KEY` ← 名字被吃空 |

所以中文中转站名会同时踩两个坑：**写进 `.env` 的键名带中文（读不到）**，而 **providers.json 里引用的键名是 `RELAY__API_KEY`（对不上）**——两边根本不是同一个 key。多个纯中文命名的中转站还会全部塌缩成同一个 `RELAY__API_KEY` 互相覆盖。

**临时解决**：中转站名只用 ASCII（如 `NL`），避开两处不一致。

**根治方案**（三处一起改，缺一不可）：
1. `claude-proxy.js:22` 改 `/^([^=\s]+)=(.*)$/` 或 `line.split('=')` 取首个 `=` 切分
2. `config_writer.rs:195` / `:635` / `:668` 与 `:128` 统一走**同一个** key 生成函数
3. 该函数对非 ASCII 名做确定性映射（如 hash 后缀或转写），而不是过滤成空串

---

## 六、当前 Runtime 修补状态

修补版代理文件：`/tmp/claude-proxy-patched.js`（**未合入源码**）

已应用修复：
- [x] Bug 1：移除 `claude-opus-5` 硬编码 + `startsWith` 收窄到 `claude-opus-4`
- [x] Bug 2：前缀剥离后回退检查 `PROVIDERS[modelId]`
- [x] Bug 3：Anthropic 直通两处（流式 + 非流式）改用 `provider.apiKey`
- [ ] Bug 5（env 编码）：**未修**
- 另含两行临时 `DEBUG` 日志（`claude-proxy.js:656` 附近打 `modelId` / `realModelId` / `PROVIDERS` keys）——合入源码前需按全局铁律确认修好后移除

启动命令（**必须显式带 `--port`**，否则默认监听 8789 而非 8689）：
```bash
node /tmp/claude-proxy-patched.js --port 8689
```

### ⚠️ 原记录中"cc-gate 监控文件 + 加 uchg 锁"的说法不成立

经查证**这套机制在代码中并不存在**：

- 全仓（`src-tauri/src/`、`Cargo.toml`）搜 `uchg` / `chflags` / `immutable` / `notify::` / `fs::watch` / `Watcher` → **零命中**，既无文件监控也无加锁代码
- 实测文件 flags 全为空（`ls -lO` 显示 `-`），`providers.json` / `claude-proxy.js` / `.env` 均未被锁
- 因此 `chflags nouchg ~/.mimo2codex/providers.json` 这一步是**多余的**，不需要执行

**真实的持久化风险是另一回事**——`deploy_proxy_scripts()`（`config_writer.rs:335-341`）的行为是：

```rust
// claude-proxy.js — only write if missing (avoid disk I/O on every launch)
if !cp.exists() { fs::write(&cp, include_str!("../../claude-proxy.js"))?; }
```

即**只在文件缺失时写入，存在则不动**。所以：

- 直接改 `~/.mimo2codex/claude-proxy.js` **不会**被 cc-gate 还原（这与原记录的描述相反）
- 但它**会被 cc-gate 起的进程重启后重新加载**——所以改完要 kill 掉 8689 上的旧进程
- 真正会丢改动的情况是**该文件被删除**（重装、清理 `~/.mimo2codex/`），届时会用编译进二进制的旧版覆盖回来

> 另注：该函数上方的 doc comment 写着 "`write_if_changed` is not used here because we WANT to overwrite on every apply"，与下方 `if !cp.exists()` 的实际实现**矛盾**——注释描述的是已废弃的旧行为，应一并修正。

因此手动挂修补版的正确步骤是：

1. 确认 `~/.mimo2codex/providers.json` 配置正确（**无需 `chflags`**）
2. `lsof -ti:8689 | xargs kill` 干掉 cc-gate 起的代理
3. `node /tmp/claude-proxy-patched.js --port 8689` 手动启动修补版
4. cc-gate 里点"应用"会重启代理、覆盖掉手动进程 → 需重做 2-3 步

---

## 七、根治建议（源码修改）

应在项目根 `claude-proxy.js`（编译期嵌入，见文首）中修复：

1. **Bug 1**：`isAnthropicNative` 判断后移到 `PROVIDERS` 查找之后，改为"providers.json 查不到才走官方直通"的兜底逻辑
2. **Bug 2**：改为精确匹配优先 + 前缀剥离兜底；或 gateway 换用不冲突的前缀方案
3. **Bug 3**：Anthropic 直通（`:680`/`:684`）与 OpenAI 转换（`:716`/`:720`）统一用 `provider.apiKey`
4. **env 解析**：`claude-proxy.js:22` 放宽正则；**同时**统一 `config_writer.rs` 中 4 处 env key 生成逻辑（`:128` / `:195` / `:635` / `:668`）
5. **错误信息**：`:661` 的 `Unknown token` 应区分"token 无效"与"model 未命中"两种情况，避免误导排查方向
6. **注释修正**：`config_writer.rs:327-330` 的 doc comment 与 `if !cp.exists()` 实现矛盾
7. ~~取消 providers.json 的 `uchg` 自动加锁~~ — **该机制不存在，无需处理**

---

## 八、待完善列表

以下问题在中转站接入过程中暴露，尚未修复，列入待办：

### 设计层面

1. **路由优先级不一致**
   - 硬编码 Anthropic 列表 > providers.json，而非 provider 优先
   - 应该：providers.json 查到 → 走 provider 配置；查不到 → 走硬编码直通或默认路由
   - 现状导致第三方 provider 配了 Claude 模型也无法生效（= Bug 1 的根因）

2. **Gateway `claude-` 前缀污染**
   - `claude-proxy.js:572` 给所有模型都加 `claude-` 前缀，与真正 Claude 模型名冲突
   - 真正的 Claude 模型变成 `claude-claude-opus-5`，非 Claude 模型变成 `claude-deepseek-v4-pro`
   - 应在 proxy 端做精确匹配优先 + 前缀匹配兜底，或 gateway 改用不冲突的前缀方案

3. **TOKEN_MAP 路由与 Provider 路由双轨并行**
   - `TOKEN_MAP`（`:527-532`）用 `x-api-key` 的 token 值（`ds`/`qwen`/`glm`/`mimo`）做路由决策
   - Provider 路由用实际 model 名匹配 providers.json
   - 实际是 `TOKEN_MAP[token] || realModelId`（`:655`）——**token 匹配优先于 model 名**
   - ⚠️ 这意味着：只要客户端 `x-api-key` 恰好等于 `ds`/`qwen`/`glm`/`mimo` 之一，**请求的 model 字段会被完全忽略**，静默路由到 token 对应的模型。且 `:601` 的 `|| 'ds'` 默认值让无 key 请求也落到 DeepSeek
   - 建议统一为单一路由策略（model 名优先，token 仅作显式覆盖）

4. **Anthropic vs OpenAI 端点选择缺乏智能**
   - 目前靠 `provider.anthropicEndpoint` 布尔值硬选（由 `config_writer.rs:167-172` 写入，仅当 `provider_id == "anthropic"` 且中转站配了 `anthropic_url` 时才为 true）
   - 没有自动探测：provider 到底提供什么格式？同一个中转站可能同时提供两种端点
   - 结果：同一中转站的 Anthropic 模型走直通、OpenAI 模型走转换，需在 providers.json 里拆成两个 entry（当前 `RELAY_NL_API_KEY` 确实出现在两个 provider 条目里）
   - 建议增加端点探测或允许 per-model 指定端点格式

### 运维层面

5. **修补版代理无法持久化**（原"uchg 锁过于激进"条目已订正）
   - `deploy_proxy_scripts()` 是 `if !exists` 语义，手动改动**不会**被还原
   - 但 cc-gate 点"应用"会重启代理进程、覆盖手动挂的修补版
   - 且 `~/.mimo2codex/` 被清理后会用编译进二进制的旧版覆盖回来
   - 建议：修复直接合入项目根 `claude-proxy.js` 并重新构建，不要长期依赖 `/tmp` 修补版

6. **代理重启无热加载**
   - `loadEnv()` / `loadProviders()` 只在启动时执行一次（`:73-74`，模块顶层）
   - 修改 providers.json 或 .env 后必须重启代理
   - 建议加文件监听（`fs.watch`）自动 reload provider 列表

7. **调试日志只能看 stderr，无结构化日志**
   - 出问题时只能靠 `console.error` 看日志
   - 建议加请求 ID 追踪、结构化日志级别（DEBUG/INFO/ERROR）

### 代码质量

8. **`loadEnv()` 正则 `\w+` 不支持非 ASCII，且与 Rust 侧生成逻辑不一致**
   - `RELAY_非线ANTHROPIC_API_KEY` 读不到值（已实测确认）
   - 更严重的是 Rust 侧两处生成方式不同（保留中文 vs 过滤成空串），详见第五节
   - 修复：放宽正则 + 统一 4 处 env key 生成逻辑

9. **Anthropic 直通路径硬编码 `anthropic-version: 2023-06-01`**
   - 四处硬编码（`:628` / `:632` 官方直通，`:680` / `:684` provider 直通），值其实是一致的
   - 问题不是"不一致"，而是**不可配置**——有的中转站可能需要特定的 `anthropic-version`
   - 建议放到 providers.json 的 per-provider 配置里

10. **无请求级超时/重试差异化配置**
    - `httpRequest()` 是 `timeout: 120000`（2 分钟，`:86`）
    - `streamPassthrough()` / `openaiStreamToAnthropicSSE()` 是 `timeout: 300000`（5 分钟，`:110` / `:167`）
    - 三处均为硬编码，不同中转站响应速度差异大，应该 per-provider 可配

11. **默认端口不一致**
    - `claude-proxy.js:12` 默认 `8789`，但 cc-gate 总是显式传 `8689`（`proxy_manager.rs:245`）
    - 手动启动忘带 `--port` 时会静默监听错误端口，表现为"改了没用"
    - 建议默认值改为 8689 与 cc-gate 对齐
