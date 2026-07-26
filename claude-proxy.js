#!/usr/bin/env node
// claude-proxy.js — Anthropic Messages API → OpenAI Chat Completions
// Usage: node claude-proxy.js [--port 8789]
// Auto-discovers providers from ~/.mimo2codex/.env + providers.json

const http = require('http');
const https = require('https');
const fs = require('fs');
const path = require('path');
const os = require('os');

const PORT = parseInt(process.argv[process.argv.indexOf('--port') + 1]) || 8789;
const HOME = os.homedir();
const ENV_FILE = path.join(HOME, '.mimo2codex', '.env');
const PROVIDERS_FILE = path.join(HOME, '.mimo2codex', 'providers.json');

// ── Load API keys from .env ──────────────────────────────────
function loadEnv() {
  const env = {};
  if (fs.existsSync(ENV_FILE)) {
    fs.readFileSync(ENV_FILE, 'utf8').split('\n').forEach(line => {
      const m = line.match(/^(\w+)=(.+)$/);
      if (m) env[m[1]] = m[2].trim();
    });
  }
  return env;
}

// ── Load provider endpoints from providers.json ──────────────
function loadProviders(env) {
  const providers = {};
  if (fs.existsSync(PROVIDERS_FILE)) {
    const data = JSON.parse(fs.readFileSync(PROVIDERS_FILE, 'utf8'));
    for (const p of (data.providers || [])) {
      for (const m of (p.models || [])) {
        providers[m.id] = {
          baseUrl: p.baseUrl,
          apiKey: env[p.envKey] || '',
          defaultModel: m.id,
          displayName: m.displayName || m.id,
          contextWindow: m.contextWindow || 131072,
          maxOutputTokens: m.maxOutputTokens || 16384,
          anthropicEndpoint: p.anthropicEndpoint || false,
          anthropicModel: p.anthropicModel || null,
        };
      }
    }
  }
  // Built-in: DeepSeek (always available via default provider)
  if (!providers['deepseek-v4-pro']) {
    providers['deepseek-v4-pro'] = {
      baseUrl: 'https://api.deepseek.com/v1',
      apiKey: env.DS_API_KEY || env.DEEPSEEK_API_KEY || '',
      defaultModel: 'deepseek-v4-pro',
      displayName: 'DeepSeek V4 Pro',
      contextWindow: 1000000,
      maxOutputTokens: 393216,
    };
  }
  if (!providers['deepseek-v4-flash']) {
    providers['deepseek-v4-flash'] = {
      baseUrl: 'https://api.deepseek.com/v1',
      apiKey: env.DS_API_KEY || env.DEEPSEEK_API_KEY || '',
      defaultModel: 'deepseek-v4-flash',
      displayName: 'DeepSeek V4 Flash',
      contextWindow: 1000000,
      maxOutputTokens: 393216,
    };
  }
  return providers;
}

const env = loadEnv();
const PROVIDERS = loadProviders(env);

console.error(`Loaded ${Object.keys(PROVIDERS).length} providers: ${Object.keys(PROVIDERS).join(', ')}`);

// ── HTTP request helper ──────────────────────────────────────
function httpRequest(url, opts, body) {
  return new Promise((resolve, reject) => {
    const u = new URL(url);
    const mod = u.protocol === 'https:' ? https : http;
    const req = mod.request(u, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', ...opts.headers },
      timeout: 120000,
    }, res => {
      let data = '';
      res.on('data', c => data += c);
      res.on('end', () => {
        try { resolve({ status: res.statusCode, body: JSON.parse(data) }); }
        catch { resolve({ status: res.statusCode, body: data }); }
      });
    });
    req.on('error', reject);
    req.on('timeout', () => { req.destroy(); reject(new Error('timeout')); });
    req.write(JSON.stringify(body));
    req.end();
  });
}

