#!/bin/bash
# CC-Gate release script - creates GitHub Release + uploads Mac DMG + Windows exe
set -e

TOKEN=$(security find-internet-password -s github.com -w 2>/dev/null)
if [ -z "$TOKEN" ]; then
  echo "GitHub token not found in keychain, enter manually:"
  read -s -p "GitHub token: " TOKEN
  echo
fi

export TOKEN
export REPO="gongminami-pixel/cc-gate"
export TAG="v0.1.0"
export VERSION="0.1.0"
export DMG="src-tauri/target/release/bundle/dmg/CC-Gate_0.1.0_x64.dmg"
export EXE="src-tauri/target/x86_64-pc-windows-msvc/release/cc-gate.exe"
export MAC_SHA=$(shasum -a 256 "$DMG" | awk '{print $1}')
export WIN_SHA=$(shasum -a 256 "$EXE" | awk '{print $1}')

echo "=== SHA256 ==="
echo "Mac DMG:  $MAC_SHA"
echo "Win exe:  $WIN_SHA"
echo ""

python3 - "$TOKEN" "$REPO" "$TAG" "$VERSION" "$DMG" "$EXE" "$MAC_SHA" "$WIN_SHA" << 'PYEOF'
import subprocess, json, os, sys, urllib.request, urllib.error

token = sys.argv[1]
repo = sys.argv[2]
tag = sys.argv[3]
version = sys.argv[4]
dmg = sys.argv[5]
exe = sys.argv[6]
mac_sha = sys.argv[7]
win_sha = sys.argv[8]

def api(method, url, data=None, is_binary=False):
    req = urllib.request.Request(url, method=method)
    req.add_header("Authorization", f"token {token}")
    if data is not None:
        if is_binary:
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
        except: return {"message": body, "status": e.code}

# Check if release already exists
print("Checking existing release...")
existing = api("GET", f"https://api.github.com/repos/{repo}/releases/tags/{tag}")

if "id" in existing:
    print(f"Release already exists (id={existing['id']}), reusing...")
    upload_url = existing["upload_url"].split("{")[0]
else:
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

- Tool detection with progressive loading (one-by-one live status)
- Claude Opus 4.5 -> Opus 5 (context 200K -> 1M)
- GPT-5.1 Codex -> GPT-5.6
- GLM-5.2 context 128K -> 1M
- Remote model catalog auto-update (models-catalog.json from GitHub)
- README with open-source documentation"""

    print("Creating release...")
    release = api("POST", f"https://api.github.com/repos/{repo}/releases", {
        "tag_name": tag,
        "name": f"CC-Gate {tag}",
        "body": body,
        "draft": False,
    })
    if "upload_url" not in release:
        print(f"ERROR creating release: {json.dumps(release, indent=2)}")
        sys.exit(1)
    upload_url = release["upload_url"].split("{")[0]
    print(f"Release created: {release['html_url']}")

# Upload Mac DMG
print(f"Uploading Mac DMG ({os.path.getsize(dmg)} bytes)...")
with open(dmg, "rb") as f:
    result = api("POST", f"{upload_url}?name=CC-Gate_{version}_x64.dmg", f.read(), is_binary=True)
    if "name" in result:
        print(f"  OK: {result['name']} ({result.get('size', '?')} bytes)")
    else:
        print(f"  FAIL: {json.dumps(result)}")

# Upload Windows exe
print(f"Uploading Windows exe ({os.path.getsize(exe)} bytes)...")
with open(exe, "rb") as f:
    result = api("POST", f"{upload_url}?name=cc-gate.exe", f.read(), is_binary=True)
    if "name" in result:
        print(f"  OK: {result['name']} ({result.get('size', '?')} bytes)")
    else:
        print(f"  FAIL: {json.dumps(result)}")

print(f"\nDone: https://github.com/{repo}/releases/tag/{tag}")
PYEOF
