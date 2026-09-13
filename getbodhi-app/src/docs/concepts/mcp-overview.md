---
title: 'MCP Overview'
description: 'Model Context Protocol in Bodhi: per-user instances, three auth methods, the playground, and the MCP proxy for external apps'
order: 5
---

# MCP Overview

**Model Context Protocol (MCP)** is an open protocol that lets a model talk to external tools — file systems, databases, APIs, search engines — through a standard request/response shape. An MCP server publishes a list of tools; a client (the model, in our case) calls them; the results flow back into the conversation.

Bodhi takes the protocol's "client" responsibilities and turns them into a managed, multi-user feature: server registry, per-user instances with whitelisting, a built-in tool playground, and a proxy so external apps can route their own MCP traffic through Bodhi's auth gate.

This page is the orientation. Per-feature workflows live under `/docs/features/mcps/*` (forward references — they land in a later phase).

## How Bodhi structures MCP

```
   ┌──────────────────────────────────────────────────────────────┐
   │ Server registry (admin-curated, optional)                    │
   │   Pre-registered server templates — name, URL, auth scheme,  │
   │   client_id for OAuth — published to the workspace by Admin. │
   └────────────────────┬─────────────────────────────────────────┘
                        │ instantiated by user
                        ▼
   ┌──────────────────────────────────────────────────────────────┐
   │ Per-user MCP instance                                        │
   │   Connection state, tool whitelist, OAuth tokens (if any),   │
   │   stored per user. Owned by the user; only they can use it.  │
   └────────────────────┬─────────────────────────────────────────┘
                        │ exposes whitelisted tools to
                        ▼
   ┌──────────────────────────────────────────────────────────────┐
   │ Bodhi chat / playground / external app                       │
   │   Calls tools via the model loop or directly.                │
   └──────────────────────────────────────────────────────────────┘
```

Three things to internalize:

1. **An admin can publish servers; users instantiate them.** A pre-registered "GitHub MCP" server in the catalog isn't usable yet — each user must create their own MCP instance from it (which, for OAuth servers, also means each user does their own OAuth authorization).
2. **Tools are whitelisted per instance.** When you connect to an MCP server, Bodhi fetches its tool list. You decide which ones are allowed to fire in your chats. Non-whitelisted tools are blocked even if the model tries to call them.
3. **Instances are user-scoped.** Your MCP setup doesn't leak to other users on the same Bodhi instance — credentials and tokens are isolated.

## Three authentication methods

Different MCP servers expect different auth shapes. Bodhi supports the three you'll encounter in the wild:

### Header-based (static API key or bearer token)

The simplest case. The server expects a header on every request, and you paste the value once when creating the instance. Common for self-hosted or developer-tool MCPs.

- _Use this if:_ the MCP server publishes "send `Authorization: Bearer ...`" or "send `X-API-Key: ...`" in its docs.

### OAuth2 with preregistered client

The MCP server publishes a `client_id` (and optionally `client_secret`) to use with its OAuth2 endpoint. Admins enter those once into the pre-registered server catalog; users then click "Authorize" on their MCP instance, complete OAuth at the server, and Bodhi stores the token bundle.

- _Use this if:_ the MCP server's docs say "register at our developer portal, get a client_id, configure your client with these endpoints."

### OAuth2 with Dynamic Client Registration (DCR)

The MCP server implements RFC 7591 (dynamic client registration) and RFC 8414 (server metadata discovery). Bodhi auto-discovers the OAuth endpoints, registers itself as a client on the fly, and runs the same authorization flow without any preregistered IDs.

- _Use this if:_ the MCP server advertises `.well-known/oauth-authorization-server` and supports DCR. (You'll know — the server's connect flow will be a one-click "Connect with OAuth" experience.)

For a "which one do I pick" decision page, see `/docs/features/mcps/auth-methods` (forward reference).

## The MCP playground

Bodhi ships with a built-in tool playground at `/ui/mcps/<id>/playground`. For a given MCP instance, you see:

- The full tool list (whitelisted and not).
- Each tool's input schema and description.
- A form-based runner — fill in the inputs, hit Execute, see the JSON response.

Use the playground to debug tool integrations, check whether a server is reachable, or sanity-check input schemas before letting the model invoke a tool.

## The MCP proxy (for external apps)

Bodhi exposes a special endpoint that lets external apps speak MCP **through** Bodhi:

```
POST /bodhi/v1/apps/mcps/{instance_id}/mcp
```

When an external app calls this endpoint with a Bodhi-issued ExternalApp token (obtained via the app-access-request flow), Bodhi forwards the MCP traffic to the upstream server, applying the same whitelist and audit checks as if the user had called the tool from chat. This is how third-party agents, IDE plugins, and browser extensions can use MCPs without ever seeing the user's MCP credentials.

The MCP proxy preserves all of Bodhi's protections:

- Tool whitelist enforced.
- Per-tool authorization (the app must have been approved for this MCP instance during access request).
- Credentials never leave Bodhi.

For the curl/SDK examples, see `/docs/api-compatibility/mcp-proxy` (forward reference).

## Where to go next

Forward references — these pages land in a later phase:

- `/docs/features/mcps/overview` — UI tour of MCPs.
- `/docs/features/mcps/setup` — connecting your first MCP server.
- `/docs/features/mcps/auth-methods` — picking between Header / OAuth2 preregistered / DCR.
- `/docs/features/mcps/playground` — using the tool playground.
- `/docs/features/mcps/pre-registered-servers` — admin-curated server catalog.
- `/docs/features/mcps/usage` — using MCPs in chat.

That wraps the **Concepts** section. From here you can dive into **[Features](/docs/features)** to see these ideas applied page-by-page, or jump to **[API Compatibility](/docs/api-compatibility/overview)** if you're integrating from code.
