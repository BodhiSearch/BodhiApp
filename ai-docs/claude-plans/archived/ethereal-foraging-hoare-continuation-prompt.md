# Continuation Prompt: Complete toolset_id → scope/scope_uuid Refactoring

## Context

This is a continuation of a large refactoring task to replace `toolset_id` with `scope` and `scope_uuid` throughout the BodhiApp codebase. The refactoring makes `scope_uuid` environment-aware (different UUIDs for dev vs prod environments) and adds `toolset_types` array to API responses.

**Previous Plan:** See `ai-docs/claude-plans/ethereal-foraging-hoare.md` for full details and current status.

## What Has Been Completed (40% done)

### ✅ Foundation Layers (100% complete)
1. **Database Schema** - Migration files updated to use `scope`/`scope_uuid` instead of `toolset_id`
2. **Domain Objects** (`crates/objs`) - All types updated, tests passing (425/425)
3. **Database Service** (`crates/services/src/db`) - SqliteDbService updated with `is_production` field and environment-specific seeding
4. **App Builder** - Services now receive `is_production` parameter
5. **Test Utilities** - TestDbService updated with new method names

### ⚠️ Partial Implementation (~60% complete)
**File:** `crates/services/src/tool_service.rs`

**Completed:**
- Added `is_production` field to DefaultToolService struct
- Updated `builtin_toolsets()` to generate environment-specific scope_uuid
- Updated test helpers and mock expectations
- Implemented `toolset_row_to_model()` with scope lookup from registry
- Updated helper conversion functions

**Remaining Work (compilation errors present):**
1. Update field accesses: `row.toolset_type` → `row.scope_uuid`, `toolset_def.toolset_id` → `toolset_def.scope_uuid`
2. Update trait method signatures (parameters renamed from toolset_type to scope_uuid)
3. Implement `get_toolset_types_for_scopes()` method (new requirement)
4. Update `is_app_client_registered_for_toolset()` to compare `scope_id` instead of `id` in Keycloak JSON
5. Fix all ToolsetRow and ToolsetWithTools constructions
6. Update test DefaultToolService::new() calls to pass `is_production` parameter
7. Fix DbService method calls (use `get_app_toolset_config_by_scope` instead of old name)

## What Needs to Be Done (60% remaining)

### Priority 1: Fix services crate compilation
**File:** `crates/services/src/tool_service.rs`
- Current status: Has compilation errors due to field name mismatches
- Approach: Systematic field renaming and method signature updates
- Tests: Must pass `cargo test --package services --lib`

### Priority 2: Update downstream crates (in dependency order)
1. **commands** - Likely minimal changes
2. **server_core** - Should not need changes
3. **auth_middleware** - Update `toolset_auth_middleware.rs` to use scope_uuid
4. **routes_oai** - Should not need changes
5. **routes_app** - Major updates to routes_toolsets.rs and toolsets_dto.rs
6. **routes_all** - Should not need changes
7. **server_app** - Should not need changes
8. **lib_bodhiserver** - Already complete
9. **integration-tests** - Update test_live_agentic_chat_with_exa.rs

### Priority 3: Frontend updates (React/TypeScript)
- 7 files need updates to use scope_uuid instead of toolset_id
- Mock data and test fixtures need updates
- UI rebuild required after changes

### Priority 4: API regeneration
```bash
cargo run --package xtask openapi
cd ts-client && npm run generate
make ci.ts-client-check
```

## Key Implementation Details

### Scope Derivation Pattern (CRITICAL)
The `scope` field in `Toolset` domain object is **NOT stored in database**. It must be derived:

```rust
fn toolset_row_to_model(&self, row: ToolsetRow) -> Toolset {
  // Lookup scope from registry using scope_uuid
  let scope = Self::builtin_toolsets(self.is_production)
    .iter()
    .find(|def| def.scope_uuid == row.scope_uuid)
    .map(|def| def.scope.clone())
    .unwrap_or_else(|| "scope_unknown".to_string());

  Toolset {
    scope_uuid: row.scope_uuid,  // From DB
    scope,                        // Derived from registry
    // ... rest
  }
}
```

### Environment-Specific UUIDs
```rust
let scope_uuid = if is_production {
  "7a89e236-9d23-4856-aa77-b52823ff9972"  // Production
} else {
  "4ff0e163-36fb-47d6-a5ef-26e396f067d6"  // Development
};
```

### DbService Method Names
Two methods for app toolset config lookup:
- `get_app_toolset_config_by_scope_uuid(&self, scope_uuid: &str)` - UUID-based
- `get_app_toolset_config_by_scope(&self, scope: &str)` - Scope string-based

Use `_by_scope` variant when looking up by scope string in implementation code.

### Keycloak Integration Change
In `is_app_client_registered_for_toolset()`, change JSON field comparison:
- Before: Compare `toolset.get("id")` with `toolset_id`
- After: Compare `toolset.get("scope_id")` with `scope_uuid`

## Recommended Approach for Fresh Session

1. **Start by fixing services/tool_service.rs:**
   - Run `cargo build --package services 2>&1 | grep "error\[E"` to see all errors
   - Use targeted sed replacements for bulk field renames
   - Fix method signatures in trait and implementation
   - Ensure tests pass: `cargo test --package services --lib`

2. **Test each crate in dependency order:**
   ```bash
   for crate in commands server_core auth_middleware routes_oai routes_app routes_all server_app integration-tests; do
     cargo test --package $crate --lib
   done
   ```

3. **Fix compilation errors systematically:**
   - Field access errors first (easiest)
   - Method signature mismatches
   - Missing method implementations
   - Test mock updates

4. **Only after backend compiles, update frontend:**
   - Regenerate OpenAPI specs
   - Update TypeScript client
   - Update React components
   - Rebuild UI
   - Run UI tests

## Success Criteria

- ✅ All backend crates compile without errors
- ✅ All backend tests pass: `make test.backend`
- ✅ OpenAPI spec regenerates successfully
- ✅ TypeScript client generates without errors
- ✅ Frontend compiles and UI tests pass: `make test.ui`
- ✅ Integration tests pass with new schema

## Files to Focus On

**Immediate attention needed:**
1. `crates/services/src/tool_service.rs` (has compilation errors)

**After tool_service.rs compiles:**
2. `crates/auth_middleware/src/toolset_auth_middleware.rs`
3. `crates/routes_app/src/routes_toolsets.rs`
4. `crates/routes_app/src/toolsets_dto.rs`
5. `crates/integration-tests/tests/test_live_agentic_chat_with_exa.rs`

**Frontend (last priority):**
6. `crates/bodhi/src/hooks/useToolsets.ts`
7. `crates/bodhi/src/app/ui/toolsets/admin/page.tsx`
8. Other UI files as needed

## Notes for Implementation

- Focus on Rust changes first - get backend compiling and tested
- Use systematic replacements (sed) for bulk field renames
- Test each crate before moving to next in dependency order
- Frontend changes only after backend is stable
- The plan file has full details on all required changes
