# mimo2codex 代理安装与配置完整教程

> **适用系统：** macOS 13+ / Windows 10+

## 简介

mimo2codex 是一个本地代理，将 OpenAI Codex CLI 的 Responses API
翻译为上游 Chat Completions，让 Codex 和 Claude Code 能对接
DeepSeek、GLM、Qwen、MiMo 等国产模型。

**当前版本：** 0.5.27
**源码：** https://github.com/NousResearch/mimo2codex

---

## 一、环境要求

| 依赖 | 最低版本 | 说明 |
|------|----------|------|
| Node.js | 18+ | 推荐用 nvm 管理 |
| npm | 9+ | 随 Node.js 自带 |
| 操作系统 | macOS 13+ / Windows 10+ | |
| 终端 | Terminal / PowerShell / Windows Terminal | |

---

## 二、安装 nvm + Node.js

### macOS

```bash
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash
```

关闭终端，重新打开，或执行：

```bash
source ~/.zshrc   # zsh 用户
source ~/.bashrc  # bash 用户
```

```bash
nvm install 20
nvm use 20
nvm alias default 20
```

### Windows

**推荐用 nvm-windows：**

1. 下载安装包：
   https://github.com/coreybutler/nvm-windows/releases
   选择 `nvm-setup.exe`，安装

2. 打开 **PowerShell 或 CMD（以管理员身份运行）**：

```powershell
nvm install 20
nvm use 20
```

3. 验证：

```powershell
node --version
# v20.x.x
npm --version
# 10.x.x
```

> **备选：** 也可以直接用 Node.js 官方安装包（nodejs.org），
> 但 nvm 让你可以随时切换版本，推荐。

---

## 三、安装 mimo2codex

### macOS

```bash
npm install -g mimo2codex
```

### Windows（PowerShell）

```powershell
npm install -g mimo2codex
```

> **不要**加 `sudo`（macOS）或以管理员身份（Windows 用 nvm 时不需要）。

验证安装：

```bash
mimo2codex --version
# 0.5.27
```

---

## 四、配置

### 4.1 创建配置目录

**macOS:**

```bash
mkdir -p ~/.mimo2codex
```

**Windows（PowerShell）:**

```powershell
mkdir $env:USERPROFILE\.mimo2codex
```

### 4.2 配置 API Key（`.env`）

在配置目录下创建 `.env` 文件。

**文件位置：**

| 系统 | 路径 |
|------|------|
| macOS | `~/.mimo2codex/.env` |
| Windows | `%USERPROFILE%\.mimo2codex\.env` |

**内容：**

```bash
# DeepSeek — https://platform.deepseek.com/api_keys
DEEPSEEK_API_KEY=sk-y...y

# 智谱 GLM — https://open.bigmodel.cn/usercenter/apikeys
GLM_API_KEY=yo...n

# 阿里 Qwen — https://bailian.console.aliyun.com/
QWEN38_API_KEY=sk...n

# 小米 MiMo — https://platform.xiaomimimo.com/#/console/api-keys
MIMO_API_KEY=sk...n
```

**macOS 设置文件权限：**

```bash
chmod 600 ~/.mimo2codex/.env
```

**⚠️ `.env` 包含明文密钥，永远不要提交到 Git 或分享。**

### 4.3 配置 Provider（`providers.json`）

在配置目录下创建 `providers.json`。

**文件位置：**

| 系统 | 路径 |
|------|------|
| macOS | `~/.mimo2codex/providers.json` |
| Windows | `%USERPROFILE%\.mimo2codex\providers.json` |

**内容：**

```json
{
  "providers": [
    {
      "id": "deepseek-direct",
      "name": "DeepSeek",
      "baseUrl": "https://api.deepseek.com/v1",
      "envKey": "DEEPSEEK_API_KEY",
      "defaultModel": "deepseek-v4-pro",
      "models": [
        {
          "id": "deepseek-v4-pro",
          "displayName": "DeepSeek V4 Pro",
          "contextWindow": 1000000,
          "maxOutputTokens": 393216
        },
        {
          "id": "deepseek-v4-flash",
          "displayName": "DeepSeek V4 Flash",
          "contextWindow": 1000000,
          "maxOutputTokens": 393216
        }
      ]
    },
    {
      "id": "glm-direct",
      "name": "智谱GLM",
      "baseUrl": "https://open.bigmodel.cn/api/paas/v4",
      "envKey": "GLM_API_KEY",
      "defaultModel": "glm-5.2",
      "features": {
        "forceParallelToolCalls": true
      },
      "models": [
        {
          "id": "glm-5.2",
          "displayName": "GLM-5.2",
          "contextWindow": 1000000,
          "maxOutputTokens": 16384
        }
      ]
    },
    {
      "id": "qwen38-direct",
      "name": "阿里Qwen3.8",
      "baseUrl": "https://dashscope.aliyuncs.com/compatible-mode/v1",
      "envKey": "QWEN38_API_KEY",
      "defaultModel": "qwen3.8-max-preview",
      "models": [
        {
          "id": "qwen3.8-max-preview",
          "displayName": "Qwen3.8 Max Preview",
          "contextWindow": 1048576,
          "maxOutputTokens": 65536
        }
      ]
    },
    {
      "id": "xiaomi-direct",
      "name": "小米MiMo",
      "baseUrl": "https://api.xiaomimimo.com/v1",
      "envKey": "MIMO_API_KEY",
      "defaultModel": "mimo-v2.5-pro",
      "models": [
        {
          "id": "mimo-v2.5-pro",
          "displayName": "MiMo V2.5 Pro",
          "contextWindow": 131072,
          "maxOutputTokens": 16384
        }
      ]
    }
  ]
}
```

