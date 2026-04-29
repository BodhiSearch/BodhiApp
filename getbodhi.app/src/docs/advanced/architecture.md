---
title: 'Architecture'
description: 'How a request travels through Bodhi App: from the wire to the inference engine and back'
order: 0
---

# Architecture

This page describes Bodhi App's runtime architecture from a self-hoster's perspective. The audience is operators who want to understand _what happens between port 1135 and the model_ — without reading source code. If you only need a mental model of _what_ Bodhi does, [Concepts → Overview](/docs/concepts/overview) is the better starting point.

## The big picture

```
                ┌──────────────────────────────────────────────────┐
                │                    Bodhi App                     │
                │                                                  │
   client ────► │  Reverse proxy (your nginx/Caddy/cloud LB)       │
                │           │                                      │
                │           ▼                                      │
                │  HTTP server  ──►  Auth middleware stack         │
                │                          │                       │
                │                          ▼                       │
                │                    Route handler                 │
                │                          │                       │
                │                          ▼                       │
                │              Service layer (business logic)      │
                │              ┌─────────┴───────────┐             │
                │              ▼                     ▼             │
                │       llama.cpp process     remote provider      │
                │       (local GGUF)          (OpenAI/Anthropic/   │
                │                              Gemini/Groq/...)    │
                └──────────────────────────────────────────────────┘
```

Three observations matter:

- **Single OS process.** Bodhi App is one binary. There is no message broker, no worker pool, no internal RPC. Local inference runs as a _child_ process of Bodhi (one llama-server per active alias), not as a separate service you deploy.
- **Auth is a chain, not a switch.** Every request walks through several middleware steps in order. Most surprising errors at the gateway come from understanding which step rejected the request.
- **The route handler picks the destination.** Whether your `/v1/chat/completions` ends up at llama.cpp or at Anthropic is decided after auth, by resolving the `model` field in the request body against your catalog.

## The auth middleware stack

Every request that reaches a protected endpoint passes through a chain of small steps. Each step has a single job; if any step rejects, the request is denied with a structured error envelope. The chain varies slightly by route group, but the pieces are:

- **Token / session resolution.** Reads the `Authorization: Bearer <token>` header (or the session cookie set by the built-in UI). Validates the token's hash against the database, looks up the user, attaches the resolved identity to the request. Tokens with stripped or revoked scope are rejected here.
- **Per-format header rewriting.** Routes that imitate a third-party API have their own pre-step. The Anthropic compat layer accepts `x-api-key`, the Gemini compat layer accepts `x-goog-api-key` (or `?key=...`); both accept Bearer too. The header-rewriting step normalises these into the same Bearer-shaped identity used by the rest of the chain. **You always send a Bodhi token; Bodhi rewrites the upstream provider header server-side when proxying.**
- **External-app validator.** Routes under `/bodhi/v1/apps/...` run a separate validator that looks at the calling app's registration and the resource consent it was granted. This is what gates an external app's MCP-proxy or Bodhi-API call without giving it the full power of a user session.
- **MCP-proxy validator.** The MCP proxy path (`/bodhi/v1/apps/mcps/{id}/mcp`) uses a tighter validator that ties the calling app's identity to the specific MCP instance being proxied.

You don't configure these directly — they're applied automatically based on the route. The point is to know where to look when a request is unexpectedly rejected: a `401` at this layer means token resolution failed; a `403` here means the resolved identity didn't have the required role or scope.

For the role/scope matrix, see [Reference → Roles and Scopes](/docs/reference/roles-and-scopes). For error envelope shapes, see [API Compatibility → Error Format](/docs/api-compatibility/error-format).

## Three request walkthroughs

### 1. `/v1/chat/completions` against a local alias

A developer's app posts an OpenAI-shaped request:

```bash
POST /v1/chat/completions
Authorization: Bearer bodhiapp_...
{ "model": "llama3:8b-instruct", "messages": [...], "stream": true }
```

