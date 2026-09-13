---
title: 'OpenAI Chat Completions'
description: 'Use /v1/chat/completions with any OpenAI SDK — streaming, tool calling, and the Bodhi-specific model resolution rules'
order: 1
---

# OpenAI Chat Completions

`POST /v1/chat/completions` is the most common way to talk to Bodhi. Anything that already speaks the OpenAI Chat Completions wire format — the official SDK, LangChain, LlamaIndex, the Vercel AI SDK, your hand-rolled curl script — works against this endpoint with two changes: point `base_url` at Bodhi, swap the API key for a Bodhi token.

## Auth

```
Authorization: Bearer <bodhi-api-token>
```

The token is a Bodhi-issued token, not a raw OpenAI key. See [API Tokens](/docs/features/auth/api-tokens) for how to mint one. The same token works against every endpoint on this server (chat completions, embeddings, responses, models, anthropic, gemini, MCP proxy).

If you call this endpoint from inside the Bodhi web UI, the session cookie authenticates you and no `Authorization` header is needed.

## Model resolution

The `model` field is matched against Bodhi's **combined catalog**:

- If it matches a **local model alias** (defined under [Models → Aliases](/docs/features/models/model-alias)), the request runs locally via llama.cpp.
- If it matches an **API model** (defined under [Models → API Models](/docs/features/models/api-models)), Bodhi forwards the request to the configured upstream provider, rewriting headers as needed.

The client doesn't know which path was taken — and that's the point. You can swap a local alias for an API-model alias with the same name and your client keeps working.

`GET /v1/models` returns the merged list (local aliases + API models). Use it to discover what's available before hardcoding anything.

For setting up remote providers (OpenAI, Anthropic, Gemini, Groq, OpenRouter, etc.), see [API Models](/docs/features/models/api-models).

## Streaming

Set `"stream": true` and the response comes back as Server-Sent Events (`Content-Type: text/event-stream`), one delta per chunk, terminated by a `data: [DONE]` line. This is identical to OpenAI's wire format, so the official SDK's streaming iterators work without modification — including for API-model requests proxied to upstream providers.

## Tool / function calling

Pass a `tools` array in OpenAI's standard shape. The server returns tool calls in `choices[].message.tool_calls`. For a worked example of an agentic loop that wires this up to MCP-discovered tools, see [Developer → Getting Started](/docs/developer/getting-started) and [SDK Advanced Patterns](/docs/developer/bodhi-js-sdk/advanced).

If you're using local model aliases, tool support depends on the underlying GGUF — check the alias's chat template and the model card. Most modern instruct models trained on tool data work; older base models won't.

## Examples

### curl

```bash
curl -X POST http://localhost:1135/v1/chat/completions \
  -H "Authorization: Bearer $BODHI_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "your-bodhi-alias",
    "messages": [
      {"role": "system", "content": "You are a helpful assistant."},
      {"role": "user", "content": "Explain MCP in two sentences."}
    ],
    "stream": false
  }'
```

For streaming, add `"stream": true` and read the response body line by line as SSE.

### Python (OpenAI SDK)

```python
from openai import OpenAI

client = OpenAI(
    base_url="http://localhost:1135/v1",
    api_key=os.environ["BODHI_TOKEN"],  # not a raw OpenAI key
)

# Non-streaming
resp = client.chat.completions.create(
    model="your-bodhi-alias",
    messages=[{"role": "user", "content": "Hello"}],
)
print(resp.choices[0].message.content)

# Streaming
stream = client.chat.completions.create(
    model="your-bodhi-alias",
    messages=[{"role": "user", "content": "Count to 5."}],
    stream=True,
)
for chunk in stream:
    delta = chunk.choices[0].delta.content or ""
    print(delta, end="", flush=True)
```

The only thing different from talking to OpenAI directly is `base_url` and `api_key`. Everything downstream — tool calling, streaming, structured output, JSON mode (where the underlying model supports it) — is the same wire format.

## Common gotchas

- **`model` not found.** The name must match a configured alias or API model exactly. Hit `/v1/models` to see what's available.
- **API-model alias misconfigured.** If the upstream provider rejects the request, Bodhi forwards the upstream's error envelope. The Bodhi-native errors (token, scope, alias-not-found) come back in [Bodhi's error format](/docs/api-compatibility/error-format). The OpenAI-shaped errors come back through the OpenAI envelope. The `error-format` page disambiguates.
- **CORS for browser clients.** `/v1/*` is configured with permissive CORS so browser apps can call it directly. Session-only endpoints aren't.

## Full schema

See Swagger UI at `http://<your-bodhi-instance>/swagger-ui` for the complete request/response schema, every supported parameter, and live "try it out" against your running instance. The local default is `http://localhost:1135/swagger-ui`.