---

## 五、启动代理

### 5.1 前台启动（调试推荐）

**macOS:**

```bash
# 加载环境变量
set -a
source <(grep -v '^\s*#' ~/.mimo2codex/.env | grep -v '^\s*$')
set +a

# 启动
mimo2codex --model ds
```

**Windows（PowerShell）:**

```powershell
# 加载环境变量
Get-Content "$env:USERPROFILE\.mimo2codex\.env" |
    Where-Object { $_ -notmatch '^\s*(#|$)' } |
    ForEach-Object {
        $key, $val = $_ -split '=', 2
        [Environment]::SetEnvironmentVariable($key, $val)
    }

# 启动
mimo2codex --model ds
```

> `--model ds` 锁定为 DeepSeek。
> 可选：`glm` / `qwen` / `mimo`。
> 默认端口 **8788**。指定端口：`mimo2codex --port 8688 --model ds`

### 5.2 后台自启

#### macOS（launchd）

创建 `~/Library/LaunchAgents/com.mimo2codex.plist`：

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.mimo2codex</string>
    <key>ProgramArguments</key>
    <array>
        <string>/bin/bash</string>
        <string>-c</string>
        <string>
            cd $HOME/.mimo2codex
            if [ -f $HOME/.mimo2codex/.env ]; then
                set -a
                source &lt;(grep -v '^\s*#' $HOME/.mimo2codex/.env | grep -v '^\s*$')
                set +a
            fi
            export PATH="$HOME/.nvm/versions/node/v$(node -v | cut -c2-)/bin:$PATH"
            exec mimo2codex --model ds --no-admin
        </string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/tmp/mimo2codex.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/mimo2codex.log</string>
</dict>
</plist>
```

```bash
launchctl load ~/Library/LaunchAgents/com.mimo2codex.plist
```

#### Windows（任务计划程序）

**方法一：开机启动文件夹**

1. 创建启动脚本 `%USERPROFILE%\.mimo2codex\start.bat`：

```batch
@echo off
cd /d %USERPROFILE%\.mimo2codex
for /f "usebackq tokens=1,* delims==" %%a in (.env) do (
    if not "%%a"=="" if not "%%a:~0,1%"=="#" set "%%a=%%b"
)
call mimo2codex --model ds --no-admin
```

2. `Win+R` → `shell:startup` → 把 `start.bat` 的快捷方式放进去。

**方法二：任务计划程序（更可靠）**

1. `Win+R` → `taskschd.msc`
2. 创建基本任务 → 触发器：**用户登录时**
3. 操作：启动程序 `cmd.exe`，参数 `/c %USERPROFILE%\.mimo2codex\start.bat`
4. 勾选 **使用最高权限运行**

### 5.3 验证代理

```bash
curl http://127.0.0.1:8788/v1/models
```

应返回 JSON 模型列表。Windows 用 PowerShell：

```powershell
Invoke-WebRequest -Uri http://127.0.0.1:8788/v1/models | Select-Object -Expand Content
```

---

## 六、对接各工具

### 6.1 Claude Code

编辑配置文件：

| 系统 | 路径 |
|------|------|
| macOS | `~/.claude/settings.json` |
| Windows | `%USERPROFILE%\.claude\settings.json` |

```json
{
  "env": {
    "ANTHROPIC_BASE_URL": "http://127.0.0.1:8689"
  },
  "model": "claude-opus-4-5"
}
```

> 端口 8689 是 claude-proxy（需要额外部署，见第七章）。

### 6.2 Codex CLI

编辑配置文件：

| 系统 | 路径 |
|------|------|
| macOS | `~/.codex/config.toml` |
| Windows | `%USERPROFILE%\.codex\config.toml` |

```toml
model_provider = "custom"
model = "deepseek-v4-pro"

