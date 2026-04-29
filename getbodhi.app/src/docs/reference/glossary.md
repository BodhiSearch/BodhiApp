---
title: 'Glossary'
description: 'Alphabetical glossary of terms used across the Bodhi App docs'
order: 4
---

# Glossary

Quick definitions for terms used throughout the docs. Each entry links to the canonical page where applicable.

**Admin** — The highest assignable role. Full system access including app settings and Admin role assignment. See [Roles and scopes](/docs/reference/roles-and-scopes).

**Alias** — A named handle for an inference target. Bodhi has three alias kinds — User aliases (a GGUF file plus inference parameters), Model aliases (auto-generated from a model file), and API model aliases (a remote provider plus credentials). See [Models, aliases, files](/docs/concepts/models-aliases-files).

**API model** — An alias backed by a remote provider (OpenAI, Anthropic, Gemini, Groq, OpenRouter, or any OpenAI-compatible base URL) rather than a local GGUF file. Uses the same `/v1/chat/completions` interface. See [API models](/docs/features/models/api-models).

**API token** — A long-lived credential of the form `bodhiapp_<random>.<client_id>` used to authenticate programmatic requests. Carries a scope (`User` or `PowerUser`). See [API tokens](/docs/features/auth/api-tokens).

**App access request** — A flow where a third-party app requests access to specific MCP instances and APIs on a user's Bodhi. The user reviews and approves resource-by-resource. See [App access management](/docs/features/auth/app-access-management).

**Bodhi-native envelope** — Bodhi's own error JSON shape, used by `/bodhi/v1/*`. A strict superset of the OpenAI error envelope. See [Error format](/docs/api-compatibility/error-format).

**`BODHI_HOME`** — Root directory for all Bodhi state — settings, databases, logs, and secrets. Defaults to `~/.bodhi` on desktop and `/data/bodhi` in the Docker images. See [Environment variables](/docs/reference/env-vars).

**DCR (Dynamic Client Registration)** — RFC 7591 flow that lets Bodhi register itself as an OAuth client with an MCP server at connect time, with no manual setup. Requires the server to expose RFC 8414 metadata. See [MCP auth methods](/docs/features/mcps/auth-methods).

**Embedding** — A fixed-length vector representing the semantic content of a text. Bodhi exposes embeddings via `/v1/embeddings` for both local models and configured API models. See [OpenAI embeddings](/docs/api-compatibility/openai-embeddings).

**External app** — An app that has been granted resource consent through the app access request flow. Receives an OAuth token tagged with the specific MCPs and API models it can use. See [App access management](/docs/features/auth/app-access-management).

**GGUF** — The on-disk file format used by llama.cpp for quantized model weights. Bodhi downloads GGUF files from HuggingFace and loads them into the embedded llama.cpp server. See [Model files](/docs/features/models/model-files).

**Guest** — Authenticated user without an assigned role. Cannot reach feature pages until an Admin or Manager approves their access request. See [User access requests](/docs/features/auth/user-access-requests).

**HuggingFace cache** — The directory under `$HF_HOME/hub` where downloaded GGUF files live. Shared across local tools that respect `HF_HOME`. See [Model downloads](/docs/features/models/model-downloads).

**llama.cpp** — The embedded C++ inference engine that runs local GGUF models. Bodhi spawns a llama.cpp server subprocess per loaded model. See [Inference stack](/docs/advanced/inference-stack).

**Manager** — Role between PowerUser and Admin. Can approve user access requests and manage user roles up to and including Manager. Cannot promote/demote Admins or edit app settings. See [Roles and scopes](/docs/reference/roles-and-scopes).

**MCP (Model Context Protocol)** — Open protocol for tool servers that LLMs can call mid-conversation. Bodhi connects as a client to MCP servers and exposes their tools to the chat UI and the API. See [MCP overview](/docs/concepts/mcp-overview).

**MCP instance** — A user-scoped configured connection to an MCP server, including auth credentials and a per-instance tool whitelist. See [MCP setup](/docs/features/mcps/setup).

