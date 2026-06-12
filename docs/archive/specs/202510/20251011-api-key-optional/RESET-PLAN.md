# API Key Optional Feature - RESET Implementation Plan
**Spec ID**: 20251011-api-key-optional
**Status**: RESET - Starting Fresh with Ownership Fix
**Last Updated**: 2025-10-11

## 🔄 Reset Reason
Compilation error due to lifetime issues with `Option<&str>` in trait method signature.
**Solution**: Use `Option<String>` for ownership instead of references.

---

## 📋 Phased Implementation Plan

### **PHASE 1: Database Schema Migration** ⏳
**Agent**: `general-purpose`
**Objective**: Create migration for nullable API key fields and verify with tests

**Tasks**:
1. Review existing migration files:
   - `crates/services/migrations/0005_optional_api_keys.up.sql`
   - `crates/services/migrations/0005_optional_api_keys.down.sql`
2. Run database migration tests:
   ```bash
   cargo test -p services -- db::migrations
   cargo test -p services -- db::service::tests
   ```

**Success Criteria**:
- Migration files correct
- All database tests pass
- Schema supports nullable `encrypted_api_key`, `salt`, `nonce`

**Logging**:
- `ai-docs/specs/20251011-api-key-optional/phase-1-db-migration.log`
- `ai-docs/specs/20251011-api-key-optional/phase-1-context.md`

---

### **PHASE 2: Service Layer Ownership Fix** ⏳
**Agent**: `general-purpose`
**Objective**: Fix DbService trait to use `Option<String>` instead of `Option<&str>`

**Core Changes Required**:
```rust
// DbService trait signature
async fn create_api_model_alias(
  &self,
  alias: &ApiAlias,
  api_key: Option<String>  // Changed from Option<&str>
) -> Result<(), DbError>;

async fn update_api_model_alias(
  &self,
  alias: &str,
  model: &ApiAlias,
  api_key: Option<String>  // Changed from Option<&str>
) -> Result<(), DbError>;
```

**Files to Update**:
1. `crates/services/src/db/service.rs`
   - Update trait method signatures
   - Update implementation to handle owned String
   - Adjust encryption calls
2. `crates/services/src/test_utils/db.rs`
   - Update TestDbService wrapper
3. `crates/services/src/test_utils/objs.rs`
   - Update seed function calls
4. `crates/services/src/data_service.rs`
   - Update all call sites

**Testing**:
```bash
cargo test -p services
```

**Success Criteria**:
- No compilation errors
- All service tests pass
- mockall works with owned types

**Logging**:
- `ai-docs/specs/20251011-api-key-optional/phase-2-service-layer.log`
- `ai-docs/specs/20251011-api-key-optional/phase-2-context.md`

---

### **PHASE 3: Backend API Layer** ⏳
**Agent**: `general-purpose`
**Objective**: Update route handlers for optional API key with proper conversions

**Files to Update**:
1. `crates/routes_app/src/routes_api_models.rs`
   - Update `create_api_model_handler`
   - Update `update_api_model_handler`
   - Convert `Option<String>` properly for service calls
2. Remove implementation comments from previous attempt

**Testing**:
```bash
cargo test -p routes_app -- routes_api_models
```

**Success Criteria**:
- All route tests pass
- Proper conversion between API layer and service layer
- No inline comments about implementation

**Logging**:
- `ai-docs/specs/20251011-api-key-optional/phase-3-routes.log`
- `ai-docs/specs/20251011-api-key-optional/phase-3-context.md`

---

### **PHASE 4: Full Backend Verification** ⏳
**Agent**: `general-purpose`
**Objective**: Run all backend tests and format code

**Tasks**:
```bash
cargo test
cargo fmt --all
```

**Success Criteria**:
- Zero test failures across all crates
- Code properly formatted
- No warnings

**Logging**:
- `ai-docs/specs/20251011-api-key-optional/phase-4-backend-verify.log`
- `ai-docs/specs/20251011-api-key-optional/phase-4-context.md`

---

### **PHASE 5: TypeScript Client Regeneration** ⏳
**Agent**: `general-purpose`
**Objective**: Regenerate TypeScript types from updated OpenAPI spec

**Tasks**:
```bash
cargo run --package xtask openapi
cd ts-client && npm run generate
```

**Success Criteria**:
- OpenAPI spec reflects optional api_key
- TypeScript types updated
- No type errors

**Logging**:
- `ai-docs/specs/20251011-api-key-optional/phase-5-ts-client.log`
- `ai-docs/specs/20251011-api-key-optional/phase-5-context.md`

---

### **PHASE 6: Frontend Schema & Components** ⏳
**Agent**: `general-purpose`
**Objective**: Update frontend schemas and components (already done, verify no comments)

**Tasks**:
1. Review and clean up:
   - `crates/bodhi/src/schemas/apiModel.ts`
   - `crates/bodhi/src/components/api-models/form/ApiKeyInput.tsx`
   - `crates/bodhi/src/components/api-models/ApiModelForm.tsx`
   - `crates/bodhi/src/components/api-models/hooks/useApiModelForm.ts`
2. Remove any implementation comments

**Success Criteria**:
- No inline comments about implementation
- Code is clean and self-documenting

**Logging**:
- `ai-docs/specs/20251011-api-key-optional/phase-6-frontend-components.log`
- `ai-docs/specs/20251011-api-key-optional/phase-6-context.md`

