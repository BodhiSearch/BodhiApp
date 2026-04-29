---
title: 'Overview'
description: 'Functional entry point for the API Compatibility section — endpoint map, unified auth, and how each compat layer is laid out'
order: 0
---

# API Compatibility — Overview

This section is the **user manual for each compat layer** Bodhi exposes. If you're looking for the mental model — _why_ Bodhi speaks four wire formats at once — start at [Concepts → API Compatibility](/docs/concepts/api-compatibility). The pages here answer the next question: _how do I actually use it?_

Each per-format page covers Bodhi-specific gotchas (auth, model resolution, streaming, rejection rules) plus one or two copy-paste-runnable examples. Endpoint schemas are intentionally **not** duplicated — that's what Swagger UI is for (see below).

## The endpoint map

| Path                                  | Format           | Purpose                                          |
| ------------------------------------- | ---------------- | ------------------------------------------------ |
| `/v1/chat/completions`                | OpenAI           | Chat completions, streaming, tool calling        |
| `/v1/responses`                       | OpenAI Responses | Async polling for reasoning / long-running tasks |
| `/v1/embeddings`                      | OpenAI           | Text embeddings                                  |
| `/v1/models`                          | OpenAI           | Combined catalog (local aliases + API models)    |
| `/anthropic/v1/messages`              | Anthropic        | Anthropic Messages, streaming, tool use          |
| `/anthropic/v1/models`                | Anthropic        | Anthropic-shaped model listing                   |
| `/v1beta/models/...`                  | Gemini           | `generateContent`, `embedContent`, etc.          |
| `/api/tags`, `/api/show`, `/api/chat` | Ollama           | Legacy Ollama clients (deprecated)               |
| `/bodhi/v1/apps/mcps/{id}/mcp`        | MCP              | Authenticated MCP proxy                          |

All of these resolve to the same unified inference layer. Switching wire format does not switch models.

## Auth model — one Bearer token, regardless of compat layer

Every compat layer accepts the same auth at the gateway: a Bodhi API token (or a session cookie if you're calling from the built-in UI).

```
Authorization: Bearer <bodhi-api-token>
```

This is true even when you call `/anthropic/v1/messages` (which natively expects `x-api-key`) or `/v1beta/...` (which natively expects `x-goog-api-key`). Bodhi accepts the Bearer token, validates it against your token record, and rewrites the outbound headers server-side when proxying to a remote provider. **You never put a raw provider key in the request.** Provider keys are stored once in the API-model record and reused for every call.

The two native provider headers are _also_ accepted as a transport convenience:

- `/anthropic/v1/messages` accepts `x-api-key: <bodhi-api-token>` — your existing Anthropic SDK can point at Bodhi without code changes.
- `/v1beta/...` accepts `x-goog-api-key: <bodhi-api-token>` (or `?key=<bodhi-api-token>`) — likewise for Gemini SDKs.

But the **value** of those headers is always a Bodhi token. Bodhi does the rewrite to the upstream provider's expected header on the way out.

If you need to mint a token, see [API Tokens](/docs/features/auth/api-tokens). If you need to understand which scope your token needs, see [Auth and Roles](/docs/concepts/auth-and-roles).

## Source of truth: the embedded Swagger UI

Every running Bodhi instance ships an interactive OpenAPI explorer:

```
http://<your-bodhi-instance>/swagger-ui
```

For a default local install that's `http://localhost:1135/swagger-ui`. The page is generated from the live `openapi.json` and stays in sync with the binary you're running, so it always reflects the exact request/response shapes, query parameters, and error envelopes that your version supports.

The pages in this section deliberately stop at "what to send and what comes back conceptually" — every per-format page below ends with a **Full schema** pointer back to Swagger UI. If you find yourself wondering about a field that isn't documented here, it's in Swagger UI.

You can also open Swagger UI from the Bodhi app menu (**API Documentation**). See [OpenAPI Reference](/docs/developer/openapi-reference) for more on the spec generation pipeline.

## Where to next

Pick the page that matches the SDK or wire format you already have:

- **[OpenAI Chat Completions](/docs/api-compatibility/openai-chat-completions)** — `/v1/chat/completions`. Streaming, tools, the most common path.
- **[OpenAI Responses](/docs/api-compatibility/openai-responses)** — `/v1/responses`. Async polling for reasoning models. **Pure pass-through to the upstream provider** — read the gotchas first.
- **[OpenAI Embeddings](/docs/api-compatibility/openai-embeddings)** — `/v1/embeddings`. RAG, retrieval, classification.
- **[Anthropic Messages](/docs/api-compatibility/anthropic-messages)** — `/anthropic/v1/messages`. Includes Anthropic-OAuth (no API key) and `x-api-key` header rewriting.
- **[Gemini](/docs/api-compatibility/gemini)** — `/v1beta/*`. `x-goog-api-key` and `?key=` handling.
- **[Ollama](/docs/api-compatibility/ollama)** — `/api/*`. Deprecated; kept for legacy clients.
- **[MCP Proxy](/docs/api-compatibility/mcp-proxy)** — `/bodhi/v1/apps/mcps/{id}/mcp`. Use Bodhi as an authenticated MCP front door.
- **[Error Format](/docs/api-compatibility/error-format)** — the two error envelopes (Bodhi-native vs OpenAI-shaped) and how to tell which one you'll see.

If you're new to the `model` field's resolution rules, the [Models overview](/docs/features/models/overview) explains how a single name can resolve to either a local GGUF file or a remote API model — transparently to the client.
