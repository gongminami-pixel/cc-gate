#!/bin/bash
# CC-Gate macOS 环境安装脚本
# 自动安装 Node.js + npm + mimo2codex（如未安装）
set -e

echo "=== CC-Gate 环境检测 ==="

# ── 1. Node.js & npm ──────────────────────────────────────
if command -v node >/dev/null 2>&1; then
    echo "✓ Node.js $(node --version)"
    echo "✓ npm $(npm --version)"
elif command -v brew >/dev/null 2>&1; then
    echo "✗ Node.js 未安装，正在通过 Homebrew 安装..."
    brew install node
    echo "✓ Node.js 安装完成"
elif [ -f "$HOME/.nvm/nvm.sh" ]; then
    echo "✗ Node.js 未安装但 nvm 可用，正在安装..."
    export NVM_DIR="$HOME/.nvm"
    [ -s "$NVM_DIR/nvm.sh" ] && . "$NVM_DIR/nvm.sh"
    nvm install 20
    nvm use 20
    nvm alias default 20
    echo "✓ Node.js 安装完成"
else
    echo "✗ 未找到 Homebrew 或 nvm，请手动安装 Node.js: https://nodejs.org"
    exit 1
fi

# ── 2. mimo2codex ─────────────────────────────────────────
if command -v mimo2codex >/dev/null 2>&1; then
    echo "✓ mimo2codex $(mimo2codex --version)"
else
    echo "✗ mimo2codex 未安装，正在安装..."
    npm install -g mimo2codex
    echo "✓ mimo2codex 安装完成"
fi

echo "=== 环境检测完成 ==="
