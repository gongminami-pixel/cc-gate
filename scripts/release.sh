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

# Use Python for API calls - easier error handling
python3 << PYEOF
import subprocess, json, os

token = os.environ["TOKEN"]
repo = os.environ["REPO"]
tag = os.environ["TAG"]
version = os.environ["VERSION"]
dmg = os.environ["DMG"]
exe = os.environ["EXE"]
mac_sha = os.environ["MAC_SHA"]
win_sha = os.environ["WIN_SHA"]

def api(method, url, data=None, raw=False):
    import urllib.request
    req = urllib.request.Request(url, method=method)
    req.add_header("Authorization", f"token {token}")
    if data is not None:
        if isinstance(data, bytes):
            req.add_header("Content-Type", "application/octet-stream")
            req.data = data
        else:
            req.add_header("Content-Type", "application/json")
            req.data = json.dumps(data).encode()
    try:
        with urllib.request.urlopen(req) as resp:
            return json.loads(resp.read())
    except urllib.error.HTTPError as e:
        body = e.read().decode()
        try: return json.loads(body)
        except: return {"message": body}

# Check if release already exists
print("Checking existing release...")
existing = api("GET", f"https://api.github.com/repos/{repo}/releases/tags/{tag}")

if "id" in existing:
    print(f"Release already exists (id={existing['id']}), reusing...")
    upload_url = existing["upload_url"].split("{")[0]
else:
    # Create release
    print("Creating release...")
    body = f"""## Download

| Platform | File | SHA256 |
|----------|------|--------|
| macOS | [CC-Gate_{version}_x64.dmg](https://github.com/{repo}/releases/download/{tag}/CC-Gate_{version}_x64.dmg) | `{mac_sha}` |
| Windows | [cc-gate.exe](https://github.com/{repo}/releases/download/{tag}/cc-gate.exe) | `{win_sha}` |

### Verify

```bash
# macOS
shasum -a 256 CC-Gate_{version}_x64.dmg
# Windows (PowerShell)
Get-FileHash cc-gate.exe -Algorithm SHA256
```

### Changes

- Tool detection with progressive loading (one-by-one with live status)
- Claude Opus 4.5 -> Opus 5 (context 200K -> 1M)
- GPT-5.1 Codex -> GPT-5.6
- GLM-5.2 context 128K -> 1M
- Remote model catalog auto-update (models-catalog.json from GitHub raw)
- README with open-source documentation"""

    release = api("POST", f"https://api.github.com/repos/{repo}/releases", {
        "tag_name": tag,
        "name": f"CC-Gate {tag}",
        "body": body,
        "draft": False,
    })
    if "upload_url" not in release:
        print(f"ERROR creating release: {release}")
        exit(1)
    upload_url = release["upload_url"].split("{")[0]
    print(f"Release created: {release['html_url']}")

# Upload assets
print(f"Upload URL: {upload_url}")

# Mac DMG
print(f"Uploading Mac DMG ({os.path.getsize(dmg)} bytes)...")
with open(dmg, "rb") as f:
    result = api("POST", f"{upload_url}?name=CC-Gate_{version}_x64.dmg", f.read(), raw=True)
    if "name" in result:
        print(f"  OK: {result['name']} ({result.get('size', '?')} bytes)")
    else:
        print(f"  FAIL: {result}")

# Windows exe
print(f"Uploading Windows exe ({os.path.getsize(exe)} bytes)...")
with open(exe, "rb") as f:
    result = api("POST", f"{upload_url}?name=cc-gate.exe", f.read(), raw=True)
    if "name" in result:
        print(f"  OK: {result['name']} ({result.get('size', '?')} bytes)")
    else:
        print(f"  FAIL: {result}")

print(f"\nDone: https://github.com/{repo}/releases/tag/{tag}")
PYEOF
