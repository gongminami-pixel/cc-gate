<script setup lang="ts">
import { computed, onBeforeUnmount, ref } from "vue";
import { checkOneTool, saveToolCache } from "../ipc/api";
import type { ToolStatus } from "../types/models";

type SlotStatus = "checking" | "installed" | "missing";

interface ToolSlot {
  command: string;
  name: string;
  status: SlotStatus;
  version: string | null;
  install_cmd: string;
  link: string;
  category: string;
}

const toolSlots = ref<ToolSlot[]>([
  { command: "node",    name: "Node.js & npm",   status: "checking", version: null, install_cmd: "", link: "", category: "runtime" },
  { command: "python3", name: "Python 3",         status: "checking", version: null, install_cmd: "", link: "", category: "runtime" },
  { command: "codex",   name: "Codex CLI",        status: "checking", version: null, install_cmd: "", link: "", category: "tool" },
  { command: "claude",  name: "Claude Code CLI",  status: "checking", version: null, install_cmd: "", link: "", category: "tool" },
  { command: "aider",   name: "Aider",            status: "checking", version: null, install_cmd: "", link: "", category: "tool" },
  { command: "bash",    name: "Shell",            status: "checking", version: null, install_cmd: "", link: "", category: "runtime" },
]);

const checking = ref(false);
const allDone = ref(false);
let abort = false;

const missingTools = computed(() => toolSlots.value.filter(t => t.status === "missing"));
const allOk = computed(() => allDone.value && missingTools.value.length === 0);

function resetSlots() {
  for (const s of toolSlots.value) {
    s.status = "checking";
    s.version = null;
    s.install_cmd = "";
    s.link = "";
  }
  allDone.value = false;
  abort = false;
}

function applyResult(result: ToolStatus) {
  const slot = toolSlots.value.find(s => s.command === result.command);
  if (!slot) return;
  slot.status = result.installed ? "installed" : "missing";
  slot.version = result.version;
  slot.install_cmd = result.install_cmd;
  slot.link = result.link;
}

async function runChecks() {
  if (checking.value) return;
  checking.value = true;
  resetSlots();

  const collected: ToolStatus[] = [];
  for (const slot of toolSlots.value) {
    if (abort) break;
    const r = await checkOneTool(slot.command);
    if (r) {
      applyResult(r);
      collected.push(r);
    }
  }

  // Save to Rust cache for future visits
  if (collected.length > 0) {
    await saveToolCache(collected);
  }

  allDone.value = true;
  checking.value = false;
}

// Start on mount
runChecks();

onBeforeUnmount(() => {
  abort = true;
});

function statusLabel(s: ToolSlot): string {
  if (s.status === "checking") return "检测中…";
  if (s.status === "installed") return s.version ? `已安装 ${s.version}` : "已安装";
  return "未安装";
}
</script>

<template>
  <section class="page">
    <header class="page-header">
      <h2>工具检测</h2>
      <button class="btn ghost" :disabled="checking" @click="runChecks">
        {{ checking ? '检测中…' : '🔄 重新检测' }}
      </button>
    </header>

    <!-- 状态概览 -->
    <div v-if="allDone" class="status-bar" :class="{ ok: allOk }">
      <span v-if="allOk">✅ 全部工具已就绪</span>
      <span v-else>🔧 检测到 {{ missingTools.length }} 个工具未安装</span>
    </div>
    <div v-else class="status-bar">
      <span>⏳ 正在检测…</span>
    </div>

    <p class="desc" style="margin:0 0 16px">
      CLI 形式的 Agent（Codex、Claude Code、Aider）需要先安装对应工具才能使用。
      安装完成后 <strong>新开终端</strong> 即可使用 alias 命令。
    </p>

    <div class="tool-list">
      <div v-for="t in toolSlots" :key="t.command" class="tool-row" :class="{ checking: t.status === 'checking' }">
        <div class="tool-row-info">
          <span class="tool-row-name">{{ t.name }}</span>
          <template v-if="t.status === 'checking'">
            <span class="badge pending">{{ statusLabel(t) }}</span>
          </template>
          <template v-else-if="t.status === 'installed'">
            <span class="badge on">已安装</span>
            <span v-if="t.version" class="dim tool-ver">{{ t.version }}</span>
          </template>
          <template v-else>
            <span class="badge warn">未安装</span>
            <code v-if="t.install_cmd" class="tool-row-cmd">{{ t.install_cmd }}</code>
          </template>
        </div>
        <a v-if="t.link" :href="t.link" target="_blank" class="btn ghost" style="font-size:13px">下载 →</a>
      </div>
    </div>
  </section>
</template>

<style scoped>
.status-bar {
  display: flex; align-items: center; gap: 10px;
  padding: 12px 16px; border-radius: var(--radius-md);
  font-size: 15px; font-weight: 700;
  background: var(--warn-soft); color: var(--warn-fg);
  border: 1px solid var(--warn-bd);
  margin-bottom: 14px;
}
.status-bar.ok {
  background: var(--ok-soft, rgba(46,160,67,0.08));
  color: var(--ok-fg, #2ea043);
  border-color: var(--ok-bd, #2ea043);
}
.desc { font-size: 13px; color: var(--fg-muted); line-height: 1.6; }
.tool-list { display: flex; flex-direction: column; gap: 6px; }
.tool-row { display: flex; align-items: center; justify-content: space-between; padding: 10px 14px; background: var(--surface); border-radius: var(--radius-md); border: 1px solid var(--border); transition: background 0.15s; }
.tool-row.checking { background: var(--surface-soft); }
.tool-row-info { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }
.tool-row-name { font-weight: 600; font-size: 14px; min-width: 140px; }
.tool-row-cmd { font-family: "SF Mono", "Menlo", monospace; font-size: 13px; color: var(--accent-soft-fg); background: var(--surface-soft); padding: 2px 8px; border-radius: 4px; }
.tool-ver { font-size: 11px; }
.badge.pending {
  background: var(--surface-soft); color: var(--fg-muted);
  border: 1px solid var(--border); padding: 1px 8px; border-radius: 10px;
  font-size: 12px;
}
</style>
