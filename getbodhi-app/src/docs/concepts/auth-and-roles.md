---
title: 'Auth and Roles'
description: 'OAuth2 PKCE, four roles (User / PowerUser / Manager / Admin), session cookies vs API tokens, and how scopes work'
order: 4
---

# Auth and Roles

Every request to Bodhi — UI click, curl call, agent tool invocation — passes through the same auth gate. This page covers the four ideas you need to navigate it: roles, scopes, login flow, and token types.

## The four roles

Bodhi uses a hierarchical role model. Each role inherits everything its lower neighbors can do, plus some extras.

| Role          | One-line capability summary                                                                                                                                        |
| ------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| **User**      | Chat with models, call inference + embedding APIs, manage their own MCP instances and chats.                                                                       |
| **PowerUser** | Everything User can do, plus: download models, create/edit user aliases, configure API models, mint API tokens, register apps.                                     |
| **Manager**   | Everything PowerUser can do, plus: approve user access requests, manage user roles (cannot promote/demote Admins), maintain the pre-registered MCP server catalog. |
| **Admin**     | Everything Manager can do, plus: full system access, edit app settings, manage Admins, view all tenant data.                                                       |

The first user to log in via OAuth becomes Admin automatically. Subsequent users start with no role (Guest) and must request access. An admin or manager approves the request and chooses the assigned role.

(Two implicit roles also exist: **Anonymous** for unauthenticated requests on public endpoints, and **Guest** for authenticated users without an assigned role. You won't normally interact with these — they exist to model "logged in but waiting for approval.")

## OAuth2 PKCE login, in one paragraph

Bodhi delegates identity to an external OAuth2 provider (`https://id.getbodhi.app/` by default). When you click "Sign in" in the UI, Bodhi generates a PKCE challenge, redirects you to the provider, you authenticate, and the provider redirects back to Bodhi with an authorization code. Bodhi exchanges the code for tokens, validates the JWT, sets a secure session cookie, and you're logged in. There is no password ever stored in Bodhi itself.

For programmatic clients, the same OAuth flow can be used end-to-end (this is how third-party apps register and request access — see [App Access Management](/docs/features/auth/app-access-management)). For most server-to-server scripting, though, **API tokens** are simpler.

## Two token types

Bodhi accepts two kinds of credentials on incoming requests:

### 1. Session cookies (browser)

Set after OAuth login and used by the built-in UI (chat, models, MCPs, settings). Cookies are HttpOnly, Secure, SameSite, and refreshed silently as long as the user is active. You don't manage these directly — the browser does.

### 2. API tokens (programmatic)

Mint an API token from the Tokens page (PowerUser+). The token shape is `bodhiapp_<random>.<client_id>`. Send it as `Authorization: Bearer <token>` on any API call.

Each API token has a **scope**:

- **User scope** — token can do what a User-role user can do (chat, embeddings, list models).
- **PowerUser scope** — token can do what a PowerUser can do (everything in User scope, plus model management, alias creation, API-model config, MCP setup).

Scopes are upper-bounded by the issuing user's role: a User-role human cannot mint a PowerUser-scope token. Tokens can be deactivated or rotated at any time from the Tokens page.

Note: API tokens do **not** grant Manager or Admin capabilities. Manager/Admin operations are session-only by design — there is no API-token equivalent for "approve a user access request."

## What gets checked, where

For each request, Bodhi resolves an `AuthContext` — a tagged union that records both _who_ is calling and _how_:

- `Anonymous` — no credentials, only public endpoints accessible.
- `Session` — browser session cookie, includes the user's role.
- `ApiToken` — Bearer token, includes the issuing user's role and the token's scope.
- `ExternalApp` — token issued via the app-access-request flow, scoped to specific MCPs and APIs the app was approved for.

Routes are tagged with the minimum role / scope they require. If your context doesn't satisfy the requirement, Bodhi returns 401/403 before the handler runs.

## Where it gets concrete

Forward references — these pages cover the workflows in detail (they land in a later phase):

- `/docs/features/auth/overview` — the same role/scope picture mapped page-by-page to UI capabilities.
- `/docs/features/auth/user-management` — admin/manager workflow for managing users and roles.
- `/docs/features/auth/user-access-requests` — the request → approve → re-login dance.
- `/docs/features/auth/api-tokens` — minting, scoping, rotating, and revoking tokens.
- `/docs/features/auth/app-access-management` — third-party apps requesting MCP/API access via OAuth.

## Common confusions

- **"My Bearer token works here but not there."** The endpoint probably requires a higher scope than your token has, or it's session-only (Manager/Admin operations).
- **"I logged in but I can't see the Models page."** You're a Guest until an admin assigns you a role. Submit an access request from `/ui/request-access/`.
- **"Why can't my token approve access requests?"** Approval is intentionally session-only. Operators must be present in a browser to make trust decisions.
- **"I rotated my token; the old one still seems to work for a few seconds."** Tokens are checked against an in-memory cache; revocation is near-instant but not strictly synchronous. Wait a few seconds and it'll fail.

Up next: **[MCP Overview](/docs/concepts/mcp-overview)** — the last of the five core concepts, covering Model Context Protocol, tool registries, and the MCP proxy.
