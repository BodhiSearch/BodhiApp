---
title: 'App Access Requests'
description: 'Resource consent model for third-party apps — access request API flow, user review, role privileges, and MCP instance approval'
order: 255
---

# App Access Requests

Third-party apps access Bodhi resources through a consent-based access request flow. Apps request access to specific resources (MCP servers, API endpoints), and the user reviews and approves or denies the request. This ensures users maintain control over which apps can use their local AI infrastructure.

## Resource Consent Model

When an app needs access to a user's Bodhi resources, it creates an access request specifying:

- **MCP servers** -- which MCP server URLs the app needs (identified by URL)
- **Requested role** -- the privilege level the app is asking for (`scope_user_user` or `scope_user_power_user`)

The user reviews this request and decides:

- Which MCP instances to grant (the user may have multiple instances of the same MCP server)
- What role level to approve (can downgrade from what was requested)
- Whether to approve or deny entirely

Future resource types will include workspaces and agents.

## API Flow

### Step 1: Create Access Request

The app sends an unauthenticated POST to create a draft access request:

```
POST /bodhi/v1/apps/request-access
Content-Type: application/json

{
  "app_client_id": "your-oauth-client-id",
  "flow_type": "popup",
  "requested_role": "scope_user_user",
  "requested": {
    "mcp_servers": [
      { "url": "http://localhost:3000/mcp" }
    ]
  }
}
```

**Response (201 Created):**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "draft",
  "review_url": "http://localhost:1135/ui/apps/access-requests/review?id=550e8400-e29b-41d4-a716-446655440000"
}
```

The `review_url` is where the user reviews and approves the request. The app opens this URL for the user (in a popup or via redirect, depending on `flow_type`).

#### Flow Types

| Flow Type | Behavior |
|-----------|----------|
| `popup` | App opens the review URL in a popup window. After the user decides, the window closes and the app detects the result. |
| `redirect` | App redirects to the review URL. After the user decides, Bodhi redirects back to the app's `redirect_url`. Requires `redirect_url` in the request body. |

### Step 2: User Reviews the Request

The user navigates to the `review_url` (they must be logged in to Bodhi). The review page shows:

- The requesting app's client ID
- The requested role level
- The requested MCP servers, with the user's available instances for each

The user can:

- **Select MCP instances** to grant for each requested MCP server URL
- **Downgrade the role** from what was requested (e.g., approve as `scope_user_user` when `scope_user_power_user` was requested)
- **Approve** the request with their selections
- **Deny** the request entirely

### Step 3: App Polls for Status

While the user is reviewing, the app polls for the access request status:

```
GET /bodhi/v1/apps/access-requests/{id}?app_client_id=your-oauth-client-id
```

The `app_client_id` query parameter is required for security -- only the app that created the request can poll its status.

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

**Possible status values:**

| Status | Meaning |
|--------|---------|
| `draft` | Awaiting user review |
| `approved` | User approved; `access_request_scope` is available for token exchange |
| `denied` | User denied the request |
| `failed` | Processing error |

### Step 4: Token Exchange

Once the status is `approved`, the app uses the `access_request_scope` value as a scope parameter during the OAuth token exchange. This yields an access token scoped to the approved resources.

With this token, the app can access the `/bodhi/v1/apps/` endpoints for the granted resources.

### Step 5: Access Granted Resources

With the scoped token, the app can call external app endpoints:

```
GET /bodhi/v1/apps/mcps
Authorization: Bearer <scoped-token>
```

```
GET /bodhi/v1/apps/mcps/{id}
Authorization: Bearer <scoped-token>
```

```
POST /bodhi/v1/apps/mcps/{id}/tools/{tool_name}/execute
Authorization: Bearer <scoped-token>
Content-Type: application/json

{
  "params": { "text": "Hello from my app" }
}
```

```
POST /bodhi/v1/apps/mcps/{id}/tools/refresh
Authorization: Bearer <scoped-token>
```

## Draft Expiry

Draft access requests expire after **10 minutes**. If the user does not act within this window, the request transitions to expired status. The app must create a new access request if this happens.

## Privilege Escalation Protection

The access request system enforces strict privilege boundaries:

1. **Approved role cannot exceed requested role.** If the app requests `scope_user_user`, the reviewer cannot upgrade it to `scope_user_power_user`.

2. **Approved role cannot exceed the reviewer's own role.** A user with `User` role can only grant `scope_user_user`. A user with `PowerUser` or higher role can grant up to `scope_user_power_user`.

3. **MCP instance ownership is verified.** The reviewer can only grant MCP instances they own. The server validates that each selected instance belongs to the reviewing user and is enabled.

## API Reference

| Endpoint | Method | Auth | Description |
|----------|--------|------|-------------|
| `/bodhi/v1/apps/request-access` | POST | None | Create a draft access request |
| `/bodhi/v1/apps/access-requests/{id}` | GET | None | Poll access request status (requires `app_client_id` query param) |
| `/bodhi/v1/access-requests/{id}/review` | GET | Session | Get review page data |
| `/bodhi/v1/access-requests/{id}/approve` | PUT | Session | Approve with role and resource selections |
| `/bodhi/v1/access-requests/{id}/deny` | POST | Session | Deny the request |
| `/bodhi/v1/apps/mcps` | GET | Bearer | List MCP instances accessible to the app |
| `/bodhi/v1/apps/mcps/{id}` | GET | Bearer | Get a specific MCP instance |
| `/bodhi/v1/apps/mcps/{id}/tools/refresh` | POST | Bearer | Refresh tools for an MCP instance |
| `/bodhi/v1/apps/mcps/{id}/tools/{tool_name}/execute` | POST | Bearer | Execute a tool on an MCP instance |

For the full API specification with request/response schemas, see the [OpenAPI Reference](/docs/developer/openapi-reference) or visit `/swagger-ui` on your Bodhi instance.
