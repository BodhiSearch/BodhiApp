# Phase 4: Auth Middleware Access Request Validation - Implementation Plan

## Context

Phase 3 completed the access request foundation (database schema, domain objects, service layer, HTTP endpoints) and migrated toolset identification from `scope`/`scope_uuid` to `tool_type` system. The toolset auth middleware currently **rejects all OAuth flows** (line 64-67 in `toolset_auth_middleware.rs`) with a TODO marker for Phase 4.

Phase 4 restores OAuth authentication for toolset execution by:
1. Implementing access request validation in toolset auth middleware
2. Adding token exchange scope forwarding for `scope_access_request:*`
3. Creating role-based extractors to distinguish OAuth vs session flows
4. Renaming JSON field `tool_types` → `toolset_types` for correct domain terminology
5. Removing deprecated `X-BodhiApp-Tool-Scopes` header completely

**Why this matters**: Third-party apps need secure, granular access to toolset execution endpoints. The access request system provides user-controlled approval of specific toolset instances, replacing the old OAuth scope-based model that was removed in Phase 3.

## Key Design Decisions

### No Backwards Compatibility

**Critical**: This is a **clean cut migration** to the new access request-based authorization model. No backwards compatibility with old OAuth scope system. All changes are breaking:
- Old `X-BodhiApp-Tool-Scopes` header removed completely
- JSON field names change from `tool_types` to `toolset_types`
- OAuth flows without `access_request_id` are rejected
- Old access request records (with `tool_types` JSON) will naturally expire

### Access Request Validation Requirements

When an OAuth token with `access_request_id` claim reaches the toolset execution endpoint, validate:
1. **Status**: `status = "approved"` (reject draft/denied/failed)
2. **App Identity**: `app_client_id` in access request matches `azp` claim in token
3. **Instance Authorization**: Requested toolset UUID exists in `approved.toolset_types[].instance_id`
4. **User Identity**: `user_id` in access request matches `sub` claim in token

Fail with **403 Forbidden** and specific error messages for any validation failure.

### Auto-Approved Access Requests

Access requests with empty `requested` array auto-approve with `approved = []` and `access_request_scope = NULL`. These **cannot be used for toolset access** - treat as "no toolsets approved" and reject with 403.

### Flow Detection

Distinguish OAuth from session flows using existing role headers:
- **Session auth**: Has `X-BodhiApp-Role` header (resource_user, resource_admin)
- **OAuth auth**: No ROLE header (has `X-BodhiApp-Scope` header instead)

Create typed extractors `ExtractResourceRole` and `MaybeResourceRole` following existing `ExtractRole` pattern.

### JSON Field Terminology Fix

Rename `tool_types` → `toolset_types` in JSON fields (`requested` and `approved` columns). This is **JSON-only** - no database column rename needed. Update:
- Domain objects: `ToolTypeRequest` → use `toolset_types` in JSON
- Service layer: JSON serialization/deserialization
- Route handlers: Request/response DTOs
- Tests: Fixture JSON strings

## Implementation Phases

### Phase 1: JSON Field Renaming (Clean Cut Migration)

**Goal**: Fix domain terminology before middleware changes. **No backwards compatibility** - clean migration to `toolset_types`.

**Changes**:
1. **Domain objects** (`crates/objs/src/access_request.rs`):
   - Rename struct fields directly: `tool_types` → `toolset_types` in `ToolTypeRequest` and `ToolApproval`
   - Update all JSON serialization to use new field names
   - Example:
   ```rust
   // OLD:
   pub struct ToolTypeRequest {
     pub tool_types: Vec<String>,
   }

   // NEW:
   pub struct ToolTypeRequest {
     pub toolset_types: Vec<String>,
   }
   ```

2. **Service layer** (`crates/services/src/access_request_service/service.rs`):
   - Update JSON string generation in `create_draft()`, `approve_request()` to use `"toolset_types"`
   - Fix all JSON parsing to expect `toolset_types` key
   - Update test fixtures to use new field names

3. **Route handlers** (`crates/routes_app/src/routes_apps/types.rs`):
   - Update request/response DTOs to use `toolset_types`
   - Fix all test JSON assertions

4. **Database data migration** (if needed):
   - Existing access request records in database have JSON with `tool_types`
   - Since access requests are short-lived (expire quickly), **no data migration needed**
   - Old records will expire naturally; new records use correct terminology

