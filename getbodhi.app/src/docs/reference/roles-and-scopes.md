---
title: 'Roles and Scopes'
description: 'Capability matrix — what User, PowerUser, Manager, and Admin can do, and how API token scopes interact with the issuing user role'
order: 2
---

# Roles and Scopes

Bodhi uses a role-based access control model with four assignable roles and two API token scopes. This page is the lookup table — for the conceptual overview, see [Auth and Roles](/docs/concepts/auth-and-roles).

## The role hierarchy

Roles are strictly ordered. Each role inherits everything the role below it can do.

```
User  →  PowerUser  →  Manager  →  Admin
```

Two non-assignable roles also exist:

- **Anonymous** — unauthenticated request; only public endpoints accessible.
- **Guest** — authenticated user without an assigned role (waiting on access approval). Cannot reach feature pages.

You will never see Anonymous or Guest in any "assign a role" UI — they are inferred from the auth context.

## The two API token scopes

API tokens carry a scope that further restricts what the token can do, even when the user behind it has a higher role.

| Scope       | Equivalent role ceiling                                 |
| ----------- | ------------------------------------------------------- |
| `User`      | User-level capabilities only                            |
| `PowerUser` | PowerUser-level capabilities (no Manager/Admin actions) |

There is intentionally no `Manager` or `Admin` token scope. Manager and Admin actions (approving access requests, editing app settings, managing users) are session-only by design. A human must be present in a browser to make those decisions.

## Effective capability rule

For an API token call:

```
effective_capability = min(user_role, token_scope)
```

A PowerUser who mints a User-scoped token can only perform User-level operations with that token, even though they could do more with their session. A User who tries to mint a PowerUser-scoped token is rejected — token scope is upper-bounded by the issuing user's role.

Session-based requests (browser cookies) use the user's role directly — there is no scope component.

## Capability matrix

✓ = allowed for that role/scope · — = not allowed

| Action                                                                               | Anonymous | Guest | User | PowerUser | Manager | Admin | Token: User | Token: PowerUser |
| ------------------------------------------------------------------------------------ | --------- | ----- | ---- | --------- | ------- | ----- | ----------- | ---------------- |
| View login page, request access                                                      | ✓         | ✓     | ✓    | ✓         | ✓       | ✓     | —           | —                |
| Chat with a model (UI)                                                               | —         | —     | ✓    | ✓         | ✓       | ✓     | —           | —                |
| Call `/v1/chat/completions`, `/v1/embeddings`, `/anthropic/v1/messages`, `/v1beta/*` | —         | —     | ✓    | ✓         | ✓       | ✓     | ✓           | ✓                |
| Call `/v1/responses` (async polling)                                                 | —         | —     | ✓    | ✓         | ✓       | ✓     | ✓           | ✓                |
| List models (`/v1/models`)                                                           | —         | —     | ✓    | ✓         | ✓       | ✓     | ✓           | ✓                |
| Browse own MCP instances + chats                                                     | —         | —     | ✓    | ✓         | ✓       | ✓     | —           | —                |
| Download a GGUF model from HuggingFace                                               | —         | —     | —    | ✓         | ✓       | ✓     | —           | ✓                |
| Create / edit a user model alias                                                     | —         | —     | —    | ✓         | ✓       | ✓     | —           | ✓                |
| Configure an API model (OpenAI / Anthropic / Gemini / Groq)                          | —         | —     | —    | ✓         | ✓       | ✓     | —           | ✓                |
| Connect to an MCP server, manage own MCP instances                                   | —         | —     | —    | ✓         | ✓       | ✓     | —           | ✓                |
| Mint API tokens                                                                      | —         | —     | —    | ✓         | ✓       | ✓     | —           | —                |
| Register an external app, review its access requests                                 | —         | —     | —    | ✓         | ✓       | ✓     | —           | —                |
| Use the MCP proxy (`/bodhi/v1/apps/mcps/{id}/mcp`)                                   | —         | —     | ✓    | ✓         | ✓       | ✓     | ✓           | ✓                |
| Approve user access requests                                                         | —         | —     | —    | —         | ✓       | ✓     | —           | —                |
| Manage users, change user roles (≤ Manager)                                          | —         | —     | —    | —         | ✓       | ✓     | —           | —                |
| Manage the pre-registered MCP server catalog (MCP store)                             | —         | —     | —    | —         | ✓       | ✓     | —           | —                |
| Edit app settings (`/ui/settings/`)                                                  | —         | —     | —    | —         | —       | ✓     | —           | —                |
| Promote / demote Admins                                                              | —         | —     | —    | —         | —       | ✓     | —           | —                |
| View developer dev pages                                                             | —         | —     | —    | —         | —       | ✓     | —           | —                |

A few notes on the rows above:

- "Manage users, change user roles (≤ Manager)" — a Manager can assign User, PowerUser, or Manager. Only an Admin can assign Admin or change another Admin's role.
- "Use the MCP proxy" — the proxy endpoint additionally requires the calling identity to have _resource consent_ for the target MCP instance. Sessions get this implicitly (you own your instances); external apps and tokens get it through the access-request flow.
- "Configure API model" rows include both adding new providers and rotating their keys. Reading the list of configured API models (without keys) is allowed at User level.

## Reading the matrix from a token's perspective

A token's effective capability is the _intersection_ of (a) what its scope allows and (b) what the issuing user's role allows. So:

- Manager user + User-scope token = **User** capabilities only.
- PowerUser user + PowerUser-scope token = **PowerUser** capabilities.
- Admin user + PowerUser-scope token = **PowerUser** capabilities. Admin-only actions (settings page, role assignment) cannot be performed with any token.
- User user + (attempted) PowerUser-scope token = **rejected at mint time**.

## External app tokens

Third-party applications request access via a separate flow. They receive an `ExternalApp` auth context bound to a list of resources (specific MCP instances, specific API model aliases). Within that bound list they get the equivalent of User scope; outside it they get nothing. See [App access management](/docs/features/auth/app-access-management).

## Related

- [Auth and Roles](/docs/concepts/auth-and-roles) — the conceptual model behind this matrix.
- [Auth overview](/docs/features/auth/overview) — feature-page tour of auth in the UI.
- [API tokens](/docs/features/auth/api-tokens) — minting and rotating tokens.
- [User management](/docs/features/auth/user-management) — assigning roles.
- [Error codes](/docs/reference/error-codes) — what 401 vs 403 actually mean in each envelope.
