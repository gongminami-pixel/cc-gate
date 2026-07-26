# Harness Handoff

_Last updated: 2026-07-26T14:20:00+08:00_

## Goal

构建 CC-Gate — Tauri 2 + Vue 3 桌面应用，统一管理 10 AI Agent 的模型/路由/Key/Shell。开源在 https://github.com/gongminami-pixel/cc-gate。

## State snapshot

- **Branch**: main
- **Commit**: 40bf058 docs(harness): 同步 — Release 上传完成 + 模型参数更新
- **Uncommitted**: proxy_manager.rs 重写 + PageStartup.vue 代理状态栏 + PageHome.vue 断连保护 + config_writer.rs defaultModel 修复 + commands 清理
- **Tag**: v0.1.0

## Context you must load (JIT)

- `src-tauri/src/proxy_manager.rs` — 代理管理核心：node 路径发现 + 端口占用清理 + 双验证存活性
- `src-tauri/src/config_writer.rs` — 配置写入（含 providers.json defaultModel 新增）
- `src/components/PageStartup.vue` — 启动页：自启 + 代理状态实时显示
- `src/components/PageHome.vue` — 首页：断连保护
- `models-catalog.json` — 远程模型目录源

## What works

- ✅ **3 代理无条件下启动**：App 打开即全起（mimo2codex:8688, claude-proxy:8689, chat-proxy:8690）
- ✅ **代理状态实时显示**：启动页"代理状态"栏，3 秒轮询，绿色呼吸灯动画
- ✅ **代理存活性双验证**：try_wait + TCP connect，防假活
- ✅ **node 路径自动发现**：nvm（优先有 mimo2codex 的版本）→ fnm → volta → Homebrew
- ✅ **端口占用自动清理**：启动前 lsof kill 占端口的僵尸进程
- ✅ **启动前写配置**：先 write_providers（保证 defaultModel 完整）再起代理
- ✅ **首页断连保护**：即将重启 claude-proxy 时弹 confirm
- ✅ 工具检测渐进式加载
- ✅ 远程模型目录自动更新
- ✅ GitHub Releases v0.1.0 已有 Mac DMG

## What's broken

- ⚠️ DMG 打包偶尔因旧 app 未退出失败（需先 pkill 再 build）
- ⚠️ 用户说 mimicodex 码偶有显示问题（非本会话范围）

## Next actions (ordered)

1. 双端构建 + GitHub Release 更新（即将执行）
2. 模型参数校准（qwen3.8/mimo 上下文）
3. OpenCode 配置策略调研

## Beware

- **mimo2codex 路径**：跟 node 同一个 bin 目录（nvm 的 bin/mimo2codex），不是 `~/.mimo2codex/` 下的 .js 文件
- **providers.json**：每个 provider 必须有 `defaultModel` 字段，不然 mimo2codex 启动报退出码 2
- **macOS GUI PATH**：Tauri app 不继承用户 shell PATH，所有外部命令须完整路径
- **lsof kill**：启动代理前用 `lsof -ti :PORT` 杀占端口的旧进程
- **v0.1.0 tag 已推送 GitHub**：release.sh 上传时会覆盖已有 assets
