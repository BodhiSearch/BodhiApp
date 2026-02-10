# Phase 3: Backend API Endpoints

## Purpose

Implement HTTP route handlers for the new access request flow: create draft, fetch details, review, approve, deny.

## Dependencies

- **Phase 2**: Service layer (repository + AuthService) implemented

## Key Endpoints

### 3a. POST /bodhi/v1/apps/request-access (revamped)

**File**: `crates/routes_app/src/routes_auth/request_access.rs` (rewrite existing)

**Handler**: `request_access_handler`

**Request**:
```json
{
  "app_client_id": "app-abc123def456",
  "flow_type": "popup",            // "redirect" | "popup"
  "redirect_uri": "http://...",    // required if flow_type == "redirect"
  "tools": [
    {"tool_type": "builtin-exa-search"}
  ]
}
```

**Flow**:
1. Validate request:
   - `flow_type` is "redirect" or "popup"
   - If "redirect", `redirect_uri` is required and valid
   - `tools` is non-empty
   - `app_client_id` exists (via `secret_service`)
2. Generate UUID for `access_request_id`
3. Compute `expires_at` = now + 10 minutes
4. Compute `review_url` = `{base_url}/ui/apps/review-access/{access_request_id}?flow_type={flow_type}`
5. Compute `scopes`: `["scope_resource-{resource_client_id}", "scope_access_request:{access_request_id}"]`
6. Insert draft into `app_access_requests` table
7. Return `{ access_request_id, review_url, scopes }`

**Error handling**: Create `AppRequestAccessError` enum with variants:
- `InvalidFlowType` — flow_type not "redirect" or "popup"
- `MissingRedirectUri` — flow_type is "redirect" but no redirect_uri
- `EmptyToolsRequest` — tools list is empty
- `AppRegInfoNotFound` — app client not registered

### 3b. GET /bodhi/v1/apps/request-access/:id (new)

**Handler**: `get_access_request_handler`

**Flow**:
1. Look up access request by id
2. If not found → 404
3. If draft and `expires_at < now` → return status "expired"
4. Return full access request details

**Response**:
```json
{
  "id": "<uuid>",
  "status": "draft" | "approved" | "denied" | "expired",
  "app_client_id": "...",
  "tools_requested": [{"tool_type": "builtin-exa-search"}],
  "tools_approved": ["<instance-uuid>"],       // only when approved
  "resource_scope": "scope_resource-xyz789abc", // only when approved
  "access_request_scope": "scope_access_request:<uuid>", // only when approved
  "created_at": "...",
  "updated_at": "..."
}
```

> **Important**: The app uses `resource_scope` and `access_request_scope` from this response to launch OAuth flow with correct scopes.

### 3c. GET /bodhi/v1/apps/access-request/:id/review (new, session-auth)

**File**: `crates/routes_app/src/routes_auth/access_request_review.rs` (new)

**Handler**: `get_access_request_review_handler`

**Auth**: Session auth required (user logged in)

**Purpose**: Frontend review page fetches this to display access request with enriched tool info.

**Response**:
```json
{
  "id": "<uuid>",
  "app_client_id": "...",
  "flow_type": "popup",
  "tools_requested": [
    {
      "tool_type": "builtin-exa-search",
      "display_name": "Exa Web Search",
      "user_instances": [
        { "id": "<uuid>", "name": "My Exa Search", "enabled": true, "has_api_key": true }
      ]
    }
  ],
  "expires_at": "..."
}
```

**Flow**:
1. Extract user_id from session
2. Look up access request
3. For each tool_type in tools_requested:
   - Fetch toolset type metadata (display_name)
   - Fetch user's instances of that type (enabled, with API key)
4. Return enriched response

### 3d. POST /bodhi/v1/apps/access-request/:id/approve (new, session-auth)

**Handler**: `approve_access_request_handler`

**Auth**: Session auth required

**Request**:
```json
{
  "tools_approved": ["<toolset-instance-uuid>"]
}
```

