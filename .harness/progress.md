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
