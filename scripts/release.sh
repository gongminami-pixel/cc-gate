#!/bin/bash
# CC-Gate release script - creates GitHub Release + uploads Mac DMG + Windows exe
set -e

TOKEN=$(security find-internet-password -s github.com -w 2>/dev/null)
if [ -z "$TOKEN" ]; then
  echo "GitHub token not found in keychain, enter manually:"
  read -s -p "GitHub token: " TOKEN
  echo
fi

REPO="gongminami-pixel/cc-gate"
TAG="v0.1.16"
VERSION="0.1.16"
DMG="src-tauri/target/release/bundle/dmg/CC-Gate_0.1.16_x64.dmg"
EXE="/tmp/CC-Gate_0.1.16_x64-setup.exe"
MAC_SHA=$(shasum -a 256 "$DMG" | awk '{print $1}')
WIN_SHA=$(shasum -a 256 "$EXE" | awk '{print $1}')

echo "=== SHA256 ==="
echo "Mac DMG:  $MAC_SHA"
echo "Win exe:  $WIN_SHA"
echo ""

AUTH="Authorization: token $TOKEN"
API="https://api.github.com/repos/$REPO"

# Check if release already exists
echo "Checking existing release..."
EXISTING_ID=$(curl -sS -H "$AUTH" "$API/releases/tags/$TAG" | python3 -c "import sys,json; print(json.load(sys.stdin).get('id',''))" 2>/dev/null || echo "")

if [ -n "$EXISTING_ID" ]; then
  echo "Release already exists (id=$EXISTING_ID), fetching upload URL..."
  UPLOAD_URL=$(curl -sS -H "$AUTH" "$API/releases/$EXISTING_ID" | python3 -c "import sys,json; print(json.load(sys.stdin)['upload_url'].split('{')[0])")
else
  # Write release body JSON to temp file to avoid shell escaping hell
  BODY_FILE=$(mktemp)
  python3 << PYEOF > "$BODY_FILE"
import json
body = """## Download

| Platform | File | SHA256 |
|----------|------|--------|
| macOS | [CC-Gate_${VERSION}_x64.dmg](https://github.com/${REPO}/releases/download/${TAG}/CC-Gate_${VERSION}_x64.dmg) | \`${MAC_SHA}\` |
| Windows | [CC-Gate_${VERSION}_x64-setup.exe](https://github.com/${REPO}/releases/download/${TAG}/CC-Gate_${VERSION}_x64-setup.exe) | \`${WIN_SHA}\` |

### Verify

\`\`\`bash
# macOS
shasum -a 256 CC-Gate_${VERSION}_x64.dmg
# Windows (PowerShell)
Get-FileHash CC-Gate_${VERSION}_x64-setup.exe -Algorithm SHA256
\`\`\`

### Changes (v0.1.16)

- 无后缀 alias（codex / claude / aider）= 官方原生调用（codex→官方 OpenAI gpt-5.5，claude→api.anthropic.com，aider→裸命令）；带后缀 alias 仍走本地代理
- 移除非线智能中转（代码/注释/测试全清），providers.json 仅剩 4 个官方直连 provider（DeepSeek / GLM / Qwen / MiMo）
- 快速添加中转弹窗预设只留 OpenRouter
- Shell 集成页别名列表新增原生条目展示
- 代理层修复：count_tokens 端点 + isAnthropicNative 检测
- 新增回归测试（原生/代理 alias 生成断言）与无 GUI 应用配置工具
"""
print(json.dumps({"tag_name": "${TAG}", "name": "CC-Gate ${TAG}", "body": body, "draft": False}))
PYEOF

  echo "Creating release..."
  RELEASE_RESP=$(curl -sS -X POST -H "$AUTH" -H "Content-Type: application/json" --data-binary "@$BODY_FILE" "$API/releases")
  rm -f "$BODY_FILE"

  HTML_URL=$(echo "$RELEASE_RESP" | python3 -c "import sys,json; print(json.load(sys.stdin).get('html_url',''))")
  UPLOAD_URL=$(echo "$RELEASE_RESP" | python3 -c "import sys,json; print(json.load(sys.stdin)['upload_url'].split('{')[0])")
  echo "Release created: $HTML_URL"
fi

echo ""

# Upload Mac DMG
echo "Uploading Mac DMG ($(ls -lh "$DMG" | awk '{print $5}'))..."
curl -sS -X POST -H "$AUTH" -H "Content-Type: application/octet-stream" \
  --data-binary "@$DMG" \
  "$UPLOAD_URL?name=CC-Gate_${VERSION}_x64.dmg" > /dev/null
echo "  OK"

# Upload Windows setup exe
echo "Uploading Windows setup exe ($(ls -lh "$EXE" | awk '{print $5}'))..."
curl -sS -X POST -H "$AUTH" -H "Content-Type: application/octet-stream" \
  --data-binary "@$EXE" \
  "$UPLOAD_URL?name=CC-Gate_${VERSION}_x64-setup.exe" > /dev/null
echo "  OK"

echo ""
echo "Done: https://github.com/$REPO/releases/tag/$TAG"
echo ""
echo "=== SHA256 (for reference) ==="
echo "Mac:  $MAC_SHA"
echo "Win:  $WIN_SHA"