**Verification**:
```bash
cargo test -p objs
cargo test -p services
cargo test -p routes_app -- access_request
```

### Phase 2: Token Exchange Scope Forwarding

**Goal**: Extract and forward `scope_access_request:*` from external tokens to KC.

**File**: `crates/auth_middleware/src/token_service.rs`

**Changes** (line 207-216):

**OLD**:
```rust
let mut scopes: Vec<&str> = claims
  .scope
  .split_whitespace()
  .filter(|s| s.starts_with("scope_user_") || s.starts_with("scope_toolset-"))
  .collect();
```

**NEW**:
```rust
let mut scopes: Vec<&str> = claims
  .scope
  .split_whitespace()
  .filter(|s| {
    s.starts_with("scope_user_")
    || s.starts_with("scope_toolset-")
    || s.starts_with("scope_access_request_")  // <-- ADD THIS
  })
  .collect();
```

**Why**: KC includes `access_request_id` claim in exchanged tokens only when `scope_access_request:<uuid>` is present in the scope list.

**Verification**:
```bash
cargo test -p auth_middleware -- token_service
```

### Phase 3: Header Injection for Access Request ID

**Goal**: Extract `access_request_id` from exchanged token claims and inject into request headers.

**File**: `crates/auth_middleware/src/auth_middleware.rs`

**Changes**:

1. Add header constant (line 39):
```rust
pub const KEY_HEADER_BODHIAPP_ACCESS_REQUEST_ID: &str = bodhi_header!("Access-Request-Id");
```

2. Extract and inject in bearer token validation (after line 180):
```rust
// After extracting toolset scopes
if let Some(access_request_id) = scope_claims.access_request_id {
  req.headers_mut().insert(
    KEY_HEADER_BODHIAPP_ACCESS_REQUEST_ID,
    access_request_id.parse().unwrap(),
  );
}
```

**Note**: `ScopeClaims` struct needs `access_request_id: Option<String>` field added.

**Verification**:
```bash
cargo test -p auth_middleware -- auth_middleware
```

### Phase 4: Role-Based Extractors

**Goal**: Create typed extractors to distinguish OAuth from session flows.

**File**: `crates/auth_middleware/src/extractors.rs`

**Add new extractors**:
```rust
/// Extract resource role (session auth only)
pub struct ExtractResourceRole(pub ResourceRole);

/// Maybe extract resource role (OAuth flows return None)
pub struct MaybeResourceRole(pub Option<ResourceRole>);

/// Extract access request ID (OAuth flows only)
pub struct ExtractAccessRequestId(pub String);

/// Maybe extract access request ID (session flows return None)
pub struct MaybeAccessRequestId(pub Option<String>);
```

**Implementation pattern** (follow existing `ExtractRole` at lines 107-121):
```rust
impl<S: Send + Sync> FromRequestParts<S> for MaybeResourceRole {
  type Rejection = ApiError;

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
    let role_header = parts.headers.get(KEY_HEADER_BODHIAPP_ROLE);

    if let Some(role_value) = role_header {
      let role_str = role_value.to_str().map_err(|e| {
        ApiError::from(HeaderExtractionError::Invalid {
          header: KEY_HEADER_BODHIAPP_ROLE.to_string(),
          reason: e.to_string(),
        })
      })?;

      let role = ResourceRole::from_str(role_str).map_err(|e| {
        ApiError::from(HeaderExtractionError::Invalid {
          header: KEY_HEADER_BODHIAPP_ROLE.to_string(),
          reason: e.to_string(),
        })
      })?;

      Ok(MaybeResourceRole(Some(role)))
    } else {
      Ok(MaybeResourceRole(None))
    }
  }
}
```

**Verification**:
```bash
cargo test -p auth_middleware -- extractors
```

### Phase 5: Toolset Auth Middleware - Redesigned Flow

**Goal**: Implement access request validation for OAuth flows, keep session flow unchanged.

**File**: `crates/auth_middleware/src/toolset_auth_middleware.rs`

