---
title: 'API Compatibility'
description: 'Why Bodhi exposes OpenAI, Anthropic, Gemini, and Ollama wire formats simultaneously — and what that means for your client'
order: 3
---

# API Compatibility

Bodhi exposes the same underlying inference layer through several wire formats at once. Point an OpenAI SDK at it, an Anthropic SDK at it, a Gemini SDK at it, and an Ollama client at it — they all work simultaneously, talking to the same models with the same auth.

This page explains the mental model. The functional, per-format usage notes (with curl examples and gotchas) live under [API Compatibility](/docs/api-compatibility/overview).

## The endpoint map

```
   /v1/chat/completions       ──► OpenAI Chat Completions (streaming, tools)
   /v1/responses              ──► OpenAI Responses API (async polling, reasoning)
   /v1/embeddings             ──► OpenAI Embeddings
   /v1/models                 ──► OpenAI model listing (combined catalog)
   /api/*                     ──► Ollama (deprecated — kept for legacy clients)
   /anthropic/v1/messages     ──► Anthropic Messages
   /anthropic/v1/models       ──► Anthropic model listing
   /v1beta/models/...         ──► Google Gemini (generateContent, embedContent, etc.)
   /bodhi/v1/apps/mcps/{id}/mcp  ──► MCP proxy (forward MCP traffic via Bodhi's auth)
```

All of these resolve to the **same** unified inference layer. A request to `/v1/chat/completions` and a request to `/anthropic/v1/messages` for the same underlying model produce equivalent answers — only the wire format changes.

## Why this matters

Most teams have an existing AI integration. It might be:

- Built directly against OpenAI's `chat.completions.create` — works as-is by changing `base_url`.
- Built against Anthropic's `messages.create` — works as-is by changing `base_url` and the auth header.
- Built against Google's GenAI SDK — works as-is for the `/v1beta/*` surface.
- An older Ollama integration — still works (with the caveat that we'll eventually deprecate this).

You can adopt Bodhi without rewriting a single client. Switch the base URL, swap the API key for a Bodhi API token, and you're done.

The same property holds in reverse: build your app against Bodhi's OpenAI-compatible endpoints, and it remains portable to OpenAI itself, OpenRouter, Groq, Together AI, or any OpenAI-shaped provider.

## How auth is unified

Every native cloud provider has its own auth header convention:

- OpenAI: `Authorization: Bearer <key>`
- Anthropic: `x-api-key: <key>`
- Gemini: `x-goog-api-key: <key>` (or `?key=<key>` query param)

Bodhi normalizes all of this to **one** scheme at the gateway: `Authorization: Bearer <bodhi-api-token-or-session-cookie>`. When Bodhi proxies to a remote provider, it rewrites headers to the provider's expected format using the credentials you stored in the API-model record.

In other words: **even when you call `/anthropic/v1/messages` or `/v1beta/*`, you send a Bodhi Bearer token, not the raw provider key.** Bodhi holds the provider keys; clients hold Bodhi tokens.

This gives you, for free:

- A single auth surface for clients regardless of which compat layer they use.
- Per-user API tokens with scopes (User, PowerUser) that you can rotate or revoke.
- Audit and rate-limiting at one chokepoint.
- Anthropic OAuth: clients never see the OAuth token bundle — Bodhi refreshes it server-side.

## Local vs remote, transparent to the client

When a request arrives, Bodhi resolves the `model` field against its catalog:

- If it matches a **local alias**, the request runs against llama.cpp and the response is reshaped into the wire format you asked for.
- If it matches an **API model**, the request is forwarded to the configured remote provider with the right headers.

The client doesn't know — and doesn't need to — which path was taken. You can swap a local alias for an API-model alias (or vice versa) by changing the `model` value alone.

## The full schema lives in Swagger UI

This documentation is **functional and narrative** — it explains how to use each compat layer, what the gotchas are, and gives copy-paste curl examples. It deliberately does not duplicate every request/response field, because the running Bodhi instance ships an embedded Swagger UI at:

```
https://<your-bodhi-host>/swagger-ui
```

That page is generated from the live OpenAPI spec and is always the source of truth for parameters, response shapes, and error envelopes.

## Where to go next

The per-format pages (forward references — they land in a later phase) cover the things you'll trip on:

- `/docs/api-compatibility/overview` — entry point with the embedded Swagger link.
- `/docs/api-compatibility/openai-chat-completions` — streaming, tools, gotchas.
- `/docs/api-compatibility/openai-responses` — the async-polling pattern for reasoning models.
- `/docs/api-compatibility/openai-embeddings` — embeddings for RAG.
- `/docs/api-compatibility/anthropic-messages` — `x-api-key` rewriting, tool use schema.
- `/docs/api-compatibility/gemini` — `x-goog-api-key` and `?key=` handling.
- `/docs/api-compatibility/ollama` — what's supported, what's deprecated.
- `/docs/api-compatibility/mcp-proxy` — using Bodhi as an authenticated MCP front door.
- `/docs/api-compatibility/error-format` — the two error envelopes you'll see.

Or step back to the broader picture: **[Auth and Roles](/docs/concepts/auth-and-roles)** explains the token model that ties all of these endpoints together.
