# Context Archive: Revamp /apps/request-access — User-Controlled App Access Request Flow

> Gathered 2026-02-10 during planning interview and codebase exploration.

---

## 1. Requirements Summary (from user interview)

### Core Change
Transform `/apps/request-access` from a server-to-server flow (app → BodhiApp → Keycloak scope registration) to a **server-to-user-approval** flow where users explicitly review, select tool instances, and approve/deny access.

### New Flow
1. App calls `POST /apps/request-access` with `{ tools: [{ tool_type: "builtin-exa-search" }], flow_type: "redirect"|"popup", redirect_uri: "..." }`
2. BodhiApp creates a **draft** `AppAccessRequest` in local DB (no user attached yet)
3. Returns `{ access_request_id, review_url, scopes: ["scope_resource-<id>", "scope_access_request:<uuid>"] }`
4. App opens `review_url` in popup (Chrome extension) or redirect (3rd party app)
5. User logs in if needed (URL stored in browser sessionStorage for post-login redirect)
6. User reviews requested tools, maps tool_type to specific configured instance, approves/denies
7. On approval: BodhiApp calls Keycloak SPI to register consent description
8. For redirect: browser redirects to app's `redirect_uri`; for popup: `window.close()`
9. App launches OAuth flow with dynamic scopes including `scope_access_request:<uuid>`
10. Keycloak consent screen shows registered description
11. Token includes `access_request_id` claim
12. BodhiApp's auth_middleware validates `access_request_id` against local DB
13. Downstream APIs use access_request_id to look up approved tool instances

### Tool Identification
- Moving from UUID-based tool scoping to **human-identifiable tool type** identifiers
- Format: `builtin-exa-search` (not a UUID)
- Users can have multiple configured instances of same tool type
- User selects which specific instance to grant access to during approval
- Only instance UUID stored in approval (instance record has type info)

---

## 2. Interview Q&A

### Keycloak Role
**Q**: Is Keycloak being removed from the access request lifecycle entirely?
**A**: Keycloak still handles OAuth token issuance (login, authorization code flow), but BodhiApp manages permission semantics locally via `access_request_id` DB lookup. Keycloak mints the token with the dynamic scope claim.

### Narrowing Access
**Q**: What dimensions can users modify when narrowing access?
**A**: MVP is tool selection only. App requests access to a tool type, user can deny but allow other API access. User maps tool_type to a specific configured instance.

### Post-Approval Flow
**Q**: What does the invoking app receive after approval?
**A**: OAuth flow is still required. Two modes:
- **Redirect**: For 3rd party apps. App provides `redirect_uri` with its own query params encoded. After approval, browser redirects to that URI.
- **Popup**: For Chrome extensions. Extension detects window close, uses UUID from draft response to poll status.

Apps send `flow_type` (redirect/popup) in the draft request. The returned review URL encodes the behavior. Once approval completes, the app launches OAuth with scopes: `scope_resource-*` + `scope_access_request:<uuid>`. Token received has `aud: resource-client-id` and `access_request_id: "<uuid>"` claim.

During token exchange, same scopes are forwarded, so exchanged token also has `access_request_id` claim. Auth middleware validates it. The approved tools from the access request are also injected for downstream APIs.

### Auth Context for Popup
**Q**: Is the user already authenticated when popup opens?
**A**: User may need to log in. Store the review URL in browser session. After login, redirect to approve page. Until approval, the draft request has no user_id. On approval, user_id is attached. During token validation, `sub` (from token) must match access request `user_id`.

### Draft Request Payload
**Q**: What does the app send when creating a draft?
**A**: `{ tools: [{ tool_type: "builtin-exa-search" }] }` — minimal and flexible. Future entities (MCP, storage) can be added with backwards-compatible format. User sees tool instances for the requested types and selects one per type.

### Middleware Performance
**Q**: Cache or per-request DB lookup for access_request_id validation?
**A**: Per-request SQLite lookup. Caching was only needed to avoid Keycloak network calls. SQLite is local and fast. No premature optimization.

