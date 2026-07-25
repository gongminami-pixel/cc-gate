<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import type { AppConfig } from "../types/models";
import { getShellInfo, type ShellInfo } from "../ipc/api";

const props = defineProps<{ config: AppConfig | null }>();
const shellInfo = ref<ShellInfo | null>(null);

function aliasShort(slug: string): string {
  const m: Record<string, string> = {
    "deepseek-v4-pro": "ds", "deepseek-v4-flash": "ds-flash",
    "glm-5.2": "glm", "qwen3.8-max-preview": "qwen", "qwen-max": "qwen-max",
    "mimo-v2.5-pro": "mimo", "mimo-v2.5": "mimo-v2.5",
    "claude-opus-4-5": "opus", "gpt-5.1-codex": "gpt",
  };
  return m[slug] || slug;
}

const codexAliases = computed(() => {
  const slugs = props.config?.agent_models?.["codex_cli"] ?? [];
  return slugs.map(s => ({ alias: `codex-${aliasShort(s)}`, model: s, tool: "Codex CLI" }));
});
const claudeAliases = computed(() => {
  const slugs = props.config?.agent_models?.["claude_cli"] ?? [];
  return slugs.map(s => ({ alias: `claude-${aliasShort(s)}`, model: s, tool: "Claude Code CLI" }));
});
const aiderAliases = computed(() => {
  const slugs = props.config?.agent_models?.["aider"] ?? [];
  return slugs.map(s => ({ alias: `aider-${aliasShort(s)}`, model: s, tool: "Aider CLI" }));
});
const allAliases = computed(() => [...codexAliases.value, ...claudeAliases.value, ...aiderAliases.value]);

const proxyPorts = computed(() => props.config?.proxy_ports ?? { claude_proxy: 8689, chat_proxy: 8690, mimo2codex: 8688 });

const platformLabel = computed(() => {
  const os = shellInfo.value?.platform_os ?? "";
  if (os === "macos") return "macOS";
  if (os === "linux") return "Linux";
  if (os === "windows") return "Windows";
  return os || "当前系统";
});

const platformIcon = computed(() => {
  const os = shellInfo.value?.platform_os ?? "";
  if (os === "macos") return "🍎";
  if (os === "linux") return "🐧";
  if (os === "windows") return "🪟";
  return "💻";
});

onMounted(async () => { try { shellInfo.value = await getShellInfo(); } catch {} });
</script>

<template>
  <section class="page">
    <header class="page-header"><h2>Shell 集成</h2></header>

    <div class="card">
      <div class="card-head">自动生效</div>
      <div class="card-body">
        <p class="desc">
          在首页勾选 CLI Agent 的模型并点「应用」后，alias 自动写入
          <strong>{{ shellInfo?.config_file || '~/.zshrc' }}</strong>。执行
          执行 <code>{{ shellInfo?.reload_cmd || 'source ~/.zshrc' }}</code> 即可使用，无需额外操作。
          <span v-if="shellInfo?.platform_os === 'windows'" class="ps-note">Windows 上已同时写入 Git-Bash 和 PowerShell（$PROFILE），两处自动生效。</span>
        </p>

        <div class="platform-card mt8">
          <div class="platform-icon">{{ platformIcon }}</div>
          <div>
            <div class="platform-name">检测到 {{ platformLabel }}</div>
            <div class="platform-detail dim">
              配置文件：<code>{{ shellInfo?.config_file || '~/.zshrc' }}</code>
            </div>
            <div class="platform-detail dim">
              生效命令：<code>{{ shellInfo?.reload_cmd || 'source ~/.zshrc' }}</code>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="card mt12">
      <div class="card-head">当前 alias</div>
      <div class="card-body">
        <div v-if="allAliases.length === 0" class="dim sec-empty">
          还没有勾选 CLI Agent 的模型。去首页勾选后点「应用」即可。
        </div>
        <div v-else class="alias-list">
          <div v-for="a in allAliases" :key="a.alias" class="alias-row">
            <code class="alias-name">{{ a.alias }}</code>
            <span class="alias-model">{{ a.model }}</span>
            <span class="alias-tool dim">{{ a.tool }}</span>
          </div>
        </div>
      </div>
    </div>

    <div class="card mt12">
      <div class="card-head">用法</div>
      <div class="card-body">
        <div class="usage-item"><code>codex-ds</code><span>Codex CLI · DeepSeek V4 Pro</span></div>
        <div class="usage-item"><code>claude-glm</code><span>Claude Code CLI · GLM-5.2</span></div>
        <div class="usage-item"><code>aider-mimo</code><span>Aider CLI · MiMo V2.5 Pro</span></div>

        <div class="mt12 dim" style="font-size:13px;line-height:1.6">
          Alias 指向本地代理：Codex → <code>127.0.0.1:{{ proxyPorts.mimo2codex }}</code>，
          Claude → <code>127.0.0.1:{{ proxyPorts.claude_proxy }}</code>，
          Aider → <code>127.0.0.1:{{ proxyPorts.chat_proxy }}</code>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.card-head { padding: 14px 18px 0; font-size: 16px; font-weight: 700; }
.card-body { padding: 10px 18px 16px; }
.desc { font-size: 14px; color: var(--fg-muted); line-height: 1.6; margin: 0; }
.desc code, .desc strong { font-size: 14px; color: var(--accent-soft-fg); }
.sec-empty { font-size: 14px; padding: 12px 0; }
.platform-card {
  display: flex; align-items: center; gap: 14px;
  background: var(--surface-soft); border: 1px solid var(--border);
  border-radius: var(--radius-lg); padding: 14px 16px;
}
.platform-icon { font-size: 28px; }
.platform-name { font-size: 14px; font-weight: 700; }
.platform-detail { font-size: 13px; margin-top: 2px; }
.platform-detail code { font-size: 13px; color: var(--accent-soft-fg); }
.alias-list { display: flex; flex-direction: column; gap: 4px; }
.alias-row { display: flex; align-items: center; gap: 14px; padding: 8px 10px; border-bottom: 1px solid var(--border); font-size: 14px; }
.alias-row:last-child { border-bottom: none; }
.alias-name { font-family: "SF Mono", "Menlo", monospace; font-size: 14px; font-weight: 700; color: var(--accent); min-width: 160px; }
.alias-model { color: var(--fg-muted); min-width: 200px; }
.alias-tool { font-size: 12px; }
.usage-item { display: flex; align-items: baseline; gap: 12px; font-size: 14px; line-height: 1.6; margin-bottom: 6px; }
.usage-item code { font-family: "SF Mono", "Menlo", monospace; font-size: 14px; font-weight: 700; color: var(--accent); min-width: 140px; }
.ps-note { display: block; margin-top: 6px; font-size: 13px; color: var(--warn-fg); }
</style>
