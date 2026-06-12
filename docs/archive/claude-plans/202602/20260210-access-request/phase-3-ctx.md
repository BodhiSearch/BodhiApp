# Phase 3 Context: API Endpoints - Q&A Session

## Session Summary

Comprehensive interview conducted before Phase 3 implementation planning. All design decisions captured below.

---

## 1. End-to-End Flow

**Q: Who calls POST /apps/request-access and how?**

The third-party app uses **Local Network Access (LNA)** browser feature to directly call the BodhiApp server running locally or on the user's network.

**Flow:**
1. App frontend calls `POST /bodhi/v1/apps/request-access` via LNA
2. Request body: `{app_client_id, flow_type, redirect_url?, requested: {tool_types: [{tool_type:'builtin-exa-search'}]}}`
3. Response: `{access_request_id, review_url, status: 'draft', ...}`
4. App opens `review_url` in popup or redirect (based on `flow_type`)
5. User sees review page (login required if not authenticated)
6. User approves/denies
7. Popup: window closes, parent app detects close, polls status
8. Redirect: redirects to app's redirect_url, app fetches status

**Key insight:** `access_request_id` must be appended to `redirect_url` by the server since it's not known at request time.

---

## 2. Authentication

**Q: What auth does POST /apps/request-access use?**

- CORS is already handled by BodhiApp
- POST endpoint is **completely unauthenticated** (no session, no token)
- The `app_client_id` in the body provides identification
- The review page (GET /review) uses **session auth** (user must be logged in)
- If user not logged in when visiting review_url: `post_login_url` stored in session, user redirected to login, after login redirected back to review page

---

## 3. Result Notification

**Q: How does the third-party app learn the approval/denial result?**

- **Redirect flow:** Nothing appended to the app's redirect_url. App already knows `access_request_id` (it can encode it in its own redirect_url). After redirect, app calls GET to fetch latest status.
- **Popup flow:** Parent window opened the popup, detects when child window closes. Then fetches latest access request status using the id.
- **UI requirement:** If `flow_type=redirect`, review page redirects to stored `redirect_url` after action. If `flow_type=popup`, review page closes the window after action.

---

## 4. App Client Registration (app_client_id)

**Q: Where does app_client_id come from?**

- `app_client_id` IS a **Keycloak client_id**
- Third-party apps register as OAuth clients in the BodhiApp KC realm
- Validation: Call KC endpoint `/users/apps/{app-client-id}/info` passing user access token (for rate limiting/tracking)
- Returns app name and description for display on review page
- 404 from KC means app client doesn't exist → return error
- Store `app_name` and `app_description` columns in DB

**Tech Debt:** Add `/apps/{app-client-id}/info` endpoint to Keycloak SPI. User will create this KC endpoint. AuthService will have a method to call it.

**Note:** Since POST /apps/request-access is unauthenticated (no user token), the KC validation with user token may happen at review time. The AuthService method can use admin token or user token depending on context.

---

## 5. Tool Type Identifier

**Q: What identifier do apps use to request tool types?**

- A **new `tool_type` field** on ToolsetDefinition (e.g., `'builtin-exa-search'`)
- **Remove `scope` and `scope_uuid` from ToolsetDefinition** - moving away from KC scope-based permissions
- New permission model: single `access_request` claim in token, BodhiApp checks access_request table
- This is a **clean, complete removal** with no backwards compatibility

---

## 6. Permission Model Change

**Q: How does the new auth model work?**

- **Old:** KC scope claims on OAuth token → auth middleware checks scopes → tool access
- **New:** Token has `access_request_id` claim → BodhiApp looks up access_request in DB → checks `approved` column for tool permissions
- The `access_request_scope` (e.g., `scope_access_request:<uuid>`) is a dynamic KC scope that adds the access_request claim to the token
- Apps include `access_request_scope` in their OAuth flow to get the claim

---

## 7. Auto-Approve Flow

**Q: What happens when no tools are requested?**

- If `requested` is empty or has no `tool_types`: **auto-approve**
- Server directly calls KC `register_access_request_consent()` (same method as manual approve)
- Returns response with `status: 'approved'`, `resource_scope`, `access_request_scope`
- App sees 'approved' status and proceeds directly to OAuth flow (no review needed)

---

## 8. Request/Response Structure

**Q: What's the `requested` JSON wrapper?**

- API request body uses `requested: {tool_types: [{tool_type: '...'}]}`
- Future-proof: when new entity types emerge (MCP, workspace), add new keys under `requested`
- DB column renamed from `tools_requested` to `requested` (modify migration directly, no release yet)
- DB column renamed from `tools_approved` to `approved`
- `resource_scope` and `access_request_scope` columns unchanged (still relevant for KC)

