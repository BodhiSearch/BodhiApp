# Plan: Add toolset_types to ListToolsetsResponse, Replace toolset_id with scope/scope_uuid

## Summary
1. Replace `toolset_id` with `scope` and `scope_uuid` throughout the codebase
2. Make `scope_uuid` environment-aware (different UUIDs for dev vs prod)
3. Add `toolset_types` array to `ListToolsetsResponse` showing status (Enabled/Disabled/NotInstalled)
4. Update Keycloak comparison from `id`→`toolset_id` to `scope_id`→`scope_uuid`

## Scope Values
- **scope**: `scope_toolset-builtin-exa-web-search` (same for all environments)
- **scope_uuid (dev)**: `4ff0e163-36fb-47d6-a5ef-26e396f067d6`
- **scope_uuid (prod)**: `7a89e236-9d23-4856-aa77-b52823ff9972`

---

## ✅ Phase db-schema: Database Schema Changes - COMPLETED

### File: `crates/services/migrations/0007_toolsets_config.up.sql` - COMPLETED

**Changes to app_toolset_configs:**
1. ✅ Removed `toolset_id` column
2. ✅ Added `scope TEXT NOT NULL UNIQUE` column
3. ✅ Added `scope_uuid TEXT NOT NULL` column
4. ✅ Removed seed INSERT (moved to programmatic seeding)
5. ✅ Updated indexes

**Changes to toolsets (user instances):**
1. ✅ Replaced `toolset_type TEXT` with `scope_uuid TEXT`
2. ✅ Updated indexes

### File: `crates/services/migrations/0007_toolsets_config.down.sql` - COMPLETED
- ✅ Removed old index references
- ✅ Added drop for new indexes

---

## ✅ Phase domain-types: Update Domain Objects - COMPLETED

### File: `crates/objs/src/toolsets.rs` - COMPLETED

**Completed Changes:**
1. ✅ Updated ToolsetDefinition with scope_uuid + scope
2. ✅ Updated ToolsetWithTools with scope_uuid + scope
3. ✅ Updated AppToolsetConfig with scope + scope_uuid
4. ✅ Updated Toolset with scope_uuid + scope (scope NOT in DB, derived from registry)
5. ✅ Added ToolsetTypeInfo enum
6. ✅ Updated ToolsetScope (removed toolset_id field)
7. ✅ Removed toolset_id() and scope_for_toolset_id() methods
8. ✅ Removed BUILTIN_EXA_WEB_SEARCH_ID constant
9. ✅ Removed test for scope_for_toolset_id()

**All tests pass:** 425 passed

---

## ✅ Phase db-service: Update DbService - COMPLETED

### File: `crates/services/src/db/service.rs` - COMPLETED

**Completed Changes:**
1. ✅ Added is_production field to SqliteDbService
2. ✅ Added seed_toolset_configs() method with environment-specific scope_uuid
3. ✅ Updated migrate() to call seed_toolset_configs()
4. ✅ Renamed method: list_toolsets_by_type → list_toolsets_by_scope_uuid
5. ✅ Renamed method: get_app_toolset_config → get_app_toolset_config_by_scope_uuid
6. ✅ Added new method: get_app_toolset_config_by_scope
7. ✅ Updated all SQL queries to use scope/scope_uuid columns

### File: `crates/services/src/db/objs.rs` - COMPLETED
- ✅ Updated AppToolsetConfigRow (replaced toolset_id with scope + scope_uuid)
- ✅ Updated ToolsetRow (replaced toolset_type with scope_uuid)

### File: `crates/services/src/test_utils/db.rs` - COMPLETED
- ✅ Updated SqliteDbService::new() call to pass is_production (false for tests)
- ✅ Renamed method: list_toolsets_by_type → list_toolsets_by_scope_uuid
- ✅ Renamed method: get_app_toolset_config → get_app_toolset_config_by_scope_uuid
- ✅ Added new method: get_app_toolset_config_by_scope