---

### **PHASE 7: Frontend Unit Tests** ⏳
**Agent**: `general-purpose`
**Objective**: Clean up tests and verify they pass

**Tasks**:
1. Review `crates/bodhi/src/components/api-models/ApiModelForm.test.tsx`
2. Remove inline comments about implementation
3. Run tests:
   ```bash
   cd crates/bodhi && npm test -- ApiModelForm.test.tsx
   ```

**Success Criteria**:
- All frontend tests pass
- No inline comments
- Clean test code

**Logging**:
- `ai-docs/specs/20251011-api-key-optional/phase-7-frontend-tests.log`
- `ai-docs/specs/20251011-api-key-optional/phase-7-context.md`

---

### **PHASE 8: UI Rebuild** ⏳
**Agent**: `general-purpose`
**Objective**: Rebuild embedded UI with all changes

**Tasks**:
```bash
make clean.ui
make build.ui
```

**Success Criteria**:
- UI builds successfully
- No build errors
- Embedded UI contains all updates

**Logging**:
- `ai-docs/specs/20251011-api-key-optional/phase-8-ui-rebuild.log`
- `ai-docs/specs/20251011-api-key-optional/phase-8-context.md`

---

### **PHASE 9: Integration Testing** ⏳
**Agent**: `general-purpose`
**Objective**: Run full test suite

**Tasks**:
```bash
make test
```

**Success Criteria**:
- All tests pass (backend + frontend + integration)
- Zero failures

**Logging**:
- `ai-docs/specs/20251011-api-key-optional/phase-9-integration.log`
- `ai-docs/specs/20251011-api-key-optional/phase-9-context.md`

---

### **PHASE 10: Documentation** ⏳
**Agent**: `general-purpose`
**Objective**: Create final documentation

**Deliverables**:
- `ai-docs/specs/20251011-api-key-optional/README.md`
- `ai-docs/specs/20251011-api-key-optional/ARCHITECTURE.md`
- `ai-docs/specs/20251011-api-key-optional/TESTING.md`

**Success Criteria**:
- Complete feature documentation
- Architecture diagrams
- Test coverage report

**Logging**:
- `ai-docs/specs/20251011-api-key-optional/phase-10-docs.log`
- `ai-docs/specs/20251011-api-key-optional/phase-10-context.md`

---

## 🔑 Key Technical Decision

### Ownership vs References
**Problem**: `mockall::automock` doesn't support `Option<&str>` in trait methods (lifetime issues)

**Solution**: Use `Option<String>` for ownership

**Impact**:
- Service layer receives owned `String`
- Minimal allocation overhead (one-time per API call)
- Clean trait signatures without lifetime annotations
- Works seamlessly with mockall

**Conversion Pattern**:
```rust
// API layer receives Option<String> from JSON
let api_key: Option<String> = payload.api_key;

// Pass to service layer as-is
db_service.create_api_model_alias(&alias, api_key).await?;

// Service layer signature
async fn create_api_model_alias(
  &self,
  alias: &ApiAlias,
  api_key: Option<String>  // Owned
) -> Result<(), DbError>;
```

---

## 📊 Progress Tracking

- [ ] Phase 1: Database Migration
- [ ] Phase 2: Service Layer Ownership Fix
- [ ] Phase 3: Backend API Layer
- [ ] Phase 4: Full Backend Verification
- [ ] Phase 5: TypeScript Client Regeneration
- [ ] Phase 6: Frontend Schema & Components
- [ ] Phase 7: Frontend Unit Tests
- [ ] Phase 8: UI Rebuild
- [ ] Phase 9: Integration Testing
- [ ] Phase 10: Documentation

---

## 🚀 Execution Strategy

### Sequential Execution with Test Gates
1. Each phase runs sequentially
2. Tests MUST pass before proceeding to next phase
3. Compilation errors block progression
4. Agent logs all activities to phase-specific log files
5. Agent documents insights in phase-specific context files

### Rollback Strategy
If any phase fails:
1. Document failure in phase log
2. Fix the issue
3. Re-run phase tests
4. Only proceed when tests pass

---

## 📁 Files to Modify

### Backend
1. `crates/services/src/db/service.rs` - Trait signature fix
2. `crates/services/src/test_utils/db.rs` - Test wrapper
3. `crates/services/src/test_utils/objs.rs` - Seed functions
4. `crates/services/src/data_service.rs` - Call sites
5. `crates/routes_app/src/routes_api_models.rs` - Route handlers

### Frontend (Clean Comments)
6. `crates/bodhi/src/schemas/apiModel.ts`
7. `crates/bodhi/src/components/api-models/form/ApiKeyInput.tsx`
8. `crates/bodhi/src/components/api-models/ApiModelForm.tsx`
9. `crates/bodhi/src/components/api-models/hooks/useApiModelForm.ts`
10. `crates/bodhi/src/components/api-models/ApiModelForm.test.tsx`

---

## ✅ Final Acceptance Criteria

- [ ] All backend tests pass (`cargo test`)
- [ ] All frontend tests pass (`npm test`)
- [ ] UI builds successfully (`make build.ui`)
- [ ] Full test suite passes (`make test`)
- [ ] No inline implementation comments
- [ ] Complete documentation
- [ ] Feature ready for production

**Estimated Time**: 3-4 hours for complete reset and verification
