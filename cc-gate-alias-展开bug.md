# cc-gate alias 生成缺陷：zsh 递归展开导致 flag 重复

> 2026-07-28 发现，影响 codex-* 系列 alias

## 现象

```bash
$ codex-ds
error: the argument '--dangerously-bypass-approvals-and-sandbox' cannot be used multiple times
```

## 根因

cc-gate 生成 `.zshrc` alias 时，同时创建了两个相关 alias：

```bash
# 基础版（带 flag）
alias codex='CC_GATE_MODEL="deepseek-v4-pro" ... codex --dangerously-bypass-approvals-and-sandbox ...'

# 具体模型版（也带 flag，且末尾引用 codex）
alias codex-ds='CC_GATE_MODEL="deepseek-v4-pro" ... codex --dangerously-bypass-approvals-and-sandbox ...'
```

zsh 的 alias 展开机制：当 `codex-ds` 被求值时，末尾的裸 `codex` 会被识别为 alias 并展开成 `codex` alias 的完整内容。结果就是 `--dangerously-bypass-approvals-and-sandbox` 出现两次。

展开过程：

```
codex-ds
  → CC_GATE_MODEL="..." ... codex --dangerously-bypass-approvals-and-sandbox ...
                              ^^^^  zsh 展开 codex alias
  → CC_GATE_MODEL="..." ... CC_GATE_MODEL="..." ... codex --dangerously-bypass-approvals-and-sandbox ... --dangerously-bypass-approvals-and-sandbox ...
                                                                                                                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ 重复!
```

## 影响范围

所有 `codex-*` 别名（`codex-ds`、`codex-glm`、`codex-qwen`、`codex-mimo` 等），其命令部分引用裸 `codex` 的，都会触发此问题。

同理，`claude-*` 别名末尾引用裸 `claude` 的也可能受影响（取决于 `claude` 基础 alias 是否包含冲突的 flag）。

## 修复方案

### 方案 A（推荐）：末尾命令加反斜杠防展开

```bash
# 改前
alias codex-ds='... \codex --dangerously-bypass-...'

# 改后
alias codex-ds='... \codex --dangerously-bypass-...'
```

`\codex` 告诉 zsh 跳过 alias 展开，直接执行原生命令。

### 方案 B：基础 alias 不带 flag

```bash
# codex 只设默认 env，不带命令参数
alias codex='CC_GATE_MODEL="deepseek-v4-pro" OPENAI_API_KEY=*** \codex'

# 各具体 alias 自己带完整参数
alias codex-ds='CC_GATE_MODEL="deepseek-v4-pro" OPENAI_API_KEY=*** \codex --dangerously-bypass-approvals-and-sandbox -c model="deepseek-v4-pro" ...'
```

## 源码修改位置

cc-gate 中生成 `.zshrc` alias 的代码，在写入 `codex-*` alias 时，命令名应使用 `\codex` 或 `command codex`。

同等检查 `claude-*` 和 `aider-*` alias 生成逻辑。
