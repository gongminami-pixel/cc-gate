<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import {
  getUsageSummary, getPerModelUsage, getRecentLogs, importUsageData,
  type UsageSummary, type PerModelUsage, type LogEntry,
} from "../ipc/api";

const summary = ref<UsageSummary | null>(null);
const perModel = ref<PerModelUsage[]>([]);
const logs = ref<LogEntry[]>([]);
const loading = ref(false);
const selectedSlot = ref<string>("今天");

async function refreshAll() {
  loading.value = true;
  try {
    await importUsageData();
    const [s, p, l] = await Promise.all([
      getUsageSummary(), getPerModelUsage(), getRecentLogs(50),
    ]);
    summary.value = s; perModel.value = p; logs.value = l;
  } catch (e: any) { console.error(e); }
  finally { loading.value = false; }
}

const currentSlot = computed(() => perModel.value.find(s => s.label === selectedSlot.value));
const currentModels = computed(() => currentSlot.value?.models ?? []);
const maxCost = computed(() => Math.max(...currentModels.value.map(m => m.cost_usd), 0.001));

function selectSlot(label: string) { selectedSlot.value = label; }

function fmtUSD(n: number): string { return `$${n.toFixed(4)}`; }
function fmtTokens(n: number): string { return n >= 1_000_000 ? `${(n/1_000_000).toFixed(1)}M` : n >= 1_000 ? `${(n/1_000).toFixed(0)}K` : String(n); }
function fmtTime(ts: string): string { if (!ts) return ""; const m = ts.match(/(\d{2}:\d{2})/); return m ? m[1] : ts.slice(0, 16); }
function proxyBadge(p: string): string { return p || "off"; }
function barPct(cost: number): string { return `${Math.round(cost / maxCost.value * 100)}%`; }

onMounted(refreshAll);
</script>

