# Harness Handoff

_Last updated: 2026-08-03T04:36:11+08:00_

## Goal

构建 CC-Gate — Tauri 2 + Vue 3 桌面应用，统一管理 10 AI Agent 的模型/路由/Key/Shell。开源在 https://github.com/gongminami-pixel/cc-gate。

## State snapshot

- **Branch**: main
- **Commit**: cc44b60 docs(harness): 同步记忆 — 开源项目 push 规则决策
- **Tag**: v0.1.13
- **Released**: v0.1.13 GitHub Release（macOS DMG + Windows exe 双包 + SHA256，2026-08-02 发布）
- **Uncommitted** (3 files, 实质改动):
  - `claude-proxy.js`: SSE 空内容过滤（`d.content !== ''` 防空字符串伪触发）
  - `src-tauri/src/proxy_manager.rs`: macOS lsof 绝对路径 `/usr/sbin/lsof`（GUI 不继承 PATH）
  - `src-tauri/tauri.conf.json`: version bump `0.1.12` → `0.1.13`

## Context you must load (JIT)

- `src-tauri/src/config_writer.rs` — 配置写入（alias 生成含 `\codex`/`\claude`/`\aider` 防展开）
- `claude-proxy.js` — 代理路由核心（SSE 流式处理）
- `chat-proxy.js` — chat 代理逻辑
- `src-tauri/src/proxy_manager.rs` — 端口管理 + lsof 路径
- `src-tauri/src/commands/usage.rs` — 日志诊断出口
- `src/components/PageAbout.vue` — 诊断信息页
- `windows-vm-build-guide.md` — Windows 虚拟机构建 Runbook
- `~/.config/gh/hosts.yml` — GitHub CLI 认证（gongminami-pixel）

## What works

- ✅ v0.1.13 GitHub Release 已发布（2026-08-02）：macOS DMG + Windows exe 双包 + SHA256
- ✅ claude-proxy.js SSE 空内容过滤修复
- ✅ proxy_manager.rs macOS lsof 绝对路径修复（GUI 不继承 /usr/sbin PATH）
- ✅ 版本号 bump 到 0.1.13
- ✅ zsh alias 递归展开 bug 已修复（`\codex`/`\claude`/`\aider`）
- ✅ 诊断信息页（版本号 + 日志尾部 + 一键复制）
- ✅ Claude alias 加 `--permission-mode bypassPermissions --thinking disabled`（仅 deepseek 加 thinking disabled）

### v0.1.13 SHA256
- **macOS**: `7db74b7b23fd17ab3d48901afaf2732e18c7c63bdffda0ca48f88087f24b125b`
- **Windows**: `ef870caf68fea629ab76fddea4dca4d86008ae7e90c97ce1d21f4b958f49d824`

## What's broken

- ⚠️ 镜像站点加载卡顿（网络问题，非 CC-Gate bug）

## Next actions (ordered)

1. 提交工作区 3 个未提交改动（SSE 修复 + lsof 路径修复 + version bump）
2. 推送 commit 到 GitHub 并同步更新 Release 的 Release Notes（如有代码提交 SHA）
3. 用户验证 v0.1.13 修复效果
4. 如有新需求，继续迭代

## Beware

- **Windows 构建**：用 `win-vm-build` skill；每次用全新 `cc-gate-build` 目录，`rmdir /s /q` 清理
- **tar 打包**必须含 `claude-proxy.js chat-proxy.js scripts/`（`include_str!` 依赖）
- **relay_env_key() 是单真源**，所有写 API key 的地方必须调用它
- **per-model alias 末尾命令名**必须用 `\codex`/`\claude`/`\aider` 防 zsh 递归展开
- **GitHub 操作**：`gh` CLI 通过 `~/.config/gh/hosts.yml` 认证（gongminami-pixel），操作前验证 `gh auth status`
- **构建上传后**：必须更新 GitHub Release 的 SHA256 值
- **gh release upload --clobber** 可覆盖同文件名资产，无需 delete release
- **★本项目是开源项目**（https://github.com/gongminami-pixel/cc-gate）：每次"同步加提交"后必须额外 `git push origin main`，与全局 CLAUDE.md "只本地不 push"不同
- **★v0.1.13 是全新 release tag**（非覆盖旧 tag），与之前 v0.1.12 四批覆盖更新不同
