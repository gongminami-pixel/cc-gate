# .harness/

Harness Framework 的**会话状态层**。跨对话、跨模型、跨上下文压缩的持久化记忆。

Managed by: `harness-framework` skill (`~/.claude/skills/harness-framework/`)

## 如何使用

| 场景 | 对 AI 说 |
|---|---|
| 开启新对话（新 chat、换模型、恢复会话） | `读取记忆` / `加载记忆` / `载入记忆` / `恢复记忆` / `召回记忆` |
| 离场前 / 切换模型 / 切换对话前 | `同步记忆` / `刷新记忆` / `校准记忆` |
| 扩展 L3 仓库知识 | `构建记忆` / `建立记忆` / `重建记忆` |
| 定期体检漂移 | `/harness-framework --audit` |

注：同步组会自动合并旧的 `checkpoint` + `handoff` 动作；compaction 前后的自保护由 skill 自动触发，无需手动操作。

完整说明见 skill 的 SKILL.md。

## 文件职责

| 文件/目录 | 性质 | 职责 |
|---|---|---|
| `progress.md` | append-only | 动作日志 |
| `decisions.md` | append-only | 决策账本（含 Supersedes 链） |
| `handoff.md` | 覆盖式 | 当前会话交接单（下一个 AI 先读这份） |
| `waypoints/` | 只增不改 | 检查点快照（压缩/切换前的保险） |
| `context/` | 动态 | JIT 加载的上下文切片 |

## 零漂移五守则

1. **文件即真相**——上下文窗口里的记忆易失，磁盘上的才算数
2. **证据锚定**——所有代码相关结论附 `file:line` 或 `commit:sha`
3. **ISO8601 时间戳**——禁用"刚才/昨天/最近"
4. **每次召回先复核**——读回的状态要用 git/read 验证一项
5. **Pivot 留痕**——方向变更用 Supersedes 链，不删旧条目

## 与 project-memory/ 的边界

- `project-memory/`（若有）= **L3 仓库知识**：静态、架构、领域模型。由 `repo-mem-builder` skill 维护
- `.harness/`（此目录）= **L2 会话状态**：动态、进度、决策。由 `harness-framework` skill 维护

两者协同：boot 时 handoff.md 的 "Context you must load" 段会指向 project-memory/ 的相关文件，按需 JIT 加载。

Created: {{ISO8601}}
