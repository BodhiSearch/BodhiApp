# Plan: Complete toolset_id → scope/scope_uuid Refactoring

## Summary
Continue and complete the refactoring to replace `toolset_id` with `scope` and `scope_uuid` throughout the codebase. The work is approximately 40% complete.

## Scope Values (Reference)
- **scope**: `scope_toolset-builtin-exa-web-search` (same for all environments)
- **scope_uuid (dev)**: `4ff0e163-36fb-47d6-a5ef-26e396f067d6`
- **scope_uuid (prod)**: `7a89e236-9d23-4856-aa77-b52823ff9972`

## Key Design Decisions
1. **Database stores `scope_uuid`** - UUID is the primary identifier in database
2. **`scope` is derived** - Looked up from registry using `scope_uuid` at runtime
3. **API uses `scope_uuid`** - Path parameters and request bodies use `scope_uuid`
4. **Environment-specific UUIDs** - Different Keycloak realms have different scope_uuids

---

## Phase services-fix: Fix services/tool_service.rs (PRIORITY 1)

### File: `crates/services/src/tool_service.rs`

**1. Add `is_production` field to DefaultToolService struct (line ~186):**
```rust
pub struct DefaultToolService {
  db_service: Arc<dyn DbService>,
  exa_service: Arc<dyn ExaService>,
  time_service: Arc<dyn TimeService>,
  is_production: bool,  // ADD THIS
}
```

**2. Update constructor to accept `is_production` (line ~193):**
```rust
pub fn new(
  db_service: Arc<dyn DbService>,
  exa_service: Arc<dyn ExaService>,
  time_service: Arc<dyn TimeService>,
  is_production: bool,  // ADD THIS
) -> Self {
  Self {
    db_service,
    exa_service,
    time_service,
    is_production,  // ADD THIS
  }
}
```

**3. Fix field access errors - replace `toolset_type` with `scope_uuid`:**
- Line 388: `t.toolset_type` → `t.scope_uuid`
- Line 491: `toolset_type: toolset_type.to_string()` → `scope_uuid: scope_uuid.to_string()`
- Line 549: `existing.toolset_type` → `existing.scope_uuid`
- Line 579: `toolset_type: existing.toolset_type` → `scope_uuid: existing.scope_uuid`
- Line 633: `instance.toolset_type` → `instance.scope_uuid`
- Line 651: `instance.toolset_type` → `instance.scope_uuid`
- Line 652: `instance.toolset_type.clone()` → `instance.scope_uuid.clone()`
- Line 657: `instance.toolset_type` → `instance.scope_uuid`
- Line 662: `instance.toolset_type.as_str()` → match on `instance.scope_uuid.as_str()` or derive scope first
- Line 664: `instance.toolset_type` → `instance.scope_uuid`

**4. Fix `toolset_def.toolset_id` → `toolset_def.scope_uuid`:**
- Line 394: `toolset_def.toolset_id` → `toolset_def.scope_uuid`
- Line 411: `toolset_def.toolset_id` → `toolset_def.scope_uuid`
- Line 679: `def.toolset_id` → `def.scope_uuid`

**5. Fix ToolsetWithTools construction (line 415-421):**
```rust
result.push(ToolsetWithTools {
  scope_uuid: toolset_def.scope_uuid.clone(),
  scope: toolset_def.scope.clone(),
  name: toolset_def.name,
  description: toolset_def.description,
  app_enabled,
  tools: toolset_def.tools,
});
```

**6. Fix DbService method calls:**
- Lines 710, 715, 743: `get_app_toolset_config(toolset_id)` → `get_app_toolset_config_by_scope(scope)`
  - Note: These methods check if a `toolset_id` exists by checking tool definitions. Need to update validation logic.

**7. Fix AppToolsetConfigRow construction (line 745-752):**
```rust
let config = AppToolsetConfigRow {
  id: existing.as_ref().map(|e| e.id).unwrap_or(0),
  scope: scope.to_string(),       // WAS: toolset_id
  scope_uuid: scope_uuid.to_string(),  // ADD THIS
  enabled,
  updated_by: updated_by.to_string(),
  created_at: existing.as_ref().map(|e| e.created_at).unwrap_or(now_ts),
  updated_at: now_ts,
};
```

**8. Fix execute method matching (line 662-664):**
The match should use `scope` instead of `toolset_type`. Need to derive scope first:
```rust
// Get scope from registry
let scope = Self::builtin_toolsets(self.is_production)
  .iter()
  .find(|def| def.scope_uuid == instance.scope_uuid)
  .map(|def| def.scope.clone())
  .unwrap_or_default();

match scope.as_str() {
  "scope_toolset-builtin-exa-web-search" => self.execute_exa_method(&api_key, method, request).await,
  _ => Err(ToolsetError::ToolsetNotFound(instance.scope_uuid)),
}
```

**9. Fix is_app_client_registered_for_toolset Keycloak JSON comparison (line 782-786):**
```rust
Ok(toolsets.iter().any(|t| {
  t.get("scope_id")  // WAS: "id"
    .and_then(|v| v.as_str())
    .map(|scope_id| scope_id == scope_uuid)  // Compare with scope_uuid
    .unwrap_or(false)
}))
```

