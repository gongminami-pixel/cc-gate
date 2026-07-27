#!/usr/bin/env python3
"""Update GitHub Release body with new SHA256 values."""
import json, subprocess, sys

token = subprocess.run(
    ["security", "find-internet-password", "-s", "github.com", "-w"],
    capture_output=True, text=True
).stdout.strip()
if not token:
    print("No token found")
    sys.exit(1)

api = "https://api.github.com/repos/gongminami-pixel/cc-gate"
version = "0.1.0"
tag = "v0.1.0"
repo = "gongminami-pixel/cc-gate"

# Calculate SHA256s
dmg_path = "/Users/ami/pro/python/py3/破卷相关/cc-x-llm/src-tauri/target/release/bundle/dmg/CC-Gate_0.1.0_x64.dmg"
exe_path = "/Users/ami/pro/python/py3/破卷相关/cc-x-llm/src-tauri/target/x86_64-pc-windows-msvc/release/cc-gate.exe"

mac_sha_result = subprocess.run(["shasum", "-a", "256", dmg_path], capture_output=True, text=True)
mac_sha = mac_sha_result.stdout.split()[0] if mac_sha_result.returncode == 0 else "PENDING"

win_sha = "PENDING (rebuild on Windows)"
import os
if os.path.exists(exe_path):
    win_sha_result = subprocess.run(["shasum", "-a", "256", exe_path], capture_output=True, text=True)
    win_sha = win_sha_result.stdout.split()[0] if win_sha_result.returncode == 0 else win_sha

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

- **3-proxy always-on**: All three proxies (mimo2codex/claude-proxy/chat-proxy) start automatically when the app launches — no manual toggles needed
- **Startup page status panel**: Real-time proxy status with breathing-dot animation and PID display
- **Apply-button guard**: Confirmation dialog before restarting claude-proxy to prevent disconnecting the current chat session
- **Proxy stability fixes**: Auto-detect node path (nvm/fnm/volta/Homebrew), auto-kill port squatters, dual-verification liveness check (try_wait + TCP connect)
- Tool detection with progressive loading (one-by-one live status)
- Claude Opus 4.5 -> Opus 5 (context 200K -> 1M)
- GPT-5.1 Codex -> GPT-5.6
- GLM-5.2 context 128K -> 1M
- Remote model catalog auto-update (models-catalog.json from GitHub)
- README with open-source documentation
"""

import urllib.request
payload = json.dumps({"body": body}).encode("utf-8")
req = urllib.request.Request(
    f"{api}/releases/359924195",
    data=payload,
    headers={
        "Authorization": f"token {token}",
        "Content-Type": "application/json",
        "User-Agent": "cc-gate-release",
    },
    method="PATCH",
)
try:
    with urllib.request.urlopen(req) as resp:
        result = json.loads(resp.read())
        print(f"Release updated: {result['html_url']}")
        print(f"macOS SHA256: {mac_sha}")
        print(f"Windows SHA256: {win_sha}")
except urllib.error.HTTPError as e:
    print(f"HTTP Error: {e.code}")
    print(e.read().decode()[:500])
