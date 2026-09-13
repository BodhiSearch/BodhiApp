---
title: 'Auth Overview'
description: 'Roles vs token scopes, session cookies vs API tokens, and the four feature pages that follow'
order: 240
---

# Auth at a glance

Bodhi App ships a single auth model that works the same way whether you click a button in the UI, run a curl from a CI job, or wire an external app through the bodhi-js SDK. This page is the map: pick a role, see what it can do, pick a credential type, and jump to the feature page that walks the workflow.

For the deeper mental model — OAuth2 PKCE flow, the `AuthContext` types, why scopes are a strict subset of roles — see [Concepts: Auth and Roles](/docs/concepts/auth-and-roles).

## Roles → capabilities

Bodhi has four assignable roles plus two implicit ones (`Anonymous` for unauthenticated requests, `Guest` for authenticated users awaiting approval). The four assignable roles are hierarchical: each row inherits everything above it.

| Capability                          | User | PowerUser | Manager | Admin |
| ----------------------------------- | :--: | :-------: | :-----: | :---: |
| Chat UI + chat / embeddings APIs    |  ✅  |    ✅     |   ✅    |  ✅   |
| List models / aliases               |  ✅  |    ✅     |   ✅    |  ✅   |
| Manage own MCP instances            |  ✅  |    ✅     |   ✅    |  ✅   |
| Mint / rotate own API tokens        |  ❌  |    ✅     |   ✅    |  ✅   |
| Download model files                |  ❌  |    ✅     |   ✅    |  ✅   |
| Create / edit user model aliases    |  ❌  |    ✅     |   ✅    |  ✅   |
| Configure API models (OpenAI etc.)  |  ❌  |    ✅     |   ✅    |  ✅   |
| Register external apps              |  ❌  |    ✅     |   ✅    |  ✅   |
| Approve user access requests        |  ❌  |    ❌     |  ✅\*   |  ✅   |
| Change user roles                   |  ❌  |    ❌     |  ✅\*   |  ✅   |
| Maintain pre-registered MCP catalog |  ❌  |    ❌     |   ✅    |  ✅   |
| View / edit app settings            |  ❌  |    ❌     |   ❌    |  ✅   |
| Manage Admins                       |  ❌  |    ❌     |   ❌    |  ✅   |

\* Manager can act on Users, PowerUsers, and other Managers — never on Admins.

The first user to log in via OAuth becomes Admin. Everyone after that lands on `/ui/request-access/` and waits for approval.

## Token types

Two credentials types are accepted on incoming requests. Pick one based on whether you're a human in a browser or a program calling APIs.

### Session cookies (browser)

Set on OAuth login. Used by the built-in UI: chat, models, MCPs, settings. Cookies are HttpOnly, Secure, SameSite, refreshed silently while you're active. No manual handling — the browser does it.

Session cookies carry your full assigned role, so all four roles' capabilities are available through the UI.

### API tokens (programmatic)

Mint from the [Tokens page](/docs/features/auth/api-tokens) (PowerUser+). The format is:

```
bodhiapp_<base64url_random>.<client_id>
```

Send as `Authorization: Bearer <token>`. Tokens are SHA-256 hashed before storage; the original is shown exactly once at creation.

## Token scopes vs user roles (don't mix these up)

This is the most common point of confusion, so:

- **User roles** (assigned to humans): `User`, `PowerUser`, `Manager`, `Admin`. Four values. Control session-based access.
- **Token scopes** (attached to API tokens): `User`, `PowerUser`. **Two values only.** Control programmatic access.

There is **no** Manager-scope or Admin-scope token. Manager and Admin operations — approving access requests, editing settings, changing roles — are session-only by design. Operators must be present in a browser to make trust decisions.

A token's scope is also upper-bounded by the issuing user's role:

- A User-role human cannot mint _any_ API token (need PowerUser+ to access the Tokens page).
- A PowerUser human can mint a User-scope or PowerUser-scope token.
- An Admin human can mint a User-scope or PowerUser-scope token (still no Admin-scope).

Translation: tokens are a _strict subset_ of what their issuer can do via session.

External apps (third-party apps integrated via bodhi-js SDK) follow the same rule — they receive `User` or `PowerUser` scoped tokens at most. Details on [App Access Management](/docs/features/auth/app-access-management).

## "What you can do" decision tree

Pick the page that matches your task:

- **You're a new user staring at "Access Request Pending"** → [User Access Requests](/docs/features/auth/user-access-requests). Walks through requesting access, the wait, and what happens after approval.
- **You're a Manager or Admin and the queue has pending users** → [User Management](/docs/features/auth/user-management). Approve, reject, change roles, remove users.
- **You're a developer or PowerUser scripting against the API** → [API Tokens](/docs/features/auth/api-tokens). Mint, scope, rotate, revoke.
- **A third-party app wants access to your MCPs / API models** → [App Access Management](/docs/features/auth/app-access-management). Granular per-resource consent, scoped OAuth tokens, 10-minute review window.

## Where this connects

- Adding an MCP server with auth? See [MCP Auth Methods](/docs/features/mcps/auth-methods).
- Configuring an API model that calls a remote provider? See [API Models](/docs/features/models/api-models).
- The full role × endpoint matrix lives in `/docs/reference/roles-and-scopes` (reserved — lands in the reference tier).
- Deeper architectural notes (OAuth2 PKCE, AuthContext types, request lifecycle) live in `/docs/advanced/security-model` (reserved).

## Common pitfalls

- **"My token works for chat but not for managing models."** Chat needs User scope; model management needs PowerUser scope. Mint a new token with the higher scope — scopes are immutable after creation.
- **"I'm Admin but my token can't approve a user access request."** Correct. Approval is session-only. Open the UI in a browser.
- **"I logged in but can only see Request Access."** You're a Guest until an admin approves you. See [User Access Requests](/docs/features/auth/user-access-requests).
- **"I rotated my token and the old one still authenticates briefly."** Tokens are validated against an in-memory cache; revocation is near-instant but not strictly synchronous. A few seconds and it'll fail.