---

## ✅ Phase app-builder: Pass is_production - COMPLETED

### File: `crates/lib_bodhiserver/src/app_service_builder.rs` - COMPLETED

**Completed Changes:**
- ✅ Updated get_or_build_db_service() to get is_production from SettingService and pass to SqliteDbService
- ✅ Updated get_or_build_tool_service() to get is_production from SettingService and pass to DefaultToolService

---

## ⚠️ Phase tool-service: Update ToolService - PARTIAL (60% complete)

### File: `crates/services/src/tool_service.rs` - IN PROGRESS

**Completed Changes:**
1. ✅ Added is_production field to DefaultToolService struct
2. ✅ Updated new() constructor to accept is_production parameter
3. ✅ Updated builtin_toolsets() to accept is_production and generate environment-specific scope_uuid
4. ✅ Updated all builtin_toolsets() calls to pass self.is_production
5. ✅ Updated test helper app_config_enabled() to use scope + scope_uuid
6. ✅ Updated test helper test_toolset_row() to use scope_uuid
7. ✅ Updated test mock expectations: expect_get_app_toolset_config → expect_get_app_toolset_config_by_scope
8. ✅ Updated toolset_row_to_model() to lookup scope from registry and return Toolset with both fields
9. ✅ Updated app_row_to_config() to use scope + scope_uuid fields

**Remaining Work:**
1. ❌ Update all field accesses in implementation methods:
   - `row.toolset_type` → `row.scope_uuid`
   - `toolset_def.toolset_id` → `toolset_def.scope_uuid`
   - `existing.toolset_type` → `existing.scope_uuid`
   - `instance.toolset_type` → `instance.scope_uuid`
2. ❌ Update trait method signatures:
   - `is_type_enabled(&self, toolset_type: &str)` → `is_type_enabled(&self, scope_uuid: &str)`
   - `set_app_toolset_enabled` parameters
   - `get_type` parameters
   - `create` parameters
3. ❌ Implement `get_toolset_types_for_scopes()` method
4. ❌ Update `is_app_client_registered_for_toolset()` to compare scope_id instead of id
5. ❌ Update all ToolsetRow constructions to include scope_uuid field
6. ❌ Update all test DefaultToolService::new() calls to pass is_production (false)
7. ❌ Fix method calls to DbService (get_app_toolset_config → get_app_toolset_config_by_scope)
8. ❌ Update all ToolsetWithTools constructions to use scope_uuid + scope

**Compilation Errors:** Multiple field access errors remaining

---

## ❌ Phase routes: Update Route Handlers - NOT STARTED

### Files Not Started:
- `crates/routes_app/src/routes_toolsets.rs`
- `crates/routes_app/src/toolsets_dto.rs`
- `crates/routes_app/src/lib.rs`

**Required Changes:**
1. Update endpoint paths: `/toolset_types/{type_id}/app-config` → `/toolset_types/{scope_uuid}/app-config`
2. Update handler parameter extraction
3. Add toolset_types population in list_toolsets_handler
4. Update all DTOs: ToolsetResponse, ToolsetTypeResponse, CreateToolsetRequest, ListToolsetsResponse

---

## ❌ Phase auth-middleware: Update Auth Middleware - NOT STARTED

### File: `crates/auth_middleware/src/toolset_auth_middleware.rs` - NOT STARTED

**Required Changes:**
1. Change `toolset.toolset_type` → `toolset.scope_uuid`
2. Update method calls to use scope_uuid parameters
3. Remove ToolsetScope::scope_for_toolset_id() usage
4. Update scope comparison to use toolset.scope field

---

## ❌ Phase frontend: Update React Components - NOT STARTED