<template>
  <section class="page">
    <header class="page-header">
      <h2>用量统计</h2>
      <button class="btn" :disabled="loading" @click="refreshAll">{{ loading ? "刷新…" : "刷新" }}</button>
    </header>

    <!-- Summary -->
    <div class="grid" v-if="summary">
      <div class="card padded stat-card"><div class="stat-label">今日费用</div><div class="stat-value">{{ fmtUSD(summary.today_cost_usd) }}</div></div>
      <div class="card padded stat-card"><div class="stat-label">本月费用</div><div class="stat-value">{{ fmtUSD(summary.month_cost_usd) }}</div></div>
      <div class="card padded stat-card"><div class="stat-label">今日 Token</div><div class="stat-value">{{ fmtTokens(summary.today_tokens) }}</div></div>
      <div class="card padded stat-card"><div class="stat-label">总请求</div><div class="stat-value">{{ summary.total_requests.toLocaleString() }}</div></div>
    </div>
    <div v-else-if="!loading" class="card padded"><div class="empty-state"><h3>暂无数据</h3><p class="muted">代理会自动记录每次请求</p></div></div>

    <!-- Per-model time selector -->
    <div v-if="perModel.length > 0" class="mt12">
      <h3 style="font-size:16px;font-weight:700;margin:0 0 10px">按模型分时段</h3>

      <!-- Slot tabs -->
      <div class="slot-tabs">
        <button
          v-for="slot in perModel" :key="slot.label"
          class="slot-tab"
          :class="{ active: selectedSlot === slot.label }"
          @click="selectSlot(slot.label)"
        >
          {{ slot.label }}
          <span class="slot-count" v-if="slot.models.length > 0">{{ slot.models.length }}</span>
        </button>
      </div>

      <!-- Model list for selected slot -->
      <div class="card mt8">
        <div v-if="currentModels.length === 0" class="slot-empty">该时段暂无数据</div>
        <div v-for="m in currentModels" :key="m.model" class="slot-row">
          <div class="slot-row-info">
            <span class="slot-model">{{ m.display_name || m.model }}</span>
            <span class="slot-meta dim">{{ m.requests }} 次 · {{ fmtTokens(m.tokens) }} tokens</span>
          </div>
          <div class="slot-bar-wrap">
            <div class="slot-bar" :style="{ width: barPct(m.cost_usd) }"></div>
          </div>
          <span class="slot-cost">{{ fmtUSD(m.cost_usd) }}</span>
        </div>
      </div>
    </div>

    <!-- Recent requests -->
    <div v-if="logs.length > 0" class="card mt12">
      <div style="padding:12px 16px;border-bottom:1px solid var(--border);font-weight:600;font-size:13px">最近请求</div>
      <div class="log-table-wrap">
        <table class="log-table">
          <thead><tr><th>时间</th><th>模型</th><th>代理</th><th style="text-align:right">输入</th><th style="text-align:right">输出</th><th style="text-align:right">费用</th></tr></thead>
          <tbody>
            <tr v-for="l in logs" :key="l.id">
              <td class="dim">{{ fmtTime(l.created_at) }}</td>
              <td>{{ l.model }}</td>
              <td><span class="badge" :class="proxyBadge(l.proxy)">{{ l.proxy }}</span></td>
              <td class="num">{{ fmtTokens(l.prompt_tokens) }}</td>
              <td class="num">{{ fmtTokens(l.completion_tokens) }}</td>
              <td class="num cost">{{ fmtUSD(l.cost_usd) }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </section>
</template>

<style scoped>
/* ── Summary ──────────────────────────── */
.stat-card { text-align: center; padding: 18px 16px !important; }
.stat-label { font-size: 13px; color: var(--fg-muted); margin-bottom: 4px; }
.stat-value { font-size: 28px; font-weight: 700; font-variant-numeric: tabular-nums; letter-spacing: -0.02em; }

/* ── Slot tabs ────────────────────────── */
.slot-tabs { display: flex; gap: 6px; flex-wrap: wrap; }
.slot-tab {
  padding: 7px 16px; border-radius: var(--radius-pill);
  border: 2px solid var(--border-strong); background: var(--surface);
  color: var(--fg-muted); font-size: 13px; font-weight: 500; cursor: pointer;
  transition: background 0.15s, border-color 0.15s, color 0.15s;
  display: flex; align-items: center; gap: 6px;
}
.slot-tab:hover { background: var(--tap-target-on); color: var(--fg); }
.slot-tab.active { background: var(--accent); border-color: var(--accent); color: var(--accent-fg); }
.slot-count { font-size: 10px; padding: 0 4px; border-radius: 999px; background: rgba(255,255,255,0.25); }
.slot-tab.active .slot-count { background: rgba(255,255,255,0.3); }

/* ── Model list ───────────────────────── */
.slot-empty { text-align: center; padding: 30px 0; color: var(--fg-dim); font-size: 13px; }

.slot-row {
  display: flex; align-items: center; gap: 14px;
  padding: 10px 16px; border-bottom: 1px solid var(--border);
}
.slot-row:last-child { border-bottom: none; }

.slot-row-info {
  width: 210px; min-width: 160px;
  display: flex; flex-direction: column; gap: 2px;
}
.slot-model { font-weight: 600; font-size: 14px; }
.slot-meta { font-size: 12px; }

.slot-bar-wrap { flex: 1; height: 12px; background: var(--surface-soft); border-radius: 6px; overflow: hidden; border: 1px solid var(--border); }
.slot-bar { height: 100%; border-radius: 5px; min-width: 2px; background: var(--accent); opacity: 0.8; transition: width 0.4s ease; }
.slot-cost { font-size: 14px; font-weight: 700; font-variant-numeric: tabular-nums; min-width: 85px; text-align: right; }

/* ── Log table ────────────────────────── */
.log-table-wrap { overflow-x: auto; }
.log-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.log-table th { text-align: left; padding: 8px 12px; font-weight: 600; color: var(--fg-muted); font-size: 12px; text-transform: uppercase; letter-spacing: 0.04em; border-bottom: 1px solid var(--border); }
.log-table td { padding: 7px 12px; border-bottom: 1px solid var(--border); white-space: nowrap; }
.log-table tr:last-child td { border-bottom: none; }
.num { text-align: right; font-variant-numeric: tabular-nums; }
.cost { font-weight: 600; }
</style>
