# CC-Gate

**一个桌面应用，统一管理你所有 AI 编程工具的模型配置。**

---

## 🆚 与 CC Switch 的最大区别

两者的根本差异在于：**CC Switch 有"供应商"这个中间层**——你想用什么模型，取决于当前激活了哪个供应商。CC-Gate 没有供应商概念——所有厂商的模型平等地写在一起，代理按模型名自动路由。

这个差异在 CLI 和桌面端表现为不同的体验：

### CLI 模式

| | CC Switch | CC-Gate |
|---|---|---|
| 多窗口并行 | ❌ 切到 DeepSeek 后，**所有终端**都只能用 DeepSeek | ✅ `codex-ds` 一个窗口、`codex-glm` 另一个窗口、`codex-mimo` 第三个窗口——**3 个终端 3 个模型同时跑，互不干扰** |
| 怎么换模型 | 先切供应商，全局生效 | 换 alias = 换个终端窗口 |

### 桌面端模式

| | CC Switch | CC-Gate |
|---|---|---|
| 对话内切模型 | ❌ `/model` 只显示当前供应商的几个模型。想用跨厂商的？→ 退出桌面端 → 切供应商 → 重开 | ✅ `/model` 菜单里 DeepSeek、GLM、Qwen、MiMo **全部混排**，直接选，不需要退出重开 |
| 模型列表 | 单厂商内部模型 | 所有厂商模型在一张列表里 |

**CC Switch 是"先选店再点菜"，CC-Gate 是"所有菜在一张菜单上"。** 代理层自动识别模型名、自动路由到对应厂商——用户根本不需要知道"供应商"这个概念。

---

## 解决的问题

你有多个 AI 编程工具（Codex CLI、Codex 桌面端、Claude Code CLI、Claude Desktop、Hermes、OpenCode、OpenClaw、Aider、Cursor、Reasonix），每个都要配置模型。不同工具配置文件格式不同、位置不同、参数名不同——手动维护极易出错。

CC-Gate 把所有这些集中到**一个 GUI** 里：

- 图形化勾选哪些 Agent 用哪些模型
- 每个模型选"直连"还是"走中转"
- 22 个提供商的 API Key 统一管理
- 3 个代理进程自动启停
- Shell alias 自动注入（终端敲 `codex-ds` 直接开搞）
- **远程模型目录自动更新**（厂商出新模型，不用重装软件）

## 架构

```
 GUI (Tauri 2 + Vue 3)
      │
      ├─ 写配置文件 ──→ ~/.codex/config.toml
      │                  ~/.codex/cc-switch-model-catalog.json
      │                  ~/.claude/settings.json
      │                  ~/.zshrc / ~/.bashrc
      │                  ~/.hermes/config.yaml
      │                  ~/.config/opencode/config.toml
      │                  ~/.openclaw/openclaw.json
      │
      ├─ 管理代理进程 ──→ mimo2codex   (:8688, Responses API 翻译)
      │                   claude-proxy (:8689, Anthropic→Chat 翻译)
      │                   chat-proxy   (:8690, Chat Completions 透传)
      │
      └─ 模型目录 ──→ models-catalog.json (GitHub raw, 远程自动更新)
```

**核心思路：按 API 协议分组，不是按 Agent 分组。** 3 个代理端口覆盖全部 10 个 Agent，不存在 10 个端口。

## 功能

### 🤖 Agent→模型 分���

10 个 Agent，每个独立勾选可用模型。Codex Desktop 和 Codex CLI 可以选不同的模型集合。勾完点"应用"，所有配置文件一次性写入。

### 🔀 模型路由

每个模型可选 **直连** 还是 **走中转站**。同一个 deepseek-v4-pro，Aider 可以直连、Codex CLI 可以走你的中转——互不干扰。

### 🔑 API Key 管理

内置 22 个主流提供商的 API Key 管理。Key 写入 `~/.mimo2codex/.env`，代理运行时读取，不落明文在配置文件里。

### ⌨️ Shell 集成

自动在 `.zshrc` / `.bashrc` / PowerShell Profile 注入 alias：

```bash
codex-ds        # Codex CLI + DeepSeek V4 Pro
codex-glm       # Codex CLI + GLM-5.2
claude-ds       # Claude Code CLI + DeepSeek V4 Pro
claude-mimo     # Claude Code CLI + MiMo V2.5 Pro
aider-ds        # Aider + DeepSeek V4 Pro
# ... 更多组合
```

### 🔧 工具检测

自动检测 Node.js、Python、Codex CLI、Claude Code CLI、Aider 是否已安装，显示版本号和安装指引。

### 🚀 启动项管理

3 个代理进程 + App 自身支持开机自启（macOS launchd）。

### 📡 远程模型目录

启动时自动从 GitHub 拉取最新模型列表。厂商出新模型（如 deepseek-v5），维护者只需更新 `models-catalog.json` 并 push，**所有用户自动获取**，不需要重装 CC-Gate。

## 支持的 Agent

| Agent | 类型 | 代理端口 | 协议 |
|-------|------|---------|------|
| Codex CLI | CLI | :8688 | Responses API |
| Codex Desktop | 桌面端 | :8688 | Responses API |
| Codex Reasonix | CLI | :8688 | Responses API |
| Claude Code CLI | CLI | :8689 | Anthropic Messages |
| Claude Desktop | 桌面端 | :8689 | Anthropic Messages |
| Hermes | CLI | :8690 | Chat Completions |
| OpenCode | CLI | :8690 | Chat Completions |
| OpenClaw | CLI | :8690 | Chat Completions |
| Aider | CLI | :8690 | Chat Completions |
| Cursor | CLI | :8690 | Chat Completions |

## 支持的模型（首批，持续更新中）

| 模型 | 提供商 | 上下文 |
|------|--------|--------|
| DeepSeek V4 Pro | DeepSeek | 1M |
| DeepSeek V4 Flash | DeepSeek | 1M |
| GLM-5.2 | 智谱 AI | 128K |
| Qwen3.8 Max Preview | 阿里云 | 1M |
| Qwen-Max | 阿里云 | 128K |
| MiMo V2.5 Pro | 小米 | 128K |
| MiMo V2.5 | 小米 | 1M |
| Claude Opus 4.5 | Anthropic | 200K |
| GPT-5.1 Codex | OpenAI | 1M |

> 更多模型通过 `models-catalog.json` 远程更新，无需重装软件。

## 安装

### 下载安装包

从 [Releases](../../releases) 页面下载最新版 `.dmg`（Mac）或 `.exe`（Windows）。

### 从源码构建

**前置要求：**
- Node.js ≥ 20
- Rust toolchain（`rustup`）
- pnpm 或 npm

```bash
git clone https://github.com/gongminami-pixel/cc-gate.git
cd cc-gate
npm install
npx tauri build
```

构建产物在 `src-tauri/target/release/bundle/`。

## 安全

- API Key 存储在 `~/.mimo2codex/.env`，不会写入源码或配置文件
- 代理监听 `127.0.0.1`，不接受外部连接
- 用量记录默认关闭，需要时手动开启

## 许可证

MIT

## 致谢

CC-Gate 的设计深受 [CC Switch](https://github.com/cexll/myclaude) 的启发。我们将"全局供应商切换"升级为 alias 级"多模型并行"——每个终端窗口独立选择模型，互不干扰。并扩展到了 10 个 Agent。

---

**CC-Gate — 一个 GUI 管所有 AI 工具。**