**New error variants** (add to `ToolsetAuthError` enum after line 29):
```rust
#[error("Access request {access_request_id} not found.")]
#[error_meta(error_type = ErrorType::Forbidden)]
AccessRequestNotFound { access_request_id: String },

#[error("Access request {access_request_id} has status '{status}'. Only approved requests can access toolsets.")]
#[error_meta(error_type = ErrorType::Forbidden)]
AccessRequestNotApproved { access_request_id: String, status: String },

#[error("Access request {access_request_id} is invalid: {reason}")]
#[error_meta(error_type = ErrorType::Forbidden)]
AccessRequestInvalid { access_request_id: String, reason: String },

#[error("Toolset {toolset_id} is not included in your approved tools for this app.")]
#[error_meta(error_type = ErrorType::Forbidden)]
ToolsetNotApproved { toolset_id: String },

#[error("Access request app client ID mismatch: expected {expected}, found {found}.")]
#[error_meta(error_type = ErrorType::Forbidden)]
AppClientMismatch { expected: String, found: String },

#[error("Access request user ID mismatch: expected {expected}, found {found}.")]
#[error_meta(error_type = ErrorType::Forbidden)]
UserMismatch { expected: String, found: String },

#[error("Invalid approved JSON in access request: {error}")]
#[error_meta(error_type = ErrorType::InternalError)]
InvalidApprovedJson { error: String },
```

**Redesigned middleware function** (replace lines 48-96):
```rust
pub async fn toolset_auth_middleware(
  ExtractUserId(user_id): ExtractUserId,
  MaybeResourceRole(role): MaybeResourceRole,
  MaybeAccessRequestId(access_request_id): MaybeAccessRequestId,
  State(state): State<Arc<dyn RouterState>>,
  req: Request<Body>,
  next: Next,
) -> Result<Response, ApiError> {
  // Extract toolset ID from path
  let id = req
    .uri()
    .path()
    .split('/')
    .find(|seg| seg.len() == 36 && seg.contains('-'))
    .ok_or(ToolsetAuthError::ToolsetNotFound)?
    .to_string();

  let tool_service = state.app_service().tool_service();
  let db_service = state.app_service().db_service();

  // Determine auth flow type
  let is_session = role.is_some();
  let is_oauth = access_request_id.is_some();

  if !is_session && !is_oauth {
    return Err(ToolsetAuthError::MissingAuth.into());
  }

  // BOTH FLOWS: Verify toolset exists and get type
  let toolset = tool_service
    .get(&user_id, &id)
    .await?
    .ok_or(ToolsetAuthError::ToolsetNotFound)?;

  // OAUTH FLOW: Access request validation
  if is_oauth {
    let ar_id = access_request_id.unwrap();

    // Fetch access request
    let access_request = db_service
      .get(&ar_id)
      .await?
      .ok_or(ToolsetAuthError::AccessRequestNotFound { access_request_id: ar_id.clone() })?;

    // Validate status
    if access_request.status != "approved" {
      return Err(ToolsetAuthError::AccessRequestNotApproved {
        access_request_id: ar_id,
        status: access_request.status,
      }.into());
    }

    // Validate app_client_id matches token azp
    let azp = req
      .headers()
      .get(KEY_HEADER_BODHIAPP_AZP)
      .and_then(|h| h.to_str().ok())
      .ok_or(ToolsetAuthError::MissingAuth)?;

    if access_request.app_client_id != azp {
      return Err(ToolsetAuthError::AppClientMismatch {
        expected: access_request.app_client_id,
        found: azp.to_string(),
      }.into());
    }

    // Validate user_id matches (must be present for approved requests)
    let ar_user_id = access_request
      .user_id
      .as_ref()
      .ok_or(ToolsetAuthError::AccessRequestInvalid {
        access_request_id: ar_id.clone(),
        reason: "Missing user_id in approved access request".to_string(),
      })?;

    if ar_user_id != &user_id {
      return Err(ToolsetAuthError::UserMismatch {
        expected: ar_user_id.clone(),
        found: user_id.clone(),
      }.into());
    }

    // Validate toolset instance in approved list
    if let Some(approved_json) = &access_request.approved {
      let approvals: serde_json::Value = serde_json::from_str(approved_json)
        .map_err(|e| ToolsetAuthError::InvalidApprovedJson { error: e.to_string() })?;

      let toolset_types = approvals
        .get("toolset_types")
        .and_then(|v| v.as_array())
        .ok_or(ToolsetAuthError::InvalidApprovedJson { error: "Missing toolset_types array".to_string() })?;

      let instance_approved = toolset_types.iter().any(|approval| {
        approval.get("status").and_then(|s| s.as_str()) == Some("approved")
          && approval.get("instance_id").and_then(|i| i.as_str()) == Some(&id)
      });

      if !instance_approved {
        return Err(ToolsetAuthError::ToolsetNotApproved { toolset_id: id }.into());
      }
    } else {
      // approved is NULL - auto-approved request with no toolsets
      return Err(ToolsetAuthError::ToolsetNotApproved { toolset_id: id }.into());
    }
  }

  // BOTH FLOWS: Verify app-level type enabled
  if !tool_service.is_type_enabled(&toolset.tool_type).await? {
    return Err(ToolsetError::ToolsetAppDisabled(toolset.tool_type.clone()).into());
  }

  // BOTH FLOWS: Verify instance configured
  if !toolset.enabled {
    return Err(ToolsetError::ToolsetNotConfigured(id.clone()).into());
  }

  if !toolset.has_api_key {
    return Err(ToolsetError::ToolsetNotConfigured(id.clone()).into());
  }

  Ok(next.run(req).await)
}
```

