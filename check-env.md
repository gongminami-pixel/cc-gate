# 开发环境检测脚本

> 一键检测 macOS / Windows 上 Node.js、npm、Python3、Codex CLI、Claude CLI、Shell 的安装状态。

---

## 使用方法

### macOS / Linux

```bash
bash <(curl -s https://example.com/check-env.sh)
# 或本地运行：
bash check-env.sh
```

### Windows PowerShell

```powershell
.\check-env.ps1
```

---

## 检测清单

| 序号 | 工具 | macOS 检测命令 | Windows 检测命令 |
|------|------|---------------|-----------------|
| 1 | Node.js | `node --version` | `node --version` |
| 2 | npm | `npm --version` | `npm --version` |
| 3 | Python3 | `python3 --version` | `python --version` |
| 4 | Codex CLI | `codex --version` | `codex --version` |
| 5 | Claude CLI | `claude --version` | `claude --version` |
| 6 | Shell | `echo $SHELL` | `$PSVersionTable.PSVersion` |

---

## macOS Shell 脚本

保存为 `check-env.sh`，运行 `chmod +x check-env.sh && ./check-env.sh`：

```bash
#!/bin/bash
# 开发环境检测脚本 — macOS/Linux
# 用法: chmod +x check-env.sh && ./check-env.sh

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

check() {
    local name="$1"
    local cmd="$2"
    local version_cmd="$3"

    if command -v "$cmd" &>/dev/null; then
        local ver
        ver=$($version_cmd 2>&1 | head -1)
        echo -e "${GREEN}✓${NC} $name — $ver — $(command -v "$cmd")"
    else
        echo -e "${RED}✗${NC} $name — 未安装"
    fi
}

echo "========================================"
echo "  开发环境检测 — $(date '+%Y-%m-%d %H:%M')"
echo "========================================"
echo ""

check "Node.js"   "node"    "node --version"
check "npm"       "npm"     "npm --version"
check "Python3"   "python3" "python3 --version"
check "Codex CLI" "codex"   "codex --version"
check "Claude CLI" "claude" "claude --version"

echo ""
echo "Shell: $SHELL — $($SHELL --version 2>&1 | head -1)"
echo ""

# 检查 nvm（Node 版本管理器）
if [ -d "$HOME/.nvm" ] || command -v nvm &>/dev/null; then
    echo -e "${GREEN}✓${NC} nvm — $(nvm --version 2>/dev/null || echo '已安装')"
else
    echo -e "${RED}✗${NC} nvm — 未安装"
fi

# 检查代理端口
echo ""
echo "--- 代理端口 ---"
for port in 8788 8688 8689 8690; do
    if lsof -tiTCP:$port -sTCP:LISTEN &>/dev/null; then
        echo -e "${GREEN}✓${NC} 端口 $port — 运行中"
    else
        echo "  端口 $port — 未占用"
    fi
done

echo ""
echo "========================================"
echo "  检测完成"
echo "========================================"
```

---

## Windows PowerShell 脚本

保存为 `check-env.ps1`，右键 "使用 PowerShell 运行"，或在终端执行 `.\check-env.ps1`：

```powershell
<#
.SYNOPSIS
    开发环境检测脚本 — Windows
.DESCRIPTION
    检测 Node.js, npm, Python3, Codex CLI, Claude CLI 安装状态
.EXAMPLE
    .\check-env.ps1
#>

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  开发环境检测 — $(Get-Date -Format 'yyyy-MM-dd HH:mm')" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

function Check-Tool {
    param($Name, $Command, $VersionArgs)
    try {
        $ver = & $Command $VersionArgs 2>&1 | Select-Object -First 1
        $path = (Get-Command $Command -ErrorAction Stop).Source
        Write-Host "✓ $Name — $ver — $path" -ForegroundColor Green
    } catch {
        Write-Host "✗ $Name — 未安装" -ForegroundColor Red
    }
}

Check-Tool "Node.js"   "node"    @("--version")
Check-Tool "npm"       "npm"     @("--version")

# Python 检测（Windows 上可能是 python 或 python3）
try {
    $pyVer = python --version 2>&1
    $pyPath = (Get-Command python -ErrorAction Stop).Source
    Write-Host "✓ Python — $pyVer — $pyPath" -ForegroundColor Green
} catch {
    try {
        $pyVer = python3 --version 2>&1
        $pyPath = (Get-Command python3 -ErrorAction Stop).Source
        Write-Host "✓ Python3 — $pyVer — $pyPath" -ForegroundColor Green
    } catch {
        Write-Host "✗ Python — 未安装" -ForegroundColor Red
    }
}

Check-Tool "Codex CLI"  "codex"  @("--version")
Check-Tool "Claude CLI" "claude" @("--version")

Write-Host ""
Write-Host "PowerShell: $($PSVersionTable.PSVersion)" -ForegroundColor White

# 检查 nvm-windows
try {
    $nvmVer = nvm version 2>&1
    Write-Host "✓ nvm-windows — $nvmVer" -ForegroundColor Green
} catch {
    Write-Host "✗ nvm-windows — 未安装" -ForegroundColor Red
}

# 检查代理端口
Write-Host ""
Write-Host "--- 代理端口 ---" -ForegroundColor White
@(8788, 8688, 8689, 8690) | ForEach-Object {
    $conn = netstat -ano 2>$null | Select-String "127.0.0.1:$_"
    if ($conn) {
        Write-Host "✓ 端口 $_ — 运行中" -ForegroundColor Green
    } else {
        Write-Host "  端口 $_ — 未占用"
    }
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  检测完成" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
```

---

## 手动逐条检测

不想用脚本的话，在终端逐条执行：

### macOS / Linux

```bash
node --version && which node
npm --version && which npm
python3 --version && which python3
codex --version && which codex
claude --version && which claude
echo "$SHELL" && $SHELL --version | head -1
```

### Windows PowerShell

```powershell
node --version; (Get-Command node).Source
npm --version; (Get-Command npm).Source
python --version 2>&1; (Get-Command python -ErrorAction SilentlyContinue).Source
codex --version; (Get-Command codex).Source
claude --version; (Get-Command claude).Source
$PSVersionTable.PSVersion
```

---

## 期望输出示例

```
✓ Node.js   — v20.20.2 — /Users/ami/.nvm/versions/node/v20.20.2/bin/node
✓ npm       — 10.8.2  — /Users/ami/.nvm/versions/node/v20.20.2/bin/npm
✓ Python3   — 3.11.1  — ~/.hermes/hermes-agent/.venv/bin/python3
✓ Codex CLI — 0.145.0 — /Users/ami/.nvm/versions/node/v20.20.2/bin/codex
✓ Claude CLI — 2.1.205 — /Users/ami/.nvm/versions/node/v20.20.2/bin/claude
Shell: /bin/zsh — zsh 5.9
```

---

*适用于 macOS 13+ / Windows 10+*
