# Harness Progress Log

_Append-only. Newest at bottom. ISO8601 timestamps only._

---

## 2026-07-25T22:45:00+08:00 — init: harness framework bootstrapped
- **touched**: .harness/README.md .harness/progress.md .harness/decisions.md .harness/handoff.md
- **action**: 通过 harness-framework skill 懒 init 路径创建 .harness/ 骨架
- **outcome**: 4 文件就绪，waypoints/ 和 context/ 子目录已建立
- **next**: 首次 git 提交 + 双端构建

## 2026-07-25T23:00:00+08:00 — work: 完成 Agent 配置全覆盖 + 工具检测独立页
- **touched**: src-tauri/src/config_writer.rs src-tauri/src/types.rs src-tauri/src/tool_check.rs src-tauri/src/paths.rs src-tauri/src/error.rs src-tauri/Cargo.toml src/components/PageTools.vue src/components/PageHome.vue src/components/Sidebar.vue src/App.vue
- **action**:
  - 修正 codex_cli/claude_cli/aider 的 writes_providers: true，让模型进 providers.json
  - Reasonix writes_catalog: true，共享 Codex model catalog
  - write_model_catalog 合并 Codex CLI + Desktop 模型
  - write_codex_config 合并 Codex Desktop + Reasonix 模型
  - write_claude_settings 合并 Claude CLI + Desktop 模型，默认模型取第一个
  - 新增 write_hermes_config：serde_yaml 解析合并 ~/.hermes/config.yaml
  - 新增 write_openclaw_config：JSON5 兼容解析合并 ~/.openclaw/openclaw.json
  - 工具检测缓存（OnceLock<Mutex<Vec<ToolStatus>>>）+ refresh()
  - 工具检测从首页挪到独立 PageTools.vue + 左侧菜单"🔧 工具检测"
  - 首页应用按钮 dirty-aware（有改动亮、无改动灰"✓ 已保存"）
- **outcome**: cargo check + vue-tsc 通过，零错误
- **next**: 双端构建 + 本地提交

## 2026-07-25T23:10:00+08:00 — handoff_ready: 同步前交接
- **touched**: .harness/handoff.md .harness/progress.md .harness/decisions.md
- **action**: 重写 handoff.md，追加 progress 条目，写入 decisions 条目
- **outcome**: handoff 反映最新状态
- **next**: 首次 git 提交 + 双端构建

## 2026-07-26T00:30:00+08:00 — work: 后台构建（隐藏用量/模型参数菜单 + 乱码修复）
- **touched**: src/components/Sidebar.vue src/components/PageAbout.vue chat-proxy.js claude-proxy.js
- **action**: 
  - 侧边栏注释掉"用量统计"和"模型参数"菜单项（代码保留）
  - chat-proxy.js 和 claude-proxy.js 注释掉 recordUsage() 调用（代码保留）
  - PageAbout.vue "统���管理" 字节级修复为"统一管理"
- **outcome**: Mac 构建成功，已安装到 /Applications
- **next**: 回答用户关于模型版本同步的问题

## 2026-07-26T01:30:00+08:00 — work: 远程模型目录自动更新 feature
- **touched**: models-catalog.json src-tauri/src/model_catalog.rs src-tauri/Cargo.toml src-tauri/src/types.rs src-tauri/src/config_store.rs src-tauri/src/commands/config.rs src-tauri/src/error.rs src-tauri/src/lib.rs src/components/PageHome.vue src/types/models.ts src/ipc/api.ts .gitignore
- **action**:
  - 新建 models-catalog.json（9 个模型完整定义，放仓库根目录）
  - 新建 model_catalog.rs：fetch_remote_catalog + read_catalog_cache + save_catalog_cache + merge_remote_models
  - 启动时后台静默拉取远程 catalog，有新模型自动合并入本地配置
  - 首页模型列表 header 加"检查模型更新"按钮，新模型显示"新"badge
  - merge 逻辑：远程参数覆盖本地但保留 enabled 状态；远程新模型默认不勾选
  - 离线兜底：缓存 → builtin_models()
  - AppConfig 新增 model_catalog_version 字段
  - 新增 check_model_updates Tauri command
  - 前端监听 config-changed 事件自动刷新
  - reqwest (rustls-tls) 依赖
  - From<String> for AppError
  - .gitignore 加 .claude/ 排除私有会话状态
- **outcome**: Mac 构建成功 + 安装到 /Applications + git push 到 GitHub
- **next**: 用户测试"检查模型更新"（远程 URL 已生效）

