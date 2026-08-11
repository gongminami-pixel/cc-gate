# Claude Code 流式代理修复指南

## 目标文件

`~/.mimo2codex/claude-proxy.js`

## 问题清单

修复前该文件存在 4 个 bug：

| # | Bug | 症状 |
|---|-----|------|
| 1 | 双重 message_stop | Claude Code 每句话回复两次 |
| 2 | 缺失 tool_use 的 SSE 事件 | 工具调用报 "tool call could not be parsed" |
| 3 | output_tokens 硬编码为 0 | 状态栏看不到 token 数 |
| 4 | message_start 里 input_tokens=0 | 状态栏完全不显示 |

---

## 修复方法：替换整个 `openaiStreamToAnthropicSSE` 函数体

找到第 193 行的变量声明区域，从 `let msgStarted = false;` 开始，
到 `upstreamRes.on('end', () => {` 回调结束（约到第 282 行的 `});`），
**整段替换**为下面的代码。

### 替换范围（旧代码）

从：
```
      let msgStarted = false;
      let blockStarted = false;
      let inputTokens = 0;
      ...
      upstreamRes.on('end', () => {
        ...
        resolve();
      });
```

到这一段结束的 `});` 以及紧接着的 `upstreamRes.on('error', reject);` **之前**。

### 新代码（完整替换）

```javascript
      let msgStarted = false;
      let blockIdx = -1;
      let blockKind = null;
      let inputTokens = 0;
      let outputTokens = 0;
      let finalStopReason = 'end_turn';
      let finished = false;
      const msgId = `msg_${Date.now()}`;
      const tcMap = new Map();           // tool call index → {id, name}
      const pending = [];                // buffer chunks before message_start

      // ── helpers ──────────────────────────────────────────
      function closeBlock() {
        if (blockIdx >= 0) {
          clientRes.write(`event: content_block_stop\ndata: ${
            JSON.stringify({type:'content_block_stop',index:blockIdx})
          }\n\n`);
          blockIdx = -1; blockKind = null;
        }
      }

      function flushMsgStart() {
        if (msgStarted) return;
        clientRes.write(`event: message_start\ndata: ${
          JSON.stringify({type:'message_start',message:{
            id:msgId,type:'message',role:'assistant',content:[],
            model:modelId,stop_reason:null,stop_sequence:null,
            usage:{input_tokens:inputTokens}
          }})
        }\n\n`);
        msgStarted = true;
        for (const fn of pending) fn();
        pending.length = 0;
      }

      function emitFinal() {
        if (finished) return; finished = true;
        flushMsgStart();
        closeBlock();
        clientRes.write(`event: message_delta\ndata: ${
          JSON.stringify({type:'message_delta',delta:{
            stop_reason:finalStopReason,stop_sequence:null
          },usage:{output_tokens:outputTokens}})
        }\n\n`);
        clientRes.write(`event: message_stop\ndata: ${
          JSON.stringify({type:'message_stop'})
        }\n\n`);
      }

      // ── data handler ──────────────────────────────────────
      let buffer = '';
      upstreamRes.on('data', chunk => {
        buffer += chunk.toString();
        const lines = buffer.split('\n');
        buffer = lines.pop() || '';

        for (const line of lines) {
          const s = line.trim();
          if (!s.startsWith('data: ')) continue;
          const p = s.slice(6).trim();
          if (p === '[DONE]') { emitFinal(); clientRes.end(); resolve(); return; }

          let obj;
          try { obj = JSON.parse(p); } catch { continue; }
          const ch = (obj.choices || [])[0] || {};
          const d = ch.delta || {};
          const fr = ch.finish_reason;

          // input tokens — DeepSeek sends this in the LAST chunk
          if (obj.usage?.prompt_tokens) {
            inputTokens = obj.usage.prompt_tokens;
            flushMsgStart();         // now we have the real value, flush everything
          }
          // output tokens + stop reason from the finish_reason chunk
          if (fr) {
            if (obj.usage?.completion_tokens) outputTokens = obj.usage.completion_tokens;
            if (fr === 'tool_calls') finalStopReason = 'tool_use';
            else if (fr === 'length' || fr === 'max_tokens') finalStopReason = 'max_tokens';
            else finalStopReason = 'end_turn';
          }

          // ── text content ──────────────────────────────
          const doText = () => {
            if (d.content != null) {
              if (blockKind !== 'text') closeBlock();
              if (blockIdx < 0) {
                blockIdx = 0; blockKind = 'text';
                clientRes.write(`event: content_block_start\ndata: ${
                  JSON.stringify({type:'content_block_start',index:0,
                    content_block:{type:'text',text:''}})
                }\n\n`);
              }
              clientRes.write(`event: content_block_delta\ndata: ${
                JSON.stringify({type:'content_block_delta',index:0,
                  delta:{type:'text_delta',text:d.content}})
              }\n\n`);
            }
          };

          // ── tool calls ────────────────────────────────
          const doTools = () => {
            if (!d.tool_calls) return;
            for (const tc of d.tool_calls) {
              const i = tc.index;
              if (!tcMap.has(i)) tcMap.set(i, { id: tc.id || '', name: '' });
              const e = tcMap.get(i);
              if (tc.id) e.id = tc.id;
              if (tc.function?.name) e.name = tc.function.name;

              if (blockKind !== 'tool_use' || blockIdx !== i) closeBlock();
              if (blockIdx < 0) {
                blockIdx = i; blockKind = 'tool_use';
                clientRes.write(`event: content_block_start\ndata: ${
                  JSON.stringify({type:'content_block_start',index:i,
                    content_block:{type:'tool_use',id:e.id,name:e.name,input:{}}})
                }\n\n`);
              }
              if (tc.function?.arguments) {
                clientRes.write(`event: content_block_delta\ndata: ${
                  JSON.stringify({type:'content_block_delta',index:i,
                    delta:{type:'input_json_delta',partial_json:tc.function.arguments}})
                }\n\n`);
              }
            }
          };

          // ── dispatch ──────────────────────────────────
          if (!msgStarted && !inputTokens) {
            // Defer: message_start hasn't been sent yet, buffer content
            pending.push(doText);
            pending.push(doTools);
          } else {
            flushMsgStart();
            doText();
            doTools();
          }
        }
      });

      // ── end handler ────────────────────────────────────────
      upstreamRes.on('end', () => {
        if (!finished && buffer.trim().startsWith('data: ') && buffer.trim().slice(6).trim() !== '[DONE]') {
          try {
            const obj = JSON.parse(buffer.trim().slice(6).trim());
            const d = (obj.choices || [])[0]?.delta || {};
            if (d.content != null) {
              if (blockKind !== 'text') closeBlock();
              if (blockIdx < 0) { blockIdx = 0; blockKind = 'text';
                clientRes.write(`event: content_block_start\ndata: ${
                  JSON.stringify({type:'content_block_start',index:0,
                    content_block:{type:'text',text:''}})
                }\n\n`);
              }
              clientRes.write(`event: content_block_delta\ndata: ${
                JSON.stringify({type:'content_block_delta',index:0,
                  delta:{type:'text_delta',text:d.content}})
              }\n\n`);
            }
          } catch {}
        }
        emitFinal();
        if (!clientRes.writableEnded) clientRes.end();
        resolve();
      });
```