### Revocation
**Q**: Can access requests be revoked after approval?
**A**: User can revoke, but NOT part of MVP.

### Multi-Draft Behavior
**Q**: What happens if same app creates multiple drafts?
**A**: New draft each time. Simple for MVP.

### Token Exchange Headers
**Q**: What information injected for downstream handlers?
**A**: Inject `access_request_id` only via `X-BodhiApp-Access-Request-Id` header. Handlers look up approved instances from DB when needed.

### Consent Screen Text
**Q**: What does the Keycloak consent screen show?
**A**: Simple bullet points: `"- Exa Web Search\n- ..."` — one line per approved tool type display name.

### Scope
**Q**: Backend only or full stack?
**A**: Full stack: Backend+Rust tests first, then Frontend+component tests with mock API, then E2E tests with live Keycloak and Exa.

### Access Request States
**Q**: State machine?
**A**: Simple: `draft` → `approved` | `denied`. No intermediate "pending_review" state.

### Draft Expiry
**Q**: Should drafts have TTL?
**A**: 10-minute TTL. Status check returns "expired" if exceeded. Approved requests don't expire.

### Status Check Endpoint
**Q**: How does the app check status?
**A**: `GET /apps/request-access/:id` returns full access request details including status, approved tools.

### KC Dynamic Scope
**Q**: Does KC need configuration for `scope_access_request:<uuid>`?
**A**: Already configured. `scope_access_request` is an optional dynamic scope at realm level. All clients have it.

### Old Flow
**Q**: What happens to existing `app_client_toolset_configs` table?
**A**: Drop entirely in new migration. No backwards compatibility.

### KC SPI Endpoint (Updated)
**Q**: Is this a custom SPI?
**A**: Yes, custom SPI. But we reuse the **existing** endpoint path:
- `POST {auth_url}/realms/{realm}/bodhi/resources/request-access`
- Use **user token** (not client service token) in Authorization header
- New request format: `{ app_client_id: ..., access_request_id: ..., description: string }`
- This allows Keycloak to extract resource-client, app-client, user-uuid and store the access_request + description in a table
- When KC receives dynamic scope `scope_access_request:<uuid>`, it displays the stored description
- The SPI already adds resource client scope to app client
- `scope_access_request` client scope is optional at realm level, no additional scope registration needed

### Draft Response Scopes
**Q**: What scopes in draft response?
**A**: Just `scope_resource-*` and `scope_access_request:<uuid>`. App already knows its own token-level scopes.

### E2E Testing
**Q**: Mock or real Keycloak?
**A**: Real Keycloak e2e tests against dev instance.

---

## 3. Current Architecture (Exploration Results)

### Current `/apps/request-access` Handler
**File**: `crates/routes_app/src/routes_auth/request_access.rs`

Current flow is server-to-server:
1. Validate app registration info from SecretService
2. Check cache by `app_client_id` + version
3. On cache miss, call Keycloak `POST /realms/{realm}/bodhi/resources/request-access`
4. Cache response in `app_client_toolset_configs` table
5. Return `AppAccessResponse { scope, toolsets: Vec<AppClientToolset>, app_client_config_version }`

### Current Auth Middleware Token Flow
**File**: `crates/auth_middleware/src/auth_middleware.rs`

Headers injected after token validation:
- `X-BodhiApp-Token`, `X-BodhiApp-Username`, `X-BodhiApp-Role`
- `X-BodhiApp-Scope`, `X-BodhiApp-User-Id`
- `X-BodhiApp-Tool-Scopes` — space-separated `scope_toolset-*` scopes from token
- `X-BodhiApp-Azp` — authorized party (app-client ID)

Token exchange in `handle_external_client_token()`:
1. Parse external token, extract issuer + azp
2. Validate issuer/audience
3. Extract `scope_user_*` and `scope_toolset-*` scopes
4. Call `auth_service.exchange_app_token()` with scopes
5. Return exchanged access token + resource scope + original azp

### Current Toolset Auth Middleware
**File**: `crates/auth_middleware/src/toolset_auth_middleware.rs`

