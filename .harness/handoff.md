# Harness Handoff

_Last updated: 2026-07-27T14:40:00+08:00_

## Goal

构建 CC-Gate — Tauri 2 + Vue 3 桌面应用，统一管理 10 AI Agent 的模型/路由/Key/Shell。开源在 https://github.com/gongminami-pixel/cc-gate。

## State snapshot

- **Branch**: main
- **Commit**: 56b3c09 feat: 代理全员启动 + 启动页实时状态 + 首页断连保护
- **Uncommitted**:
  - `claude-proxy.js` — SSE 流代理 4 bug 修复（in project root）
  - `src-tauri/src/commands/config.rs` — (从上次遗留)
  - `src-tauri/src/config_writer.rs` — (从上次遗留)
  - `src-tauri/src/lib.rs` — (从上次遗留)
  - `src-tauri/src/proxy_manager.rs` — (从上次遗留)
  - `src/components/PageHome.vue` — (从上次遗留)
  - `src/ipc/api.ts` — (从上次遗留)
  - 新增文件: `scripts/update_release.py`, `scripts/update_release_body.py`, `src-tauri/src/backup.rs`, `claude-proxy-sse-fix.md`, `status-bar-fix-report.md`
- **Tag**: v0.1.0（需要 bump 到 v0.1.1）

## Context you must load (JIT)

- `src-tauri/src/proxy_manager.rs` — 代理管理核心：node 路径发现 + 端口占用清理 + 双验证存活性
- `src-tauri/src/config_writer.rs` — 配置写入（含 providers.json defaultModel 新增）
- `src/components/PageStartup.vue` — 启动页：自启 + 代理状态实时显示
- `src/components/PageHome.vue` — 首页：断连保护
- `models-catalog.json` — 远程模型目录源
- `claude-proxy.js` — SSE 流代理脚本（Anthropic→OpenAI 协议转换，已修复 4 bug）

## What works

- ✅ **3 代理无条件下启动**：App 打开即全起（mimo2codex:8688, claude-proxy:8689, chat-proxy:8690）
- ✅ **代理状态实时显示**：启动页"代理状态"栏，3 秒轮询，绿色呼吸灯动画
- ✅ **代理存活性双验证**：try_wait + TCP connect，防假活
- ✅ **node 路径自动发现**：nvm（优先有 mimo2codex 的版本）→ fnm → volta → Homebrew
- ✅ **端口占用自动清理**：启动前 lsof kill 占端口的僵尸进程
- ✅ **启动前写配置**：先 write_providers（保证 defaultModel 完整）再起代理
- ✅ **首页断连保护**：即将重启 claude-proxy 时弹 confirm
- ✅ **SSE 流修复**（2026-07-27）：claude-proxy.js 修复 4 个 bug
  - 双重 message_stop → `emitFinal()` 互斥位
  - 缺失 tool_use SSE 事件 → `tcMap` + `doTools()`
  - output_tokens=0 → 从 finish_reason chunk 正确读取
  - input_tokens=0 → `pending[]` 缓冲区等待 DeepSeek 最后 chunk 到达
- ✅ `/v1/models` 接口返回 context_window + max_output_tokens
- ✅ 状态栏配置：模型名(亮青) | 目录 | ctx: Xk/1.0M | $x.xx
- ✅ 工具检测渐进式加载
- ✅ 远程模型目录自动更新
- ✅ GitHub Releases v0.1.0 已有 Mac DMG

## What's broken

- ⚠️ DMG 打包偶尔因旧 app 未退出失败（需先 pkill 再 build）
- ⚠️ Claude Code 不认 `/v1/models` 的 context_window 字段，状态栏硬编码各模型上下文窗口做 workaround
- ⚠️ cc-gate 有文件备份机制可能还原 claude-proxy.js，源码修复需随构建打包进 app

## Next actions (ordered)

1. **本次会话**：git 提交 + push + 双端构建 + GitHub Release 上传 + SHA256 更新
2. 下次版本 bump 到 v0.1.1
3. claude-proxy.js 嵌入 Tauri bundle（打包进 Resources）防被 cc-gate backup 还原
4. OpenCode 配置策略调研
5. Windows 构建支持（当前仅 Mac 端）

## Beware

- **mimo2codex 路径**：跟 node 同一个 bin 目录（nvm 的 bin/mimo2codex），不是 `~/.mimo2codex/` 下的 .js 文件
- **providers.json**：每个 provider 必须有 `defaultModel` 字段，不然 mimo2codex 启动报退出码 2
- **macOS GUI PATH**：Tauri app 不继承用户 shell PATH，所有外部命令须完整路径
- **lsof kill**：启动代理前用 `lsof -ti :PORT` 杀占端口的旧进程
- **claude-proxy.js 双重部署**：项目根有源码，运行时在 `~/.mimo2codex/claude-proxy.js`（cc-gate 读取）+ `/tmp/claude-proxy-fixed.js`（cc-gate 备份机制写入）
- **v0.1.0 tag 已推送 GitHub**：release.sh 上传时会覆盖已有 assets
- **DeepSeek stream**：thinking 必须 disabled（`thinking: { type: 'disabled' }`），否则 tool_calls 进 reasoning_content 导致 Claude Code 解析失败
