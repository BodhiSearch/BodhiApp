---
title: 'Building Third-Party Apps'
description: 'End-to-end lifecycle for apps built on BodhiApp — OAuth client registration, user consent flow, scoped token usage, and the security model'
order: 252
---

# Building Third-Party Apps on BodhiApp

This guide covers the complete lifecycle for building an application that connects to a user's BodhiApp instance: how to register your app, how users grant your app access, and how your app calls BodhiApp APIs using the approved credentials — without ever touching the user's underlying provider API keys.

For SDK-based React apps the [Getting Started](/docs/developer/getting-started) guide is the fastest path to code. This page focuses on the underlying protocol so you can implement it in any language or framework.

## Trust Model

BodhiApp acts as a resource server sitting between your application and the user's local AI infrastructure. The design enforces three properties:

- **Your app never sees provider API keys.** Keys for OpenAI, Anthropic, HuggingFace, and other upstream providers are stored encrypted in BodhiApp. Your app receives a scoped OAuth token that grants access to BodhiApp's proxy endpoints — not to the underlying providers directly.
- **User consent is explicit and granular.** Users choose which MCP server instances your app can reach and what role level to grant. They can downgrade the scope or deny the request entirely.
- **Tokens are revocable and per-user.** Each user's approval produces a separate scoped token. Revoking one user's access has no effect on others.

## Step 1: Register Your App

To call BodhiApp APIs you need an `app_client_id` — an OAuth 2.0 client identifier issued by the upstream OAuth server.

Register your application at [https://developer.getbodhi.app](https://developer.getbodhi.app) to obtain your `app_client_id`. This is the identifier you embed in your application and send when initiating access requests.

> **TODO: verify** — the developer portal and self-service registration flow are not yet publicly released. If you are building during the private beta, contact the BodhiApp team for your client ID.

There is no `/bodhi/v1/apps/register` endpoint in BodhiApp itself. Registration happens at the upstream OAuth provider level, not at the per-instance level.

## Step 2: Request Access from the User

Once you have an `app_client_id`, your app initiates an access request against the user's BodhiApp instance. This is an unauthenticated call — no token is needed because no token has been granted yet.

### 2a. Create a Draft Access Request

```bash
curl -X POST http://localhost:1135/bodhi/v1/apps/request-access \
  -H "Content-Type: application/json" \
  -d '{
    "app_client_id": "your-app-client-id",
    "flow_type": "popup",
    "requested_role": "scope_user_user",
    "requested": {
      "version": "1",
      "mcp_servers": [
        { "url": "https://mcp.example.com/mcp" }
      ]
    }
  }'
```

The `version` field inside `requested` is mandatory. The only supported version today is `"1"`. Future resource types will use the same versioned envelope.

**Response (201 Created):**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "draft",
  "review_url": "http://localhost:1135/ui/apps/access-requests/review?id=550e8400-e29b-41d4-a716-446655440000"
}
```

### 2b. Direct the User to the Review URL

Open `review_url` in a browser — either in a popup (`flow_type: "popup"`) or by redirecting the user's current tab (`flow_type: "redirect"`). For redirect flow, include a `redirect_url` in the request body so BodhiApp knows where to send the user after they decide.

The user must be logged in to their BodhiApp instance. The review page shows:

- Your app's client ID (and name/description if configured in the OAuth provider)
- The role you requested
- Each requested MCP server URL, with the user's available instances to choose from

The user can approve some resources and exclude others, downgrade the role, or deny entirely.

### 2c. Poll for Status

While the user reviews, poll the status endpoint using your `app_client_id` as a query parameter:

```bash
curl "http://localhost:1135/bodhi/v1/apps/access-requests/550e8400-e29b-41d4-a716-446655440000?app_client_id=your-app-client-id"
```

**Response when approved:**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "approved",
  "requested_role": "scope_user_user",
  "approved_role": "scope_user_user",
  "access_request_scope": "scope_access_request:550e8400-e29b-41d4-a716-446655440000"
}
```

| Status     | Meaning                                         |
| ---------- | ----------------------------------------------- |
| `draft`    | Awaiting user review (expires after 10 minutes) |
| `approved` | User approved; `access_request_scope` is ready  |
| `denied`   | User denied the request                         |
| `failed`   | Processing error                                |

The `app_client_id` query parameter is required on the polling endpoint. Requests with a mismatched client ID return 404, preventing enumeration by other parties.

### 2d. Exchange for a Scoped Token

Use `access_request_scope` as the `scope` parameter during the OAuth token exchange with the upstream provider. This yields an access token scoped to exactly the resources the user approved.

> **TODO: verify** — the specific OAuth token exchange endpoint and parameters depend on the upstream OAuth server configuration. Consult your OAuth provider documentation, using `access_request_scope` as the scope value.

For a detailed walkthrough of the full access request protocol including edge cases and expiry handling, see [App Access Requests](/docs/developer/app-access-requests).

## Step 3: Call BodhiApp APIs with the Approved Token

