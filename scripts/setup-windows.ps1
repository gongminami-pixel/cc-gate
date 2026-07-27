# CC-Gate Windows 环境安装脚本
# 自动安装 Node.js + npm + mimo2codex（如未安装）
# 用法：powershell -ExecutionPolicy Bypass -File setup-windows.ps1

$ErrorActionPreference = "Stop"

Write-Host "=== CC-Gate 环境检测 ===" -ForegroundColor Cyan

# ── 1. Node.js & npm ──────────────────────────────────────
$nodeOk = $false
try { $nv = node --version 2>$null; if ($LASTEXITCODE -eq 0) { Write-Host "✓ Node.js $nv" -ForegroundColor Green; $nodeOk = $true } } catch {}

if (-not $nodeOk) {
    Write-Host "✗ Node.js 未安装，正在通过 winget 安装..." -ForegroundColor Yellow
    winget install OpenJS.NodeJS.LTS --silent --accept-package-agreements --accept-source-agreements
    refreshenv
    node --version
    Write-Host "✓ Node.js 安装完成" -ForegroundColor Green
}

# ── 2. npm ────────────────────────────────────────────────
$npmOk = $false
try { $nv = npm --version 2>$null; if ($LASTEXITCODE -eq 0) { Write-Host "✓ npm $nv" -ForegroundColor Green; $npmOk = $true } } catch {}

# ── 3. mimo2codex ─────────────────────────────────────────
$mimoOk = $false
try { $mv = mimo2codex --version 2>$null; if ($LASTEXITCODE -eq 0) { Write-Host "✓ mimo2codex $mv" -ForegroundColor Green; $mimoOk = $true } } catch {}

if (-not $mimoOk) {
    if ($nodeOk) {
        Write-Host "✗ mimo2codex 未安装，正在安装..." -ForegroundColor Yellow
        npm install -g mimo2codex
        mimo2codex --version
        Write-Host "✓ mimo2codex 安装完成" -ForegroundColor Green
    } else {
        Write-Host "✗ 无法安装 mimo2codex：Node.js 不可用" -ForegroundColor Red
    }
}

Write-Host "=== 环境检测完成 ===" -ForegroundColor Cyan
