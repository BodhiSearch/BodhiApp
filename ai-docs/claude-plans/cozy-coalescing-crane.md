# Fix Toolsets OAuth Authorization and E2E Tests

## Summary

Fix middleware authorization for toolsets endpoints to properly restrict OAuth token access, and update E2E tests accordingly.

---

## Phase routes-config: Update Route Configuration

**File**: `crates/routes_all/src/routes.rs`

### Current Issue
- `user_session_apis` uses `api_auth_middleware(ResourceRole::User, None, Some(UserScope::User))` - allows OAuth for CRUD (wrong)
- `user_oauth_apis` (execute) only has `toolset_auth_middleware` - missing `api_auth_middleware` layer

### Changes Required

**1. Rename existing `user_oauth_apis` to `toolset_exec_apis`** (execute endpoint)

**2. Modify `user_session_apis`** - Session-only CRUD operations:
```rust
let user_session_apis = Router::new()
  .route(ENDPOINT_TOOLSETS, post(create_toolset_handler))      // POST /toolsets
  .route(&format!("{ENDPOINT_TOOLSETS}/{{id}}"), get(get_toolset_handler))    // GET /toolsets/{id}
  .route(&format!("{ENDPOINT_TOOLSETS}/{{id}}"), put(update_toolset_handler)) // PUT /toolsets/{id}
  .route(&format!("{ENDPOINT_TOOLSETS}/{{id}}"), delete(delete_toolset_handler)) // DELETE /toolsets/{id}
  .route_layer(from_fn_with_state(
    state.clone(),
    move |state, req, next| {
      api_auth_middleware(ResourceRole::User, None, None, state, req, next) // Session-only
    },
  ));
```

**3. Create new `user_oauth_apis`** - List endpoint with OAuth support:
```rust
let user_oauth_apis = Router::new()
  .route(ENDPOINT_TOOLSETS, get(list_toolsets_handler))  // GET /toolsets (list)
  .route_layer(from_fn_with_state(
    state.clone(),
    move |state, req, next| {
      api_auth_middleware(ResourceRole::User, None, Some(UserScope::User), state, req, next)
    },
  ));
```

**4. Update `toolset_exec_apis`** - Add api_auth_middleware before toolset_auth_middleware:
```rust
let toolset_exec_apis = Router::new()
  .route(&format!("{ENDPOINT_TOOLSETS}/{{id}}/execute/{{method}}"), post(execute_toolset_handler))
  .route_layer(from_fn_with_state(state.clone(), toolset_auth_middleware))
  .route_layer(from_fn_with_state(
    state.clone(),
    move |state, req, next| {
      api_auth_middleware(ResourceRole::User, None, Some(UserScope::User), state, req, next)
    },
  ));
```

**5. Update router merge** - Include all three groups in the final router merge.

---

## Phase ui-attr: Add data-test-uuid Attribute

**File**: `crates/bodhi/src/app/ui/toolsets/page.tsx`

Add `data-test-uuid` attribute to toolset row elements for easier test access:
```tsx
<TableCell data-testid={`toolset-name-${toolset.id}`} data-test-uuid={toolset.id} ...>
```

**After changes**: Run `make build.ui-rebuild`

---

## Phase test-revert: Revert Tests 10-11 to Expect 401

**File**: `crates/lib_bodhiserver_napi/tests-js/specs/toolsets/toolsets-auth-restrictions.spec.mjs`

### Tests 10-11: Currently expect 200, must revert to 401

The tests were incorrectly modified to expect 200. Per authorization requirements, OAuth tokens should get 401 for CRUD operations.

**Test-10 (GET):**
- Test name: "GET /toolsets/{id} with OAuth token returns 401 (session-only)"
- Endpoint: `/toolsets/${toolsetUuid}` (correct)
- Expectation: `expect(response.status).toBe(401)`
- Remove response body assertions

**Test-11 (PUT):**
- Test name: "PUT /toolsets/{id} with OAuth token returns 401 (session-only)"
- Endpoint: `/toolsets/${toolsetUuid}` (correct)
- Expectation: `expect(response.status).toBe(401)`
- Remove response body assertions

### Describe block rename
- "OAuth Token - Toolset CRUD Endpoints" → "OAuth Token - Toolset CRUD Endpoints (Session-Only)"

### UUID Access Changes
Replace `page.evaluate` fetch with data-test-uuid attribute:

**In beforeEach:**
```javascript
// After configureToolsetWithApiKey, navigate to toolsets and get UUID from attribute:
await toolsetsPage.navigateToToolsets();
const row = sessionPage.locator('[data-testid-type="builtin-exa-web-search"]').first();
toolsetUuid = await row.getAttribute('data-test-uuid');
```

---

## Phase test-run: Run and Verify Tests

```bash
cd crates/lib_bodhiserver_napi && npm run test:playwright -- --grep "toolsets"
```

### Expected Results After Middleware Fix:
| Test | Expected Result |
|------|-----------------|
| Test-9 (OAuth execute) | Pass (200) |
| Test-10 (GET with OAuth) | Pass (401) |
| Test-11 (PUT with OAuth) | Pass (401) |
| Other toolset tests | Pass |

---

## Files to Modify

1. `crates/routes_all/src/routes.rs` - Route configuration and middleware
2. `crates/bodhi/src/app/ui/toolsets/page.tsx` - Add data-test-uuid attribute
3. `crates/lib_bodhiserver_napi/tests-js/specs/toolsets/toolsets-auth-restrictions.spec.mjs` - Revert tests to 401

---

## Authorization Summary

| Endpoint | Session | OAuth (scope_user_*) | API Token (scope_token_*) |
|----------|---------|----------------------|---------------------------|
| GET /toolsets (list) | ✅ | ✅ (filtered by scope) | ❌ 401 |
| POST /toolsets | ✅ | ❌ 401 | ❌ 401 |
| GET /toolsets/{id} | ✅ | ❌ 401 | ❌ 401 |
| PUT /toolsets/{id} | ✅ | ❌ 401 | ❌ 401 |
| DELETE /toolsets/{id} | ✅ | ❌ 401 | ❌ 401 |
| POST /toolsets/{id}/execute/{method} | ✅ | ✅ (with scope check) | ❌ 401 |
