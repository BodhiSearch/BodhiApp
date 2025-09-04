# Alias Refactoring Tasks & Progress

## Overview
This document tracks the phase-by-phase implementation progress for the alias system refactoring from fragmented data access to unified enum-based architecture.

## Progress Summary
- ✅ **Phase 1-2 Completed**: objs crate with tagged enum (source field elimination)
- 🔄 **Phase 3 In Progress**: services crate compilation fixes + unified architecture  
- ⏳ **Phase 4-10 Pending**: server_core, routes, commands, frontend updates

---

## ✅ Phase 1-2: Foundation (COMPLETED)

### Objs Crate Tagged Enum Implementation
- ✅ Created `ModelAlias` struct for auto-discovered models
- ✅ Updated `Alias` enum to use tagged serialization with `#[serde(tag = "source")]`
- ✅ Removed source fields from individual structs (UserAlias, ModelAlias, ApiAlias) 
- ✅ Updated `ApiAlias` Display implementation
- ✅ Fixed all objs crate tests (362 tests passing)
- ✅ Updated test YAML files to include source field
- ✅ Verified tagged enum serialization produces clean output
- ✅ Formatted code with `cargo fmt`

**Key Achievement**: Eliminated duplicate source field serialization issue while maintaining clean JSON/YAML output.

---

## ✅ Phase 3: Services Crate Unified Architecture (COMPLETED)

### Immediate Compilation Fixes
- ✅ Change `AliasSource::RemoteApi` → `AliasSource::Api` (6 locations)
  - `crates/services/src/db/service.rs:768, 859, 1326, 1359, 1401, 1462, 1493, 1525, 1541`
  - `crates/services/src/test_utils/objs.rs:22`
- ✅ Remove `source` parameter from `ApiAlias::new()` calls (9 locations)
- ✅ Remove direct `source` field assignments in ApiAlias struct creation
- ✅ Update HubService: `UserAliasBuilder` → `ModelAliasBuilder`
- ✅ Remove `.source()` calls from HubService implementation

### Unified Data Service Architecture  
- ✅ Add `db_service: Arc<dyn DbService>` to `LocalDataService` struct
- ✅ Update `LocalDataService::new()` constructor signature
- ✅ Make `DataService` trait methods async:
  - `list_aliases(&self) -> Result<Vec<UserAlias>>` → `async fn list_aliases(&self) -> Result<Vec<Alias>>`
  - `find_alias(&self, alias: &str) -> Option<UserAlias>` → `async fn find_alias(&self, alias: &str) -> Option<Alias>`
- ✅ Implement unified internal logic:
  - User aliases from YAML files → `Alias::User(...)`
  - Model aliases from HubService → `Alias::Model(...)`  
  - API aliases from DbService → `Alias::Api(...)`
- ✅ Update HubService trait: `list_model_aliases() -> Vec<UserAlias>` → `Vec<ModelAlias>`

### Commands to Run
```bash
# Fix compilation
cargo check -p services

# Run tests
cargo test -p services

# Format code  
cargo fmt -p services
```

---

## ✅ Phase 4: Server Core Updates (COMPLETED)

### Model Router Simplification
- ✅ Replace multiple `find_alias()` calls with single lookup + pattern matching
- ✅ Update `RouteDestination` handling for all three alias types
- ✅ Add `.await` to async `DataService` calls
- ✅ Remove separate `db_service.get_api_model_alias()` calls

### SharedContext Interface Updates
- ✅ Update `chat_completions` method to accept `Alias` enum instead of `UserAlias`
- ✅ Add pattern matching to extract fields from User/Model aliases
- ✅ Reject API aliases with appropriate error (they route to AiApiService)
- ✅ Handle context_params differences between User and Model aliases

### Router State Updates
- ✅ Update DefaultModelRouter constructor to remove DbService dependency
- ✅ Update chat_completions call to pass Alias enum to SharedContext

### Test Updates
- ✅ Fixed all model router tests to use new Alias enum structure
- ✅ Removed outdated "priority" concept from test names and logic
- ✅ Simplified mock setups to match unified architecture

### Commands to Run
```bash
cargo check -p server_core  # ✅ PASSED
cargo test -p server_core   # ✅ PASSED (92/92 tests)
cargo fmt -p server_core    # ✅ COMPLETED
```

**Key Achievement**: Simplified routing from 3 separate service calls to 1 unified call with pattern matching. All 92 tests passing.

---

## ⏳ Phase 5-6: Routes Layer Simplification (PENDING)