**10. Update trait method parameter names (NOT method names):**
- Keep method names unchanged (e.g., `is_type_enabled`, `get_type`, `validate_type`)
- Only rename parameters from `toolset_id`/`toolset_type` to `scope_uuid`
- This avoids cascading changes to `expect_*` mock calls in tests

**11. Update all test helper functions and mock expectations:**
- Test `DefaultToolService::new()` calls need `is_production: false` parameter
- Update `test_toolset_row()` helper
- Update mock expectations to use new field names

### Verification:
```bash
cargo build --package services
cargo test --package services --lib
```

---

## Phase auth-middleware: Update Auth Middleware

### File: `crates/auth_middleware/src/toolset_auth_middleware.rs`

**Changes:**
1. Line 102: `toolset.toolset_type` → `toolset.scope_uuid`
2. Line 105: Parameter already named `toolset_type`, update call to use `scope_uuid`
3. Line 118: Same - use `scope_uuid`
4. Line 131-132: `ToolsetScope::scope_for_toolset_id(toolset_type)` was removed
   - Replace with: Lookup `scope` from `Toolset` struct (it now has `scope` field)
   - Or: Use `toolset.scope` directly since Toolset now has this field
5. Update test fixtures to use new field names

**New approach for line 131:**
```rust
// OLD: let required_scope = ToolsetScope::scope_for_toolset_id(toolset_type)
// NEW: The toolset already has the scope field
let required_scope = ToolsetScope::from_str(&toolset.scope)
  .ok_or_else(|| ToolsetError::ToolsetNotFound(toolset.scope_uuid.clone()))?;
```

### Verification:
```bash
cargo build --package auth_middleware
cargo test --package auth_middleware --lib
```

---

## Phase routes-app: Update Route Handlers

### File: `crates/routes_app/src/toolsets_dto.rs`

**Changes:**
1. `CreateToolsetRequest.toolset_type` → `CreateToolsetRequest.scope_uuid`
2. `ToolsetResponse.toolset_type` → `ToolsetResponse.scope_uuid` + `ToolsetResponse.scope`
3. `ToolsetTypeResponse.toolset_id` → `ToolsetTypeResponse.scope_uuid` + `ToolsetTypeResponse.scope`
4. Update all tests

### File: `crates/routes_app/src/routes_toolsets.rs`

**Changes:**
1. Line 59-69: `extract_allowed_toolset_types` - update to return `scope_uuid` values
   - Change `s.toolset_id()` to appropriate scope_uuid extraction
2. Line 90-97: Update route paths from `{type_id}` to `{scope_uuid}`
3. Line 133: `toolset.toolset_type` → `toolset.scope_uuid`
4. Line 179: `request.toolset_type` → `request.scope_uuid`
5. Line 370: `t.toolset_id` → `t.scope_uuid`
6. Line 379: Update ToolsetTypeResponse construction
7. Line 462-470: Update ToolsetResponse construction
8. Update all test fixtures and mock expectations

### Verification:
```bash
cargo build --package routes_app
cargo test --package routes_app --lib
```

---

## Phase integration-tests: Update Integration Tests

### File: `crates/integration-tests/tests/test_live_agentic_chat_with_exa.rs`

**Changes:**
1. Line 56: Update URL path to use `scope_uuid` instead of `builtin-exa-web-search`
2. Line 78: Update JSON body `toolset_type` → `scope_uuid`
3. Line 109: Update JSON field check `toolset_type` → `scope_uuid`

### Verification:
```bash
cargo test --package integration-tests
```

---

## Phase openapi: Regenerate API Specs

After all backend changes:
```bash
cargo run --package xtask openapi
cd ts-client && npm run generate
make ci.ts-client-check
```

---

## Phase frontend: Update React Components (if needed)

### Files to update:
1. `crates/bodhi/src/hooks/useToolsets.ts`
2. `crates/bodhi/src/app/ui/toolsets/admin/page.tsx`
3. `crates/bodhi/src/app/ui/toolsets/new/page.tsx`
4. `crates/bodhi/src/app/ui/chat/ToolsetsPopover.tsx`
5. `crates/bodhi/src/app/ui/setup/toolsets/SetupToolsetForm.tsx`
6. `crates/bodhi/src/test-utils/msw-v2/handlers/toolsets.ts`
7. `crates/bodhi/src/test-utils/fixtures/chat.ts`

**Pattern:** Replace `toolset_type` and `toolset_id` with `scope_uuid` (and add `scope` where needed for display).

### Verification:
```bash
make build.ui-rebuild
make test.ui
```

---

## Crate Processing Order
1. `services` - Fix compilation errors
2. `commands` - Likely minimal changes
3. `server_core` - No changes expected
4. `auth_middleware` - Update toolset field access
5. `routes_oai` - No changes expected
6. `routes_app` - Major DTO and route updates
7. `routes_all` - No changes expected
8. `server_app` - No changes expected
9. `lib_bodhiserver` - Already has is_production passing, verify
10. `lib_bodhiserver_napi` - No changes expected
11. `bodhi/src-tauri` - No changes expected
12. `integration-tests` - Update test URLs and JSON

---

## Final Verification
```bash
make test.backend
cargo run --package xtask openapi
cd ts-client && npm run generate
make ci.ts-client-check
make build.ui-rebuild
make test.ui
```