**MCP playground** — UI at `/ui/mcps/playground/` for invoking individual MCP tools with arbitrary inputs to inspect their responses. See [MCP playground](/docs/features/mcps/playground).

**MCP proxy** — Endpoint at `/bodhi/v1/apps/mcps/{id}/mcp` that lets external apps route their MCP traffic through Bodhi's auth gateway. See [MCP proxy](/docs/api-compatibility/mcp-proxy).

**MCP server (catalog)** — An admin-curated, workspace-wide MCP server template that users can spin instances from in one click. Stored in the MCP Store. See [Pre-registered MCP servers](/docs/features/mcps/pre-registered-servers).

**MCP store** — Admin-only catalog UI for adding, editing, and removing pre-registered MCP servers. See [Pre-registered MCP servers](/docs/features/mcps/pre-registered-servers).

**Model file** — A downloaded GGUF artifact stored in the HuggingFace cache. Identified by `(repo, filename)`. See [Model files](/docs/features/models/model-files).

**OAuth2 PKCE** — Proof Key for Code Exchange flow used for browser logins to Bodhi's identity provider (`id.getbodhi.app` by default). No password is stored in Bodhi itself. See [Auth and roles](/docs/concepts/auth-and-roles).

**PowerUser** — Role above User. Can download models, create aliases, configure API models, mint API tokens, and register external apps. Cannot approve user access requests or manage roles. See [Roles and scopes](/docs/reference/roles-and-scopes).

**Quantization** — Lossy compression of model weights from float to int. GGUF files come in many quants (Q4_K_M, Q5_K_S, Q8_0, etc.) trading quality for size and speed. See [Inference stack](/docs/advanced/inference-stack).

**Refresh token** — OAuth token used to obtain a new access token without re-authenticating. Bodhi handles refresh transparently for sessions and stored MCP credentials.

**ResourceRole** — Bodhi's term for the assignable role on a session: `Anonymous`, `Guest`, `User`, `PowerUser`, `Manager`, `Admin`. See [Roles and scopes](/docs/reference/roles-and-scopes).

**Resource consent** — Per-resource grant given by a user when approving an app access request. The app can only act on resources it was granted (specific MCP instances, specific API models). See [App access management](/docs/features/auth/app-access-management).

**Role** — One of the assignable capability tiers on a Bodhi user. See [Auth and roles](/docs/concepts/auth-and-roles) and [Roles and scopes](/docs/reference/roles-and-scopes).

**Scope** — A capability ceiling carried by an API token (`User` or `PowerUser`). The token's effective capability is `min(user_role, scope)`. See [Roles and scopes](/docs/reference/roles-and-scopes).

**Session cookie** — HttpOnly, Secure, SameSite cookie set after browser OAuth login. Used by the built-in UI and refreshed silently. See [Auth and roles](/docs/concepts/auth-and-roles).

**Setup wizard** — One-time first-run flow that resolves `BODHI_HOME`, picks the inference variant, and seats the first Admin. See [Install](/docs/install).

**System tray** — Tauri desktop's persistent icon for launching the UI, opening the home directory, and quitting. See [Desktop deployment](/docs/deployment/desktop).

**Tauri** — The cross-platform shell that wraps Bodhi's HTTP server and embedded UI into a single native desktop app. See [Desktop deployment](/docs/deployment/desktop).

**Tool whitelist** — Per-MCP-instance allow-list of tool names that can actually be invoked from chat or the proxy. Tools not on the whitelist are hidden. See [MCP setup](/docs/features/mcps/setup).

**User** — The lowest assignable role. Can chat, call `/v1/chat/completions`, list models, and manage their own MCP instances. See [Roles and scopes](/docs/reference/roles-and-scopes).

**User access request** — A flow where a newly-logged-in user requests a role from an Admin or Manager. See [User access requests](/docs/features/auth/user-access-requests).

## Related

- [Concepts overview](/docs/concepts/overview) — narrative version of the core terms.
- [Environment variables](/docs/reference/env-vars), [Settings precedence](/docs/reference/settings), [Roles and scopes](/docs/reference/roles-and-scopes), [Error codes](/docs/reference/error-codes) — the rest of the reference tier.
