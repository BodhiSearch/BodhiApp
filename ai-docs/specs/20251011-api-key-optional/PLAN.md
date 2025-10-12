# API Key Optional Feature - Implementation Plan
**Spec ID**: 20251011-api-key-optional
**Status**: Phases 1-7 COMPLETED | Phases 8-10 PENDING
**Last Updated**: 2025-10-11

## 🎯 Feature Overview
Make API key field optional in API models, similar to prefix field functionality. Users can configure API models without API keys when planning to use stored credentials.

---

## 📊 Current Progress Summary

### ✅ COMPLETED PHASES (1-7)
All foundational layers implemented and code changes applied:
- **Phase 1**: Database schema migration
- **Phase 2**: Service layer (DbService trait/impl)
- **Phase 3**: Backend API layer (DTOs, routes)
- **Phase 4**: TypeScript client regeneration
- **Phase 5**: Frontend schemas & validation
- **Phase 6**: Frontend components (ApiKeyInput, ApiModelForm)
- **Phase 7**: Frontend business logic hooks

### ⏳ REMAINING PHASES (8-10)
Testing, validation, and documentation:
- **Phase 8**: Frontend unit tests update
- **Phase 9**: Backend test verification & UI rebuild
- **Phase 10**: Integration testing & documentation

---

## 📋 Phase Breakdown

### **PHASE 8: Frontend Unit Tests Update** ⏳
**Agent**: `general-purpose`
**Dependencies**: Phases 1-7 complete
**Objective**: Update all frontend unit tests to support new checkbox functionality

**Tasks**:
1. Update `ApiModelForm.test.tsx`:
   - Add tests for `useApiKey` checkbox behavior
   - Update existing tests to check/uncheck API key checkbox
   - Test form submission with/without API key
   - Test fetch models with checkbox enabled/disabled
   - Test connection with checkbox enabled/disabled
2. Update MSW handlers if needed:
   - Verify `api-models.ts` handlers support optional api_key
   - Update mock responses to reflect new behavior
3. Update `apiModel.ts` schema tests (if any exist)

**Validation**:
```bash
cd crates/bodhi && npm test -- ApiModelForm.test.tsx
```

**Success Criteria**:
- All frontend unit tests pass
- Test coverage includes checkbox functionality
- No regression in existing functionality

**Logging**:
- Activity log: `ai-docs/specs/20251011-api-key-optional/phase-8-frontend-tests.log`
- Context insights: `ai-docs/specs/20251011-api-key-optional/phase-8-context.md`

---

### **PHASE 9: Backend Test Verification & UI Rebuild** ⏳
**Agent**: `general-purpose`
**Dependencies**: Phase 8 complete
**Objective**: Verify all backend tests pass and rebuild embedded UI

**Tasks**:
1. **Backend Test Verification**:
   ```bash
   # Run database tests
   cargo test -p services -- db::service

   # Run route tests
   cargo test -p routes_app -- routes_api_models

   # Run all backend tests
   cargo test
   ```

2. **UI Rebuild** (CRITICAL for embedded changes):
   ```bash
   make clean.ui
   make build.ui
   ```

3. **Rust Formatting**:
   ```bash
   cargo fmt --all
   ```

**Validation**:
- All backend tests pass
- UI builds successfully
- No compilation errors

**Success Criteria**:
- Zero backend test failures
- UI embedded build contains updated components
- Code properly formatted

**Logging**:
- Activity log: `ai-docs/specs/20251011-api-key-optional/phase-9-backend-ui.log`
- Context insights: `ai-docs/specs/20251011-api-key-optional/phase-9-context.md`

---

### **PHASE 10: Integration Testing & Documentation** ⏳
**Agent**: `general-purpose`
**Dependencies**: Phase 9 complete
**Objective**: Comprehensive integration testing and spec documentation

**Tasks**:
1. **Integration Tests**:
   - Run full UI test suite
   - Manual verification of create/edit flows
   - Test API model creation without API key
   - Test fetch models/test connection with stored credentials

2. **Playwright Tests** (if exist):
   ```bash
   cd crates/bodhi && npm run test:playwright
   ```

3. **Create Spec Documentation**:
   - Feature overview
   - Architecture changes
   - API contract changes
   - Migration guide
   - Testing checklist

**Validation**:
```bash
# Run all tests
make test
```

**Success Criteria**:
- All integration tests pass
- Manual testing complete
- Spec documentation created