// ── Anthropic Messages → OpenAI Chat Completions ─────────────
function anthropicToOpenAI(anthropicReq) {
  const messages = [];
  // System → system message
  if (anthropicReq.system) {
    if (typeof anthropicReq.system === 'string') {
      messages.push({ role: 'system', content: anthropicReq.system });
    } else if (Array.isArray(anthropicReq.system)) {
      for (const block of anthropicReq.system) {
        if (block.type === 'text') messages.push({ role: 'system', content: block.text });
      }
    }
  }
  // Messages — handle mixed content blocks (text, tool_use, tool_result)
  for (const msg of (anthropicReq.messages || [])) {
    if (typeof msg.content === 'string') {
      messages.push({ role: msg.role, content: msg.content });
    } else if (Array.isArray(msg.content)) {
      // Build OpenAI-format message from Anthropic content blocks
      let textParts = [];
      let toolCalls = [];
      for (const block of msg.content) {
        if (block.type === 'text') {
          textParts.push(block.text);
        } else if (block.type === 'tool_use') {
          toolCalls.push({
            id: block.id,
            type: 'function',
            function: {
              name: block.name,
              arguments: typeof block.input === 'string' ? block.input : JSON.stringify(block.input),
            },
          });
        } else if (block.type === 'tool_result') {
          // Tool result → tool message in OpenAI format
          messages.push({
            role: 'tool',
            tool_call_id: block.tool_use_id,
            content: typeof block.content === 'string' ? block.content : JSON.stringify(block.content),
          });
        }
      }
      const openaiMsg = { role: msg.role };
      if (textParts.length > 0) openaiMsg.content = textParts.join('\n');
      if (toolCalls.length > 0) openaiMsg.tool_calls = toolCalls;
      if (textParts.length > 0 || toolCalls.length > 0) {
        messages.push(openaiMsg);
      }
    }
  }
  // Translate Anthropic tools → OpenAI tools (function calling)
  const openaiReq = {
    model: anthropicReq.model,
    messages,
    max_tokens: anthropicReq.max_tokens || 4096,
    temperature: anthropicReq.temperature,
    top_p: anthropicReq.top_p,
    stop: anthropicReq.stop_sequences,
    stream: false,
  };
  if (anthropicReq.tools && anthropicReq.tools.length > 0) {
    openaiReq.tools = anthropicReq.tools.map(t => ({
      type: 'function',
      function: {
        name: t.name,
        description: t.description || '',
        parameters: t.input_schema || { type: 'object', properties: {} },
      },
    }));
    // Default to 'auto' — let model decide when to call tools
    openaiReq.tool_choice = 'auto';
  }
  return openaiReq;
}

// ── OpenAI Chat Completion → Anthropic Messages ─────────────
function openAIToAnthropic(openAIResp, model) {
  const choice = (openAIResp.choices || [])[0] || {};
  const msg = choice.message || {};
  const content = [];

  // Text content
  if (msg.content) {
    content.push({ type: 'text', text: msg.content });
  }

  // Tool calls → tool_use blocks
  if (msg.tool_calls && msg.tool_calls.length > 0) {
    for (const tc of msg.tool_calls) {
      let input;
      try {
        input = typeof tc.function.arguments === 'string'
          ? JSON.parse(tc.function.arguments)
          : tc.function.arguments;
      } catch {
        input = {};
      }
      content.push({
        type: 'tool_use',
        id: tc.id,
        name: tc.function.name,
        input,
      });
    }
  }

  // Fallback: if somehow no content at all, add empty text
  if (content.length === 0) {
    content.push({ type: 'text', text: '' });
  }

  // Determine stop_reason
  let stopReason = 'end_turn';
  if (choice.finish_reason === 'tool_calls') {
    stopReason = 'tool_use';
  } else if (choice.finish_reason === 'length' || choice.finish_reason === 'max_tokens') {
    stopReason = 'max_tokens';
  }

  return {
    id: `msg_${Date.now()}`,
    type: 'message',
    role: 'assistant',
    content,
    model,
    stop_reason: stopReason,
    stop_sequence: null,
    usage: {
      input_tokens: openAIResp.usage?.prompt_tokens || 0,
      output_tokens: openAIResp.usage?.completion_tokens || 0,
    },
  };
}

// ── Error response ───────────────────────────────────────────
function errorResponse(status, message) {
  return {
    type: 'error',
    error: { type: 'api_error', message, code: status },
  };
}

// ── Token-based routing ──────────────────────────────────────
const TOKEN_MAP = {
  'ds': 'deepseek-v4-pro',
  'qwen': 'qwen3.8-max-preview',
  'glm': 'glm-5.2',
  'mimo': 'mimo-v2.5-pro',
};

// ── Usage recording ────────────────────────────────────────
const USAGE_FILE = path.join(HOME, '.mimo2codex', 'usage.jsonl');

function modelToProvider(modelId) {
  if (modelId.startsWith('deepseek')) return 'deepseek';
  if (modelId.startsWith('glm')) return 'glm';
  if (modelId.startsWith('qwen3')) return 'qwen38';
  if (modelId.startsWith('qwen')) return 'qwen';
  if (modelId.startsWith('mimo')) return 'xiaomi';
  return 'unknown';
}

function recordUsage(modelId, usage, proxyName) {
  if (!usage || (!usage.input_tokens && !usage.prompt_tokens && !usage.total_tokens)) return;
  const prompt = usage.input_tokens || usage.prompt_tokens || 0;
  const completion = usage.output_tokens || usage.completion_tokens || 0;
  const total = usage.total_tokens || (prompt + completion);
  const record = {
    request_id: `claude-${Date.now()}-${Math.random().toString(36).slice(2,8)}`,
    model: modelId,
    provider: modelToProvider(modelId),
    prompt_tokens: prompt,
    completion_tokens: completion,
    total_tokens: total,
    proxy: proxyName,
    timestamp: new Date().toISOString(),
  };
  try {
    fs.appendFileSync(USAGE_FILE, JSON.stringify(record) + '\n');
  } catch (e) {
    console.error(`[usage] Failed to record: ${e.message}`);
  }
}

// ── /v1/models — gateway model discovery ────��───────────────
function handleModels(res) {
  console.error(`← GET /v1/models (gateway discovery)`);
  const models = Object.values(PROVIDERS).map(p => ({
    id: 'claude-' + p.defaultModel,       // claude- prefix required by CC
    type: 'model',
    display_name: p.displayName,
    created_at: '2025-01-01T00:00:00Z',
  }));
  res.writeHead(200, { 'Content-Type': 'application/json' });
  res.end(JSON.stringify({ data: models }));
}