---

## 9. Approve Endpoint

**Q: How does the approve flow work?**

- **PUT /access-request/:id/approve** (session auth)
- **Single step:** saves selections + calls KC + updates status = all in one request
- Request body: `{approved: {tool_types: [{tool_type: 'builtin-exa-search', status: 'approved', instance_id: '...'}, {tool_type: 'other-tool', status: 'denied'}]}}`
- Each tool_type in approved has individual `status` (approved/denied)
- `instance_id` only present when status = 'approved'

---

## 10. Deny Endpoint

**Q: Is deny separate from approve?**

- **Yes, keep separate** POST /access-request/:id/deny endpoint
- Need to track approve/deny at the access request level (overall status)
- **No body needed** for deny - simple action with session user_id
- Sets status = 'denied' and records user_id

---

## 11. Polling Endpoint Authentication

**Q: How is GET /apps/access-request/:id secured?**

- Require `app_client_id` as query parameter
- Validate: access_request.app_client_id must match the query param
- Return **404 (not found)** if app_client_id is missing or doesn't match
- UUID + app_client_id combination is hard to guess

---

## 12. Toolset Scope Removal (Included in Phase 3)

**Q: What's the scope of the scope removal?**

- Phase 3 includes clean removal of `scope` and `scope_uuid` from:
  - `ToolsetDefinition` struct → replace with `tool_type`
  - `Toolset` struct → replace with `tool_type`
  - `ToolService` methods → use `tool_type` instead of `scope_uuid`
  - Toolset routes → use `tool_type` instead of scope-based filtering
- `AppToolsetConfig` and `app_toolset_configs` table already dropped in migration 0010
- **New migration 0012** needed for `app_toolsets` table: add `tool_type`, drop `scope`, `scope_uuid`

---

## 13. Redirect URL Handling

**Q: How is redirect_url managed?**

- **Server appends at creation:** POST endpoint takes app's redirect_url, appends `?id=<access_request_id>` before storing in DB
- Review page uses the stored (modified) redirect_url when redirecting after action
- App's original redirect_url is not separately stored

---

## 14. Review Page for Processed Requests

**Q: What if user visits review page for already-processed request?**

- Return full access request data regardless of status
- UI renders **non-editable** view for approved/denied/expired requests
- Review endpoint returns data, UI decides rendering based on status

---

## 15. User Tool Instance Selection

**Q: What if user has no configured tool instances?**

- The verify/review screen is specifically for reviewing non-traditional resource access like tools
- If `requested: {}` or no tool_types → auto-approve (see #7)
- When tools ARE requested, user selects instances from their configured ones on the review page
- MVP: single instance per tool_type
- Future: multiple instances, ability to skip (reject partial access)

---

## 16. DB Schema Changes (Phase 3)

### Migration file modifications (unreleased, modify directly):
- `0011_app_access_requests.up.sql`: rename `tools_requested` → `requested`, `tools_approved` → `approved`, add `app_name`, `app_description` columns

### New migration:
- `0012_app_toolsets_scope_to_tool_type.up.sql`: ALTER app_toolsets - add `tool_type`, drop `scope`, `scope_uuid`

---

## 17. Tech Debt Notes

1. **KC /apps/{app-client-id}/info endpoint**: Create custom Keycloak SPI endpoint to return app name and description. Currently being built by user separately.
2. **KC /users/apps/{app-client-id}/info**: BodhiApp route (or AuthService method) to call the KC endpoint with user token for rate limiting/tracking.
3. **Multiple tool instances per tool_type**: Currently MVP is single instance selection.
4. **Partial request rejection**: User can deny specific tool_types without denying the whole request. Future: ability to skip providing instance for a tool_type.
5. **New entity types under `requested`**: MCP servers, workspace folders, etc. - future keys under `requested` object.
6. **Per-entity scopes**: Future: `tool_types: [{tool_type: '...', scopes: ['...']}, approved: {tool_types: [{...}]}]` for downscoped access.

---

## API Surface Summary

| Endpoint | Method | Auth | Purpose |
|----------|--------|------|---------|
| `/bodhi/v1/apps/request-access` | POST | None | Create draft / auto-approve |
| `/bodhi/v1/apps/access-request/:id` | GET | None (requires ?app_client_id) | Poll status |
| `/bodhi/v1/apps/access-request/:id/review` | GET | Session | Review page data |
| `/bodhi/v1/apps/access-request/:id/approve` | PUT | Session | Submit approval with selections |
| `/bodhi/v1/apps/access-request/:id/deny` | POST | Session | Deny request |