1. Reverse proxy terminates TLS, forwards to Bodhi.
2. Token resolution validates `bodhiapp_...`, attaches the user identity.
3. The chat handler resolves `llama3:8b-instruct` against the catalog. It matches a **local model alias** — a YAML record bundling a GGUF file with default inference parameters.
4. The inference layer checks whether a llama-server child process is already running for that alias. If yes, the request is forwarded to it. If no, a new llama-server is spawned with the alias's parameters and the GGUF file resolved from the HuggingFace cache.
5. llama-server streams tokens back as Server-Sent Events. The chat handler relays the stream verbatim to the client (rewriting only what's needed to match the OpenAI wire format).
6. After the configured idle timeout (`BODHI_KEEP_ALIVE_SECS`, default 300s), the llama-server process is shut down to free RAM/VRAM.

Cold starts are dominated by GGUF model loading. Warm calls reuse the running process.

### 2. `/anthropic/v1/messages` against a remote provider

A team using Claude SDKs points `ANTHROPIC_BASE_URL` at Bodhi:

```bash
POST /anthropic/v1/messages
x-api-key: bodhiapp_...
{ "model": "claude-3-5-sonnet-20241022", "messages": [...] }
```

1. The Anthropic compat layer accepts `x-api-key`. The header rewriter normalises this into Bodhi's internal Bearer identity.
2. Token resolution validates the Bodhi token, attaches the user.
3. The Anthropic handler resolves `claude-3-5-sonnet-20241022` against the catalog. It matches an **API model** — a configured remote provider (here, Anthropic) with a stored API key.
4. The proxy fetches the encrypted provider credential from the database, decrypts it in memory using `BODHI_ENCRYPTION_KEY`, and rewrites the request: outbound `x-api-key` becomes the real Anthropic key (or the Anthropic-OAuth access token, refreshed if needed).
5. Bodhi forwards the request to `https://api.anthropic.com/v1/messages`, streaming the SSE response back to the client unchanged.

The client never sees the upstream key. The key never leaves Bodhi's process unencrypted on disk.

### 3. `/bodhi/v1/apps/mcps/{id}/mcp` from a third-party app

A registered external app calls an MCP tool through Bodhi's authenticated proxy:

```bash
POST /bodhi/v1/apps/mcps/01J.../mcp
Authorization: Bearer <external-app-token>
{ "jsonrpc": "2.0", "method": "tools/call", ... }
```

1. The external-app validator confirms the calling app is registered and was granted resource consent for this user.
2. The MCP-proxy validator confirms the MCP instance ID belongs to the same user.
3. The MCP service resolves the upstream MCP server URL plus its auth-config (header / preregistered OAuth2 / DCR OAuth2), refreshes the OAuth token if needed, and forwards the JSON-RPC body upstream.
4. The response streams back to the calling app.

This lets external apps speak MCP without holding any of the upstream MCP servers' credentials. See [API Compatibility → MCP Proxy](/docs/api-compatibility/mcp-proxy) for the wire-level detail and [Concepts → MCP Overview](/docs/concepts/mcp-overview) for the model.

## Where data lives

Bodhi keeps four categories of state, each in a different place:

| Data                                                                                  | Location                                                     | Notes                                             |
| ------------------------------------------------------------------------------------- | ------------------------------------------------------------ | ------------------------------------------------- |
| **Sessions** (browser cookies)                                                        | Session DB (SQLite by default; see `BODHI_SESSION_DB_URL`)   | Used only by the built-in UI                      |
| **App data** (users, tokens, API models, MCP configs, access requests, download jobs) | App DB (SQLite by default; see `BODHI_APP_DB_URL`)           | All long-lived state                              |
| **GGUF model files**                                                                  | HuggingFace cache (`HF_HOME`, default `$BODHI_HOME/hf_home`) | Standard HF layout                                |
| **Model aliases**                                                                     | YAML files under `$BODHI_HOME/aliases/`                      | Edited via the UI or by hand                      |
| **Encrypted credentials** (API model keys, MCP OAuth client secrets/tokens)           | App DB, encrypted at rest                                    | Master key from `BODHI_ENCRYPTION_KEY`            |
| **Logs**                                                                              | `$BODHI_HOME/logs/` (rotated daily)                          | See [Observability](/docs/advanced/observability) |

For a full env-var matrix see [Reference → Environment Variables](/docs/reference/env-vars). For settings precedence (DB > YAML > Env > Default) see [Reference → Settings](/docs/reference/settings).

## What's outside the process

A few things deliberately _aren't_ Bodhi's job:

- **TLS termination and rate limiting.** Both belong at the reverse proxy. The app speaks plain HTTP internally and trusts the proxy for transport security and per-IP throttling. See [Deployment → Reverse Proxy](/docs/deployment/reverse-proxy).
- **Identity provider.** Authentication is OAuth2 PKCE against an external identity provider (default: a managed Keycloak realm). Bodhi never stores user passwords.
- **GGUF download infrastructure.** Models live on HuggingFace. Bodhi schedules downloads, but the bytes come from huggingface.co.

## Where to read next

- [Security Model](/docs/advanced/security-model) — the public-safe summary of Bodhi's security posture.
- [Inference Stack](/docs/advanced/inference-stack) — how the llama.cpp child processes are configured.
- [Performance Tuning](/docs/advanced/performance-tuning) — variant × hardware decisions.
- [Observability](/docs/advanced/observability) — logs, the queue, and the settings page.