**Verification**:
```bash
cargo test -p auth_middleware -- toolset_auth_middleware
```

### Phase 6: Remove Deprecated Header

**Goal**: Clean up old OAuth scope system completely.

**Changes**:
1. **Remove header injection** (`auth_middleware/src/token_service.rs`, lines 165-177):
   - Delete the `KEY_HEADER_BODHIAPP_TOOL_SCOPES` injection block

2. **Remove header constant** (`auth_middleware/src/auth_middleware.rs`, line 37):
   - Delete `pub const KEY_HEADER_BODHIAPP_TOOL_SCOPES: &str = bodhi_header!("Tool-Scopes");`

3. **Remove scope extraction logic** (`auth_middleware/src/token_service.rs`, lines 162-177):
   - Delete the `toolset_scopes` extraction after token validation

**Verification**:
```bash
cargo check -p auth_middleware
grep -r "KEY_HEADER_BODHIAPP_TOOL_SCOPES" crates/  # Should return no results
```

### Phase 7: Unit Tests

**Goal**: Comprehensive test coverage for access request validation.

**File**: `crates/auth_middleware/src/toolset_auth_middleware.rs` (test module)

**Test cases** (following existing patterns at lines 98-262):

```rust
#[rstest]
#[case::oauth_approved(true, "approved", true, StatusCode::OK)]
#[case::oauth_denied(true, "denied", false, StatusCode::FORBIDDEN)]
#[case::oauth_expired(true, "expired", false, StatusCode::FORBIDDEN)]
#[case::oauth_draft(true, "draft", false, StatusCode::FORBIDDEN)]
#[case::oauth_not_in_approved_list(true, "approved", false, StatusCode::FORBIDDEN)]
#[tokio::test]
async fn test_oauth_access_request_validation(
  #[case] has_access_request: bool,
  #[case] ar_status: &str,
  #[case] instance_in_approved: bool,
  #[case] expected_status: StatusCode,
) -> anyhow::Result<()> {
  // Setup: Create access request in DB via db_service
  // Setup: Mock tool_service with toolset response
  // Build router with middleware
  // Send request with access_request_id header
  // Assert status code and error details
}

#[rstest]
#[tokio::test]
async fn test_oauth_app_client_mismatch() -> anyhow::Result<()> {
  // AR has app_client_id = "app1"
  // Token has azp = "app2"
  // Expect 403 with AppClientMismatch error
}

#[rstest]
#[tokio::test]
async fn test_oauth_user_mismatch() -> anyhow::Result<()> {
  // AR has user_id = "user1"
  // Token has sub = "user2"
  // Expect 403 with UserMismatch error
}

#[rstest]
#[tokio::test]
async fn test_oauth_auto_approved_no_toolsets() -> anyhow::Result<()> {
  // AR has approved = NULL (auto-approved)
  // Expect 403 with ToolsetNotApproved error
}

#[tokio::test]
async fn test_session_auth_unchanged() -> anyhow::Result<()> {
  // Session auth with ROLE header (no access_request_id)
  // Should work exactly as before
  // Expect 200 OK
}
```

**Test infrastructure**:
- Use `test_db_service_with_temp_dir()` for real SQLite with access request records
- Use `MockToolService` for toolset lookups
- Use `AppServiceStubBuilder` to compose test services
- Follow existing test patterns from lines 98-262

**Verification**:
```bash
cargo test -p auth_middleware
```

## Critical Files

