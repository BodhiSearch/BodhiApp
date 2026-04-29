---
title: "What's New"
description: 'Highlights of the features Bodhi App shipped in Q1 2026 — Anthropic API, Responses API, MCP proxy, agentic chat, and more'
order: 2
---

# What's New

A guided tour of the features that landed in Bodhi App during Q1 2026 — grouped by theme, leading with the most user-impactful. This is _not_ a full changelog (see [GitHub Releases](https://github.com/BodhiSearch/BodhiApp/releases) for that) — it's a quick orientation if you're upgrading from a 2025 build.

## New API surfaces

### Anthropic Messages API

You can now point any Anthropic SDK at Bodhi. The `/anthropic/v1/messages` and `/anthropic/v1/models` endpoints accept the native Anthropic wire format (including `x-api-key`, streaming SSE, tool use, system blocks). Use the official `anthropic` Python or TypeScript SDK with `base_url` set to your Bodhi instance — no code changes from a real Claude integration. See [Anthropic Messages](/docs/api-compatibility/anthropic-messages).

### Anthropic OAuth — sign in with Claude

Configure an Anthropic API model with the new `AnthropicOAuth` provider type to authenticate using a Claude.ai or Anthropic Console account directly — no API key required. Bodhi runs the full OAuth flow, stores refresh tokens, and surfaces both the model and credentials to the chat UI. See [Anthropic OAuth](/docs/features/models/anthropic-oauth).

### OpenAI Responses API

`/v1/responses` implements OpenAI's async-polling Responses interface — designed for long-running, reasoning-heavy, or background workloads. Compatible with the `openai` Python SDK's `responses.create()`, `responses.retrieve()`, and `responses.cancel()`. See [OpenAI Responses](/docs/api-compatibility/openai-responses).

### Gemini compatibility

`/v1beta/*` accepts Google's native Gemini wire format with `x-goog-api-key` (or `?key=`) auth. Drop in `google-genai` and switch the base URL to talk to local models, configured API providers, or anything Bodhi exposes. See [Gemini](/docs/api-compatibility/gemini).

### MCP proxy

External apps can now route their own MCP traffic through Bodhi's auth gateway via `/bodhi/v1/apps/mcps/{id}/mcp`. The proxy enforces resource consent, applies the per-instance tool whitelist, and forwards the rest verbatim — letting third-party agents reuse the credentials a user already trusted Bodhi with. See [MCP proxy](/docs/api-compatibility/mcp-proxy).

## MCP improvements

### Unified MCP auth — three methods, one workflow

Connecting to an MCP server now offers three auth methods picked from a single dropdown:

- **Header** — static API key or bearer token sent on every request.
- **OAuth (Preregistered)** — for servers with a fixed client_id you've registered manually.
- **OAuth (Dynamic Client Registration)** — RFC 7591/8414 — Bodhi registers itself at connect time, no manual setup.

See [MCP auth methods](/docs/features/mcps/auth-methods).

### MCP store (admin-curated catalog)

Admins maintain a workspace-wide catalog of pre-registered MCP servers under `/ui/mcps/servers/`. Users see them on the MCP page and spin up an instance in one click — no manual URL or auth config. See [Pre-registered MCP servers](/docs/features/mcps/pre-registered-servers).

### MCP playground

The new playground at `/ui/mcps/playground/` lets you invoke any whitelisted MCP tool with arbitrary inputs and inspect the raw response. Useful for confirming an MCP works before adding it to a chat. See [MCP playground](/docs/features/mcps/playground).

### Agentic chat with tool calling

The chat UI now invokes MCP tools mid-conversation for any model that supports tool calling. Tool calls and results render inline, multi-step plans run automatically, and you can pick which MCPs are active per-conversation from the chat header. See [Tool calling in chat](/docs/features/chat/tool-calling).

## Security and access

### Security hardening

- **Content Security Policy** — strict CSP across the UI, with self-hosted fonts and no inline scripts (except `unsafe-eval` for the MCP SDK).
- **AuthZ middleware** — every route revalidates `(user_role, token_scope)` against the endpoint's requirement, eliminating cross-scope leak vectors.
- **SSRF guards** — outbound calls to user-supplied URLs (API model base URLs, MCP servers) are validated against an allowlist.

### App access requests with resource consent

Third-party apps now go through a consent UX modeled on OAuth scopes — they request _specific_ MCP instances and API model aliases, the user reviews each resource individually before approving. Approved apps get an `ExternalApp` token bound to exactly those resources. See [App access management](/docs/features/auth/app-access-management).

### User access request flow

New users requesting access can pick a desired role; an Admin or Manager reviews and chooses the actual role to assign. Approval invalidates the requester's session so they re-login with the new role. See [User access requests](/docs/features/auth/user-access-requests).

## UI and developer experience

### TanStack Router migration

The frontend moved to TanStack Router with file-based routing — faster client-side navigation, type-safe links, and clean integration with TanStack Query v5 for data fetching.

### Embedded Swagger UI

The full OpenAPI spec is now mounted at `/swagger-ui` on every running instance. The docs reference layer points to it instead of duplicating endpoint schemas. See [API reference](/docs/developer/openapi-reference).

### Browser extension

The new Bodhi browser extension exposes authenticated endpoints to any webpage — drop AI capabilities into any site you visit without per-tab token plumbing. Currently Chrome. See [Browser extension](/docs/developer/browser-extension).

## Deployment

### Broader Docker variant matrix

CPU (AMD64 + ARM64), CUDA, ROCm, Vulkan, MUSA, Intel, and CANN — seven variants covering most consumer and datacenter accelerators. Pick the one matching your hardware in [Docker deployment](/docs/deployment/docker).

### Tauri desktop polish

Tray icon, autostart toggle, native open-in-browser, and a cleaner `~/.bodhi` layout for settings, logs, and downloaded models. See [Desktop deployment](/docs/deployment/desktop).

## Where to go next

- New users — start at [Introduction](/docs/intro), then [Install](/docs/install).
- Upgrading from a 2025 build — skim [Concepts overview](/docs/concepts/overview) and the [API compatibility overview](/docs/api-compatibility/overview).
- Self-hosting — [Deployment overview](/docs/deployment/overview) for the desktop-vs-Docker decision.
- Building integrations — [Developer getting started](/docs/developer/getting-started).