## 2026-07-26T02:15:00+08:00 — handoff_ready: 同步记忆
- **touched**: .harness/waypoints/2026-07-26T02-15-00+08:00.md .harness/handoff.md .harness/progress.md .harness/decisions.md
- **action**: 落 waypoint + 重写 handoff + 追加 decisions
- **outcome**: 状态反映到远程模型目录 feature 完成后
- **next**: git 提交 .harness/ + 用户测试

## 2026-07-26T03:00:00+08:00 — work: README 开源说明文档 + 远程模型目录上线
- **touched**: README.md
- **action**:
  - 写 README.md 开源首页说明文档
  - ���部突出与 CC Switch 最大区别（CLI alias 多窗口并行 vs 全局单模型切换）
  - 分 CLI 和桌面端两个维度对比
  - push 到 GitHub 后模型目录 404 修复
- **outcome**: GitHub 首页可读，"检查模型更新"可正常拉取
- **next**: 工具检测体验优化

## 2026-07-26T11:00:00+08:00 — work: 工具检测渐进式加载（3 次迭代）
- **touched**: src-tauri/src/tool_check.rs src-tauri/src/lib.rs src/components/PageTools.vue src/ipc/api.ts
- **action**:
  - 第 1 版：check_progressive() + thread::spawn emit 事件 —— 失败（Tauri 在 command 期内缓冲事件）
  - 第 2 版：去掉 thread::spawn，同步 emit —— 仍然失败（同一问题）
  - 第 3 版：改为前端逐个调用 checkOneTool()（6 次独立 IPC），每调用一次 Rust 检测一个工具、返回一个结果、前端立即更新 UI —— 成功
  - 新增 saveToolCache() 命令回写缓存
  - 新增 check_one() 按名匹配检测
- **outcome**: 工具检测页面进即渲染 6 条"检测中…"，逐条亮起（已安装/未安装），体验流畅
- **next**: 模型参数校准 + 双端构建

## 2026-07-26T12:00:00+08:00 — work: 模型参数更新 + 双端构建 + Release 脚本
- **touched**: models-catalog.json src-tauri/src/types.rs src-tauri/src/config_writer.rs README.md scripts/release.sh
- **action**:
  - Claude Opus 4.5 → Opus 5（slug: claude-opus-5，上下文 200K → 1M）
  - GPT-5.1 Codex → GPT-5.6（slug: gpt-5.6）
  - GLM-5.2 上下文 128K → 1M
  - 同步更新 models-catalog.json + builtin_models() + short() alias 映射 + README 模型表
  - Mac 构建 (3.9MB DMG) + Windows 构建 (7.1MB exe)
  - 创建 scripts/release.sh（curl 创建 GitHub Release + 上传双端 MA）
  - v0.1.0 tag 已推送
- **outcome**: 双端包就绪，发布脚本就绪，用户跑 release.sh 即可上传
- **next**: 用户跑 release.sh → 下载页就绪

## 2026-07-26T12:15:00+08:00 — handoff_ready: 同步加提交
- **touched**: .harness/waypoints/2026-07-26T12-15-00+08:00.md .harness/handoff.md .harness/progress.md .harness/decisions.md
- **action**: 落 waypoint + 重写 handoff + 追加 progress/decisions + git 提交
- **outcome**: 状态完整反映到最新
- **next**: 用户跑 release.sh + 模型参数进一步校准

## 2026-07-26T12:30:00+08:00 — work: 双端安装包成功上传到 GitHub Releases
- **touched**: scripts/release.sh
- **action**: 
  - release.sh 迭代 4 次（shell 转义+Python SSL 证书+curl 混 bash 文件损坏→最终纯 curl 方案）
  - Mac DMG (3.9MB) + Windows exe (7.1MB) 上传到 v0.1.0 Release
  - SHA256 校验码写入 Release 正文
- **outcome**: 下载页 https://github.com/gongminami-pixel/cc-gate/releases/tag/v0.1.0 可正常下载
- **next**: ��型参数进一步校准（qwen3.8/mimo 上下文等）

## 2026-07-26T12:35:00+08:00 — handoff_ready: 同步加提交（Release 上传完成）
- **touched**: .harness/progress.md
- **action**: 追加 progress 条目（Release 上传成功）
- **outcome**: L2 状态更新
- **next**: 用户继续测试