**Flow**:
1. Extract user_id from session and user token (via `ExtractToken`)
2. Look up access request by id
3. Validate: status is "draft", not expired
4. Validate: each tool instance UUID belongs to the user and is enabled with API key
5. Build consent description: `"- Exa Web Search\n- ..."` from instance → toolset type → display name
6. Call `auth_service.register_access_request_consent(user_token, app_client_id, access_request_id, description)`
7. On success (201/200): update DB row — status="approved", user_id, tools_approved, resource_scope, access_request_scope (from KC response)
8. On 409 Conflict: abort — return error (UUID collision, extremely rare)
9. Return success with `{ resource_scope, access_request_scope }`

**Error handling**: Handle validation errors, expired requests, KC errors.

### 3e. POST /bodhi/v1/apps/access-request/:id/deny (new, session-auth)

**Handler**: `deny_access_request_handler`

**Flow**:
1. Extract user_id from session
2. Look up access request, validate draft + not expired
3. Update status to "denied", attach user_id
4. Return success

## Files to Create/Modify

| File | Action | Description |
|------|--------|-------------|
| `crates/routes_app/src/routes_auth/request_access.rs` | Rewrite | New POST handler |
| `crates/routes_app/src/routes_auth/access_request_review.rs` | Create | Review/approve/deny handlers |
| `crates/routes_app/src/routes_auth/mod.rs` | Modify | Register new module |
| `crates/routes_app/src/routes.rs` | Modify | Register new routes |
| `crates/routes_app/src/endpoints.rs` | Modify | Add endpoint constants |
| `crates/objs/src/errors/app_request_access_error.rs` | Create | New error enum |
| `crates/objs/src/errors/mod.rs` | Modify | Export new error |

## Research Questions

1. **Extractors**: How do we extract user_id from session? (Check `ExtractUserId` extractor)
2. **Token extraction**: How do we extract user token for KC call? (Check `ExtractToken` or similar)
3. **Base URL**: How do we get base URL for constructing `review_url`? (Check existing handlers)
4. **Resource client ID**: Where do we get `resource_client_id` for scope construction? (Check config/state)
5. **Tool validation**: How do we check if user owns a tool instance and it has API key? (Check `ToolService`)
6. **Error responses**: How do we convert domain errors to HTTP responses? (Check existing error handling)
7. **Session auth**: What's the pattern for session-protected routes? (Check existing session routes)

## Acceptance Criteria

### POST /apps/request-access
- [ ] Validates flow_type (redirect/popup)
- [ ] Validates redirect_uri required for redirect flow
- [ ] Validates tools non-empty
- [ ] Generates UUID and stores draft in DB
- [ ] Returns correct response with review_url and scopes
- [ ] Error handling for invalid inputs

### GET /apps/request-access/:id
- [ ] Returns full access request details
- [ ] Returns "expired" status for expired drafts
- [ ] Returns 404 for not found
- [ ] Includes KC-returned scopes for approved requests

### GET /apps/access-request/:id/review
- [ ] Requires session auth
- [ ] Enriches tool_type with display_name
- [ ] Includes user's available instances per tool type
- [ ] Filters instances to only enabled with API key
- [ ] Returns 401 for unauthenticated users

### POST /apps/access-request/:id/approve
- [ ] Requires session auth
- [ ] Validates tool instances belong to user
- [ ] Validates tool instances are enabled with API key
- [ ] Calls KC consent registration SPI
- [ ] Updates DB with approved status and KC-returned scopes
- [ ] Returns scopes for frontend to use
- [ ] Handles 409 Conflict from KC gracefully

### POST /apps/access-request/:id/deny
- [ ] Requires session auth
- [ ] Updates DB with denied status
- [ ] Returns success

### Testing
- [ ] Unit tests for each handler (see test-routes-app skill)
- [ ] Mock service layer (AuthService, DbService, ToolService)
- [ ] Test error paths (expired, not found, validation failures)
- [ ] `cargo test -p routes_app` passes

## Notes for Sub-Agent

- **Use test-routes-app skill**: Follow canonical test patterns for routes_app
- **Session auth**: Look at existing session-protected routes for pattern
- **Extractors**: Check `crates/auth_middleware/src/extractors.rs` for user_id extraction
- **Error handling**: Map service errors to HTTP status codes (see existing error mappers)
- **Tool validation**: ToolService likely has methods to check ownership and API key
- **Base URL**: May be in RouterState or config — check existing URL construction
- **OpenAPI**: After implementation, run `cargo run --package xtask openapi` to update specs

## Next Phase

Phase 4 will update auth middleware to use these new access requests for validation.