With the scoped token in hand, your app calls BodhiApp endpoints using a standard Bearer token header. No provider API keys, no session cookies.

### OpenAI-Compatible LLM Endpoints

These endpoints work with any `UserScope` token (`scope_user_user` or `scope_user_power_user`):

```bash
# List models available on this BodhiApp instance
curl http://localhost:1135/v1/models \
  -H "Authorization: Bearer <scoped-token>"

# Chat completion (streaming or non-streaming)
curl -X POST http://localhost:1135/v1/chat/completions \
  -H "Authorization: Bearer <scoped-token>" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "llama3",
    "messages": [{"role": "user", "content": "Hello!"}],
    "stream": false
  }'

# Text embeddings
curl -X POST http://localhost:1135/v1/embeddings \
  -H "Authorization: Bearer <scoped-token>" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "nomic-embed-text",
    "input": "Text to embed"
  }'
```

These use the `/v1/` prefix — the same OpenAI-compatible format, so existing OpenAI client libraries work unchanged.

### MCP Tool Access

After the user approves MCP server access, retrieve the granted instances and use the MCP proxy:

```bash
# List MCP instances accessible to your app
curl http://localhost:1135/bodhi/v1/apps/mcps \
  -H "Authorization: Bearer <scoped-token>"

# Get details for a specific instance
curl http://localhost:1135/bodhi/v1/apps/mcps/{id} \
  -H "Authorization: Bearer <scoped-token>"
```

Each MCP instance exposes a Streamable HTTP MCP proxy at:

```
/bodhi/v1/apps/mcps/{id}/mcp
```

Use any MCP-compatible client to connect to this path with your Bearer token. This endpoint proxies the full MCP protocol, including tool discovery and invocation.

## Role Scopes and What They Allow

When creating an access request, you choose a `requested_role`. The user approves at this level or lower.

| Scope      | Value                   | Permissions                                                                |
| ---------- | ----------------------- | -------------------------------------------------------------------------- |
| User       | `scope_user_user`       | Chat completions, embeddings, model listing, MCP tool access               |
| Power User | `scope_user_power_user` | All User permissions, plus model alias management and model file downloads |

Request the minimum scope your app needs. If your app only calls the LLM and uses MCP tools, request `scope_user_user`. Power User scope is appropriate only if your app needs to manage model aliases or download model files on behalf of the user.

Apps are capped at Power User scope — Manager and Admin roles are never available to external app tokens regardless of who approves the request.

## Example: Powering a Research Assistant App

Here is a concrete end-to-end scenario. A third-party web app called "ResearchBot" wants to use a user's local LLM and a DeepWiki MCP server to answer research questions.

1. **Registration**: ResearchBot registers at developer.getbodhi.app and receives `app_client_id: "researchbot-prod"`.

2. **Access request**: When the user first connects ResearchBot to their BodhiApp, ResearchBot calls:

   ```json
   POST /bodhi/v1/apps/request-access
   {
     "app_client_id": "researchbot-prod",
     "flow_type": "popup",
     "requested_role": "scope_user_user",
     "requested": {
       "version": "1",
       "mcp_servers": [{ "url": "https://mcp.deepwiki.com/mcp" }]
     }
   }
   ```

3. **User consent**: ResearchBot opens the `review_url` in a popup. The user sees the request, selects their DeepWiki MCP instance, and clicks Approve.

4. **Token exchange**: ResearchBot polls until status is `approved`, then exchanges `access_request_scope` for a scoped token via OAuth.

5. **API calls**: ResearchBot uses the token to call `/v1/chat/completions` for LLM responses and `/bodhi/v1/apps/mcps/{id}/mcp` to discover and call DeepWiki tools. The user's API keys for DeepWiki and the underlying LLM provider are never exposed to ResearchBot.

## Security Model

This design is materially safer than the alternative of asking users to paste API keys into your app:

| Property              | Key-sharing                     | BodhiApp consent flow                                 |
| --------------------- | ------------------------------- | ----------------------------------------------------- |
| Provider key exposure | Your app sees the key           | Your app never sees the key                           |
| Key storage           | In your app's database          | Encrypted in BodhiApp, user-controlled                |
| Revocation            | User must rotate key everywhere | User denies access in BodhiApp; other apps unaffected |
| Scope                 | Full provider access            | Limited to approved resources and role                |
| Per-user isolation    | Shared key, no isolation        | Each user's approval is a separate scoped token       |

BodhiApp verifies on every request that the token's access request record is still in `approved` status and that the token's `app_client_id` matches the record. There is no way for an approved token to exceed the scope the user granted.

## Related Documentation

- [App Access Requests](/docs/developer/app-access-requests) — Detailed protocol reference, flow types, privilege escalation rules, and the full API table
- [Getting Started](/docs/developer/getting-started) — SDK-based quick-start for React apps
- [OpenAPI Reference](/docs/developer/openapi-reference) — Interactive endpoint explorer and CORS policy
- [App Access Management](/docs/features/auth/app-access-management) — The user-facing review flow (what your users see)
