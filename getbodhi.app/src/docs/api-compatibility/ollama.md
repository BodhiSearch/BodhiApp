---
title: 'Ollama (deprecated)'
description: '/api/* compatibility for legacy Ollama clients — limited surface, kept for migration only'
order: 6
---

# Ollama Compatibility

> **Deprecated.** The Ollama compat layer is kept so existing tools and scripts that hardcoded the Ollama API can keep working while you migrate them. **For new clients, use the OpenAI-compatible endpoints under `/v1/*`** — they're better supported, get the most features, and are the focus of ongoing work.

If you have a tool that already calls Ollama's HTTP API (the CLI, a homegrown script, an old IDE plugin, etc.), point its base URL at Bodhi and it should work for the supported endpoints below.

## Supported endpoints

Bodhi implements the three Ollama endpoints that real-world clients depend on most:

| Endpoint    | Method | Purpose                                                                            |
| ----------- | ------ | ---------------------------------------------------------------------------------- |
| `/api/tags` | GET    | List available models (Bodhi's combined catalog reshaped into Ollama's tag format) |
| `/api/show` | POST   | Show model details for one tag                                                     |
| `/api/chat` | POST   | Chat with a model (streaming and non-streaming)                                    |

That's the entire supported surface. **Anything not listed here isn't supported** — `/api/generate`, `/api/pull`, `/api/push`, `/api/create`, `/api/copy`, `/api/delete`, `/api/embeddings` (the Ollama-shaped one), the model-management endpoints, and so on. Don't enumerate further; assume the rest is absent.

## Auth

```
Authorization: Bearer <bodhi-api-token>
```

Same Bodhi-issued token as every other endpoint on the server. The Ollama CLI doesn't natively send a Bearer token — see its docs for the env var (`OLLAMA_HOST` plus a custom auth header config) or use a wrapper.

## Quick check

```bash
curl http://localhost:1135/api/tags \
  -H "Authorization: Bearer $BODHI_TOKEN"
```

The `models` array in the response lists the same things `GET /v1/models` returns, just in Ollama's tag format.

## Migration tip

If you're touching the calling code anyway, switch from `/api/chat` to `/v1/chat/completions`. The OpenAI format is more widely supported, has richer tool-calling and streaming semantics, and is the path getting active investment. See [OpenAI Chat Completions](/docs/api-compatibility/openai-chat-completions) for the equivalent calls.

For embeddings, use [`/v1/embeddings`](/docs/api-compatibility/openai-embeddings) — Bodhi does not expose an Ollama-format embeddings endpoint.

## Full schema

See Swagger UI at `http://<your-bodhi-instance>/swagger-ui` for the exact request/response shapes Bodhi returns for these three endpoints. Default local URL: `http://localhost:1135/swagger-ui`.
