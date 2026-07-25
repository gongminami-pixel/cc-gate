<script setup lang="ts">
import { ref } from "vue";
import { hideMainWindow, quitApp } from "../ipc/api";
import { listen } from "@tauri-apps/api/event";

const show = ref(false);
const dontShowAgain = ref(localStorage.getItem("ccgate.closeToTray") === "1");

listen("ccgate://close-requested", () => {
  if (dontShowAgain.value) {
    hideMainWindow();
  } else {
    show.value = true;
  }
});

function onClose() {
  hideMainWindow();
  if (dontShowAgain.value) {
    localStorage.setItem("ccgate.closeToTray", "1");
  }
  show.value = false;
}

function onQuit() {
  quitApp();
}
</script>

<template>
  <div v-if="show" class="overlay">
    <div class="dialog">
      <h3>关闭窗口</h3>
      <p>CC-Gate 会继续在菜单栏运行，代理进程保持活跃。</p>
      <label class="checkbox-label">
        <input type="checkbox" v-model="dontShowAgain" />
        <span>不再提示</span>
      </label>
      <div class="actions">
        <button class="btn" @click="onQuit">退出应用</button>
        <button class="btn primary" @click="onClose">隐藏到菜单栏</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
}
.dialog {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: 24px;
  max-width: 380px;
  width: 90%;
  box-shadow: var(--shadow-lg);
}
.dialog h3 { margin: 0 0 8px; font-size: 16px; }
.dialog p { color: var(--fg-muted); font-size: 13px; margin: 0 0 12px; }
.checkbox-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--fg-muted);
  margin-bottom: 16px;
  cursor: pointer;
}
.actions { display: flex; gap: 8px; justify-content: flex-end; }
</style>