## 2026-07-26T14:20:00+08:00 — work: 启动项代理状态 + 首页断连保护 + provider defaultModel 修复
- **touched**: src-tauri/src/proxy_manager.rs src-tauri/src/commands/config.rs src-tauri/src/commands/proxy.rs src-tauri/src/config_writer.rs src/components/PageStartup.vue src/components/PageHome.vue
- **action**:
  - proxy_manager.rs 全面重写：
    - find_node() 优先搜寻有 mimo2codex 的 nvm 版本（之前按字母排序可能选错版本）
    - kill_port_occupant() 启动前 lsof kill 僵尸进程释放端口
    - port_is_listening() TCP connect 双验证存活性
    - start() spawn 后 500ms 等待 + try_wait 检测即死进程
    - status() try_wait 清理死 Child + 端口��听兜底（不在 HashMap 但端口活着→仍报 running）
  - 启动时无条件拉 3 代理（不再判断 autostart 开关）
  - 启动前先 write_providers() 确保 defaultModel 完整（不写则 mimo2codex 退出码 2）
  - 去掉 PageStartup.vue "代理进程"开关栏，保留"代理状态"栏 + 3 条功能描述
  - 代理状态呼吸灯动画 (pulse-dot 2s)
  - PageHome.vue 首页点"应用"前检测 claude-proxy 是否即将重启→弹 confirm 防断连
  - config_writer.rs providers.json 每个 provider 加 "defaultModel" 字段
  - proxy_script_for() 统一入口，mimo2codex 走 bin_dir 同目录
- **outcome**: 3 代理 App 打开即全起，启动页实时显示运行状态，首页断连保护
- **next**: 同步加提交 + 双端构建 + GitHub Release 更新

## 2026-07-26T14:20:00+08:00 — handoff_ready: 同步
- **touched**: .harness/waypoints/ .harness/handoff.md .harness/progress.md .harness/decisions.md
- **action**: 落 waypoint + 重写 handoff + 追加 progress/decisions
- **outcome**: L2 状态更新
- **next**: git 提交 + push + 双端构建

## 2026-07-27T14:25:00+08:00 — work: claude-proxy.js SSE 流代理 4 bug 修复 + 状态栏配置
- **touched**: claude-proxy.js ~/.claude/settings.json /tmp/claude-proxy-fixed.js
- **action**:
  - 修复 claude-proxy.js `openaiStreamToAnthropicSSE` 函数体 4 个 bug：
    - 双重 message_stop（#1）→ `emitFinal()` 忘设 finished 互斥
    - 缺失 tool_use SSE 事件（#2）→ 无 tcMap 追踪 + 无 doTools()
    - output_tokens 硬编码 0（#3）→ 未从 finish_reason chunk 读 completion_tokens
    - input_tokens 为 0（#4）→ DeepSeek 最后 chunk 才发 prompt_tokens，需 pending 缓冲
  - 改用 blockIdx/blockKind 追踪内容块，closeBlock() 按正确 index 发 content_block_stop
  - emitFinal() 不再调 clientRes.end()，避免 [DONE] 和 end 双重触发
  - pending[] 同时缓冲 doText + doTools
  - /v1/models 接口加 context_window + max_output_tokens 字段
  - 文件同步至 3 处：项目根 claude-proxy.js + ~/.mimo2codex/claude-proxy.js + /tmp/claude-proxy-fixed.js
  - 状态栏配置（~/.claude/settings.json）：模型名(亮青) | 目录 | ctx: Xk/1.0M | $x.xx
  - context_window 硬编码各模型正确值：deepseek/glm/mimo→1M, qwen→1048576
- **outcome**: 代理流正确，工具调用正常，状态栏显示 model+token+cost
- **next**: git 提交 + push + 双端构建 + Release 发布 + SHA256

## 2026-07-27T14:40:00+08:00 — handoff_ready: 同步
- **touched**: .harness/waypoints/2026-07-27T06-31-13+08:00.md .harness/handoff.md .harness/progress.md
- **action**: 落 waypoint + 重写 handoff + 追加 progress
- **outcome**: L2 状态完整反映 SSE 修复 + 状态栏配置
- **next**: git 提交 + push + 双端构建 + GitHub Release + SHA256