**Deliverables**:
- `ai-docs/specs/20251011-api-key-optional/README.md` - Feature specification
- `ai-docs/specs/20251011-api-key-optional/ARCHITECTURE.md` - Technical design
- `ai-docs/specs/20251011-api-key-optional/TESTING.md` - Test strategy & results

**Logging**:
- Activity log: `ai-docs/specs/20251011-api-key-optional/phase-10-integration.log`
- Context insights: `ai-docs/specs/20251011-api-key-optional/phase-10-context.md`

---

## 🏗️ Technical Architecture

### Layer Sequence (Bottom-Up)
```
Database (✅ Phase 1)
    ↓
Service Layer (✅ Phase 2)
    ↓
Backend API (✅ Phase 3)
    ↓
TypeScript Client (✅ Phase 4)
    ↓
Frontend Schema (✅ Phase 5)
    ↓
Frontend Components (✅ Phase 6)
    ↓
Frontend Logic (✅ Phase 7)
    ↓
Frontend Tests (⏳ Phase 8)
    ↓
Backend Verification (⏳ Phase 9)
    ↓
Integration Testing (⏳ Phase 10)
```

### Key Changes Summary

**Database**:
- `encrypted_api_key`, `salt`, `nonce` → nullable columns
- Migration: `0005_optional_api_keys.{up,down}.sql`

**Backend**:
- `DbService::create_api_model_alias(api_key: Option<&str>)`
- `CreateApiModelRequest.api_key: Option<String>`
- Conditional encryption logic

**Frontend**:
- Schema: `useApiKey: boolean` checkbox field
- Component: `ApiKeyInput` with checkbox control
- Conversion: Only include api_key when checkbox enabled
- Logic: Test/fetch operations support optional API key

---

## 📁 Modified Files

### Backend (13 files)
- `crates/services/migrations/0005_optional_api_keys.up.sql` [NEW]
- `crates/services/migrations/0005_optional_api_keys.down.sql` [NEW]
- `crates/services/src/db/service.rs`
- `crates/services/src/test_utils/db.rs`
- `crates/services/src/test_utils/objs.rs`
- `crates/services/src/data_service.rs`
- `crates/routes_app/src/api_models_dto.rs`
- `crates/routes_app/src/routes_api_models.rs`
- `openapi.json`

### TypeScript Client (2 files)
- `ts-client/src/types/types.gen.ts`
- `ts-client/src/openapi-typescript/openapi-schema.ts`

### Frontend (4 files)
- `crates/bodhi/src/schemas/apiModel.ts`
- `crates/bodhi/src/components/api-models/form/ApiKeyInput.tsx`
- `crates/bodhi/src/components/api-models/ApiModelForm.tsx`
- `crates/bodhi/src/components/api-models/hooks/useApiModelForm.ts`

### Pending Updates (Phase 8-10)
- `crates/bodhi/src/components/api-models/ApiModelForm.test.tsx`
- `crates/bodhi/src/test-utils/msw-v2/handlers/api-models.ts` (verify)

---

## 🚀 Execution Strategy

### Agent Communication Protocol
Each phase agent will:
1. **Log all activities** to phase-specific `.log` file
2. **Document domain insights** in phase-specific `-context.md` file
3. **Report test results** with pass/fail status
4. **Block on failures** - do not proceed if tests fail
5. **Communicate dependencies** - verify previous phase completion

### Sequential Execution
- Phases execute **sequentially** (8 → 9 → 10)
- Each phase **must complete successfully** before next phase starts
- Test failures **block progression** to next phase

### Rollback Strategy
If any phase fails:
1. Document failure in phase log
2. Identify root cause
3. Fix issues
4. Re-run phase tests
5. Only proceed when all tests pass

---

## ✅ Acceptance Criteria

### Phase 8 (Frontend Tests)
- [ ] All `ApiModelForm.test.tsx` tests pass
- [ ] Checkbox behavior tests added
- [ ] No test regressions

### Phase 9 (Backend & UI Build)
- [ ] All backend tests pass (`cargo test`)
- [ ] UI builds successfully (`make build.ui`)
- [ ] No compilation errors

### Phase 10 (Integration)
- [ ] Full test suite passes (`make test`)
- [ ] Manual testing complete
- [ ] Spec documentation created
- [ ] Feature ready for production

---

## 📝 Next Steps

1. **Review this plan** for completeness
2. **Approve execution** of remaining phases
3. **Run Phase 8**: Update frontend tests with agent logging
4. **Run Phase 9**: Verify backend tests & rebuild UI
5. **Run Phase 10**: Integration testing & documentation

**Estimated Time**: 2-3 hours for Phases 8-10