[model_providers.custom]
name = "CC-Gate"
base_url = "http://127.0.0.1:8688/v1"
wire_api = "responses"
requires_openai_auth = true
```

切换模型：

```bash
codex --model deepseek-v4-pro
codex --model glm-5.2
codex --model qwen3.8-max-preview
```

### 6.3 OpenCode CLI

编辑配置文件：

| 系统 | 路径 |
|------|------|
| macOS | `~/.config/opencode/config.yaml` |
| Windows | `%APPDATA%\opencode\config.yaml` |

```yaml
model: deepseek-v4-pro
base_url: http://127.0.0.1:8690/v1
```

---

## 七、claude-proxy 部署（Claude Code 专用）

mimo2codex 本身不翻译 Anthropic Messages 协议。
Claude Code 需要额外的 claude-proxy 做协议转换。

### 7.1 获取脚本

从 cc-gate 项目的 `~/.mimo2codex/` 目录（已部署）复制，
或直接使用修复版：

**macOS:**

```bash
cp /tmp/claude-proxy-fixed.js ~/.mimo2codex/claude-proxy-fixed.js
```

**Windows:**

```powershell
Copy-Item "C:\Users\你的用户名\.mimo2codex\claude-proxy-fixed.js" `
    "$env:USERPROFILE\.mimo2codex\claude-proxy-fixed.js"
```

### 7.2 启动

**macOS:**

```bash
node ~/.mimo2codex/claude-proxy-fixed.js --port 8689 &
```

**Windows:**

```powershell
Start-Process node -ArgumentList "$env:USERPROFILE\.mimo2codex\claude-proxy-fixed.js", "--port", "8689" `
    -WindowStyle Hidden
```

### 7.3 验证

```bash
curl http://127.0.0.1:8689/v1/models
```

---

## 八、验证清单

```bash
# 1. mimo2codex (Responses API) — 端口 8788
curl http://127.0.0.1:8788/v1/models

# 2. claude-proxy (Anthropic Messages) — 端口 8689
curl http://127.0.0.1:8689/v1/models

# 3. chat-proxy (Chat Completions) — 端口 8690
curl http://127.0.0.1:8690/v1/models
```

每条都应返回模型 JSON 列表。

---

## 九、端口与协议对照

| 端口 | 代理 | 协议 | 使用者 |
|------|------|------|--------|
| 8688 | mimo2codex | Responses API | Codex CLI |
| 8689 | claude-proxy | Messages API | Claude Code |
| 8690 | chat-proxy | Chat Completions | Hermes, OpenCode |
| 8788 | mimo2codex (独立) | Responses API | Codex CLI |
| 8789 | claude-proxy (独立) | Messages API | Claude Code |

---

## 十、常见问题

### Q: `npm install -g` 报权限错误

**macOS:** 不要用 `sudo`。使用 nvm 管理的 Node.js 自动解决。
**Windows:** 确认 nvm-windows 安装正确，不要以管理员身份运行 npm。

### Q: 启动后端口被占用

```bash
# macOS
lsof -ti:8788 | xargs kill -9

# Windows PowerShell
netstat -ano | findstr :8788
# 记下 PID，然后：
taskkill /PID 进程ID /F
```

### Q: 代理日志在哪里

- 前台启动：直接输出到终端
- macOS launchd：`/tmp/mimo2codex.log`
- Windows 任务计划：查看启动脚本所在目录

### Q: Codex 报 "Model not found"

检查 `providers.json` 中模型 ID 是否与 Codex `config.toml` 的 `model` 字段完全一致。

### Q: Claude Code 报 tool_calls 解析错误

代理脚本版本过旧，缺少 SSE 流式翻译和工具调用支持。
使用 `claude-proxy-fixed.js` 替换原始的 `claude-proxy.js`。

### Q: Windows 下 curl 不可用

用 PowerShell：

```powershell
Invoke-WebRequest -Uri http://127.0.0.1:8788/v1/models
```

或安装 curl：
```powershell
winget install curl.curl
```

### Q: Windows 下 nvm 切换版本后 node 找不到

```powershell
nvm use 20
refreshenv
```

### Q: 环境变量加载后不生效

**macOS:** 每次新开终端需要重新 source `.env`
**Windows PowerShell:** 用 `[Environment]::SetEnvironmentVariable` 加载的是进程级变量，只对当前窗口有效。写入用户环境变量（永久）用：

```powershell
[Environment]::SetEnvironmentVariable("DEEPSEEK_API_KEY", "sk-...", "User")
```