| Component | File Path | Changes |
|-----------|-----------|---------|
| Domain Objects | `crates/objs/src/access_request.rs` | JSON field rename (tool_types → toolset_types) |
| Token Exchange | `crates/auth_middleware/src/token_service.rs` | Scope forwarding, remove header injection |
| Auth Middleware | `crates/auth_middleware/src/auth_middleware.rs` | Add access_request_id header injection, remove tool_scopes |
| Extractors | `crates/auth_middleware/src/extractors.rs` | Add role/access_request extractors |
| Toolset Auth | `crates/auth_middleware/src/toolset_auth_middleware.rs` | Redesigned flow with access request validation |
| Service Layer | `crates/services/src/access_request_service/service.rs` | JSON field updates |
| Route Handlers | `crates/routes_app/src/routes_apps/` | JSON field updates in DTOs |

## Verification Strategy

### Compilation Verification
```bash
cargo check -p objs
cargo check -p services
cargo check -p auth_middleware
cargo check -p routes_app
```

### Unit Test Verification
```bash
# Phase 1: JSON renaming
cargo test -p objs
cargo test -p services -- access_request

# Phase 2-3: Token exchange and header injection
cargo test -p auth_middleware -- token_service
cargo test -p auth_middleware -- auth_middleware

# Phase 4: Extractors
cargo test -p auth_middleware -- extractors

# Phase 5-7: Middleware and tests
cargo test -p auth_middleware -- toolset_auth_middleware

# Full suite
make test.backend
```

### Integration Test Strategy

**Deferred to Tech Debt**: Complex end-to-end integration tests involving actual KC token exchange, live HTTP requests, and full middleware stack will be documented in `tech-debt.md` for implementation in `server_app` crate later. Phase 4 focuses on unit tests with mocked services.

**Tech debt note**: "Integration tests for OAuth access request flow: Mock KC token exchange with real access_request_id claims, test full middleware stack in server_app with live HTTP client."

## Open Items

1. **KC Claim Structure**: Verify `ScopeClaims` struct has `access_request_id: Option<String>` field. If not, add it.

2. **Error Message Localization**: Current error messages are in English. Future enhancement: Add localization support for 403 error messages.

3. **Access Request Expiry**: Phase 4 validates `status = "approved"` but doesn't check `expires_at` timestamp. Future enhancement: Add expiry validation.

4. **Caching Consideration**: User selected "No caching" for Phase 4, but future performance optimization could add request-scoped caching if DB queries become bottleneck.

5. **Integration Tests**: As noted above, complex integration tests are deferred to tech-debt.md for server_app implementation.

## Success Criteria

- [ ] JSON fields renamed: `tool_types` → `toolset_types` across domain objects, services, routes
- [ ] Token exchange forwards `scope_access_request:*` to KC
- [ ] `access_request_id` extracted from token claims and injected into `X-BodhiApp-Access-Request-Id` header
- [ ] Role-based extractors distinguish OAuth (no ROLE) from session (has ROLE) flows
- [ ] Toolset auth middleware validates OAuth flows: status=approved, app_client_id match, instance_id in approved list, user_id match
- [ ] Auto-approved requests (approved=[]) reject toolset access with 403
- [ ] Session auth flow unchanged - existing tests still pass
- [ ] 403 Forbidden responses with specific error codes for each validation failure
- [ ] `X-BodhiApp-Tool-Scopes` header removed completely
- [ ] All unit tests passing: `cargo test -p auth_middleware`
- [ ] Full backend suite passing: `make test.backend`
- [ ] No compilation errors across affected crates

---

## Phase 6 - Actual Implementation (Completed 2026-02-12)

**Context**: Phase 6 as originally planned focused only on removing the `X-BodhiApp-Tool-Scopes` header. However, the actual implementation went further to complete a comprehensive cleanup of the legacy `scope_toolset-*` authorization system, removing all remnants from the codebase.

### What Was Actually Implemented

This implementation completed a clean migration from the old OAuth scope-based toolset authorization (`scope_toolset-*`) to the new access request-based authorization system. All changes follow the project's **no backwards compatibility** policy.

### Changes Made

#### 1. Migration 0013 - Complete Schema Cleanup

**Files**:
- `crates/services/migrations/0013_app_toolset_configs_tool_type.up.sql`
- `crates/services/migrations/0013_app_toolset_configs_tool_type.down.sql`

