# Meta-Plan: Split Access Request Revamp into Phase Plans

## Task
Split the monolithic plan below into separate per-phase plan files under `ai-docs/claude-plans/20260210-access-request/`. Each phase plan is an **initial/seed plan** — exploratory, not prescriptive. Sub-agents will review, research, finalize, and implement each phase independently.

## Files to Create

```
ai-docs/claude-plans/20260210-access-request/
├── kickoff-prompt.md          # Overall goal, phases overview, sub-agent workflow
├── phase-0-keycloak-reqs.md   # KC SPI contract and prerequisites
├── phase-1-db-domain.md       # Database migration + domain objects
├── phase-2-service-layer.md   # DB repository + AuthService extension
├── phase-3-api-endpoints.md   # Backend route handlers
├── phase-4-auth-middleware.md  # Token claim extraction + toolset auth
├── phase-5-cleanup-old.md     # Remove old flow code
├── phase-6-frontend.md        # Review/approve page
├── phase-7-rust-tests.md      # Unit + API tests
├── phase-8-frontend-tests.md  # Component tests
└── phase-9-e2e-tests.md       # Real Keycloak e2e tests
```

## Approach
- Kickoff prompt explains overall goal, rough phases, notes that plans evolve during implementation
- Each phase plan is a seed: key requirements, files involved, acceptance criteria, references to context/other phases
- Sub-agent receives kickoff + specific phase plan, explores codebase, asks questions, finalizes plan, then implements
- Cross-reference context archive and KC integration doc instead of duplicating

---

# Original Plan (reference — content below is source material for splitting)

## Context

The current `/apps/request-access` endpoint is a server-to-server flow where an external app registers its scopes with Keycloak via BodhiApp acting as a proxy. The user has no visibility into what tools an app requests access to.

**Problem**: Apps get access to toolsets without explicit user consent for specific tool instances. Users cannot review, narrow, or deny specific tool access.

**Solution**: Transform into a user-approval flow where:
1. App creates a **draft** access request specifying tool types
2. BodhiApp returns a **review URL** + **access_request_id** + **scopes**
3. User opens URL (popup or redirect), logs in if needed, reviews requested tools, maps tool types to specific instances, and approves/denies
4. After approval, BodhiApp calls a **custom Keycloak SPI** to register the consent description
5. App launches OAuth flow with dynamic scope `scope_access_request:<uuid>`
6. Token includes `access_request_id` claim — BodhiApp's middleware validates it against local DB
7. Downstream APIs use access_request_id to look up approved tool instances

**Breaking changes**: Drop `app_client_toolset_configs` table and old Keycloak-based request-access flow entirely.

---

## Phase 0: Keycloak Requirements & SPI Contract

> **Execute first, before any BodhiApp changes.** Confirm KC changes are deployed to dev before proceeding.
>
> **KC Integration Doc**: `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/keycloak-bodhi-ext/ai-docs/claude-plans/20260210-access-request-doc-integration.md`
> **Context Archive**: `ai-docs/claude-plans/20260210-ctx-access-request.md`

### KC Dynamic Scope (already configured)
- `scope_access_request` is an optional dynamic scope at realm level
- Format: `scope_access_request:<access-request-uuid>`
- All clients (old and new) already have access to this scope

### KC SPI: Register Access Request Consent

**Endpoint**: `POST {auth_url}/realms/{realm}/bodhi/users/request-access`

> New path (`/users/` not `/resources/`). Uses **user token** (from resource client session), NOT client service account token.

