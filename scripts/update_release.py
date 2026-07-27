#!/usr/bin/env python3
"""Update GitHub Release body with new SHA256 values."""
import json, subprocess, sys
import urllib.request, ssl

# Bypass SSL issues (macOS Python 3.11 cert problem)
ctx = ssl.create_default_context()
ctx.check_hostname = False
ctx.verify_mode = ssl.CERT_NONE

token = subprocess.run(
    ["security", "find-internet-password", "-s", "github.com", "-w"],
    capture_output=True, text=True
).stdout.strip()
if not token:
    print("No token found")
    sys.exit(1)

# Calculate SHA256s
dmg = "/Users/ami/pro/python/py3/破卷相关/cc-x-llm/src-tauri/target/release/bundle/dmg/CC-Gate_0.1.0_x64.dmg"
exe = "/Users/ami/pro/python/py3/破卷相关/cc-x-llm/src-tauri/target/x86_64-pc-windows-msvc/release/cc-gate.exe"

mac_sha = subprocess.run(["shasum", "-a", "256", dmg], capture_output=True, text=True).stdout.split()[0]
win_sha = subprocess.run(["shasum", "-a", "256", exe], capture_output=True, text=True).stdout.split()[0]

version = "0.1.0"
tag = "v0.1.0"
repo = "gongminami-pixel/cc-gate"

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

- **3-proxy always-on**: All three proxies start automatically — no manual toggles
- **Startup page proxy status**: Real-time status with breathing-dot animation + PID
- **Apply-button guard**: Confirmation dialog before restarting claude-proxy
- **Proxy stability**: Auto-detect node path (nvm/fnm/volta), auto-kill port squatters, dual liveness verification
- Tool detection with progressive loading
- Claude Opus 4.5 -> Opus 5 (context 1M)
- GPT-5.1 Codex -> GPT-5.6
- GLM-5.2 context 1M
- Remote model catalog auto-update
- README documentation
"""

payload = json.dumps({"body": body}).encode("utf-8")
req = urllib.request.Request(
    "https://api.github.com/repos/gongminami-pixel/cc-gate/releases/359924195",
    data=payload,
    headers={
        "Authorization": f"token {token}",
        "Content-Type": "application/json",
        "User-Agent": "cc-gate-release",
    },
    method="PATCH",
)

try:
    with urllib.request.urlopen(req, context=ctx) as resp:
        result = json.loads(resp.read())
        print(f"OK: {result['html_url']}")
        print(f"macOS SHA256: {mac_sha}")
        print(f"Windows SHA256: {win_sha}")
except Exception as e:
    print(f"Error: {e}")
    if hasattr(e, 'read'):
        print(e.read().decode()[:500])