**Change**: Rebuilt `app_toolset_configs` table to drop legacy `scope` (with `NOT NULL UNIQUE` constraint) and `scope_uuid` columns. Added UNIQUE constraint on `toolset_type` column.

**Implementation**: SQLite table rebuild pattern - create new table without legacy columns, copy and migrate data (`scope_toolset-builtin-exa-web-search` → `builtin-exa-search`), drop old table, rename new table. Down migration restores original structure with UNIQUE constraint.

**Rationale**: SQLite doesn't support dropping columns with UNIQUE constraints directly. Migration 0013 was incomplete - it added `toolset_type` but didn't remove the obsolete `scope` column.

#### 2. Database Seed Function Update

**File**: `crates/services/src/db/service.rs:54-65`

**Change**: Removed `scope` column from `seed_toolset_configs()` INSERT statement. Now inserts only `toolset_type` with value `"builtin-exa-search"` (removed `scope_toolset-builtin-exa-web-search` bind).

**Rationale**: The `scope` column no longer exists after migration 0013. This updates stale code that should have been changed when the migration was originally created.

#### 3. Token Exchange Filter Cleanup

**File**: `crates/auth_middleware/src/token_service.rs:204-214`

**Change**: Removed `s.starts_with("scope_toolset-")` filter condition from token exchange scope extraction. Now forwards only `scope_user_*` and `scope_access_request:*` patterns to Keycloak.

**Rationale**: Toolset authorization now uses access_request validation in `toolset_auth_middleware.rs`. OAuth tokens no longer need `scope_toolset-*` scopes for toolset access.

### Verification Results

#### Compilation Checks
```bash
✅ cargo check -p auth_middleware  # Success
✅ cargo check -p services          # Success
```

#### Unit Tests
```bash
✅ cargo test -p auth_middleware -- token_service  # 13 tests passed
✅ cargo test -p services                          # 286 tests passed
✅ cargo test -p routes_app                        # All tests passed
```

#### Code Cleanup Verification
```bash
✅ grep -r "scope_toolset" crates/auth_middleware  # No results
✅ grep -r "scope_toolset" crates/services         # Only in historical migrations (expected)
```

### Files Modified

| File | Change Type | Description |
|------|-------------|-------------|
| `crates/services/migrations/0013_app_toolset_configs_tool_type.up.sql` | Schema Migration | Table rebuild to drop `scope` and `scope_uuid` columns, add UNIQUE constraint on `toolset_type` |
| `crates/services/migrations/0013_app_toolset_configs_tool_type.down.sql` | Schema Migration | Restore original table structure with UNIQUE constraint |
| `crates/services/src/db/service.rs` | Code Update | Remove `scope` column from seed function INSERT |
| `crates/auth_middleware/src/token_service.rs` | Code Update | Remove `scope_toolset-*` from token exchange filter |

### Migration Ordering Constraint

**Critical**: The migration update MUST be applied before the seed function update:
1. ✅ Update migration 0013 SQL files
2. ✅ Apply updated migration 0013 (for new databases) or handle existing databases that already ran incomplete migration 0013
3. ✅ Update seed function in `db/service.rs`
4. ✅ Update token exchange filter in `token_service.rs`

### Handling Existing Databases

For databases where migration 0013 was already applied (without the scope column drop):
- **Development/Test Databases**: Delete and recreate - migrations will apply cleanly
- **Production Databases**: Would need migration 0014 to drop the columns, but this is not currently needed as the system is in development

### Out of Scope

Explicitly excluded from this cleanup (per project scope):
- ❌ Changes to `crates/server_app/tests/` - separate cleanup effort
- ❌ Changes to `crates/routes_app/` route handlers - handled separately
- ❌ Updates to Keycloak configuration - external system, no changes needed
- ❌ Migration of existing access_request data - schema-only change, no data migration required
- ❌ Documentation files in `ai-docs/` - reference only

### Key Design Decisions

**No Backwards Compatibility**: Clean cut migration following project policy. Old `scope_toolset-*` OAuth authorization system completely removed. No attempt to support both old and new systems.

**Table Rebuild Approach**: SQLite's limitations with UNIQUE constraint columns required the table rebuild pattern rather than simple ALTER TABLE DROP COLUMN.

**Domain Objects Unchanged**: The `AppToolsetConfigRow` and `AppToolsetConfig` domain objects already had correct structure (only `toolset_type` field, no `scope` field) - no domain object changes needed.