// ── Main server ──────────────────────────────────────────────
const server = http.createServer(async (req, res) => {
  console.error(`${req.method} ${req.url}`);
  // Gateway model discovery
  if (req.method === 'GET' && req.url === '/v1/models') {
    handleModels(res);
    return;
  }

  if (req.method !== 'POST' || !req.url.startsWith('/v1/messages')) {
    res.writeHead(404, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify(errorResponse(404, 'Not found')));
    return;
  }

  // Extract token from x-api-key header (or Authorization Bearer)
  const authHeader = req.headers['x-api-key'] || '';
  const bearer = (req.headers['authorization'] || '').replace('Bearer ', '');
  const token = authHeader || bearer || 'ds';

  let body = '';
  req.on('data', c => body += c);
  req.on('end', async () => {
    let anthropicReq;
    try { anthropicReq = JSON.parse(body); }
    catch { res.writeHead(400); res.end(JSON.stringify(errorResponse(400, 'Invalid JSON'))); return; }

    const modelId = anthropicReq.model;
    // Strip claude- prefix (added by gateway model discovery)
    const realModelId = modelId.startsWith('claude-') ? modelId.slice(7) : modelId;
    // Resolve provider via token → model id → provider lookup
    const resolvedModel = TOKEN_MAP[token] || realModelId;
    const provider = PROVIDERS[resolvedModel];

    if (!provider) {
      console.error(`Unknown token: ${token}, model: ${modelId}`);
      res.writeHead(400, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify(errorResponse(400, `Unknown token: ${token}. Use: ${Object.keys(TOKEN_MAP).join(', ')}`)));
      return;
    }

    if (!provider.apiKey) {
      console.error(`No API key for ${modelId}`);
      res.writeHead(500, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify(errorResponse(500, `No API key configured for ${modelId}`)));
      return;
    }

    if (provider.anthropicEndpoint) {
      // ── Anthropic passthrough (provider speaks Anthropic natively) ──
      console.error(`→ ${modelId} → ${provider.displayName} (passthrough: ${provider.baseUrl})`);
      try {
        // Use provider's model name (or anthropicModel override for passthrough)
        const reqBody = { ...anthropicReq };
        reqBody.model = provider.anthropicModel || provider.defaultModel || modelId;
        
        const result = await httpRequest(provider.baseUrl + '/v1/messages', {
          headers: { 
            'x-api-key': provider.apiKey,
            'anthropic-version': '2023-06-01',
            'Content-Type': 'application/json'
          }
        }, reqBody);

        if (result.status === 200) {
          // recordUsage(modelId, result.body?.usage || {}, 'claude-proxy');
          res.writeHead(200, { 'Content-Type': 'application/json' });
          res.end(JSON.stringify(result.body));
        } else {
          const errMsg = result.body?.error?.message || JSON.stringify(result.body);
          console.error(`← ${modelId} ERROR ${result.status}: ${errMsg}`);
          res.writeHead(result.status, { 'Content-Type': 'application/json' });
          res.end(JSON.stringify(errorResponse(result.status, errMsg)));
        }
      } catch (e) {
        console.error(`← ${modelId} FAIL: ${e.message}`);
        res.writeHead(502, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify(errorResponse(502, `Upstream error: ${e.message}`)));
      }
    } else {
      // ── OpenAI Chat Completions translation ──
      const openaiReq = anthropicToOpenAI(anthropicReq);
      openaiReq.model = provider.defaultModel;
      const upstreamUrl = `${provider.baseUrl}/chat/completions`;
      console.error(`→ ${modelId} → ${provider.displayName} (translate: ${upstreamUrl})`);
      try {
        const result = await httpRequest(upstreamUrl, {
          headers: { 'Authorization': `Bearer ${provider.apiKey}` }
        }, openaiReq);
        if (result.status === 200) {
          const anthropicResp = openAIToAnthropic(result.body, modelId);
          // recordUsage(modelId, anthropicResp.usage || {}, 'claude-proxy');
          res.writeHead(200, { 'Content-Type': 'application/json' });
          res.end(JSON.stringify(anthropicResp));
        } else {
          const errMsg = result.body?.error?.message || JSON.stringify(result.body);
          console.error(`← ${modelId} ERROR ${result.status}: ${errMsg}`);
          res.writeHead(result.status, { 'Content-Type': 'application/json' });
          res.end(JSON.stringify(errorResponse(result.status, errMsg)));
        }
      } catch (e) {
        console.error(`← ${modelId} FAIL: ${e.message}`);
        res.writeHead(502, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify(errorResponse(502, `Upstream error: ${e.message}`)));
      }
    }
  });
});

server.listen(PORT, '127.0.0.1', () => {
  console.error(`Claude Proxy listening on http://127.0.0.1:${PORT}`);
  console.error(`Providers: ${Object.keys(PROVIDERS).join(', ')}`);
});
