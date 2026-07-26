<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import type { AgentId, AgentMeta, AppConfig, ModelDef } from "../types/models";
import { getAgentList, applyAgentConfig, checkModelUpdates } from "../ipc/api";
import { useToast } from "../composables/useToast";
import { useAppConfig } from "../composables/useAppConfig";
import { listen } from "@tauri-apps/api/event";

const toast = useToast();
const { config, refresh: refreshConfig } = useAppConfig();

const agents = ref<AgentMeta[]>([]);
const selectedAgentId = ref<AgentId | null>(null);
const applying = ref(false);
const checking = ref(false);
const newModelSlugs = ref<Set<string>>(new Set());
const catalogVersion = ref(0);
const workingModels = ref<Record<AgentId, string[]>>({} as Record<AgentId, string[]>);
const modelRouting = ref<Record<string, string>>({});

const selectedAgent = computed(() => agents.value.find(a => a.id === selectedAgentId.value));
const allModels = computed(() => config.value?.models ?? []);
const relays = computed(() => config.value?.relays ?? []);

/** 当前编辑状态是否与已保存配置有差异 */
const dirty = computed(() => {
  if (!config.value) return false;
  // compare agent_models
  const orig = config.value.agent_models;
  const cur = workingModels.value;
  for (const agent of agents.value) {
    const o = (orig[agent.id] ?? []).slice().sort().join(',');
    const c = (cur[agent.id] ?? []).slice().sort().join(',');
    if (o !== c) return true;
  }
  // compare model_routing
  const origR = config.value.model_routing ?? {};
  const curR = modelRouting.value;
  const allSlugs = new Set([...Object.keys(origR), ...Object.keys(curR)]);
  for (const s of allSlugs) {
    if ((origR[s] ?? 'direct') !== (curR[s] ?? 'direct')) return true;
  }
  return false;
});

function initWorking() {
  if (!config.value) return;
  const wm: Record<string, string[]> = {};
  for (const agent of agents.value) wm[agent.id] = [...(config.value.agent_models[agent.id] ?? [])];
  workingModels.value = wm as Record<AgentId, string[]>;
  modelRouting.value = { ...config.value.model_routing };
}

function isModelEnabled(modelSlug: string): boolean {
  if (!selectedAgentId.value) return false;
  return workingModels.value[selectedAgentId.value]?.includes(modelSlug) ?? false;
}

function toggleModel(modelSlug: string) {
  if (!selectedAgentId.value || !config.value) return;
  const cur = workingModels.value[selectedAgentId.value] ?? [];
  workingModels.value[selectedAgentId.value] = cur.includes(modelSlug) ? cur.filter(s => s !== modelSlug) : [...cur, modelSlug];
  workingModels.value = { ...workingModels.value };
}

function selectAgent(id: AgentId) { selectedAgentId.value = id; }

function routingFor(slug: string): string {
  return modelRouting.value[slug] ?? "direct";
}

function setRouting(slug: string, value: string) {
  modelRouting.value = { ...modelRouting.value, [slug]: value };
}

async function onApply() {
  if (!config.value) return;
  const cfg: AppConfig = { ...config.value, agent_models: { ...workingModels.value } as Record<AgentId, string[]>, model_routing: { ...modelRouting.value } };

  // Warn before restarting proxy that carries the current chat session
  const willRestartClaude = cfg.autostart_claude_proxy &&
    agents.value.some(a => a.proxy === "claude-proxy" && (cfg.agent_models[a.id]?.length ?? 0) > 0);
  if (willRestartClaude) {
    if (!window.confirm(
      "即将重启 claude-proxy（端口 8689）。\n\n" +
      "如果你当前正通过它连接到 CC Chat，本次操作会断开当前会话。\n\n" +
      "确定要继续吗？"
    )) {
      return;
    }
  }

  applying.value = true;
  try {
    const result = await applyAgentConfig(cfg);
    await refreshConfig();
    toast.ok(result.restarted_proxies?.length > 0 ? `已应用，重启：${result.restarted_proxies.join('、')}` : "配置已应用");
  } catch (e: any) { toast.err(e?.message ?? String(e)); }
  finally { applying.value = false; }
}

