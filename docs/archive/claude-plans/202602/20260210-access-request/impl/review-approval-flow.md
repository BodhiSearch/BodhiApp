# Review & Approval Flow

> **Purpose**: Full-stack trace of user review, approval, and denial of access requests.
> Covers: review endpoint, approve/deny handlers, privilege escalation guard, KC consent registration, frontend review page.

---

## Endpoints

| Endpoint | Method | Auth | Handler |
|----------|--------|------|---------|
| `/bodhi/v1/access-requests/{id}/review` | GET | Session | `apps_get_access_request_review` |
| `/bodhi/v1/access-requests/{id}/approve` | PUT | Session | `apps_approve_access_request` |
| `/bodhi/v1/access-requests/{id}/deny` | POST | Session | `apps_deny_access_request` |

All require session authentication. Registered in the session-auth router in `crates/routes_app/src/routes.rs`.

---

## Review: GET /access-requests/{id}/review

### Handler

**File**: `crates/routes_app/src/apps/routes_apps.rs`

1. Fetch access request by ID (no tenant filtering — allows drafts with NULL tenant)
2. Deserialize `requested` JSON to `RequestedResources`
3. Fetch user's available resources:
   - `auth_scope.tools().list()` — all user's toolset instances
   - `auth_scope.mcps().list()` — all user's MCP instances
4. **Enrich toolset types**: For each requested toolset type:
   - Get tool definition via `tools_svc.get_type()` -> name, description
   - Filter user's toolsets matching that type
   - Build `ToolTypeReviewInfo` with type metadata + available instances
5. **Enrich MCP servers**: For each requested MCP server URL:
   - Filter user's MCPs with matching server URL
   - Build `McpServerReviewInfo` with URL + available instances
6. Return `AccessRequestReviewResponse`

### Response Shape

```json
{
  "id": "01HXYZ...",
  "app_client_id": "my-app",
  "app_name": null,
  "app_description": null,
  "flow_type": "Popup",
  "status": "Draft",
  "requested_role": "scope_user_user",
  "requested": { "toolset_types": [...], "mcp_servers": [...] },
  "tools_info": [
    {
      "toolset_type": "builtin-exa-search",
      "name": "Exa Search",
      "description": "...",
      "instances": [
        { "id": "instance-ulid", "slug": "my-exa", "enabled": true, "has_api_key": true }
      ]
    }
  ],
  "mcps_info": [
    {
      "url": "https://mcp.example.com/mcp",
      "instances": [
        { "id": "mcp-ulid", "slug": "my-mcp", "enabled": true }
      ]
    }
  ]
}
```

---

## Approve: PUT /access-requests/{id}/approve

### Handler

**File**: `crates/routes_app/src/apps/routes_apps.rs`

### Request Body

```json
{
  "approved_role": "scope_user_user",
  "approved": {
    "toolsets": [
      { "toolset_type": "builtin-exa-search", "status": "approved", "instance": { "id": "instance-ulid" } }
    ],
    "mcps": [
      { "url": "https://mcp.example.com/mcp", "status": "approved", "instance": { "id": "mcp-ulid" } }
    ]
  }
}
```

### Handler Logic (10 Steps)

**Step 1-3: Extract auth context**
- `user_id` from auth scope
- `token` from auth context (needed for KC call)
- `tenant_id` from auth scope
- `approver_role` from `AuthContext::Session { role }` or `MultiTenantSession { role }`
- Missing role -> `InsufficientPrivileges` (403)

**Step 4: Compute max grantable scope**
```
if approver_role >= ResourceRole::PowerUser -> UserScope::PowerUser
else -> UserScope::User
```

**Step 5: Fetch access request** to get `requested_role`

**Step 6: Privilege escalation check (two levels)**
- `approved_role > requested_role` -> `PrivilegeEscalation` (403) — can't grant more than app asked for
- `approved_role > max_grantable` -> `PrivilegeEscalation` (403) — can't grant more than approver's level allows

**Step 7: Validate each approved toolset instance**
For each toolset with `status == Approved`:
- `instance` must be present (required)
- `auth_scope.tools().get(&instance.id)` — verifies user owns it
- Type must match the approval's `toolset_type`
- Must be `enabled`
- Must have `encrypted_api_key` (configured)

**Step 8: Validate each approved MCP instance**
For each MCP with `status == Approved`:
- `instance` must be present
- `auth_scope.mcps().get(&instance.id)` — verifies user owns it
- Server URL must match the approval's `url`
- Must be `enabled`

**Step 9: Call service layer**
```
access_request_service.approve_request(
  id, user_id, tenant_id, token,
  tool_approvals, mcp_approvals, approved_role
)
```

**Step 10: Return response**
```json
{ "status": "Approved", "flow_type": "Popup", "redirect_url": null }
```

### Service Layer: approve_request()

**File**: `crates/services/src/app_access_requests/access_request_service.rs`

1. Fetch request, validate status is Draft (not expired/processed)
2. Generate description from approvals (human-readable for KC consent screen)
3. **Call Keycloak**: `auth_service.register_access_request_consent(user_token, app_client_id, id, description)`
   - On success: extract `access_request_scope` from response
   - On **409 Conflict**: Mark as Failed via `update_failure()`, return the failed record (not an error)
   - On other errors: Return `KcRegistrationFailed`