4-layer authorization for OAuth:
1. Extract `user_id` → verify ownership
2. Check app-level type enabled (`app_toolset_configs.enabled`)
3. Check app-client registered (`app_client_toolset_configs` — **being removed**)
4. Check `scope_toolset-*` in `X-BodhiApp-Tool-Scopes` header — **being replaced**
5. Check toolset is available (enabled + has API key)

### Current Exa Tool Invocation Chain
**File**: `crates/routes_app/src/routes_toolsets/toolsets.rs`

Route: `POST /toolsets/{id}/execute/{method}`
Middleware chain: `auth_middleware` → `api_auth_middleware` → `toolset_auth_middleware` → handler
Handler delegates to `tool_service.execute()` which:
1. Fetches instance from DB, verifies ownership
2. Checks app-level type enabled
3. Checks instance enabled
4. Gets decrypted API key
5. Validates method exists in toolset definition
6. Dispatches to `execute_exa_method()` → `ExaService`

### Database Schema
- Latest migration: `0009_user_aliases` — new migration will be `0010`
- `app_client_toolset_configs` table (to be dropped): caches per-app-client toolset registration from Keycloak
- `toolsets` table: user-owned tool instances with encrypted API keys
- `app_toolset_configs` table: admin-controlled per-toolset-type enable/disable

### ToolsetScope Domain Object
**File**: `crates/objs/src/toolsets.rs`

- `ToolsetScope` struct with `scope: String` field
- Static registry: `ToolsetScope::all()`, `builtin_exa_web_search()`
- Parsing: `from_scope_string()` extracts known scopes from space-separated string
- Only one builtin: `scope_toolset-builtin-exa-web-search`

### Frontend Patterns
**File**: `crates/bodhi/src/app/ui/request-access/page.tsx` (existing user access request page — different from app access)

Patterns:
- Page wrapped with `AppInitializer` for auth enforcement
- Separate `Page` + `Content` components
- React Query hooks in `hooks/useQuery.ts` (useQuery for reads, useMutationQuery for writes)
- MSW v2 for API mocking in component tests
- `data-testid` attributes for test selectors
- Dialog component from `@radix-ui/react-dialog`
- Login redirect via `handleSmartRedirect()` in `lib/utils.ts`
- Browser sessionStorage for storing redirect URLs

### Auth Service Keycloak Integration
**File**: `crates/services/src/auth_service.rs`

- `request_access()` method calls `POST /realms/{realm}/bodhi/resources/request-access`
- Uses `get_client_access_token()` for service account auth
- Bodhi API URL: `{auth_url}/realms/{realm}/bodhi`
- Token exchange: `exchange_app_token()` calls KC token endpoint with RFC 8693 grant type

### Services DB Objects
**File**: `crates/services/src/db/objs.rs`

- `AppClientToolsetConfigRow` — to be removed
- `ToolsetRow` — stays (user tool instances)
- `AppToolsetConfigRow` — stays (admin tool type config)
- `UserAccessRequest` — existing user role access requests (separate system)

---

## 4. Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Keycloak role | Auth only, permissions local | Reduces KC dependency, faster local validation |
| Access narrowing | Tool selection only (MVP) | Keep simple, extensible later |
| States | draft → approved/denied | No intermediate states needed |
| Draft TTL | 10 minutes | Short enough to prevent stale requests |
| Multi-draft | New each time | Simple, no replacement logic |
| Token injection | access_request_id only | Handlers look up from DB as needed |
| DB lookup strategy | Per-request SQLite | No premature caching, SQLite is fast |
| Consent text | Tool type display names | Simple "- Exa Web Search" bullets |
| Revocation | Not in MVP | Can add later via user settings UI |
| Scope | Full stack (BE → FE → E2E) | Complete feature delivery |
| KC SPI endpoint | Reuse existing `/bodhi/resources/request-access` | Same endpoint, new request format with user token |
| KC auth for SPI | User token (not client service token) | Allows KC to extract user-uuid from token |
| KC scope registration | Not needed | `scope_access_request` already optional at realm level |