**Authentication**: Bearer token (**user's access token** from resource client session)

**Request**:
```json
{
  "app_client_id": "app-abc123def456",
  "access_request_id": "550e8400-e29b-41d4-a716-446655440000",
  "description": "- Exa Web Search\n- ..."
}
```

**KC extracts from token + request**:
- **resource-client**: from the token's audience
- **app-client**: from `app_client_id` field (must be public client)
- **user-uuid**: from the token's `sub` claim
- Stores `access_request_id` + `description` in KC table for consent display
- Adds `scope_resource-*` to app client if not already added

**Response (201 Created)** — first registration:
```json
{
  "scope": "scope_resource-xyz789abc",
  "access_request_id": "550e8400-e29b-41d4-a716-446655440000",
  "access_request_scope": "scope_access_request:550e8400-e29b-41d4-a716-446655440000"
}
```

**Response (200 OK)** — idempotent retry (same UUID, same context):
```json
{
  "scope": "scope_resource-xyz789abc",
  "access_request_id": "550e8400-e29b-41d4-a716-446655440000",
  "access_request_scope": "scope_access_request:550e8400-e29b-41d4-a716-446655440000"
}
```

**Error Responses**:

| Status | Error | Reason |
|--------|-------|--------|
| 400 | `"access_request_id is required"` | Missing UUID |
| 400 | `"description is required"` | Missing description |
| 400 | `"App client not found"` | Invalid `app_client_id` |
| 400 | `"Only public app clients can request access"` | Confidential client used |
| 401 | `"invalid session"` | No/invalid bearer token |
| 401 | `"service account tokens not allowed"` | Used service account instead of user token |
| 401 | `"Token is not from a valid resource client"` | User token not from resource client |
| 409 | `"access_request_id already exists for a different context"` | UUID reused for different resource/app/user combo |

**Idempotency**: Same `access_request_id` with same context → 200 OK. Same UUID for different `resource_client_id`, `app_client_id`, or `user_id` → 409 Conflict (abort, regenerate UUID).

**Behavior**: When a client triggers OAuth flow with `scope_access_request:<uuid>`, Keycloak:
1. Looks up the registered consent description for that UUID
2. Displays it on the user consent screen (e.g., "Approved permission in App: \<uuid>")
3. If user consents, includes `access_request_id: "<uuid>"` as a claim in the issued token

**Note**: `scope_access_request` is already an optional client scope at realm level — no additional scope registration needed.

### KC Token Exchange Behavior
- When token exchange is called with a token containing `access_request_id` claim, the exchanged token MUST also include `access_request_id` claim
- Token exchange scope parameter must include `scope_access_request:<uuid>` to preserve the claim
- No additional KC changes needed — dynamic scope claims pass through token exchange

### Deliverables
- [x] SPI endpoint deployed at `/bodhi/users/request-access` (confirmed in KC integration doc)
- [ ] Token exchange preserves `access_request_id` claim (verified)
- [ ] Consent screen shows description when `scope_access_request:<uuid>` is requested
- [ ] Confirm with dev that these are ready before starting Phase 1

---

## Phase 1: Database & Domain Objects

### Migration 0010: `app_access_requests` table + drop `app_client_toolset_configs`

**File**: `crates/services/migrations/0010_app_access_requests.up.sql`

```sql
CREATE TABLE IF NOT EXISTS app_access_requests (
    id TEXT PRIMARY KEY,                    -- UUID (access_request_id)
    app_client_id TEXT NOT NULL,
    flow_type TEXT NOT NULL,                -- 'redirect' | 'popup'
    redirect_uri TEXT,                      -- For redirect flow only
    status TEXT NOT NULL DEFAULT 'draft',   -- 'draft' | 'approved' | 'denied'
    tools_requested TEXT NOT NULL,          -- JSON: [{"tool_type": "builtin-exa-search"}]
    tools_approved TEXT,                    -- JSON: ["<toolset-instance-uuid>", ...] (set on approval)
    user_id TEXT,                           -- NULL until user approves (attached on approval)
    resource_scope TEXT,                    -- KC-returned "scope_resource-xyz" (set after KC call)
    access_request_scope TEXT,              -- KC-returned "scope_access_request:<uuid>" (set after KC call)
    expires_at INTEGER NOT NULL,            -- Unix timestamp, draft TTL = 10 minutes
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX idx_app_access_requests_status ON app_access_requests(status);
CREATE INDEX idx_app_access_requests_app_client ON app_access_requests(app_client_id);

DROP TABLE IF EXISTS app_client_toolset_configs;
```

**Down migration**: `0010_app_access_requests.down.sql`
```sql
DROP TABLE IF EXISTS app_access_requests;
-- Recreate app_client_toolset_configs from 0008
```

### Domain Objects

**File**: `crates/services/src/db/objs.rs` (add)

```rust
pub struct AppAccessRequestRow {
    pub id: String,              // UUID (access_request_id)
    pub app_client_id: String,
    pub flow_type: String,       // "redirect" | "popup"
    pub redirect_uri: Option<String>,
    pub status: String,          // "draft" | "approved" | "denied"
    pub tools_requested: String, // JSON
    pub tools_approved: Option<String>, // JSON
    pub user_id: Option<String>,
    pub resource_scope: Option<String>,         // KC-returned "scope_resource-xyz" (set after KC call)
    pub access_request_scope: Option<String>,   // KC-returned "scope_access_request:<uuid>" (set after KC call)
    pub expires_at: i64,
    pub created_at: i64,
    pub updated_at: i64,
}
```

**File**: `crates/objs/src/access_request.rs` (new)

```rust
pub enum AppAccessRequestStatus {
    Draft,
    Approved,
    Denied,
}

pub enum AccessRequestFlowType {
    Redirect,
    Popup,
}

pub struct ToolTypeRequest {
    pub tool_type: String,  // e.g. "builtin-exa-search"
}
```

**File**: `crates/services/src/lib.rs` — update `AppAccessRequest`/`AppAccessResponse` structs

New request:
```rust
pub struct AppAccessRequest {
    pub app_client_id: String,
    pub flow_type: String,        // "redirect" | "popup"
    pub redirect_uri: Option<String>,  // required if flow_type == "redirect"
    pub tools: Vec<ToolTypeRequest>,
}
```

New response:
```rust
pub struct AppAccessResponse {
    pub access_request_id: String,
    pub review_url: String,
    pub scopes: Vec<String>,     // ["scope_resource-<id>", "scope_access_request:<uuid>"]
}
```

### Files to modify
- `crates/services/migrations/` — new 0010 up/down
- `crates/services/src/db/objs.rs` — add `AppAccessRequestRow`
- `crates/objs/src/access_request.rs` — new file for domain enums
- `crates/objs/src/lib.rs` — re-export new types
- `crates/services/src/lib.rs` — update `AppAccessRequest`/`AppAccessResponse`

---

## Phase 2: Service Layer — DB Repository + AccessRequestService

### DB Repository

**File**: `crates/services/src/db/access_request_repository.rs` (new)

```rust
#[async_trait]
pub trait AccessRequestRepository {
    async fn create_app_access_request(&self, row: &AppAccessRequestRow) -> Result<()>;
    async fn get_app_access_request(&self, id: &str) -> Result<Option<AppAccessRequestRow>>;
    async fn update_app_access_request_approval(
        &self,
        id: &str,
        user_id: &str,
        tools_approved: &str,       // JSON
        resource_scope: &str,        // KC-returned scope
        access_request_scope: &str,  // KC-returned scope
    ) -> Result<()>;
    async fn update_app_access_request_denial(
        &self,
        id: &str,
        user_id: &str,
    ) -> Result<()>;
}
```

Add to `DbService` trait (like other repositories) and implement in `DefaultDbService`.

### AuthService Extension

**File**: `crates/services/src/auth_service.rs`

Add method to `AuthService` trait:
```rust
async fn register_access_request_consent(
    &self,
    user_token: &str,           // User's access token from resource client session
    app_client_id: &str,
    access_request_id: &str,
    description: &str,
) -> Result<RegisterAccessRequestResponse>;

pub struct RegisterAccessRequestResponse {
    pub scope: String,                  // "scope_resource-xyz789abc"
    pub access_request_id: String,      // echo back
    pub access_request_scope: String,   // "scope_access_request:<uuid>"
}
```

Implementation calls `POST {auth_url}/realms/{realm}/bodhi/users/request-access` with Bearer = **user's session token**. Request body: `{ app_client_id, access_request_id, description }`.

KC returns `{ scope, access_request_id, access_request_scope }` with 201 (new) or 200 (idempotent retry).

**Error handling**:
- 409 Conflict → `access_request_id` UUID collision for different context. BodhiApp should abort and regenerate UUID. This is extremely rare since UUIDs are generated locally.
- 401 → user token invalid/expired (surface as auth error)
- 400 → validation failure (surface as bad request)

### Remove old code
- Remove `request_access()` from `AuthService` trait and `KeycloakAuthService`
- Remove `RequestAccessRequest`/`RequestAccessResponse` structs
- Remove `AppClientToolset` struct
- Remove `get_app_client_toolset_config`/`upsert_app_client_toolset_config` from `DbService`/repository
- Remove `AppClientToolsetConfigRow`
- Remove `is_app_client_registered_for_toolset` from `ToolService`

### Files to modify
- `crates/services/src/db/access_request_repository.rs` — new
- `crates/services/src/db/service.rs` — add repository impl
- `crates/services/src/db/mod.rs` — register module
- `crates/services/src/auth_service.rs` — add `register_access_request_consent`, remove `request_access`
- `crates/services/src/tool_service/service.rs` — remove `is_app_client_registered_for_toolset`
- `crates/services/src/db/toolset_repository.rs` — remove `app_client_toolset_configs` methods
- `crates/services/src/db/objs.rs` — remove `AppClientToolsetConfigRow`

---

## Phase 3: Backend API Endpoints

### 3a. POST /bodhi/v1/apps/request-access (revamped)

**File**: `crates/routes_app/src/routes_auth/request_access.rs` (rewrite)

**Handler**: `request_access_handler`

**Flow**:
1. Validate request: `flow_type` is "redirect" or "popup"; if "redirect", `redirect_uri` is required; `tools` is non-empty
2. Generate UUID for access_request_id
3. Compute `expires_at` = now + 10 minutes
4. Compute `review_url` = `{base_url}/ui/apps/review-access/{access_request_id}?flow_type={flow_type}`
5. Compute scopes: `["scope_resource-{resource_client_id}", "scope_access_request:{access_request_id}"]`
6. Insert draft into `app_access_requests` table
7. Return `{ access_request_id, review_url, scopes }`

**Error enum**: `AppRequestAccessError` (new domain error enum)
- `InvalidFlowType` — flow_type not "redirect" or "popup"
- `MissingRedirectUri` — flow_type is "redirect" but no redirect_uri
- `EmptyToolsRequest` — tools list is empty
- `AppRegInfoNotFound` — from secret_service

### 3b. GET /bodhi/v1/apps/request-access/:id (new)

**File**: `crates/routes_app/src/routes_auth/request_access.rs` (add)

**Handler**: `get_access_request_handler`

**Flow**:
1. Look up access request by id
2. If not found → 404
3. If draft and `expires_at < now` → return status "expired"
4. Return full access request details including status, tools_requested, tools_approved

**Response**:
```json
{
  "id": "<uuid>",
  "status": "draft" | "approved" | "denied" | "expired",
  "app_client_id": "...",
  "tools_requested": [{"tool_type": "builtin-exa-search"}],
  "tools_approved": ["<instance-uuid>"],       // only when approved
  "resource_scope": "scope_resource-xyz789abc", // only when approved (from KC)
  "access_request_scope": "scope_access_request:<uuid>", // only when approved (from KC)
  "created_at": "...",
  "updated_at": "..."
}
```

> **Important**: The app uses `resource_scope` and `access_request_scope` from the GET response (after approval) to launch its OAuth flow with the correct scopes.

### 3c. POST /bodhi/v1/apps/access-request/:id/approve (new, session-auth)

**File**: `crates/routes_app/src/routes_auth/access_request_review.rs` (new)

**Handler**: `approve_access_request_handler`

**Auth**: Session auth required (user is logged in via popup/redirect)

**Request**:
```json
{
  "tools_approved": ["<toolset-instance-uuid>"]
}
```

**Flow**:
1. Extract user_id from session (via `ExtractUserId`) and user token (via `ExtractToken`)
2. Look up access request by id
3. Validate: status is "draft", not expired
4. Validate: each tool instance UUID belongs to the user and is enabled with API key
5. Build consent description: `"- Exa Web Search\n- ..."` from instance → toolset type → display name
6. Call `auth_service.register_access_request_consent(user_token, app_client_id, access_request_id, description)`
7. On success (201/200): update DB row — status="approved", user_id, tools_approved, resource_scope, access_request_scope (from KC response)
8. On 409 Conflict: abort — return error to frontend (extremely rare UUID collision)
9. Return success with `{ resource_scope, access_request_scope }` for frontend to pass back to app

**Post-approval behavior** (handled by frontend):
- If `flow_type == "redirect"` → frontend redirects to `redirect_uri`
- If `flow_type == "popup"` → frontend closes window

### 3d. POST /bodhi/v1/apps/access-request/:id/deny (new, session-auth)

**Handler**: `deny_access_request_handler`

**Flow**:
1. Look up access request, validate draft + not expired
2. Update status to "denied", attach user_id
3. Return success
4. Frontend handles redirect/close

### 3e. GET /bodhi/v1/apps/access-request/:id/review (new, session-auth)

**Handler**: `get_access_request_review_handler`

**Auth**: Session auth required

**Purpose**: Frontend review page fetches this to display the access request details with tool type info and user's available instances.

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

This enriches the raw request with display names and the user's available tool instances.

### Files to modify/create
- `crates/routes_app/src/routes_auth/request_access.rs` — rewrite
- `crates/routes_app/src/routes_auth/access_request_review.rs` — new
- `crates/routes_app/src/routes_auth/mod.rs` — register new module
- `crates/routes_app/src/routes.rs` — register new routes
- `crates/routes_app/src/endpoints.rs` — add endpoint constants

---

## Phase 4: Auth Middleware Changes

### 4a. Token claim extraction

**File**: `crates/auth_middleware/src/auth_middleware.rs`

In the bearer token flow (after token exchange), extract `access_request_id` from the exchanged token's claims:
- New header constant: `KEY_HEADER_BODHIAPP_ACCESS_REQUEST_ID` = `"X-BodhiApp-Access-Request-Id"`
- If token has `access_request_id` claim → inject header

### 4b. Token exchange scope forwarding

**File**: `crates/services/src/token_service.rs` (or equivalent)

In `handle_external_client_token()`:
- Extract `scope_access_request:*` from external token scopes (in addition to existing `scope_toolset-*`)
- Forward to `auth_service.exchange_app_token()` scope list
- Per KC integration doc: scope parameter must include `scope_access_request:<uuid>` and `scope_resource-*` for the exchanged token to contain `access_request_id` claim

### 4c. Toolset auth middleware update

**File**: `crates/auth_middleware/src/toolset_auth_middleware.rs`

Replace OAuth-specific checks (steps 3-4):

**Old** (remove):
- Check `is_app_client_registered_for_toolset(azp, scope_uuid)` via `app_client_toolset_configs`
- Check `scope_toolset-*` in `X-BodhiApp-Tool-Scopes`

**New** (add):
- Extract `access_request_id` from `X-BodhiApp-Access-Request-Id` header
- Look up access request from DB: validate status=approved, not expired, `user_id` matches
- Verify the requested toolset instance UUID is in `tools_approved` list

### 4d. Typed extractor

**File**: `crates/auth_middleware/src/extractors.rs`

Add `ExtractAccessRequestId` and `MaybeAccessRequestId` extractors for the new header.

### Files to modify
- `crates/auth_middleware/src/auth_middleware.rs` — add header constant, inject `access_request_id`
- `crates/auth_middleware/src/toolset_auth_middleware.rs` — replace OAuth checks
- `crates/auth_middleware/src/extractors.rs` — new extractors
- `crates/services/src/auth_service.rs` — exchange_app_token scope handling

---

## Phase 5: Remove Old Flow Code

After new flow is working, remove all old request-access artifacts:

- Remove `X-BodhiApp-Tool-Scopes` header injection from `auth_middleware.rs`
- Remove `ToolsetScope::from_scope_string()` usage in middleware (keep type for other uses if needed)
- Remove `toolset_scopes` module constants if no longer used
- Remove `AppClientToolset` from services
- Remove `is_app_client_registered_for_toolset` from `ToolService` trait + mock
- Clean up `services/src/lib.rs` exports

### Files to modify
- `crates/auth_middleware/src/auth_middleware.rs`
- `crates/services/src/tool_service/service.rs`
- `crates/services/src/lib.rs`

---

## Phase 6: Frontend — Review/Approve Page

### 6a. New page: `/ui/apps/review-access/[id]/page.tsx`

**File**: `crates/bodhi/src/app/ui/apps/review-access/[id]/page.tsx`

**Behavior**:
1. Extract `id` from URL params, `flow_type` from query params
2. If user not logged in → store current URL in sessionStorage → redirect to login
3. After login → redirect back to review page (use existing `AppInitializer` pattern)
4. Fetch `GET /bodhi/v1/apps/access-request/{id}/review`
5. Display:
   - App name (`app_client_id`)
   - Requested tools with display names
   - For each tool type: dropdown/radio to select which instance to grant
   - Approve / Deny buttons
   - Expiry countdown (optional)
6. On Approve: `POST /bodhi/v1/apps/access-request/{id}/approve` with selected instance UUIDs
7. On Deny: `POST /bodhi/v1/apps/access-request/{id}/deny`
8. After action:
   - If `flow_type == "redirect"` → `window.location.href = redirect_uri`
   - If `flow_type == "popup"` → `window.close()`

### 6b. API hooks

**File**: `crates/bodhi/src/hooks/useQuery.ts` (extend)

```typescript
export function useAccessRequestReview(id: string) { ... }
export function useApproveAccessRequest(id: string, options?) { ... }
export function useDenyAccessRequest(id: string, options?) { ... }
```

### 6c. Route constant

**File**: `crates/bodhi/src/lib/constants.ts`

```typescript
export const ROUTE_APP_REVIEW_ACCESS = '/ui/apps/review-access';
```

### Files to create/modify
- `crates/bodhi/src/app/ui/apps/review-access/[id]/page.tsx` — new
- `crates/bodhi/src/hooks/useQuery.ts` — new hooks
- `crates/bodhi/src/lib/constants.ts` — new route constant

---

## Phase 7: Rust Unit & API Tests

### 7a. Service layer tests

- `AccessRequestRepository` CRUD tests (create, get, update approval, update denial)
- Draft expiry logic test (created 10min ago → status "expired")
- `register_access_request_consent` with mockito mock for KC endpoint (`/bodhi/users/request-access` with user token)
- Test 201 Created response parsing (first call)
- Test 200 OK response parsing (idempotent retry)
- Test 409 Conflict handling (UUID collision — abort)

### 7b. Route handler tests

- `POST /apps/request-access`: valid request → 200 with access_request_id, review_url, scopes
- `POST /apps/request-access`: invalid flow_type → 400
- `POST /apps/request-access`: redirect without redirect_uri → 400
- `POST /apps/request-access`: empty tools → 400
- `GET /apps/request-access/:id`: found → 200 with full details
- `GET /apps/request-access/:id`: not found → 404
- `GET /apps/request-access/:id`: expired draft → 200 with status "expired"
- `POST /apps/access-request/:id/approve`: valid → 200, DB updated
- `POST /apps/access-request/:id/approve`: expired → 400
- `POST /apps/access-request/:id/approve`: already approved → 400
- `POST /apps/access-request/:id/approve`: invalid instance UUID → 400
- `POST /apps/access-request/:id/deny`: valid → 200
- `GET /apps/access-request/:id/review`: enriched response with user instances

### 7c. Auth middleware tests

- Token with `access_request_id` claim → header injected
- Toolset middleware: OAuth with valid access_request_id → allowed
- Toolset middleware: OAuth with expired access_request_id → denied
- Toolset middleware: OAuth with access_request_id but instance not in approved list → denied
- Toolset middleware: Session auth → unchanged (no access_request_id check)

---

## Phase 8: Frontend Component Tests

### Test patterns (following existing MSW v2 pattern)

**File**: `crates/bodhi/src/app/ui/apps/review-access/[id]/page.test.tsx`

Mock API handlers:
- `mockAccessRequestReview()` — returns review data with tool types and instances
- `mockApproveAccessRequest()` — returns success
- `mockDenyAccessRequest()` — returns success

Test cases:
- Renders requested tools with display names
- User can select tool instances from dropdown
- Approve button calls approve API with selected instances
- Deny button calls deny API
- Shows expired state when request is expired
- Shows error when request not found
- After approve with popup flow → calls `window.close()`
- After approve with redirect flow → navigates to redirect_uri
- Loading states during API calls

---

## Phase 9: E2E Tests (Real Keycloak)

### Prerequisites
- Keycloak dev instance with:
  - `scope_access_request` dynamic scope configured
  - Custom SPI for consent registration deployed
  - Test app client registered
- Exa API key configured for test user's tool instance

### Test Journeys

**Journey 1: Happy path — Popup flow (Chrome extension pattern)**
1. App calls `POST /apps/request-access` with `flow_type=popup`, tools=[builtin-exa-search]
2. Assert response has `access_request_id`, `review_url`, `scopes`
3. Open review_url in browser
4. Login as test user
5. Review page shows "Exa Web Search" with user's instance
6. Select instance, click Approve
7. Assert DB: access_request status=approved, user_id set, tools_approved has instance UUID
8. App launches OAuth flow with `scope_access_request:<uuid>` + `scope_resource-*`
9. Complete OAuth consent
10. Assert token has `access_request_id` claim
11. Call `POST /toolsets/{instance-id}/execute/search` with bearer token
12. Assert: Exa search executes successfully

**Journey 2: Happy path — Redirect flow (3rd party app)**
1. App calls `POST /apps/request-access` with `flow_type=redirect`, `redirect_uri=http://localhost:9999/callback`
2. Open review_url
3. Login, approve
4. Assert browser redirected to `http://localhost:9999/callback`
5. Complete OAuth flow, verify token, call toolset API

**Journey 3: Deny flow**
1. Create draft request
2. Open review_url, login
3. Click Deny
4. Assert DB: status=denied
5. App polls `GET /apps/request-access/:id` → sees "denied"
6. App tries OAuth flow → consent page should not show approved tools

**Journey 4: Expired draft**
1. Create draft request
2. Wait 10 minutes (or manipulate time in test)
3. Poll `GET /apps/request-access/:id` → status "expired"
4. Open review_url → shows expired message

**Journey 5: Token validation — wrong user**
1. User A approves access request
2. User B tries to use the token (different `sub` claim)
3. Auth middleware rejects with access_request invalid error

**Journey 6: Revoked/tampered access_request_id**
1. Complete approval flow, get token
2. Delete access request from DB (simulating revocation)
3. Call toolset API → middleware rejects (not found)

---

## Verification

### Backend verification
```bash
cargo check -p objs
cargo check -p services
cargo check -p auth_middleware
cargo check -p routes_app
cargo test -p services
cargo test -p auth_middleware
cargo test -p routes_app
make test.backend
```

### Frontend verification
```bash
cd crates/bodhi && npm test
make build.ui-clean && make build.ui
```

### E2E verification
- Requires running Keycloak dev instance with SPI deployed
- Run e2e test suite against live environment

---

## Key Files Reference

| Area | File | Action |
|------|------|--------|
| Migration | `crates/services/migrations/0010_app_access_requests.up.sql` | Create |
| Migration | `crates/services/migrations/0010_app_access_requests.down.sql` | Create |
| Domain objects | `crates/objs/src/access_request.rs` | Create |
| DB row types | `crates/services/src/db/objs.rs` | Modify |
| DB repository | `crates/services/src/db/access_request_repository.rs` | Create |
| DB service | `crates/services/src/db/service.rs` | Modify |
| Auth service | `crates/services/src/auth_service.rs` | Modify (add consent, remove old) |
| Tool service | `crates/services/src/tool_service/service.rs` | Modify (remove old checks) |
| API request/response | `crates/services/src/lib.rs` | Modify |
| Route handler | `crates/routes_app/src/routes_auth/request_access.rs` | Rewrite |
| Review endpoints | `crates/routes_app/src/routes_auth/access_request_review.rs` | Create |
| Route registration | `crates/routes_app/src/routes.rs` | Modify |
| Endpoints | `crates/routes_app/src/endpoints.rs` | Modify |
| Auth middleware | `crates/auth_middleware/src/auth_middleware.rs` | Modify |
| Toolset middleware | `crates/auth_middleware/src/toolset_auth_middleware.rs` | Modify |
| Extractors | `crates/auth_middleware/src/extractors.rs` | Modify |
| Frontend page | `crates/bodhi/src/app/ui/apps/review-access/[id]/page.tsx` | Create |
| Frontend hooks | `crates/bodhi/src/hooks/useQuery.ts` | Modify |
| Frontend constants | `crates/bodhi/src/lib/constants.ts` | Modify |
| Toolset repo | `crates/services/src/db/toolset_repository.rs` | Modify (remove old) |
