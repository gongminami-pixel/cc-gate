# CC-Gate Windows 环境安装（非交互式）
# 检测 & 提示缺失，然后尽力安装
$ErrorActionPreference = "Continue"

# Node.js — 只检测，不强制安装（winget 需要管理员权限）
try {
    $n = node --version 2>$null
    if ($LASTEXITCODE -eq 0) { Write-Host "node $n" }
} catch { }

# npm
try {
    $n = npm --version 2>$null
    if ($LASTEXITCODE -eq 0) { Write-Host "npm $n" }
} catch { }

# mimo2codex — 有 node/npm 就自动安装
try {
    $m = mimo2codex --version 2>$null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "mimo2codex $m"
    } else {
        throw "not found"
    }
} catch {
    try {
        Write-Host "Installing mimo2codex..."
        npm install -g mimo2codex 2>&1
        $m2 = mimo2codex --version 2>$null
        if ($LASTEXITCODE -eq 0) { Write-Host "mimo2codex $m2 installed" }
    } catch { }
}
