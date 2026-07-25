<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { checkTools } from "../ipc/api";
import type { ToolStatus } from "../types/models";

const toolStatuses = ref<ToolStatus[]>([]);
const toolCheckDone = ref(false);
const toolChecking = ref(false);

const missingTools = computed(() => toolStatuses.value.filter(t => !t.installed));
const allOk = computed(() => toolCheckDone.value && missingTools.value.length === 0);

async function refreshTools() {
  toolChecking.value = true;
  try { toolStatuses.value = await checkTools(true); }
  finally { toolChecking.value = false; }
}

onMounted(async () => {
  toolStatuses.value = await checkTools();
  toolCheckDone.value = true;
});
</script>

<template>
  <section class="page">
    <header class="page-header">
      <h2>工具检测</h2>
      <button class="btn ghost" :disabled="toolChecking" @click="refreshTools">
        {{ toolChecking ? '检测中…' : '🔄 重新检测' }}
      </button>
    </header>

    <div v-if="!toolCheckDone" class="dim" style="padding:40px;text-align:center">检测中…</div>

    <template v-else>
      <!-- 状态概览 -->
      <div class="status-bar" :class="{ ok: allOk }">
        <span v-if="allOk">✅ 全部工具已就绪</span>
        <span v-else>🔧 检测到 {{ missingTools.length }} 个工具未安装</span>
      </div>

      <p class="desc" style="margin:0 0 16px">
        CLI 形式的 Agent（Codex、Claude Code、Aider）需要先安装对应工具才能使用。
        安装完成后 <strong>新开终端</strong> 即可使用 alias 命令。
      </p>

      <div class="tool-list">
        <div v-for="t in toolStatuses" :key="t.command" class="tool-row">
          <div class="tool-row-info">
            <span class="tool-row-name">{{ t.name }}</span>
            <template v-if="t.installed">
              <span class="badge on">已安装</span>
              <span v-if="t.version" class="dim tool-ver">{{ t.version }}</span>
            </template>
            <template v-else>
              <span class="badge warn">未安装</span>
              <code class="tool-row-cmd">{{ t.install_cmd }}</code>
            </template>
          </div>
          <a v-if="t.link" :href="t.link" target="_blank" class="btn ghost" style="font-size:13px">下载 →</a>
        </div>
      </div>
    </template>
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
.tool-row { display: flex; align-items: center; justify-content: space-between; padding: 10px 14px; background: var(--surface); border-radius: var(--radius-md); border: 1px solid var(--border); }
.tool-row-info { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }
.tool-row-name { font-weight: 600; font-size: 14px; min-width: 140px; }
.tool-row-cmd { font-family: "SF Mono", "Menlo", monospace; font-size: 13px; color: var(--accent-soft-fg); background: var(--surface-soft); padding: 2px 8px; border-radius: 4px; }
.tool-ver { font-size: 11px; }
</style>
