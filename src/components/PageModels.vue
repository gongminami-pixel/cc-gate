<script setup lang="ts">
import type { AppConfig } from "../types/models";

defineProps<{ config: AppConfig | null }>();

function fmtTokens(n: number): string { return n >= 1_000_000 ? `${(n/1_000_000).toFixed(1)}M` : n >= 1_000 ? `${(n/1_000).toFixed(0)}K` : String(n); }

function providerLabel(p: string): string {
  const m: Record<string, string> = {
    deepseek: "DeepSeek", glm: "智谱 GLM", qwen: "阿里 Qwen-Max",
    qwen38: "阿里 Qwen3.8", xiaomi: "小米 MiMo",
    anthropic: "Anthropic Opus", openai: "OpenAI GPT",
  };
  return m[p] || p;
}
</script>

<template>
  <section class="page">
    <header class="page-header">
      <h2>模型参数</h2>
      <span class="desc dim">
        价格以各厂商官网为准，Opus / GPT 为估算值
      </span>
    </header>

    <div class="card mt12" v-if="config">
      <div class="table-wrap">
        <table class="param-table">
          <thead>
            <tr>
              <th>厂商</th>
              <th>模型</th>
              <th>slug</th>
              <th>上下文</th>
              <th>最大输出</th>
              <th>入价 / 1K</th>
              <th>出价 / 1K</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="m in config.models" :key="m.slug">
              <td class="provider-cell">{{ providerLabel(m.provider) }}</td>
              <td class="name-cell">{{ m.display_name }}</td>
              <td><code>{{ m.slug }}</code></td>
              <td>{{ fmtTokens(m.context_window) }}</td>
              <td>{{ fmtTokens(m.max_output_tokens) }}</td>
              <td>${{ m.input_price_per_1k.toFixed(4) }}</td>
              <td>${{ m.output_price_per_1k.toFixed(4) }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
    <div v-else class="dim" style="padding:48px;text-align:center">Loading…</div>
  </section>
</template>

<style scoped>
.desc { font-size: 13px; }
.table-wrap { overflow-x: auto; }

.param-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 14px;
}
.param-table thead {
  position: sticky; top: 0; z-index: 2;
}
.param-table th {
  text-align: left;
  padding: 10px 14px;
  font-weight: 700;
  color: var(--fg-muted);
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  border-bottom: 2px solid var(--border-strong);
  background: var(--surface);
  white-space: nowrap;
}
.param-table td {
  padding: 9px 14px;
  border-bottom: 1px solid var(--border);
  white-space: nowrap;
}
.param-table tbody tr:hover { background: var(--tap-target-on); }

.provider-cell { color: var(--fg-muted); font-size: 13px; }
.name-cell { font-weight: 600; }
.param-table code { font-size: 13px; color: var(--accent-soft-fg); }
</style>
