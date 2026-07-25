<script setup lang="ts">
import { onMounted, ref } from "vue";
import type { AppConfig } from "../types/models";
import { saveConfig, setAppAutostart, getAppAutostartStatus } from "../ipc/api";
import { useToast } from "../composables/useToast";
import { useAppConfig } from "../composables/useAppConfig";

const props = defineProps<{ config: AppConfig | null }>();
const toast = useToast();
const { refresh } = useAppConfig();

const autostartEnabled = ref(false);
const autostartBusy = ref(false);

async function loadAutostart() {
  try { autostartEnabled.value = (await getAppAutostartStatus()).enabled; } catch { autostartEnabled.value = false; }
}
async function onToggleAutostart() {
  autostartBusy.value = true;
  try { const r = await setAppAutostart(!autostartEnabled.value); autostartEnabled.value = r.enabled; toast.ok(r.enabled ? "已启用登录时自动启动" : "已关闭登录时自动启动"); }
  catch (e: any) { toast.err(e?.message ?? String(e)); } finally { autostartBusy.value = false; }
}
async function onToggleProxyAutostart(key: "autostart_mimo2codex" | "autostart_claude_proxy" | "autostart_chat_proxy") {
  if (!props.config) return;
  props.config[key] = !props.config[key];
  try { await saveConfig(props.config); toast.ok("已保存"); await refresh(); }
  catch (e: any) { toast.err(e?.message ?? String(e)); }
}

onMounted(loadAutostart);
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
      <div class="card-head">代理进程</div>
      <div class="card-body">
        <p class="sec-desc">App 启动时自动拉起以下代理进程。关闭则需手动在「首页」点应用后重启。</p>
        <div class="toggle-row"><span>mimo2codex (端口 :8688)</span><label class="toggle"><input type="checkbox" :checked="config?.autostart_mimo2codex" @change="onToggleProxyAutostart('autostart_mimo2codex')" /><span class="slider"></span></label></div>
        <div class="toggle-row"><span>claude-proxy (端口 :8689)</span><label class="toggle"><input type="checkbox" :checked="config?.autostart_claude_proxy" @change="onToggleProxyAutostart('autostart_claude_proxy')" /><span class="slider"></span></label></div>
        <div class="toggle-row"><span>chat-proxy (端口 :8690)</span><label class="toggle"><input type="checkbox" :checked="config?.autostart_chat_proxy" @change="onToggleProxyAutostart('autostart_chat_proxy')" /><span class="slider"></span></label></div>
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
</style>