### Files Not Started:
- `crates/bodhi/src/hooks/useToolsets.ts`
- `crates/bodhi/src/app/ui/toolsets/admin/page.tsx`
- `crates/bodhi/src/app/ui/toolsets/new/page.tsx`
- `crates/bodhi/src/app/ui/chat/ToolsetsPopover.tsx`
- `crates/bodhi/src/app/ui/setup/toolsets/SetupToolsetForm.tsx`
- `crates/bodhi/src/test-utils/msw-v2/handlers/toolsets.ts`
- `crates/bodhi/src/test-utils/fixtures/chat.ts`

---

## ❌ Phase openapi: Regenerate API Specs - NOT STARTED

```bash
cargo run --package xtask openapi
cd ts-client && npm run generate
```

---

## ❌ Phase testing: Update Tests - NOT STARTED

### Backend tests not updated:
- Integration tests in `crates/integration-tests/tests/test_live_agentic_chat_with_exa.rs`
- Other service tests

### Frontend tests not updated:
- Mock data
- Test selectors

---

## Implementation Notes & Corrections

### Key Discovery: Scope Derivation Pattern
**Important:** The `scope` field in the `Toolset` domain object is NOT stored in the database. When converting `ToolsetRow` → `Toolset`, the `toolset_row_to_model()` method must look up the scope from the toolset definition registry using the `scope_uuid`:

```rust
fn toolset_row_to_model(&self, row: ToolsetRow) -> Toolset {
  let scope = Self::builtin_toolsets(self.is_production)
    .iter()
    .find(|def| def.scope_uuid == row.scope_uuid)
    .map(|def| def.scope.clone())
    .unwrap_or_else(|| "scope_unknown".to_string());

  Toolset {
    id: row.id,
    name: row.name,
    scope_uuid: row.scope_uuid,
    scope,  // Derived from registry
    // ... rest
  }
}
```

### Method Signature Change
The `toolset_row_to_model()` method signature changed from static to instance method to access `self.is_production`:
- Before: `fn toolset_row_to_model(row: ToolsetRow) -> Toolset`
- After: `fn toolset_row_to_model(&self, row: ToolsetRow) -> Toolset`

This requires updating all call sites from `Self::toolset_row_to_model(row)` to `self.toolset_row_to_model(row)`.

### DbService Method Renamings
The trait now has TWO methods for app toolset config lookup:
1. `get_app_toolset_config_by_scope_uuid(&self, scope_uuid: &str)` - for UUID-based lookups
2. `get_app_toolset_config_by_scope(&self, scope: &str)` - for scope string-based lookups

All implementation code should use the `_by_scope` variant when looking up by scope string.

---

## Files Summary with Status

| File | Status | Progress |
|------|--------|----------|
| `crates/services/migrations/0007_toolsets_config.up.sql` | ✅ Complete | 100% |
| `crates/services/migrations/0007_toolsets_config.down.sql` | ✅ Complete | 100% |
| `crates/objs/src/toolsets.rs` | ✅ Complete | 100% |
| `crates/services/src/db/service.rs` | ✅ Complete | 100% |
| `crates/services/src/db/objs.rs` | ✅ Complete | 100% |
| `crates/services/src/test_utils/db.rs` | ✅ Complete | 100% |
| `crates/lib_bodhiserver/src/app_service_builder.rs` | ✅ Complete | 100% |
| `crates/services/src/tool_service.rs` | ⚠️ Partial | ~60% |
| `crates/auth_middleware/src/toolset_auth_middleware.rs` | ❌ Not Started | 0% |
| `crates/routes_app/src/routes_toolsets.rs` | ❌ Not Started | 0% |
| `crates/routes_app/src/toolsets_dto.rs` | ❌ Not Started | 0% |
| `crates/routes_app/src/lib.rs` | ❌ Not Started | 0% |
| `crates/services/src/test_utils/db.rs` | ✅ Complete | 100% |
| `crates/integration-tests/tests/test_live_agentic_chat_with_exa.rs` | ❌ Not Started | 0% |
| Frontend files (7 files) | ❌ Not Started | 0% |

**Overall Progress:** ~40% complete (backend foundations done, implementation details pending)