function fmtTokens(n: number): string { return n >= 1_000_000 ? `${(n/1_000_000).toFixed(1)}M` : n >= 1_000 ? `${(n/1_000).toFixed(0)}K` : String(n); }

async function onCheckUpdates() {
  if (!config.value) return;
  checking.value = true;
  try {
    const result = await checkModelUpdates();
    newModelSlugs.value = new Set(result.new_slugs);
    catalogVersion.value = result.version;
    await refreshConfig();
    if (result.new_models > 0) {
      toast.ok(`发现 ${result.new_models} 个新模型`);
    } else {
      toast.ok("模型列表已是最新");
    }
  } catch (e: any) {
    toast.err(e?.message ?? String(e));
  } finally {
    checking.value = false;
  }
}

let unlisten: (() => void) | null = null;
onMounted(async () => {
  agents.value = await getAgentList();
  initWorking();
  if (agents.value.length > 0) selectedAgentId.value = agents.value[0].id;

  // Listen for background catalog refreshes
  unlisten = await listen("config-changed", () => {
    refreshConfig();
  });
});
onUnmounted(() => { unlisten?.(); });
watch(config, () => { if (config.value) { initWorking(); } });
</script>

<template>
  <section class="page">
    <header class="page-header">
      <h2>首页</h2>
      <button
        class="apply-btn"
        :class="{ ready: dirty, applied: !dirty }"
        :disabled="applying || !config || !dirty"
        @click="onApply"
      >
        <span v-if="applying" class="apply-spin">⟳</span>
        <span v-else-if="dirty">应用</span>
        <span v-else>✓ 已保存</span>
      </button>
    </header>

    <div class="home-layout">
      <!-- Agent list -->
      <div class="agent-col">
        <div v-for="agent in agents" :key="agent.id"
          class="agent-item" :class="{ active: selectedAgentId === agent.id }"
          @click="selectAgent(agent.id)">
          <div class="agent-name">{{ agent.name }}</div>
          <div class="agent-meta">{{ agent.tool }} · {{ agent.type === 'cli' ? 'CLI' : '桌面端' }}</div>
        </div>
      </div>

      <!-- Model list with routing dropdown -->
      <div class="model-col">
        <template v-if="selectedAgent">
          <div class="model-header">
            {{ selectedAgent.name }} 的模型
            <span class="dim">({{ workingModels[selectedAgent.id]?.length ?? 0 }}/{{ allModels.length }})</span>
            <button class="update-btn" :disabled="checking" @click="onCheckUpdates">
              <span v-if="checking" class="update-spin">⟳</span>
              <span v-else>检查模型更新</span>
            </button>
          </div>

          <div v-for="m in allModels" :key="m.slug" class="model-check-row">
            <label class="check-label">
              <input type="checkbox" :checked="isModelEnabled(m.slug)" @change="toggleModel(m.slug)" />
              <span class="check-text">
                <span class="model-slug">
                  {{ m.slug }}
                  <span v-if="newModelSlugs.has(m.slug)" class="new-badge">新</span>
                </span>
                <span class="model-meta">{{ m.display_name }} · {{ fmtTokens(m.context_window) }} ctx · {{ fmtTokens(m.max_output_tokens) }} out</span>
              </span>
            </label>

            <!-- Routing dropdown -->
            <select class="routing-select" :value="routingFor(m.slug)" @change="setRouting(m.slug, ($event.target as HTMLSelectElement).value)">
              <option value="direct">直连</option>
              <option v-for="r in relays" :key="r.name" :value="'relay:' + r.name">{{ r.name }}</option>
            </select>
          </div>
        </template>
        <div v-else class="dim" style="padding:40px;text-align:center">← 选择左侧 Agent</div>
      </div>
    </div>

  </section>
</template>