---

## 修复后重启

替换完成后，重启 8689 端口的代理进程（不要重启 Claude Code 本身）：

```bash
kill $(lsof -tiTCP:8689 -sTCP:LISTEN) 2>/dev/null
sleep 1
# cc-gate 会在几秒内自动检测并重新拉起代理
```

---

## 验证

```bash
# 1. 文本流式
curl -s -N http://127.0.0.1:8689/v1/messages \
  -H "Content-Type: application/json" -H "x-api-key: ds" \
  -d '{"model":"claude-deepseek-v4-pro","max_tokens":30,
       "messages":[{"role":"user","content":"hi"}],"stream":true}' \
  2>&1 | grep "input_tokens"

# 应该输出类似: "input_tokens":5  （非零）

# 2. 工具调用
curl -s -N http://127.0.0.1:8689/v1/messages \
  -H "Content-Type: application/json" -H "x-api-key: ds" \
  -d '{"model":"claude-deepseek-v4-pro","max_tokens":100,
       "messages":[{"role":"user","content":"1+1"}],
       "tools":[{"name":"calc","description":"math",
        "input_schema":{"type":"object","properties":{"e":{"type":"string"}}}}],
       "stream":true}' 2>&1 | grep "^event:"

# 应该看到: message_start → content_block_start → content_block_delta ×N
#           → content_block_stop → message_delta → message_stop
# 每个 event 只出现一次，没有重复
```

---

## 关键设计说明

### 1. `pending[]` 缓冲区
DeepSeek 的 `prompt_tokens` 在流式的**最后一个** chunk 才返回，
但 Anthropic 协议要求 `message_start`（**第一个**事件）就带 `input_tokens`。
所以先缓冲前面的内容 chunk，等 `prompt_tokens` 到了再一次性发出
`message_start` + 缓冲的内容。

### 2. `emitFinal()` 互斥
`[DONE]` 和 `upstreamRes.on('end')` 都可能触发结束事件。
`finished` 标志位保证只会发一次 `message_delta` + `message_stop`。

### 3. `tcMap` 工具调用追踪
OpenAI 流式的 tool_calls 分多个 chunk 到达（第一个 chunk 带 id+name，
后续 chunk 带 arguments 片段）。`tcMap` 按 index 记录每个工具调用的状态，
确保 `content_block_start` 只发一次。

### 4. cc-gate 文件还原
⚠️ cc-gate 有文件监控/备份机制，可能会自动还原被修改的 `claude-proxy.js`。
如果修改后立刻被还原，需要：
- 关掉 cc-gate 的文件监控功能，或
- 在 cc-gate 源码中直接修改代理脚本路径或内容
