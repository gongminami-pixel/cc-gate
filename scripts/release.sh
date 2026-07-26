#!/bin/bash
# CC-Gate release script — creates GitHub Release + uploads Mac DMG + Windows exe
set -e

TOKEN=$(security find-internet-password -s github.com -w 2>/dev/null)
if [ -z "$TOKEN" ]; then
  echo "请先从钥匙串读取 GitHub token 失败，手动输入："
  read -s -p "GitHub token: " TOKEN
  echo
fi

REPO="gongminami-pixel/cc-gate"
TAG="v0.1.0"
VERSION="0.1.0"
DMG="src-tauri/target/release/bundle/dmg/CC-Gate_${VERSION}_x64.dmg"
EXE="src-tauri/target/x86_64-pc-windows-msvc/release/cc-gate.exe"

echo "=== SHA256 ==="
MAC_SHA=$(shasum -a 256 "$DMG" | awk '{print $1}')
WIN_SHA=$(shasum -a 256 "$EXE" | awk '{print $1}')
echo "Mac DMG:  $MAC_SHA"
echo "Win exe:  $WIN_SHA"
echo ""

# Create release
echo "Creating release..."
RELEASE_JSON=$(curl -sS -X POST \
  -H "Authorization: token $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"tag_name\": \"$TAG\",
    \"name\": \"CC-Gate $TAG\",
    \"body\": \"## 下载\n\n| 平台 | 下载 | SHA256 |\n|------|------|--------|\n| macOS | [CC-Gate_${VERSION}_x64.dmg](https://github.com/$REPO/releases/download/$TAG/CC-Gate_${VERSION}_x64.dmg) | \`$MAC_SHA\` |\n| Windows | [cc-gate.exe](https://github.com/$REPO/releases/download/$TAG/cc-gate.exe) | \`$WIN_SHA\` |\n\n### 校验\n\`\`\`bash\n# macOS\nshasum -a 256 CC-Gate_${VERSION}_x64.dmg\n# Windows (PowerShell)\nGet-FileHash cc-gate.exe -Algorithm SHA256\n\`\`\`\n\n### 改动\n\n- 工具检测渐进式加载（逐条检测、逐条更新）\n- Claude Opus 4.5 → Opus 5（上下文 200K → 1M）\n- GPT-5.1 Codex → GPT-5.6\n- GLM-5.2 上下文 128K → 1M\n- 远程模型目录自动更新（models-catalog.json GitHub raw）\n- README 开源说明文档\",
    \"draft\": false
  }" \
  "https://api.github.com/repos/$REPO/releases")

UPLOAD_URL=$(echo "$RELEASE_JSON" | python3 -c "import sys,json; print(json.load(sys.stdin)['upload_url'].split('{')[0])")
echo "Release created: $UPLOAD_URL"

# Upload assets
echo "Uploading Mac DMG..."
curl -sS -X POST \
  -H "Authorization: token $TOKEN" \
  -H "Content-Type: application/octet-stream" \
  --data-binary "@$DMG" \
  "$UPLOAD_URL?name=CC-Gate_${VERSION}_x64.dmg" | python3 -c "import sys,json; r=json.load(sys.stdin); print(f'  ✓ {r[\"name\"]} ({r[\"size\"]} bytes)')"

echo "Uploading Windows exe..."
curl -sS -X POST \
  -H "Authorization: token $TOKEN" \
  -H "Content-Type: application/octet-stream" \
  --data-binary "@$EXE" \
  "$UPLOAD_URL?name=cc-gate.exe" | python3 -c "import sys,json; r=json.load(sys.stdin); print(f'  ✓ {r[\"name\"]} ({r[\"size\"]} bytes)')"

echo ""
echo "=== Done ==="
echo "下载页: https://github.com/$REPO/releases/tag/$TAG"