### Routes Models Update
- ⏳ Replace manual merging in `list_local_aliases_handler()`:
  - Remove: `data_service.list_aliases()` + `db_service.list_api_model_aliases()`  
  - Replace with: Single `data_service.list_aliases().await?` call
- ⏳ Remove `UnifiedModelResponse` wrapper - use `Alias` enum directly
- ⏳ Update sorting and pagination logic
- ⏳ Update OpenAPI schema generation

### Routes API Models
- ⏳ Review `routes_api_models.rs` - may not need changes (API-specific endpoint)

### Commands to Run
```bash
cargo check -p routes_app
cargo check -p routes_oai  
cargo test -p routes_app
cargo test -p routes_oai
```

---

## ⏳ Phase 7: Commands Layer (PENDING)

### Update Commands
- ⏳ Add `.await` to `data_service.find_alias()` calls in `cmd_create.rs`
- ⏳ Update pattern matching for `Alias` enum instead of source filtering
- ⏳ Update error handling for different alias types

### Commands to Run
```bash
cargo check -p commands
cargo test -p commands
```

---

## ⏳ Phase 8: Service Construction Updates (PENDING)

### Update All LocalDataService::new() Call Sites
- ⏳ `services/src/test_utils/app.rs:150` - Add `db_service` parameter
- ⏳ `integration-tests/tests/utils/live_server_utils.rs:132` - Add `db_service`
- ⏳ Any other app service builders found during implementation

### Update DefaultAppService
- ⏳ Verify `derive_new::new` handles new parameter order automatically
- ⏳ Update any manual construction sites

---

## ⏳ Phase 9: TypeScript Generation (PENDING)

### OpenAPI & Client Updates
```bash
# Generate OpenAPI spec
cargo run --package xtask openapi

# Generate TypeScript types
cd ts-client
npm run generate
npm run build  
npm test
```

### Expected Changes
- ⏳ `UnifiedModelResponse` type removed
- ⏳ New `Alias` discriminated union type
- ⏳ Updated API response types

---

## ⏳ Phase 10: Frontend Updates (PENDING)

### UI Components
- ⏳ Update imports: `UnifiedModelResponse` → `Alias`
- ⏳ Update type guards and pattern matching  
- ⏳ Update model display logic for new structure
- ⏳ Update query hooks for new response types

### Commands to Run
```bash
cd crates/bodhi
npm run dev     # Test UI
npm test        # Run tests
```

---

## Integration Testing Checklist

### Manual Testing (After All Phases)
- [ ] Create user alias via UI → appears in unified listing
- [ ] Create user alias via CLI → appears in unified listing  
- [ ] Auto-discovered models → appear in unified listing
- [ ] API model configuration → appears in unified listing
- [ ] Chat with user-created alias → works
- [ ] Chat with auto-discovered model → works  
- [ ] Chat with API model → works
- [ ] Model selection in chat interface → all types available
- [ ] Sorting/filtering in models page → works across all types
- [ ] Copy user alias → works (other types show appropriate error)
- [ ] Delete user alias → works (other types show appropriate error)

### Performance Testing
- [ ] Compare old vs new approach response times
- [ ] Memory usage with large alias sets
- [ ] Database connection handling

### Automated Testing
```bash
# Full test suite
make test

# Individual test suites  
make test.backend
make test.ui
make test.napi
```

---

## Current Blockers & Issues

### Phase 3 Current Issues
1. **Services Compilation Errors**: Need to fix AliasSource enum references and constructor calls
2. **Async Propagation**: Making DataService async affects all consumers
3. **Dependency Injection**: Need to ensure DbService is available where LocalDataService is constructed

### Resolution Strategy
1. Fix immediate compilation errors first
2. Implement unified DataService incrementally
3. Update consumers phase by phase
4. Maintain backward compatibility during transition where possible

---

## Notes for Resumption

### If Work is Interrupted
1. Check `cargo check -p services` to see current compilation status
2. Review git status to see modified files
3. Continue from current phase in this task list
4. Use detailed code examples in plan.md for implementation guidance

### Key Implementation Files
- **Services**: `crates/services/src/data_service.rs`, `crates/services/src/db/service.rs`
- **Server Core**: `crates/server_core/src/model_router.rs`  
- **Routes**: `crates/routes_app/src/routes_models.rs`
- **Commands**: `crates/commands/src/cmd_create.rs`

### Success Validation
Each phase completion verified by:
1. `cargo check -p <crate>` passes
2. `cargo test -p <crate>` passes
3. No regressions in dependent crates
4. Manual testing of affected functionality