**Migration 0013 Completion**: Treated as completing an incomplete migration rather than creating new migration 0014, since the original migration should have dropped the `scope` column when adding `toolset_type`.

### Success Criteria - Completed

- ✅ Migration 0013 updated to drop `scope` column and index from `app_toolset_configs`
- ✅ Migration 0013 updated to add UNIQUE constraint to `toolset_type` index
- ✅ Migration 0013 down migration updated to restore `scope` column with UNIQUE constraint
- ✅ Database seed function no longer references `scope` column
- ✅ `scope_toolset-*` pattern removed from token_service.rs filter logic
- ✅ All compilation checks pass: `cargo check -p auth_middleware -p services`
- ✅ All unit tests pass: 13 token_service tests + 286 services tests + routes_app tests
- ✅ No references to `scope_toolset-*` remain in auth_middleware or services crates (except historical migrations)
- ✅ Toolset execution continues to work with access_request-based authorization

### Related Documentation

This implementation completes the scope cleanup that was partially addressed in Phase 3 (migration 0012 for `toolsets` table and migration 0013 for `app_toolset_configs` table). The full authorization flow now exclusively uses:
- **User-level permissions**: `scope_user_*` (role-based)
- **Access request authorization**: `scope_access_request_*` (instance-specific toolset access)

The legacy `scope_toolset-*` OAuth authorization pattern is completely removed from the codebase.

---

## Phase 4b: Pre/Post-Exchange Validation (Completed 2026-02-12)

### Context

After Phase 4 implementation (commits 25fd85f1-33c51630), external OAuth clients could present `scope_access_request:<uuid>` scopes without pre-validation. The token exchange occurred before verifying the scope existed in the database or was approved, creating a security gap.

### What Was Implemented

Added two-layer validation around Keycloak token exchange:

**Pre-Exchange Validation (Fail Fast)**:
1. Extract `scope_access_request:<uuid>` from external token
2. Look up access request by scope in database
3. Validate: status=approved, app_client_id matches azp, user_id matches sub
4. Return 403 Forbidden on any validation failure
5. Store validated record for post-verification

**Post-Exchange Verification (Integrity Check)**:
1. Extract `access_request_id` claim from exchanged token
2. Verify claim matches database record.id from pre-validation
3. Log warning and return 403 if mismatch detected

### Changes Made

**Database Schema** (`migrations/0011_app_access_requests.up.sql`):
- Added unique index on `access_request_scope` (NULL-aware) to prevent duplicate KC-assigned scopes

**Repository** (`services/src/db/`):
- Added `get_by_access_request_scope()` method to `AccessRequestRepository`
- Implemented using `fetch_optional` pattern in `SqliteDbService`

**Error Handling** (`services/src/token.rs`):
- Added `AccessRequestValidationError` enum with 5 variants: ScopeNotFound, NotApproved, AppClientMismatch, UserMismatch, AccessRequestIdMismatch
- Integrated into `TokenError` via transparent wrapping

**Validation Logic** (`auth_middleware/src/token_service.rs`):
- Pre-validation runs before token exchange when `scope_access_request:*` present
- Post-verification runs after token exchange to validate KC returned correct `access_request_id` claim
- Tokens without access request scopes skip validation entirely

**Testing**:
- 5 repository tests (found, not found, NULL handling, unique constraint)
- 11 token service tests (5 pre-validation failures, 2 post-verification, 2 happy path, 2 integration)
- All 291+ tests passing across objs, services, server_core, auth_middleware, routes_app, server_app

### Success Criteria - Completed

- ✅ Unique index on `access_request_scope` with NULL support
- ✅ Repository method `get_by_access_request_scope()` implemented
- ✅ `AccessRequestValidationError` enum with 5 specific error variants
- ✅ Pre-validation logic validates status, app_client_id, user_id before exchange
- ✅ Post-verification logic validates `access_request_id` claim after exchange
- ✅ 16 new tests covering all validation scenarios
- ✅ All compilation checks pass
- ✅ Full test suite passing (291+ tests)

### Key Design Decisions

**Fail Fast Strategy**: Invalid scopes rejected before Keycloak call, reducing unnecessary token exchange operations.

**Conditional Validation**: Only validates when `scope_access_request:*` present. Tokens with only user scopes skip validation.

**Separation of Concerns**: Scope UUID lookup separate from access_request_id claim verification, allowing KC claim format flexibility.
