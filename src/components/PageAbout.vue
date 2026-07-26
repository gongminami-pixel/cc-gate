<script setup lang="ts">
</script>

<template>
  <section class="page">
    <header class="page-header">
      <h2>关于 CC-Gate</h2>
      <span class="badge on">v0.1.0</span>
    </header>

    <div class="about-content">
      <p class="about-desc">
        CC-Gate 是一个多模型 AI 工具配置管理器桌面应用，
        一键管理所有 AI Agent 的模型选择、中转站、API Key、Shell Alias 和用量统计。
      </p>

      <div class="feature-grid">
        <div class="feature-card">
          <div class="feature-icon">🔄</div>
          <div class="feature-title">Agent 模型分配</div>
          <div class="feature-detail">
            为 Codex、Claude Code、Hermes、OpenClaw、Aider 等 10 个 Agent
            独立分配可用模型，CLI 与桌面端分开管理
          </div>
        </div>

        <div class="feature-card">
          <div class="feature-icon">↗↘</div>
          <div class="feature-title">模型路由</div>
          <div class="feature-detail">
            每个模型可选择「直连」或任意中转站转发请求，
            一键切换无需改代码
          </div>
        </div>

        <div class="feature-card">
          <div class="feature-icon">🔑</div>
          <div class="feature-title">API Key 管理</div>
          <div class="feature-detail">
            22 个国内外大模型提供商统一管理 API Key，
            中转站密钥集中填入，自动检测环境变量
          </div>
        </div>

        <div class="feature-card">
          <div class="feature-icon">⚡</div>
          <div class="feature-title">Shell Alias</div>
          <div class="feature-detail">
            CLI Agent 模型自动生成 alias 写入 Shell 配置，
            跨平台支持 macOS、Linux、Windows (PowerShell)
          </div>
        </div>

        <div class="feature-card">
          <div class="feature-icon">📊</div>
          <div class="feature-title">用量统计</div>
          <div class="feature-detail">
            按今天/昨天/本周/本月等 8 个时段分桶，
            查看每个模型的 Token 消耗与费用
          </div>
        </div>

        <div class="feature-card">
          <div class="feature-icon">🚀</div>
          <div class="feature-title">自动启动</div>
          <div class="feature-detail">
            3 个代理进程（mimo2codex / claude-proxy / chat-proxy）
            随 App 启动自动拉起，统���管理
          </div>
        </div>

        <div class="feature-card">
          <div class="feature-icon">🔧</div>
          <div class="feature-title">工具检测</div>
          <div class="feature-detail">
            自动检测 Node.js、Python、Codex CLI、Claude Code、Aider 等
            依赖工具是否安装，提供安装指引
          </div>
        </div>

        <div class="feature-card">
          <div class="feature-icon">🌐</div>
          <div class="feature-title">中转站配置</div>
          <div class="feature-detail">
            内置 OpenRouter / Groq / Together / DeepInfra / Cloudflare
            预设，快速填入 API 地址
          </div>
        </div>
      </div>

      <div class="about-footer">
        <p>CC-Gate 替代原 CC Switch，统一管理多模型 AI 工具的配置分发。</p>
        <p class="dim" style="margin-top:8px">端口：mimo2codex :8688 · claude-proxy :8689 · chat-proxy :8690</p>
      </div>
    </div>
  </section>
</template>

<style scoped>
.about-content { padding: 0; max-width: 720px; }
.about-desc { font-size: 14px; color: var(--fg-dim); line-height: 1.6; margin: 0 0 24px; }

.feature-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }
.feature-card {
  padding: 16px; border-radius: var(--radius-lg);
  background: var(--surface); border: 1px solid var(--border);
}
.feature-icon { font-size: 22px; margin-bottom: 6px; }
.feature-title { font-size: 14px; font-weight: 700; margin-bottom: 4px; }
.feature-detail { font-size: 12px; color: var(--fg-dim); line-height: 1.5; }

.about-footer { margin-top: 28px; padding-top: 16px; border-top: 1px solid var(--border); font-size: 13px; color: var(--fg-muted); }
</style>
