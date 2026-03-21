# Phase 4: Auth Middleware Changes

## Purpose

Update authentication middleware to extract and inject `access_request_id` from tokens, and update toolset auth middleware to validate access requests instead of old OAuth scope checks.

## Dependencies

- **Phase 2**: Service layer with `AccessRequestRepository` implemented
- **Phase 3**: API endpoints need to issue tokens with `access_request_id` claim

## Key Changes

### 4a. Token Claim Extraction and Header Injection

**File**: `crates/auth_middleware/src/auth_middleware.rs`

**Changes**:
1. Add new header constant: `KEY_HEADER_BODHIAPP_ACCESS_REQUEST_ID` = `"X-BodhiApp-Access-Request-Id"`
2. In bearer token flow (after token exchange):
   - Extract `access_request_id` from exchanged token's claims
   - If present → inject header `X-BodhiApp-Access-Request-Id: <uuid>`

**Pattern**: Follow existing header injection pattern for user_id, tool_scopes, etc.

### 4b. Token Exchange Scope Forwarding

**File**: `crates/services/src/token_service.rs` (or equivalent token exchange logic)

**Changes**:
- In `handle_external_client_token()` or similar:
  - Extract `scope_access_request:*` from external token scopes (in addition to existing `scope_toolset-*`)
  - Forward to `auth_service.exchange_app_token()` scope list
  - **Critical**: Scope parameter must include `scope_access_request:<uuid>` and `scope_resource-*` for exchanged token to contain `access_request_id` claim (per KC integration doc)

### 4c. Toolset Auth Middleware Update

**File**: `crates/auth_middleware/src/toolset_auth_middleware.rs`

**Replace OAuth-specific checks** (steps 3-4 in existing flow):

**OLD (remove)**:
- Check `is_app_client_registered_for_toolset(azp, scope_uuid)` via `app_client_toolset_configs`
- Check `scope_toolset-*` in `X-BodhiApp-Tool-Scopes`

**NEW (add)**:
- Extract `access_request_id` from `X-BodhiApp-Access-Request-Id` header
- If present (OAuth flow):
  - Look up access request from DB
  - Validate: status=approved, not expired, `user_id` matches token
  - Verify requested toolset instance UUID is in `tools_approved` list
  - If valid → allow
  - If invalid → deny with appropriate error
- If not present (session flow):
  - Existing session auth validation continues unchanged

**Keep existing**: Session auth validation for non-OAuth flows (user directly using UI)

### 4d. Typed Extractors

**File**: `crates/auth_middleware/src/extractors.rs`

Add new extractors:
```rust
pub struct ExtractAccessRequestId(pub String);
pub struct MaybeAccessRequestId(pub Option<String>);
```

Follow existing pattern for header extractors (like `ExtractUserId`, `MaybeUserId`).

## Files to Create/Modify

| File | Action | Description |
|------|--------|-------------|
| `crates/auth_middleware/src/auth_middleware.rs` | Modify | Add header constant, inject claim |
| `crates/auth_middleware/src/toolset_auth_middleware.rs` | Modify | Replace OAuth checks with access_request validation |
| `crates/auth_middleware/src/extractors.rs` | Modify | Add new extractors |
| `crates/services/src/token_service.rs` | Modify | Forward scope_access_request in token exchange |

## Research Questions

1. **Token exchange location**: Where is `handle_external_client_token()` implemented? (Search for token exchange logic)
2. **Claim extraction**: How do we decode JWT claims from exchanged token? (Check existing claim extraction)
3. **Header injection point**: Where exactly do we inject headers after token exchange? (Check existing header injection)
4. **Middleware flow**: What's the exact order of operations in toolset auth middleware? (Read existing code)
5. **Access to DB**: How does middleware access `AccessRequestRepository`? (Check existing service access in middleware)
6. **Error types**: What error should we return when access_request validation fails? (Check existing auth errors)
7. **Session detection**: How do we distinguish OAuth flow from session flow? (Check existing auth detection)

## Acceptance Criteria

### Token Claim Extraction
- [ ] `access_request_id` claim extracted from exchanged token
- [ ] Header `X-BodhiApp-Access-Request-Id` injected when claim present
- [ ] No header injection when claim absent (session flow)

### Token Exchange Scope Forwarding
- [ ] `scope_access_request:*` extracted from external token
- [ ] Scope forwarded to `exchange_app_token()` call
- [ ] Exchanged token includes `access_request_id` claim when scope present

### Toolset Auth Middleware
- [ ] OAuth flow: validates access_request_id from header
- [ ] Validates access request status is "approved"
- [ ] Validates access request not expired
- [ ] Validates user_id matches token
- [ ] Validates requested toolset instance in tools_approved list
- [ ] Session flow: unchanged validation (no access_request_id check)
- [ ] Clear error messages for validation failures

### Extractors
- [ ] `ExtractAccessRequestId` and `MaybeAccessRequestId` implemented
- [ ] Follow existing extractor patterns
- [ ] Work with Axum routing

### Code Cleanup
- [ ] Remove `X-BodhiApp-Tool-Scopes` header injection
- [ ] Remove `ToolsetScope::from_scope_string()` usage in middleware (keep type if used elsewhere)
- [ ] Remove `is_app_client_registered_for_toolset` calls

### Testing
- [ ] Unit tests for token claim extraction
- [ ] Unit tests for header injection
- [ ] Unit tests for toolset middleware with valid access_request
- [ ] Unit tests for toolset middleware with expired access_request
- [ ] Unit tests for toolset middleware with wrong user
- [ ] Unit tests for toolset middleware with missing tool instance
- [ ] Unit tests for session flow (no access_request_id)
- [ ] `cargo test -p auth_middleware` passes

## Notes for Sub-Agent

- **Read existing middleware first**: Understand current flow before modifying
- **Service access**: Middleware may need `AccessRequestRepository` injected via state
- **Error handling**: Match existing error response patterns
- **Session vs OAuth**: Existing code likely has pattern for distinguishing — follow it
- **Header constants**: Follow naming convention (KEY_HEADER_BODHIAPP_*)
- **Expiry check**: Use current timestamp comparison (check `TimeService` usage)
- **Testing**: Use mock repository for middleware tests (see existing patterns)

## Next Phase

Phase 5 will clean up remaining old flow code after new middleware is working.
