---
title: 'MCP Auth Methods'
description: 'Pick between Header, OAuth2 preregistered, and OAuth2 Dynamic Client Registration for connecting to an MCP server'
order: 2
---

# MCP Auth Methods

Bodhi supports three ways to authenticate against an MCP server. The method is set at the **server level** (an admin templates it on the published server) and inherited by every user instance that connects through it. This page explains when to pick which, what you enter in the UI, and how Bodhi handles the credentials afterwards.

> Skip ahead: if you already know which method the server expects, go straight to [Setup](/docs/features/mcps/setup).

## Header-based

Send a static API key or bearer token as an HTTP header on every request. The simplest method, and the one most local-development and self-hosted MCP servers expect.

### When to pick it

- The server's docs say **"send `Authorization: Bearer <token>`"** or **"set `X-API-Key: <key>`"**.
- You have a long-lived API key, not an OAuth flow.
- You are connecting to a trusted internal service (your own infra, a cloud-provider MCP behind a static credential).

### What the user enters

When creating an instance against a header-templated server, Bodhi shows the field names defined by the admin (`Authorization`, `X-API-Key`, etc.) and asks for the secret value. Each value is a password field with a show/hide toggle.

### How Bodhi stores the credential

The value is encrypted at rest before it touches the database. On every outbound request to the MCP server, Bodhi attaches the header automatically. There is no token-refresh dance — the credential is sent verbatim until you delete or replace the instance.

> Self-hoster note: header values are encrypted using a workspace-level key derived from `BODHI_ENCRYPTION_KEY`. Rotating that key invalidates stored credentials, so plan rotations carefully. See the advanced env-var reference for details.

## OAuth2 with preregistered client

The MCP server supports OAuth2, but does not implement Dynamic Client Registration. The server (or the IdP behind it) publishes a fixed `client_id` (and sometimes a `client_secret`), plus the `authorization_endpoint` and `token_endpoint` URLs. Admins paste those into the server template once; every user then runs the OAuth dance against their own account.

### When to pick it

- The MCP server's docs link to a developer portal where you register a client and get a `client_id`.
- The server's auth provider is well-known but does **not** expose `.well-known/oauth-authorization-server` with DCR enabled.
- You want stable client credentials that survive backend restarts.

### What the admin enters (once, on the server template)

- **Client ID** — the OAuth client identifier.
- **Client Secret** _(optional)_ — for confidential clients.
- **Authorization Endpoint** — the OAuth `/authorize` URL.
- **Token Endpoint** — the OAuth `/token` URL.
- **Scopes** _(optional)_ — space-separated.

### What the user does

When creating an instance, the user picks the OAuth template from the auth dropdown and clicks **Connect**. Bodhi:

1. Saves the form state to session storage so the form survives the redirect.
2. Redirects the browser to the server's authorization endpoint.
3. After the user approves, the OAuth provider redirects back to Bodhi's callback URL (`/ui/mcps/oauth/callback/`).
4. Bodhi exchanges the authorization code for tokens and stores them, then returns the user to the form with a green **Connected** badge.

### Token refresh

Bodhi stores both the access token and the refresh token (when the provider issues one). When the access token expires, Bodhi refreshes it on the next outbound MCP request without user interaction. If the refresh fails — refresh token revoked, scopes changed — the user sees the **Connected** state drop, and clicking **Connect** again re-runs the dance.

## OAuth2 with Dynamic Client Registration (DCR)

The MCP server implements [RFC 7591](https://www.rfc-editor.org/rfc/rfc7591) (Dynamic Client Registration) and typically [RFC 8414](https://www.rfc-editor.org/rfc/rfc8414) (server metadata discovery). Bodhi auto-discovers the OAuth endpoints, registers itself as a client on the fly, and runs the same authorization flow without any preregistered IDs. Zero-config from the user's perspective.

### When to pick it

- The MCP server advertises `.well-known/oauth-authorization-server` and supports DCR.
- You are integrating with an "ecosystem" MCP service where every consumer registers itself dynamically rather than being pre-provisioned.
- You want the lowest-friction admin experience — paste a URL, hit save.

### What the admin enters

- The MCP server URL (the rest can usually be auto-populated).
- A registration endpoint, if discovery does not produce one.
- Scopes, if the defaults are not enough.

When the admin saves, Bodhi calls the discovery endpoint, populates the OAuth endpoints, then calls the registration endpoint to obtain a fresh `client_id` and `client_secret`. Those credentials are stored against the server template just like a preregistered client.

If discovery or registration fails on the **New Server** page, Bodhi silently falls back to the pre-registered form so the admin can fill the fields manually.

### What the user does

Same as preregistered: pick the OAuth template, hit **Connect**, approve at the provider, return to the form. The DCR-registered client is invisible to end-users — they just see "OAuth".

### Token refresh

Identical to preregistered OAuth. If the IdP revokes the registered client (rare, but it happens), the admin re-saves the server template to trigger a fresh registration; users may need to reconnect.

## How Bodhi protects credentials

All three methods share the same storage guarantees:

- Credentials are encrypted at rest with a workspace-level key derived from `BODHI_ENCRYPTION_KEY`.
- OAuth tokens are scoped to a single user — your token bundle is never readable by another user on the same deployment.
- Outbound MCP requests run server-side. The browser never sees the credential after the OAuth callback returns.
- Disconnecting an instance removes the stored token (or zeroes the header value) before the row is deleted.

Self-hosters should treat `BODHI_ENCRYPTION_KEY` as long-lived and back it up alongside the database. Losing it makes existing credentials unrecoverable; rotating it invalidates them.

## Decision matrix

| Signal                                                                     | Pick this                                       |
| -------------------------------------------------------------------------- | ----------------------------------------------- |
| Docs mention `Authorization: Bearer ...` or `X-API-Key: ...`               | **Header**                                      |
| Static API key from a developer portal, no OAuth                           | **Header**                                      |
| Local or trusted-internal MCP, simplest setup wins                         | **Header**                                      |
| OAuth provider with a fixed `client_id` to register                        | **OAuth2 preregistered**                        |
| Production MCP behind a well-known IdP (Google, GitHub, etc.)              | **OAuth2 preregistered**                        |
| Server publishes `.well-known/oauth-authorization-server` and supports DCR | **OAuth2 DCR**                                  |
| Ecosystem-style MCP where every consumer self-registers                    | **OAuth2 DCR**                                  |
| You want zero-config admin setup                                           | **OAuth2 DCR** if supported, else preregistered |

## Where to next

- [Setup](/docs/features/mcps/setup) — actually create the server template and a user instance.
- [Pre-registered Servers](/docs/features/mcps/pre-registered-servers) — admin walkthrough for publishing to the catalog.
- [App Access Management](/docs/features/auth/app-access-management) — how external apps obtain MCP access through Bodhi without ever seeing these credentials.
