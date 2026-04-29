---
title: 'MCP Proxy'
description: 'Use Bodhi as an authenticated MCP front door — the per-instance proxy at /bodhi/v1/apps/mcps/{id}/mcp, JSON-RPC over HTTP, and how upstream auth is hidden from your callers'
order: 7
---

# MCP Proxy

`/bodhi/v1/apps/mcps/{instance_id}/mcp` is a transparent reverse proxy in front of one of the user's MCP server instances. Third-party apps that have completed Bodhi's access-request flow can call MCP tools through Bodhi without ever seeing the upstream MCP server's credentials.

This is the front door for **building an app on top of someone else's MCP setup**. The user grants your app access to a specific MCP instance once; from then on your app calls Bodhi with a Bodhi token and Bodhi handles every credential rotation, OAuth refresh, and tool whitelist check on your behalf.

If you're new to MCP at Bodhi, start with [MCP Overview](/docs/features/mcps/overview).

## Who this is for

- A web or desktop app that wants to use a user's GitHub, Slack, Jira, or custom MCP server — without you, the app developer, ever holding those credentials.
- An automation script that needs to invoke MCP tools on behalf of a logged-in Bodhi user.
- A second AI agent (your product) that wants to delegate tool calls to a first AI agent (Bodhi's MCP-connected setup).

You do **not** call this endpoint to wire MCP servers to _your own_ Bodhi chat. For that, just configure the MCP instance in the UI and use it from chat or the API directly. See [MCP Setup](/docs/features/mcps/setup).

## How access works

This is a **per-instance** proxy, not a per-server one. Every MCP instance the user has configured gets its own URL:

```
/bodhi/v1/apps/mcps/{instance_id}/mcp
```

Where `{instance_id}` is the UUID of the user's specific MCP-instance row. To call this URL, your app must:

1. Register itself with Bodhi (via the [access request flow](/docs/developer/app-access-requests)).
2. Be granted access by the user, naming the specific MCP instance(s) it may use.
3. Authenticate with an OAuth token issued to your app for that user's tenant.

Bodhi's access-request flow is the consent step. The user can approve a subset of the instances you asked for, and they can revoke access at any time. See [App Access Management](/docs/features/auth/app-access-management) for the user-side view.

## What's hidden from your app

The upstream MCP server can be configured with any of the three supported auth methods — static header, OAuth2 with a pre-registered client, or OAuth2 with Dynamic Client Registration. **All three are transparent to the proxy caller.** Bodhi:

- Injects the appropriate auth header (`Authorization`, `x-api-key`, or whatever the MCP-instance config specifies) into every upstream request.
- Refreshes OAuth tokens server-side when they expire.
- Enforces the tool whitelist the user configured for that instance — the same whitelist that applies in chat and the playground. Tools not on the whitelist are unreachable through the proxy.

If the upstream MCP server's OAuth refresh fails (revoked grant, expired refresh token, server outage), Bodhi returns the upstream `401` or `403` to your app verbatim. Treat it as "the user needs to reconnect this MCP server" and surface that in your UI.

For the user-facing setup of these auth methods, see [MCP Auth Methods](/docs/features/mcps/auth-methods).

## The wire protocol — MCP over HTTP, not REST

This endpoint speaks the **MCP Streamable HTTP protocol**. It is not a REST endpoint with one route per tool. The body is a JSON-RPC 2.0 message — `initialize`, `tools/list`, `tools/call`, `prompts/list`, `resources/list`, and so on. The MCP session is identified by the `mcp-session-id` header, and the protocol version by `mcp-protocol-version`. Both are forwarded to and from the upstream server.

`POST` carries a JSON-RPC request and returns either a JSON response or an SSE stream of partial results. `GET` opens a long-lived SSE channel for server-initiated messages. `DELETE` ends the session.

In practice, you do not hand-craft these messages — you use an MCP client SDK and point it at Bodhi's URL.

## Auth at the gateway

```
Authorization: Bearer <oauth-token-issued-to-your-app>
```

This is an OAuth bearer issued to your app via the access-request flow, scoped to the user that approved you. It is _not_ a long-lived API token, and it is _not_ the same Bearer used by interactive Bodhi users.

## Example

End-to-end you'd use an MCP SDK, but here's a minimal `tools/list` over raw HTTP to show the shape:

```bash
curl -X POST http://localhost:1135/bodhi/v1/apps/mcps/$INSTANCE_ID/mcp \
  -H "Authorization: Bearer $APP_OAUTH_TOKEN" \
  -H "Content-Type: application/json" \
  -H "Accept: application/json, text/event-stream" \
  -H "mcp-protocol-version: 2025-03-26" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/list",
    "params": {}
  }'
```

The response is the upstream MCP server's `tools/list` result — already filtered down to the instance's configured tool whitelist. From there, `tools/call` invokes a tool the same way it would against the upstream MCP server directly.

For a worked example using an MCP TypeScript client SDK, see [Building Apps](/docs/developer/building-apps).

## What you don't have to think about

- Upstream credentials. Ever. They never leave Bodhi.
- OAuth refresh. Bodhi runs the refresh flow when the token expires.
- Tool authorization. The whitelist is enforced server-side.
- Per-user provisioning. The user does it once in their Bodhi UI; you re-use the access for every session.

## Errors

Errors are returned in the upstream MCP server's native shape when they originate upstream — the proxy is transparent. Local errors (instance not found, instance disabled, access not granted, body too large) are returned in [Bodhi's error format](/docs/api-compatibility/error-format).

A `403` with a Bodhi-shaped envelope means the access request hasn't been approved (or has been revoked). A `401` is a token problem. A `403` with the upstream's MCP-shaped error means upstream auth failed (likely an expired OAuth grant that needs reconnecting).

## Full schema

See Swagger UI at `http://<your-bodhi-instance>/swagger-ui` for the route registration and the JSON-RPC envelope. The MCP protocol itself is specified at [modelcontextprotocol.io](https://modelcontextprotocol.io). The local default is `http://localhost:1135/swagger-ui`.
