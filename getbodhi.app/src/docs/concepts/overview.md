---
title: 'Overview'
description: 'The mental model behind Bodhi App: one server, many compatibility layers, local + remote inference, MCP tools, role-based auth'
order: 0
---

# What is Bodhi App?

Bodhi App is a **single server** that sits between your AI client (chat UI, agent framework, IDE plugin, custom app) and the actual model that answers a request. Whether that model lives on your laptop or behind a cloud provider, Bodhi presents one consistent surface: one set of endpoints, one auth token, one place to manage models and tools.

This page is your one-stop mental model. The rest of the **Concepts** section drills into the five ideas that power the app — read them in order if this is your first time, or jump straight to whichever name in the sidebar speaks to your problem.

## The unified-gateway picture

```
            ┌────────────────────────────────────────────────────┐
            │  Bodhi App (single OS process)                     │
            │                                                    │
   client ──┼──► /v1/*           (OpenAI, Ollama-compat)         │
   client ──┼──► /v1/responses   (OpenAI Responses, async)       │
   client ──┼──► /anthropic/v1/* (Anthropic Messages + OAuth)    │
   client ──┼──► /v1beta/*       (Google Gemini)                 │
   client ──┼──► /api/*          (Ollama, deprecated)            │
   client ──┼──► /bodhi/v1/apps/mcps/{id}/mcp  (MCP proxy)       │
            │                                                    │
            │      ▼ unified inference layer                     │
            │   ┌──────────────┐    ┌─────────────────────────┐  │
            │   │ llama.cpp    │    │ remote provider proxy   │  │
            │   │ (local GGUF) │    │ (OpenAI/Anthropic/...)  │  │
            │   └──────────────┘    └─────────────────────────┘  │
            │                                                    │
            │      ▲ MCP integration                             │
            │   ┌──────────────────────────────────────────┐     │
            │   │ MCP servers (per-user instances + tools) │     │
            │   └──────────────────────────────────────────┘     │
            └────────────────────────────────────────────────────┘
```

Every request hits the same OAuth2 / API-token auth gate, the same role check, the same audit trail — regardless of whether it ends up running locally or being forwarded to a third-party API.

## The five ideas you need

Bodhi looks like a single product, but mastering it means understanding five separate (and orthogonal) concepts:

1. **[Deployment modes](/docs/concepts/deployment-modes)** — Where Bodhi runs (your laptop vs. a Docker container) shapes who manages it, where data lives, and which auth flow you'll use.
2. **[Models, aliases, and files](/docs/concepts/models-aliases-files)** — A "model" can mean three different things in Bodhi: a downloaded GGUF file, a named alias that bundles a file with parameters, or a remote API model. Don't confuse them.
3. **[API compatibility](/docs/concepts/api-compatibility)** — Bodhi serves multiple wire formats simultaneously. Learn which endpoint to use and why this matters for drop-in migration.
4. **[Auth and roles](/docs/concepts/auth-and-roles)** — Four roles (User, PowerUser, Manager, Admin), two token types (session cookies, API tokens), one OAuth2 PKCE flow.
5. **[MCP overview](/docs/concepts/mcp-overview)** — How Bodhi turns Model Context Protocol from a client-side concern into a managed, multi-user, multi-server tool registry — plus the MCP proxy for external apps.

## A typical request, end-to-end

To make this concrete, here's what happens when a chat client sends a message via the OpenAI-compatible endpoint:

1. The client posts to `POST /v1/chat/completions` with a Bearer token (or session cookie if it's the built-in chat UI).
2. Bodhi authenticates the token, looks up the user, and checks they have permission for chat (User role or higher).
3. The `model` field in the request is resolved against Bodhi's catalog:
   - If it matches a **local model alias**, Bodhi launches (or reuses) a llama.cpp process for that alias.
   - If it matches an **API model alias**, Bodhi forwards the request to the remote provider, translating headers and request shape as needed.
4. The response (streamed or whole) flows back through the same path. Tool calls are intercepted: if the chat session has MCP tools enabled, Bodhi resolves them against the user's MCP instances, executes the tool, and threads the result back into the conversation.
5. Usage is logged. The session continues.

The same shape applies to `/anthropic/v1/messages`, `/v1/responses`, and `/v1beta/*` — only the wire format differs. That's the whole point.

## Who uses what

| You are...                                                        | Most-used surface                        | Where to start                                        |
| ----------------------------------------------------------------- | ---------------------------------------- | ----------------------------------------------------- |
| End-user, just want to chat                                       | Built-in Chat UI at `/ui/chat/`          | [Features → Chat](/docs/features)                     |
| Developer building an app on Bodhi                                | Bodhi JS SDK + API tokens                | [Developer Guide](/docs/developer/getting-started)    |
| Developer porting an existing OpenAI/Anthropic/Gemini integration | API compatibility endpoints              | [API Compatibility](/docs/api-compatibility/overview) |
| Self-hoster                                                       | Docker variants, env vars, reverse proxy | [Docker Deployment](/docs/deployment/docker)          |
| Admin / team lead                                                 | Users, MCP store, API model catalog      | [Features → Auth](/docs/features)                     |

## What Bodhi is not

- **Not a model trainer.** Bodhi runs and proxies models — it doesn't fine-tune them.
- **Not a multi-tenant SaaS.** This documentation covers single-tenant deployment (Tauri desktop or Docker single-tenant). Multi-tenant deployment exists but is out of scope here.
- **Not opinionated about your stack.** Any OpenAI / Anthropic / Gemini SDK that lets you set a base URL works. So does `curl`.

Ready to dig in? The next page — **[Deployment Modes](/docs/concepts/deployment-modes)** — picks the deployment shape that fits your workflow.