## 2026-07-27T16:40:00+08:00 — work: statusLine 嵌入 CC-Gate + model slug 修复 + 全局总则更新
- **touched**: src-tauri/src/config_writer.rs scripts/status-line.sh ~/.claude/CLAUDE.md
- **action**:
  - `write_claude_settings` 改 4 处：
    - default_model 自动加 `claude-` 前缀匹配 gateway /v1/models 返回的 ID
    - 部署 `scripts/status-line.sh` 到 `~/.mimo2codex/status-line.sh`
    - settings.json 写入 statusLine 配置（type=command, command="bash ~/.mimo2codex/status-line.sh"）
  - 新增 `scripts/status-line.sh`：Claude Code 状态栏脚本（模型名亮青 | 目录 | ctx: K/M简写/正确上下文 | 费用）
    - 上下文窗口硬编码覆盖（deepseek/glm/mimo→1M, qwen→1048576）
    - 百分比自己算（不依赖 Claude Code 错误值）
    - 价格按模型实际定价
  - 全局 CLAUDE.md 加 Windows 构建提示（cmd.exe, set PATH, 不用 cargo-xwin）
- **outcome**: 用户安装 CC-Gate 后 Claude Code 自动显示正确状态栏，model ID 匹配 gateway，所有 Agent 上下文窗口正确
- **next**: git 提交 + bump 0.1.2 + push + 双端构建 + GitHub Release + SHA256

## 2026-07-27T16:50:00+08:00 — handoff_ready: 同步
- **touched**: .harness/waypoints/2026-07-27T08-49-15+08:00.md .harness/handoff.md .harness/progress.md
- **action**: 落 waypoint + 重写 handoff + 追加 progress
- **outcome**: L2 完整反映 statusLine 嵌入 + model slug 修复
- **next**: bump 0.1.2 + git 提交 + push + 双端构建 + Release + SHA256

## 2026-07-28T17:00:00+08:00 — work: 中转站弹窗改造
- **touched**: src/components/PageRelayKeys.vue
- **action**:
  - 去掉内联的 `.add-relay-box`，四个输入框改为 Modal 弹窗
  - 新增 `showRelayModal` 状态控制弹窗显隐
  - 弹窗内四个字段纵向排列 + 底部 preset 快捷填入 + 取消/保存按钮
  - 点遮罩层等同于取消
  - 点"添加中转站"/"编辑"打开弹窗
  - 页面只剩中转站列表 + API Key 卡片
- **outcome**: UI 清爽，Mac 构建通过
- **next**: Windows 构建

## 2026-07-28T17:30:00+08:00 — work: 双端构建 v0.1.10 + GitHub Release 发布
- **touched**: src-tauri/tauri.conf.json (bump 0.1.9→0.1.10)
- **action**:
  - Mac `npx tauri build` 成功 (DMG 3.9MB)
  - Windows VM 构建：踩坑 Tauri 自动生成伪造 Cargo.toml 导致 `include_str!` 路径解析失败
    - 根因：`cc-x-llm` 目录残留 Tauri CLI 生成的虚拟 Cargo.toml（`path = "src/main.rs"`）
    - 解法：按 `windows-vm-build-guide.md` runbook，用全新 `cc-gate-build` 目录 + `rmdir /s /q` 清理
    - `npm run tauri -- build --bundles nsis` 成功 (exe 2.94MB)
  - 256KB×12 chunks 回传 + Python 拼接 + SHA256 校验一致
  - GitHub Release v0.1.10：删除旧版本 9 个资产，上传双端包，更新 SHA256
  - 旧 Release 标注废弃说明
- **outcome**: 双端包就绪，GitHub Release 可下载
- **next**: 同步记忆 + 提交 .harness/

## 2026-07-28T18:00:00+08:00 — work: 固化 win-vm-build skill
- **touched**: ~/.claude/skills/win-vm-build/SKILL.md
- **action**:
  - 创建通用 Windows 虚拟机构建 skill
  - 包含完整 4 步骤：tar 打包 → scp 传 VM → PowerShell 远程编译 → 256KB chunks 回传 + SHA256
  - 7 类踩坑全集、增量构建优化、故障排查
  - 触发词：双端构建、两端构建、三端构建、Windows 构建、虚拟机构建、win build、VM 构建
  - 放到 `~/.claude/skills/win-vm-build/`，待 cc-switch scan_unmanaged_skills 收编
- **outcome**: 跨项目通用，下次新项目说"双端构建"即可自动执行
- **next**: 待 cc-switch 同步后生效

## 2026-07-28T18:40:00+08:00 — handoff_ready: 同步
- **touched**: .harness/waypoints/2026-07-28T18-40-00+08:00.md .harness/handoff.md .harness/progress.md
- **action**: 落 waypoint + 重写 handoff + 追加 progress
- **outcome**: L2 完整反映 v0.1.10 弹窗改造 + 双端构建 + Release 发布 + win-vm-build skill
- **next**: codex-cli 配置写入问题排查
