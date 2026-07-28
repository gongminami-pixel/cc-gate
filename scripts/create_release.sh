#!/bin/bash
set -e
TOKEN=$(security find-internet-password -s github.com -w 2>/dev/null)
VERSION="0.1.5"
TAG="v0.1.5"
REPO="gongminami-pixel/cc-gate"
API="https://api.github.com/repos/$REPO"
AUTH="Authorization: token $TOKEN"

MAC_DMG="src-tauri/target/release/bundle/dmg/CC-Gate_${VERSION}_x64.dmg"
WIN_EXE="/tmp/CC-Gate_${VERSION}_x64-setup.exe"
MAC_SHA=$(shasum -a 256 "$MAC_DMG" | awk '{print $1}')
WIN_SHA=$(shasum -a 256 "$WIN_EXE" | awk '{print $1}')

echo "Mac: $MAC_SHA"
echo "Win: $WIN_SHA"

BODY_FILE=$(mktemp)
python3 << PYEOF > "$BODY_FILE"
import json
body = "\n".join([
    "## Download",
    "",
    "| Platform | File | SHA256 |",
    "|----------|------|--------|",
    "| macOS | [CC-Gate_${VERSION}_x64.dmg](https://github.com/${REPO}/releases/download/${TAG}/CC-Gate_${VERSION}_x64.dmg) | \`${MAC_SHA}\` |",
    "| Windows | [CC-Gate_${VERSION}_x64-setup.exe](https://github.com/${REPO}/releases/download/${TAG}/CC-Gate_${VERSION}_x64-setup.exe) | \`${WIN_SHA}\` |",
    "",
    "### Changes",
    "",
    "- Fix: 恢复后刷新配置+应用按钮始终可点击",
    "- Fix: 应用/恢复改为单按钮切换",
])
print(json.dumps({"tag_name": "${TAG}", "name": "CC-Gate ${TAG}", "body": body, "draft": False}))
PYEOF

RELEASE_RESP=$(curl -sS -X POST -H "$AUTH" -H "Content-Type: application/json" --data-binary "@$BODY_FILE" "$API/releases")
rm -f "$BODY_FILE"

UPLOAD_URL=$(echo "$RELEASE_RESP" | python3 -c "import sys,json; print(json.load(sys.stdin)['upload_url'].split('{')[0])")
echo "Upload URL: $UPLOAD_URL"

curl -sS -X POST -H "$AUTH" -H "Content-Type: application/octet-stream" --data-binary "@$MAC_DMG" "$UPLOAD_URL?name=CC-Gate_${VERSION}_x64.dmg" > /dev/null && echo "Mac OK"
curl -sS -X POST -H "$AUTH" -H "Content-Type: application/octet-stream" --data-binary "@$WIN_EXE" "$UPLOAD_URL?name=CC-Gate_${VERSION}_x64-setup.exe" > /dev/null && echo "Win OK"
echo "Done: https://github.com/$REPO/releases/tag/$TAG"
