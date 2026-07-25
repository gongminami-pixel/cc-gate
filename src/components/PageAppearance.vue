<script setup lang="ts">
import { useTheme } from "../composables/useTheme";
const { theme } = useTheme();

function setTheme(t: string) { theme.value = t as "dark" | "light" | "auto"; }
</script>

<template>
  <section class="page">
    <header class="page-header"><h2>外观</h2></header>

    <div class="card">
      <div class="card-head">主题</div>
      <div class="card-body">
        <div style="max-width:260px">
          <div v-for="opt in [
            { val: 'dark', label: '深色',    desc: '暗色背景，护眼，推荐' },
            { val: 'light', label: '浅色',   desc: '亮色背景，白天使用' },
            { val: 'auto', label: '跟随系统', desc: '自动匹配 macOS 外观设置' },
          ]" :key="opt.val" class="theme-option" :class="{ active: theme === opt.val }" @click="setTheme(opt.val)">
            <div class="theme-dot" :class="{ checked: theme === opt.val }">
              <span v-if="theme === opt.val">✓</span>
            </div>
            <div>
              <div class="theme-label">{{ opt.label }}</div>
              <div class="theme-desc dim">{{ opt.desc }}</div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.card-head { padding: 14px 18px 0; font-size: 15px; font-weight: 700; }
.card-body { padding: 12px 18px 16px; }
.theme-option {
  display: flex; align-items: flex-start; gap: 12px;
  padding: 10px 12px; cursor: pointer; border-radius: var(--radius-md);
  border: 1px solid var(--border); margin-bottom: 6px;
  transition: border-color 0.15s, background 0.15s;
}
.theme-option:hover { background: var(--tap-target-on); }
.theme-option.active { border-color: var(--accent); background: var(--accent-soft); }
.theme-dot { width: 20px; height: 20px; border-radius: 50%; border: 2px solid var(--border-strong); display: flex; align-items: center; justify-content: center; font-size: 12px; color: #fff; flex-shrink: 0; margin-top: 1px; }
.theme-dot.checked { border-color: var(--accent); background: var(--accent); }
.theme-label { font-size: 13px; font-weight: 600; }
.theme-desc { font-size: 11px; margin-top: 1px; }
</style>
