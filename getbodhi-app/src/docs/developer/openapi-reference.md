---
title: 'OpenAPI Reference'
description: 'Interactive Swagger UI, the per-format compat guides, endpoint prefixes, and CORS policy'
order: 256
---

# OpenAPI Reference

Bodhi ships an interactive, auto-generated OpenAPI explorer (Swagger UI) and a set of narrative guides for each compat layer. Together they are the source of truth for the API surface of your running instance.

- **Schemas, parameters, response shapes** — Swagger UI (`/swagger-ui`).
- **Auth, model resolution, streaming, header rewriting, gotchas, copy-paste examples** — the [API Compatibility](/docs/api-compatibility/overview) section.

This page is the orientation map between the two.

## Format-specific guides

Pick the page that matches the SDK or wire format you already speak. Each ends with a "Full schema" pointer back to Swagger UI.

- **[API Compatibility — Overview](/docs/api-compatibility/overview)** — endpoint map, the unified Bearer-token auth model, and how Bodhi normalizes provider-specific headers.
- **[OpenAI Chat Completions](/docs/api-compatibility/openai-chat-completions)** — `/v1/chat/completions`. Streaming, tool calling, and how `model` resolves to either a local alias or a remote API model.
- **[OpenAI Responses](/docs/api-compatibility/openai-responses)** — `/v1/responses`. Async polling for reasoning workloads.
- **[OpenAI Embeddings](/docs/api-compatibility/openai-embeddings)** — `/v1/embeddings`. RAG and retrieval.
- **[Anthropic Messages](/docs/api-compatibility/anthropic-messages)** — `/anthropic/v1/messages` and `/v1/messages`. `x-api-key` rewriting, `anthropic-*` header pass-through, and the Anthropic-OAuth path.
- **[Gemini](/docs/api-compatibility/gemini)** — `/v1beta/*`. `x-goog-api-key`, `?key=`, action dispatch, SSE streaming.
- **[Ollama](/docs/api-compatibility/ollama)** — `/api/*`. Deprecated, kept for legacy clients.
- **[MCP Proxy](/docs/api-compatibility/mcp-proxy)** — `/bodhi/v1/apps/mcps/{id}/mcp`. The authenticated MCP front door for third-party apps.
- **[Error Format](/docs/api-compatibility/error-format)** — the four error envelopes (Bodhi-native, OpenAI-style, Anthropic-style, Gemini-style) and how to tell them apart.

## Accessing Swagger UI

The interactive explorer is mounted at:

```
http://<your-bodhi-instance>/swagger-ui
```

For a default local install that's `http://localhost:1135/swagger-ui`. You can also open it from the Bodhi app menu (**API Documentation**).

The Swagger UI lets you:

- Browse endpoint descriptions, request/response schemas, and authentication requirements.
- Test endpoints interactively against your running instance.
- See the available authentication methods (session cookie and bearer token).

The Anthropic and Gemini compat surfaces are documented as separate specs mounted under `/api-docs/openapi-anthropic.json` and `/api-docs/openapi-gemini.json` so the Swagger picker can switch between them.

## Endpoint prefixes

Bodhi groups its API by prefix.

### `/v1/` — OpenAI-compatible endpoints

| Endpoint               | Method    | Description                                    |
| ---------------------- | --------- | ---------------------------------------------- |
| `/v1/chat/completions` | POST      | Chat completions (streaming and non-streaming) |
| `/v1/responses`        | POST, GET | Async polling Responses API for reasoning      |
| `/v1/embeddings`       | POST      | Generate text embeddings                       |
| `/v1/models`           | GET       | Combined catalog (local aliases + API models)  |

### `/anthropic/v1/` and `/v1/messages` — Anthropic-compatible endpoints

| Endpoint                    | Method | Description                              |
| --------------------------- | ------ | ---------------------------------------- |
| `/anthropic/v1/messages`    | POST   | Anthropic Messages (streaming, tool use) |
| `/v1/messages`              | POST   | Same handler as above; either path works |
| `/anthropic/v1/models`      | GET    | Anthropic-shaped catalog                 |
| `/anthropic/v1/models/{id}` | GET    | Anthropic-shaped model lookup            |

### `/v1beta/` — Gemini-compatible endpoints