<style scoped>
.home-layout { display: flex; border: 1px solid var(--border); border-radius: var(--radius-lg); overflow: hidden; background: var(--surface); min-height: 360px; }
.agent-col { width: 200px; min-width: 200px; border-right: 1px solid var(--border); background: var(--sidebar-bg); overflow-y: auto; }
.agent-item { padding: 11px 14px; cursor: pointer; border-bottom: 1px solid var(--border); transition: background 0.1s; }
.agent-item:hover { background: var(--tap-target-on); }
.agent-item.active { background: var(--accent); color: var(--accent-fg); font-weight: 600; box-shadow: 0 1px 4px rgba(0,0,0,0.12); }
.agent-name { font-size: 14px; font-weight: 600; }
.agent-meta { font-size: 11px; color: var(--fg-dim); margin-top: 1px; }

.model-col { flex: 1; padding: 14px 18px; overflow-y: auto; }
.model-header { font-size: 15px; font-weight: 600; margin-bottom: 12px; padding-bottom: 8px; border-bottom: 1px solid var(--border); }

.model-check-row { display: flex; align-items: center; gap: 12px; padding: 7px 0; border-bottom: 1px solid var(--border); }
.model-check-row:last-child { border-bottom: none; }
.check-label { display: flex; align-items: flex-start; gap: 10px; cursor: pointer; font-size: 14px; flex: 1; min-width: 0; }
.check-label input[type="checkbox"] { margin-top: 2px; width: 17px; height: 17px; accent-color: var(--accent); flex-shrink: 0; }
.check-text { display: flex; flex-direction: column; gap: 1px; min-width: 0; }
.model-slug { font-weight: 600; font-family: "SF Mono", "Menlo", monospace; font-size: 13px; color: var(--accent); }
.model-meta { font-size: 12px; color: var(--fg-dim); white-space: nowrap; }

.routing-select {
  flex-shrink: 0; width: 115px;
  border: 1px solid var(--border-strong); border-radius: var(--radius-md);
  padding: 5px 7px; font-size: 13px; background: var(--surface); color: var(--fg);
  outline: none; cursor: pointer;
}
.routing-select:focus { border-color: var(--accent); box-shadow: var(--focus-ring); }

/* ── Apply button ─────────────────────────── */
.apply-btn {
  padding: 8px 20px; border: none; border-radius: var(--radius-lg);
  font-size: 14px; font-weight: 600; cursor: pointer;
  transition: all 0.2s ease;
  outline: none; min-width: 100px;
}
.apply-btn:disabled { cursor: not-allowed; opacity: 0.5; }
.apply-btn.ready {
  background: var(--accent); color: var(--accent-fg);
  box-shadow: 0 2px 12px color-mix(in srgb, var(--accent) 35%, transparent);
}
.apply-btn.ready:hover:not(:disabled) {
  filter: brightness(1.1);
  box-shadow: 0 4px 18px color-mix(in srgb, var(--accent) 45%, transparent);
}
.apply-btn.applied {
  background: var(--surface-soft); color: var(--fg-muted);
  border: 1px solid var(--border);
}
.apply-spin {
  display: inline-block;
  animation: spin 0.8s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }

/* ── Model update button ──────────────────── */
.update-btn {
  margin-left: auto; padding: 4px 12px; border: 1px solid var(--border-strong);
  border-radius: var(--radius-md); font-size: 12px; font-weight: 500;
  background: var(--surface-soft); color: var(--fg-dim); cursor: pointer;
  transition: all 0.15s; outline: none; white-space: nowrap;
}
.update-btn:hover:not(:disabled) { border-color: var(--accent); color: var(--accent); }
.update-btn:disabled { opacity: 0.5; cursor: not-allowed; }
.update-spin { display: inline-block; animation: spin 0.8s linear infinite; }

/* ── New model badge ───────────────────────── */
.new-badge {
  display: inline-block; font-size: 10px; font-weight: 700; line-height: 1;
  padding: 1px 5px; border-radius: 8px;
  background: color-mix(in srgb, var(--accent) 20%, transparent);
  color: var(--accent); vertical-align: middle; margin-left: 4px;
}
</style>