4. Serialize `approved` JSON from tool/MCP approval arrays
5. Update DB via `update_approval(id, user_id, tenant_id, approved_json, approved_role, access_request_scope)`
   - This binds `tenant_id` (draft -> tenant-scoped)
   - Sets status to Approved

### Repository: update_approval()

1. Direct lookup (bypasses RLS) to confirm record exists
2. Start tenant transaction with the approver's tenant_id
3. Re-read within transaction, validate: still Draft, not expired
4. Update: `tenant_id`, `status`, `user_id`, `approved`, `approved_role`, `access_request_scope`, `updated_at`
5. Commit

---

## Deny: POST /access-requests/{id}/deny

### Handler

**File**: `crates/routes_app/src/apps/routes_apps.rs`

Simple flow:
1. Extract `user_id` from auth scope
2. Call `access_request_service.deny_request(id, user_id)`
3. Return `AccessRequestActionResponse { status: Denied, flow_type, redirect_url }`

### Service Layer: deny_request()

1. Fetch request, validate status is Draft
2. Call `db_service.update_denial(id, user_id)`
   - Sets `status = Denied`, `user_id = denier`
   - Does NOT set `tenant_id` (remains NULL)

---

## Role Logic

### Role Hierarchy

**ResourceRole** (session-based, from KC): `User < PowerUser < Manager < Admin`

**UserScope** (external app scope): `User < PowerUser`

**Mapping**: `ResourceRole.max_user_scope()`:
- `ResourceRole::User` -> `UserScope::User`
- `ResourceRole::PowerUser+` -> `UserScope::PowerUser`

### Approval Decision Matrix

| Approver Role | Requested Role | Can Approve? | Available Options |
|---------------|---------------|--------------|-------------------|
| resource_user | scope_user_power_user | Yes (downgrade only) | [scope_user_user] |
| resource_user | scope_user_user | Yes | [scope_user_user] |
| resource_power_user+ | scope_user_power_user | Yes | [scope_user_power_user, scope_user_user] |
| resource_power_user+ | scope_user_user | Yes | [scope_user_user] |

---

## Frontend Review Page

**File**: `crates/bodhi/src/app/ui/apps/access-requests/review/page.tsx`

### Data Fetching

- `useAppAccessRequestReview(id)` — fetches review data
- `useUser()` — fetches current user info (for role computation)

### Role Dropdown

`computeRoleOptions(requestedRole, userRole)`:
1. Map `requestedRole` to index in `SCOPE_ORDER = ['scope_user_power_user', 'scope_user_user']`
2. Map user's resource role to `maxGrantable` scope
3. Available options = all scopes from `max(requestedIndex, maxGrantableIndex)` onward
4. Default selection = first (most permissive) available option

### Tool/MCP Selection

State managed with React hooks:
- `selectedInstances`: `Record<toolset_type, instance_id>` — which instance per tool type
- `approvedTools`: `Record<toolset_type, boolean>` — checkbox state per tool type
- `selectedMcpInstances`: `Record<url, instance_id>` — which instance per MCP server
- `approvedMcps`: `Record<url, boolean>` — checkbox state per MCP

### Validation (canApprove)

For each requested tool: if approved, must have valid instances (enabled + has_api_key) and one selected.
For each requested MCP: if approved, must have valid instances (enabled) and one selected.
`approvedRole` must be non-null.

### Approval Submission

Builds `ApproveAccessRequest` from state:
- Each tool/MCP maps to `status: "approved"` or `"denied"` based on checkbox
- Approved entries include `instance: { id }` from selection
- Calls `PUT /access-requests/{id}/approve`

### Post-Action Behavior

- **Popup flow**: `window.close()` — closes the popup window
- **Redirect flow**: `window.location.href = redirect_url` — navigates to callback
- **Already processed**: Shows status badge, auto-closes popup after delay

### Post-Login Redirect

If user isn't logged in when opening the review URL:
- `sessionStorage('bodhi-return-url')` preserves the review URL across the login redirect

### Component Cards

**ToolTypeCard** (`ToolTypeCard.tsx`):
- Shows tool name, description
- Checkbox to approve/deny
- Instance select dropdown (filtered: enabled + has_api_key)
- Alert when no valid instances available

**McpServerCard** (`McpServerCard.tsx`):
- Shows server URL badge
- Checkbox to approve/deny
- Instance select dropdown (filtered: enabled)
- Alert when no valid instances available

---

## Key Files

| File | Role |
|------|------|
| `crates/routes_app/src/apps/routes_apps.rs` | All three handlers |
| `crates/routes_app/src/apps/apps_api_schemas.rs` | DTOs |
| `crates/routes_app/src/apps/error.rs` | Route errors incl. PrivilegeEscalation |
| `crates/services/src/app_access_requests/access_request_service.rs` | approve_request(), deny_request() |
| `crates/services/src/app_access_requests/access_request_repository.rs` | update_approval(), update_denial() |
| `crates/services/src/auth/auth_service.rs` | register_access_request_consent() |
| `crates/services/src/auth/auth_objs.rs` | ResourceRole, UserScope, max_user_scope() |
| `crates/bodhi/src/app/ui/apps/access-requests/review/page.tsx` | Frontend review page |
| `crates/bodhi/src/app/ui/apps/access-requests/review/ToolTypeCard.tsx` | Tool instance selector |
| `crates/bodhi/src/app/ui/apps/access-requests/review/McpServerCard.tsx` | MCP instance selector |
| `crates/bodhi/src/hooks/useAppAccessRequests.ts` | React Query hooks |
