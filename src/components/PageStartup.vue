<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import type { AppConfig, ProxyStatus } from "../types/models";
import { setAppAutostart, getAppAutostartStatus, getProxyStatus, startProxy, restartProxy } from "../ipc/api";
import { useToast } from "../composables/useToast";

const props = defineProps<{ config: AppConfig | null }>();
const toast = useToast();

const autostartEnabled = ref(false);
const autostartBusy = ref(false);
const proxyStatuses = ref<ProxyStatus[]>([]);
const busyProxy = ref<string | null>(null);
let statusTimer: ReturnType<typeof setInterval> | null = null;

async function loadAutostart() {
  try { autostartEnabled.value = (await getAppAutostartStatus()).enabled; } catch { autostartEnabled.value = false; }
}
async function loadProxyStatuses() {
  try { proxyStatuses.value = await getProxyStatus(); } catch { /* proxy manager not ready yet */ }
}
async function onToggleAutostart() {
  autostartBusy.value = true;
  try { const r = await setAppAutostart(!autostartEnabled.value); autostartEnabled.value = r.enabled; toast.ok(r.enabled ? "已启用登录时自动启动" : "已关闭登录时自动启动"); }
  catch (e: any) { toast.err(e?.message ?? String(e)); } finally { autostartBusy.value = false; }
}

async function onProxyAction(name: string, running: boolean) {
  busyProxy.value = name;
  try {
    if (running) {
      await restartProxy(name);
      toast.ok(`${name} 已重启`);
    } else {
      await startProxy(name);
      toast.ok(`${name} 已启动`);
    }
    await loadProxyStatuses();
  } catch (e: any) { toast.err(e?.message ?? String(e)); }
  finally { busyProxy.value = null; }
}

const STATUSBAR_META: { name: string; key: string; port: number; label: string; desc: string }[] = [
  {
    name: "mimo2codex", key: "mimo2codex", port: 8688, label: "mimo2codex",
    desc: "协议转换 — 把 Responses API 转为 Chat Completions，供 Codex CLI / Desktop / Reasonix 走非原生模型（GLM / Qwen / MiMo）时使用",
  },
  {
    name: "claude-proxy", key: "claude-proxy", port: 8689, label: "claude-proxy",
    desc: "协议转换 — 把 Anthropic Messages API 转为 Chat Completions，供 Claude CLI、Claude Desktop 使用",
  },
  {
    name: "chat-proxy", key: "chat-proxy", port: 8690, label: "chat-proxy",
    desc: "Chat Completions 透传 — 直接转发，供 Hermes、OpenCode、OpenClaw、Aider、Cursor 使用",
  },
];
function statusFor(key: string): ProxyStatus | undefined {
  return proxyStatuses.value.find(s => s.name === key);
}

onMounted(() => {
  loadAutostart();
  loadProxyStatuses();
  statusTimer = setInterval(loadProxyStatuses, 3000);
});
onUnmounted(() => {
  if (statusTimer) { clearInterval(statusTimer); statusTimer = null; }
});
</script>

<template>
  <section class="page">
    <header class="page-header"><h2>启动项</h2></header>

    <div class="card">
      <div class="card-head">自启</div>
      <div class="card-body">
        <div class="toggle-row"><span>登录时自动启动 CC-Gate</span><label class="toggle"><input type="checkbox" :checked="autostartEnabled" :disabled="autostartBusy" @change="onToggleAutostart" /><span class="slider"></span></label></div>
      </div>
    </div>

    <div class="card mt12">
      <div class="card-head">代理状态</div>
      <div class="card-body">
        <p class="sec-desc">当前各代理进程的运行状况。每 3 秒自动刷新。</p>
        <div v-for="m in STATUSBAR_META" :key="m.key" class="status-block">
          <div class="status-row">
            <span class="status-dot" :class="{ on: statusFor(m.key)?.running }"></span>
            <span class="status-name">{{ m.label }}</span>
            <span class="status-port">:{{ m.port }}</span>
            <span class="status-state" :class="{ running: statusFor(m.key)?.running }">
              {{ statusFor(m.key)?.running ? '运行中' : '未启动' }}
            </span>
            <span v-if="statusFor(m.key)?.running && statusFor(m.key)?.pid" class="status-pid">PID {{ statusFor(m.key)?.pid }}</span>
            <button class="proxy-action-btn" :disabled="busyProxy !== null" @click="onProxyAction(m.key, statusFor(m.key)?.running ?? false)">
              <span v-if="busyProxy === m.key" class="apply-spin">⟳</span>
              <span v-else>{{ statusFor(m.key)?.running ? '重启' : '重试' }}</span>
            </button>
          </div>
          <div class="status-desc">{{ m.desc }}</div>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.card-head { padding: 14px 18px 0; font-size: 15px; font-weight: 700; }
.card-body { padding: 10px 18px 16px; }
.sec-desc { font-size: 12px; color: var(--fg-dim); margin: 0 0 10px; }
.toggle-row { display: flex; align-items: center; justify-content: space-between; padding: 7px 0; border-bottom: 1px solid var(--border); font-size: 13px; }
.toggle-row:last-child { border-bottom: none; }

/* ── Proxy status rows ────────────────────── */
.status-block { padding: 10px 0; border-bottom: 1px solid var(--border); }
.status-block:last-child { border-bottom: none; }
.status-row { display: flex; align-items: center; gap: 8px; font-size: 13px; }
.status-dot { width: 9px; height: 9px; border-radius: 50%; background: var(--fg-dim); flex-shrink: 0; }
.status-dot.on { background: #22c55e; box-shadow: 0 0 6px rgba(34,197,94,0.5); animation: pulse-dot 2s ease-in-out infinite; }
@keyframes pulse-dot {
  0%, 100% { box-shadow: 0 0 4px rgba(34,197,94,0.4); }
  50%      { box-shadow: 0 0 12px rgba(34,197,94,0.8); }
}
.status-name { font-weight: 600; font-family: "SF Mono", "Menlo", monospace; font-size: 13px; min-width: 100px; }
.status-port { color: var(--fg-dim); font-family: "SF Mono", "Menlo", monospace; font-size: 12px; }
.status-state { font-size: 12px; color: var(--fg-dim); }
.status-state.running { color: #22c55e; font-weight: 600; }
.status-pid { font-size: 11px; color: var(--fg-muted); margin-left: auto; font-family: "SF Mono", "Menlo", monospace; }
.proxy-action-btn {
  margin-left: auto; padding: 2px 10px; border: 1px solid var(--border-strong); border-radius: var(--radius-sm);
  font-size: 11px; font-weight: 600; cursor: pointer; background: var(--surface); color: var(--fg);
  transition: all 0.1s; outline: none; white-space: nowrap;
}
.proxy-action-btn:hover:not(:disabled) { border-color: var(--accent); color: var(--accent); }
.proxy-action-btn:disabled { opacity: 0.5; cursor: not-allowed; }
.apply-spin { display: inline-block; animation: spin 0.8s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
.status-desc { font-size: 12px; color: var(--fg-dim); margin-top: 4px; margin-left: 17px; line-height: 1.5; }
</style>
