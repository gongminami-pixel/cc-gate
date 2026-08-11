#!/usr/bin/env node
// Verifies the claude-proxy.js routing/key fixes against a fake upstream.
//
//   node scripts/test-claude-proxy.cjs
//
// Builds an isolated fake HOME and copies claude-proxy.js into it — mirroring how
// deploy_proxy_scripts() ships it to ~/.mimo2codex/ — so the real config is never
// touched. (The copy is also required because the repo root is "type": "module"
// while the proxy is CommonJS; it only runs outside the package.)
//
// Each case asserts on what the FAKE UPSTREAM actually received, not on proxy log
// text, so a fix that only changes logging cannot pass.

const http = require('http');
const fs = require('fs');
const os = require('os');
const path = require('path');
const { spawn } = require('child_process');

const ROOT = path.resolve(__dirname, '..');
const PROXY_SRC = path.join(ROOT, 'claude-proxy.js');
const PROXY_PORT = 18689;
const UPSTREAM_PORT = 18700;

let received = [];   // every request the fake upstream saw

function startUpstream() {
  return new Promise(resolve => {
    const srv = http.createServer((req, res) => {
      let body = '';
      req.on('data', c => body += c);
      req.on('end', () => {
        let parsed = null;
        try { parsed = JSON.parse(body); } catch {}
        received.push({
          url: req.url,
          apiKey: req.headers['x-api-key'] || null,
          authorization: req.headers['authorization'] || null,
          anthropicVersion: req.headers['anthropic-version'] || null,
          model: parsed?.model ?? null,
        });
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({
          id: 'msg_test', type: 'message', role: 'assistant', model: parsed?.model,
          content: [{ type: 'text', text: 'ok' }], stop_reason: 'end_turn',
          usage: { input_tokens: 1, output_tokens: 1 },
          // OpenAI shape too, so the translation path can parse the same response
          choices: [{ message: { role: 'assistant', content: 'ok' }, finish_reason: 'stop' }],
        }));
      });
    });
    srv.listen(UPSTREAM_PORT, '127.0.0.1', () => resolve(srv));
  });
}

// Isolated fake HOME so the real ~/.mimo2codex is untouched.
function makeFakeHome() {
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'ccgate-test-'));
  const dir = path.join(home, '.mimo2codex');
  fs.mkdirSync(dir, { recursive: true });
  fs.copyFileSync(PROXY_SRC, path.join(dir, 'claude-proxy.js'));

  // Non-ASCII key name — the old /^(\w+)=/ regex silently dropped this line.
  // RELAY_X975EX7EBF_API_KEY is relay_env_key("非线") in the Rust writer.
  fs.writeFileSync(path.join(dir, '.env'), [
    '# comment line',
    'RELAY_X975EX7EBF_API_KEY=relay-cjk-key',
    'RELAY_NL_API_KEY=relay-nl-key',
    'DEEPSEEK_API_KEY=ds-key',
    'HAS_EQUALS_IN_VALUE=abc=def==',
    '',
  ].join('\n'));

  const base = `http://127.0.0.1:${UPSTREAM_PORT}`;
  fs.writeFileSync(path.join(dir, 'providers.json'), JSON.stringify({
    providers: [
      {
        // Relay serving a model literally named claude-opus-5 — Bug 1's victim.
        id: 'anthropic-relay-nl', name: 'NL Relay', baseUrl: base,
        envKey: 'RELAY_X975EX7EBF_API_KEY', defaultModel: 'claude-opus-5',
        anthropicEndpoint: true, anthropicModel: 'claude-opus-5',
        anthropicVersion: '2099-01-01',
        models: [{ id: 'claude-opus-5', displayName: 'NL Opus 5' }],
      },
      {
        id: 'deepseek-direct', name: 'DeepSeek', baseUrl: base,
        envKey: 'DEEPSEEK_API_KEY', defaultModel: 'deepseek-v4-pro',
        models: [{ id: 'deepseek-v4-pro', displayName: 'DeepSeek V4 Pro' }],
      },
    ],
  }, null, 2));
  return home;
}

function startProxy(home) {
  const script = path.join(home, '.mimo2codex', 'claude-proxy.js');
  const p = spawn(process.execPath, [script, '--port', String(PROXY_PORT)], {
    env: { ...process.env, HOME: home, USERPROFILE: home },
    stdio: ['ignore', 'ignore', 'pipe'],
  });
  let log = '';
  p.stderr.on('data', d => log += d.toString());
  return new Promise((resolve, reject) => {
    const t = setTimeout(() => reject(new Error('proxy did not start:\n' + log)), 8000);
    const tick = setInterval(() => {
      if (/listening/i.test(log)) { clearTimeout(t); clearInterval(tick); resolve({ p, getLog: () => log }); }
    }, 100);
    p.on('exit', c => { clearTimeout(t); clearInterval(tick); reject(new Error(`proxy exited (${c}):\n` + log)); });
  });
}

function post(model, apiKey) {
  return new Promise((resolve, reject) => {
    const payload = JSON.stringify({ model, max_tokens: 16, messages: [{ role: 'user', content: 'hi' }] });
    const req = http.request({
      host: '127.0.0.1', port: PROXY_PORT, path: '/v1/messages', method: 'POST',
      headers: { 'Content-Type': 'application/json', 'x-api-key': apiKey },
    }, res => {
      let d = ''; res.on('data', c => d += c);
      res.on('end', () => { let b = null; try { b = JSON.parse(d); } catch {} resolve({ status: res.statusCode, body: b, raw: d }); });
    });
    req.on('error', reject);
    req.end(payload);
  });
}