| Endpoint                          | Method | Description                                                                                            |
| --------------------------------- | ------ | ------------------------------------------------------------------------------------------------------ |
| `/v1beta/models`                  | GET    | List Gemini-format models                                                                              |
| `/v1beta/models/{model}`          | GET    | Gemini model lookup                                                                                    |
| `/v1beta/models/{model}:{action}` | POST   | Action dispatch — `:generateContent`, `:streamGenerateContent`, `:embedContent`, `:batchEmbedContents` |

### `/api/` — Ollama-compatible endpoints (deprecated)

Kept for legacy clients. See [Ollama](/docs/api-compatibility/ollama) for the supported subset.

### `/bodhi/v1/` — Bodhi-specific endpoints

Bodhi-specific functionality lives under `/bodhi/v1/`. This includes user management, MCP CRUD and auth-config, model management, settings, tokens, access requests, and more. See Swagger UI for the full list.

### `/bodhi/v1/apps/` — External app endpoints

Third-party apps that have completed the [access request flow](/docs/developer/app-access-requests) use endpoints under `/bodhi/v1/apps/`. These have permissive CORS so a browser-based app on a different origin can call them.

| Endpoint                              | Method            | Description                                                                             |
| ------------------------------------- | ----------------- | --------------------------------------------------------------------------------------- |
| `/bodhi/v1/apps/request-access`       | POST              | Create an access request (unauthenticated)                                              |
| `/bodhi/v1/apps/access-requests/{id}` | GET               | Poll access request status                                                              |
| `/bodhi/v1/apps/mcps`                 | GET               | List MCP instances the app has access to                                                |
| `/bodhi/v1/apps/mcps/{id}`            | GET               | Get one MCP instance's metadata                                                         |
| `/bodhi/v1/apps/mcps/{id}/mcp`        | POST, GET, DELETE | MCP Streamable HTTP proxy — JSON-RPC over HTTP, with upstream auth injected server-side |

The proxy is a transparent MCP-protocol pass-through, not a REST per-tool surface. See [MCP Proxy](/docs/api-compatibility/mcp-proxy) for the JSON-RPC envelope shape and a worked example.

## CORS policy

CORS is applied per route group, not globally:

- **Session endpoints** (login, logout, OAuth callbacks) — restrictive CORS, same-origin only. Protects session-based authentication from cross-site attacks.
- **API and external app endpoints** (`/v1/*`, `/anthropic/v1/*`, `/v1beta/*`, `/api/*`, `/bodhi/v1/apps/*`) — permissive CORS. Enables third-party clients to call Bodhi from their own domain.

## Authentication

Two methods, both documented in detail under [API Compatibility — Overview](/docs/api-compatibility/overview):

- **Session cookie** — used by the Bodhi web UI and by browser flows like access-request review.
- **Bearer token** — `Authorization: Bearer <bodhi-token>`. Used by SDKs, scripts, and external apps. The Anthropic and Gemini compat layers also accept their native header (`x-api-key`, `x-goog-api-key`, `?key=`) as a transport convenience — the value is still a Bodhi token, not a raw provider key.

```bash
curl http://localhost:1135/v1/models \
  -H "Authorization: Bearer $BODHI_TOKEN"
```

To mint a token, see [API Tokens](/docs/features/auth/api-tokens).

## Quick start examples

### Chat completion

```bash
curl -X POST http://localhost:1135/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $BODHI_TOKEN" \
  -d '{
    "model": "your-model-alias",
    "messages": [
      {"role": "system", "content": "You are a helpful assistant."},
      {"role": "user", "content": "Hello!"}
    ]
  }'
```

### List models

```bash
curl http://localhost:1135/v1/models \
  -H "Authorization: Bearer $BODHI_TOKEN"
```

### Generate embeddings

```bash
curl -X POST http://localhost:1135/v1/embeddings \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $BODHI_TOKEN" \
  -d '{
    "model": "your-embedding-model",
    "input": "Text to generate embeddings for"
  }'
```

For Anthropic, Gemini, and MCP-proxy examples, see the per-format pages linked at the top.

## Keeping the spec current

The OpenAPI specification is auto-generated from the backend on every build. As the codebase evolves, Swagger UI always reflects the API surface of the binary you're running. The narrative compat guides under [API Compatibility](/docs/api-compatibility/overview) cover behaviour and gotchas; Swagger UI is the schema reference.