function getModels() {
  return new Promise((resolve, reject) => {
    http.get({ host: '127.0.0.1', port: PROXY_PORT, path: '/v1/models' }, res => {
      let d = ''; res.on('data', c => d += c);
      res.on('end', () => { try { resolve(JSON.parse(d)); } catch (e) { reject(e); } });
    }).on('error', reject);
  });
}

let pass = 0, fail = 0;
function check(name, cond, detail) {
  if (cond) { pass++; console.log(`  ✅ ${name}`); }
  else { fail++; console.log(`  ❌ ${name}\n       ${detail}`); }
}

(async () => {
  const upstream = await startUpstream();
  const home = makeFakeHome();
  const { p: proxy } = await startProxy(home);
  try {
    console.log('\n── Bug 5: loadEnv reads non-ASCII key names ──');
    // The CJK-named key is what the relay provider references. If loadEnv dropped
    // that line, provider.apiKey is empty and the proxy 500s before any upstream call.
    received = [];
    let r = await post('claude-claude-opus-5', 'proxy');
    check('non-ASCII env key resolved (no "No API key" 500)',
      r.status === 200, `status=${r.status} body=${r.raw.slice(0, 200)}`);

    console.log('\n── Bug 1: relay-configured claude-* is NOT hijacked to api.anthropic.com ──');
    check('request reached the fake upstream, not Anthropic',
      received.length === 1, `upstream saw ${received.length} request(s)`);
    check('routed to the relay /v1/messages',
      received[0]?.url === '/v1/messages', `url=${received[0]?.url}`);

    console.log('\n── Bug 3: passthrough sends the PROVIDER key, not the client token ──');
    check('x-api-key === relay key from .env',
      received[0]?.apiKey === 'relay-cjk-key',
      `got ${JSON.stringify(received[0]?.apiKey)} (client sent "proxy")`);
    check('per-provider anthropic-version honored',
      received[0]?.anthropicVersion === '2099-01-01', `got ${received[0]?.anthropicVersion}`);

    console.log('\n── Bug 2: model name is not mangled by a blind slice(7) ──');
    check('upstream received "claude-opus-5", not "opus-5"',
      received[0]?.model === 'claude-opus-5', `model=${received[0]?.model}`);

    // Bypassing gateway discovery: send the raw providers.json name.
    received = [];
    r = await post('claude-opus-5', 'proxy');
    check('raw name (discovery bypassed) still routes',
      r.status === 200 && received[0]?.model === 'claude-opus-5',
      `status=${r.status} model=${received[0]?.model}`);

    console.log('\n── gateway-prefixed non-Claude model still strips correctly ──');
    received = [];
    r = await post('claude-deepseek-v4-pro', 'proxy');
    check('deepseek routed via OpenAI translation',
      r.status === 200 && received[0]?.url === '/chat/completions',
      `status=${r.status} url=${received[0]?.url}`);
    check('translation path uses provider key as Bearer',
      received[0]?.authorization === 'Bearer ds-key', `got ${received[0]?.authorization}`);

    console.log('\n── TOKEN_MAP no longer overrides an explicit model ──');
    // x-api-key "ds" is a legacy shorthand for deepseek. It must not silently
    // redirect an explicitly requested relay model.
    received = [];
    r = await post('claude-claude-opus-5', 'ds');
    check('explicit model wins over token shorthand',
      received[0]?.model === 'claude-opus-5', `model=${received[0]?.model}`);

    console.log('\n── unroutable model: clear error, no "Unknown token" red herring ──');
    r = await post('claude-totally-made-up', 'proxy');
    check('400 with model-centric message',
      r.status === 400 && /No provider configured for model/.test(r.body?.error?.message || ''),
      `status=${r.status} msg=${r.body?.error?.message}`);

    console.log('\n── /v1/models discovery still prefixes with claude- ──');
    const models = await getModels();
    const ids = (models.data || []).map(m => m.id);
    check('claude-claude-opus-5 advertised', ids.includes('claude-claude-opus-5'), `ids=${ids.join(', ')}`);
    check('claude-deepseek-v4-pro advertised', ids.includes('claude-deepseek-v4-pro'), `ids=${ids.join(', ')}`);

    console.log('\n── source hygiene ──');
    const src = fs.readFileSync(PROXY_SRC, 'utf8');
    check('default port is 8689, no 8789 left', !/\|\|\s*8789/.test(src), 'found "|| 8789"');
    check('no DEBUG scaffolding left in proxy', !/DEBUG modelId/.test(src), 'stray DEBUG log found');
    check('no hardcoded ANTHROPIC_MODELS list', !/ANTHROPIC_MODELS/.test(src), 'list still present');
  } finally {
    proxy.kill();
    upstream.close();
    fs.rmSync(home, { recursive: true, force: true });
  }

  console.log(`\n${fail === 0 ? '✅' : '❌'} ${pass} passed, ${fail} failed\n`);
  process.exit(fail === 0 ? 0 : 1);
})().catch(e => { console.error('harness error:', e); process.exit(1); });
