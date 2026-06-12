# Phase 4 Final Context: Pre/Post-Exchange Validation - Q&A Session

## Session Summary

Comprehensive interview conducted after Phase 4 implementation to identify gaps and plan verification/enhancement. Phases 1-6 were implemented (commits 25fd85f1 through 33c51630). This session clarifies the missing pre-token-exchange validation and post-exchange verification logic.

---

## 1. What Has Been Implemented (Phases 1-6)

**Phase 1-2 (commit 25fd85f1):**
- ✅ JSON field renaming: `tool_types` → `toolset_types` in domain objects, services, routes
- ✅ Token exchange scope forwarding: Added `scope_access_request:*` prefix to scope filtering in token_service.rs

**Phase 3-4 (commit f4a6ce1b):**
- ✅ Added `access_request_id: Option<String>` field to ScopeClaims struct
- ✅ Header constant: `KEY_HEADER_BODHIAPP_ACCESS_REQUEST_ID`
- ✅ Header injection: Extract access_request_id from exchanged token claims, inject into request headers
- ✅ Extractor: `MaybeAccessRequestId` for optional access_request_id extraction

**Phase 5 (commit 70ee94eb):**
- ✅ Toolset auth middleware redesign with dual auth flow (session vs OAuth)
- ✅ 7 new error variants for access request validation
- ✅ OAuth validation: status=approved, app_client_id match, user_id match, instance in approved list
- ✅ Comprehensive test coverage (145 auth_middleware tests, 286 services tests, 422 routes_app tests)

**Phase 6 (commit 33c51630):**
- ✅ Removed deprecated `X-BodhiApp-Tool-Scopes` header completely
- ✅ Cleaned up old scope-based authorization system

---

## 2. Identified Gaps

**Critical Missing Features:**
1. ❌ No pre-token-exchange validation of `scope_access_request:<uuid>` existence in DB
2. ❌ No verification that app_client_id, user_id, status match BEFORE token exchange
3. ❌ No post-exchange verification that `access_request_id` claim matches record.id
4. ❌ No database index + unique constraint on `access_request_scope` column
5. ❌ No repository method: `get_by_access_request_scope()`

**Current Flow Problem:**
- External token with `scope_access_request:<uuid>` goes directly to token exchange
- No validation that the scope exists or is approved
- Validation only happens downstream in toolset_auth_middleware
- This allows invalid/expired/denied scopes to reach Keycloak

---

## 3. Validation Flow Architecture

**Q: When should access_request_scope validation happen?**

**Pre-Token-Exchange Validation (in token_service.rs validate_bearer_token):**
1. Extract `scope_access_request:<uuid>` from external token scopes
2. Look up record in DB by `access_request_scope = "scope_access_request:<uuid>"`
3. Verify record exists
4. Verify `record.app_client_id === token.azp`
5. Verify `record.user_id === token.sub`
6. Verify `record.status === "approved"`
7. If any check fails: return 403 Forbidden with specific error code
8. If all pass: include scope in token exchange scopes

**Post-Token-Exchange Verification:**
1. Extract `access_request_id` claim from exchanged token
2. Verify `access_request_id === record.id` (the primary key, NOT the scope uuid)
3. If match: inject `X-BodhiApp-Access-Request-Id` header
4. If mismatch: return 403 Forbidden with specific error code

**Rationale:** Pre-validation fails fast for invalid scopes, preventing unnecessary Keycloak calls. Post-verification ensures KC returned the correct claim for the approved request.

---

## 4. Scope vs Access Request ID Relationship

**Q: What's the relationship between scope_access_request:<uuid> and access_request_id claim?**

**Current Design:**
- `scope_access_request:<uuid>` is in the incoming external token (set by app after user approval)
- This scope value is stored in DB as `access_request_scope` field
- After token exchange, KC returns `access_request_id` claim
- The `access_request_id` claim should equal `record.id` (the primary key)

**Future Flexibility:**
- Right now `<uuid>` in scope and claim are the same
- But we're keeping it flexible so they can diverge in future
- **Lookup logic:** Always look up by `access_request_scope`, then match `access_request_id` claim against `record.id`

**Why Not Match Scope UUID Directly:**
- Allows KC to use different identifier in claim if needed
- Keeps DB primary key as source of truth
- Enables future scenarios where scope is a different format than claim

---

## 5. Validation Trigger Conditions

**Q: Should validation run for ALL bearer tokens or only specific cases?**

**Decision: Conditional Validation**
- Validation runs ONLY for external bearer tokens
- AND ONLY when `scope_access_request:*` prefix exists in token scopes
- If scope not present: skip validation entirely (e.g., auto-approved requests, user-only scopes)

**Implementation:**
- In `validate_bearer_token`, after extracting claims from external token
- Check if any scope starts with `"scope_access_request:"`
- If yes: run pre-validation
- If no: proceed with token exchange without validation

**Where NOT to Validate:**
- `inject_optional_auth_info` middleware path (optional auth)
- Session-based authentication flows
- Tokens with only `scope_user_*` or `scope_resource-*` scopes

---

## 6. Error Handling Design

**Q: What error codes should be returned for validation failures?**

**Decision: New Dedicated Error Enum**
- Create `AccessRequestValidationError` enum with specific variants
- Add as transparent variant to `TokenError` enum
- Variants:
  - `ScopeNotFound` - No record with matching access_request_scope
  - `NotApproved` - Record status is not "approved"
  - `AppClientMismatch` - record.app_client_id != token.azp
  - `UserMismatch` - record.user_id != token.sub
  - `AccessRequestIdMismatch` - access_request_id claim != record.id

**Error Response Pattern:**
- All pre-validation failures: 403 Forbidden
- Post-verification mismatch: 403 Forbidden
- Each error includes: error code, user-friendly message, structured args

**Transparency:**
- `TokenError::AccessRequestValidation(#[from] AccessRequestValidationError)` with `#[error(transparent)]`
- Error code delegates to inner enum (e.g., `"access_request_validation_error-scope_not_found"`)

---

## 7. Database Schema Changes

**Q: Should access_request_scope have a unique constraint? What about NULLs?**

**Decision: UNIQUE Constraint Allowing NULLs**
- Add `CREATE UNIQUE INDEX idx_access_request_scope_unique ON app_access_requests(access_request_scope) WHERE access_request_scope IS NOT NULL;`
- Allows multiple NULL values (auto-approved requests)
- Prevents duplicate non-NULL scopes (KC-assigned scopes)

**Migration Strategy:**
- Modify existing migration `0011_app_access_requests.up.sql` (no new migration)
- No backwards compatibility required (no production data)
- Direct constraint addition - if duplicates exist, migration fails

**NULL Handling:**
- NULL `access_request_scope` means auto-approved request
- These requests don't participate in OAuth token exchange flow
- They use `scope_resource-*` access only, no `scope_access_request:*` needed

---

## 8. Repository Method Design

**Q: How should get_by_access_request_scope handle multiple results?**

**Decision: Return Error on Multiple**
- Method signature: `async fn get_by_access_request_scope(&self, scope: &str) -> Result<Option<AppAccessRequestRow>, DbError>`
- Returns `Ok(None)` if not found
- Returns `Ok(Some(row))` if exactly one found
- Returns `Err(DbError::MultipleRecords)` if multiple found (should be impossible with unique constraint)

**SQL Pattern:**
```sql
SELECT * FROM app_access_requests
WHERE access_request_scope = ?
LIMIT 2  -- Fetch 2 to detect multiples
```

**Validation Logic:**
- If 0 rows: return Ok(None)
- If 1 row: return Ok(Some(row))
- If 2 rows: return Err (unique constraint violated)

---

## 9. Service Dependency Architecture

**Q: Should TokenService depend directly on DbService for access request lookup?**

**Decision: Direct DbService Dependency**
- TokenService already has DbService for token cache operations
- Add access request lookup directly via `db_service.get_by_access_request_scope()`
- No need for AuthService intermediary
- Keeps validation logic localized in TokenService

**Dependency Chain:**
```
validate_bearer_token
  → db_service.get_by_access_request_scope(scope)
  → validate record (status, app_client_id, user_id)
  → auth_service.exchange_app_token(scopes including access_request scope)
  → verify access_request_id claim matches record.id
```

---

## 10. Test Coverage Requirements

**Q: What test scenarios are required for complete coverage?**

**Pre-Validation Failure Tests (Required):**
- Test: scope not found in DB → 403 with ScopeNotFound error
- Test: status=draft → 403 with NotApproved error
- Test: status=denied → 403 with NotApproved error
- Test: app_client_id mismatch → 403 with AppClientMismatch error
- Test: user_id mismatch → 403 with UserMismatch error

**Post-Verification Tests (Required):**
- Test: access_request_id claim matches record.id → continues, header injected
- Test: access_request_id claim != record.id → 403 with AccessRequestIdMismatch error

**Happy Path Integration (Required):**
- Test: external token with valid scope_access_request:*, all validation passes, token exchange succeeds, header injected correctly

**NULL Scope Handling (Required):**
- Test: external token without scope_access_request:* prefix → validation skipped entirely
- Test: DB record with NULL access_request_scope → not affected by unique constraint
- Test: multiple NULL access_request_scope values allowed

**Repository Method Tests:**
- Test: get_by_access_request_scope with existing scope → returns record
- Test: get_by_access_request_scope with non-existent scope → returns None
- Test: get_by_access_request_scope with NULL → returns None (no match)

**Migration Tests:**
- Test: unique constraint prevents duplicate non-NULL access_request_scope
- Test: multiple NULL values allowed

---

## 11. Verification Scope

**Q: Should verification include e2e tests in lib_bodhiserver_napi/tests-js?**

**Decision: Unit + API Tests Only**
- Complete all unit tests in auth_middleware, services, routes_app crates
- No e2e tests in this verification iteration
- E2e tests are separate phase after verification complete
- E2e will test complete user journey with real app token and exchange token flow

**Test Levels:**
1. **Unit tests:** Repository methods, validation logic, error handling
2. **Integration tests:** Token service with mock KC, complete pre/post validation flow
3. **API tests:** Routes with mocked services, end-to-end request/response validation
4. **E2e tests (separate phase):** Real Keycloak, real token exchange, full user journey

---

## 12. Phase 5 Cleanup Verification

**Q: Should we verify Phase 5 cleanup beyond what Phase 6 already removed?**

**Decision: Phase 6 Was Thorough, Skip Additional Verification**
- Phase 6 (commit 33c51630) already removed:
  - `KEY_HEADER_BODHIAPP_TOOL_SCOPES` header constant
  - Toolset scope extraction and injection
  - All references to old scope-based authorization
- No additional cleanup verification needed
- Phase 6 commit message documents all changes

**Verification Confidence:**
- 145 auth_middleware tests passing
- 422 routes_app tests passing
- All compilation checks successful
- Grep searches confirmed no remaining `scope_toolset-*` references

---

## 13. Documentation Updates

**Q: Should phase documentation be updated after implementation?**

**Decision: Yes, Append to phase-4-plan.md**
- Add "Phase 4b: Pre/Post-Exchange Validation" section
- Document new validation logic similar to Phase 6 appendix pattern
- Include implementation details, verification results, files modified
- Append at end of phase-4-plan.md rather than creating new file

**Documentation Pattern:**
```markdown
## Phase 4b - Pre/Post-Exchange Validation (Completed 2026-02-12)

**Context**: After Phase 4 implementation, identified gap in pre-token-exchange
validation. This enhancement adds comprehensive validation before and after token
exchange to ensure scope integrity.

### What Was Actually Implemented
[Details of changes]

### Files Modified
[List of files with descriptions]

### Verification Results
[Test results and compilation checks]
```

---

## 14. Implementation Timeline

**Verification Implementation Focus:**
1. Database schema: Add unique index to migration 0011
2. Repository: Add get_by_access_request_scope method
3. Error types: Create AccessRequestValidationError enum
4. Token service: Add pre-validation logic in validate_bearer_token
5. Token service: Add post-verification logic after token exchange
6. Unit tests: All failure scenarios + happy path + NULL handling
7. Integration tests: Complete token exchange flow with validation
8. Documentation: Append Phase 4b section to phase-4-plan.md

**NOT in Scope:**
- E2e tests in lib_bodhiserver_napi/tests-js (separate iteration)
- Frontend changes (access request review page)
- Keycloak SPI changes (already done in Phase 0)

---

## 15. Key Design Principles

**Fail Fast:**
- Pre-validation catches invalid scopes before Keycloak call
- Reduces unnecessary token exchange operations
- Provides specific error codes for debugging

**Flexibility for Future:**
- Scope UUID and access_request_id claim kept separate
- Allows KC to change claim format without breaking validation
- Database primary key remains source of truth

**Clean Architecture:**
- Validation logic centralized in TokenService
- Clear separation: pre-validation, exchange, post-verification
- No duplication with toolset_auth_middleware (different validation layers)

**Security in Depth:**
- Multiple validation layers: pre-exchange, post-exchange, toolset middleware
- Each layer validates different aspects of authorization
- Failed validation always returns 403 Forbidden, never continues

**Testability:**
- Conditional validation enables testing both paths (with/without scope)
- Mock DbService for isolated TokenService tests
- Integration tests verify complete flow without real Keycloak

---

## 16. Migration Notes

**Modifying Existing Migration:**
- Migration 0011 introduced access_request_scope column
- We're adding unique constraint to same migration (no new migration)
- No production data exists, so no backwards compatibility needed

**Why No New Migration:**
- Feature is still in development
- No production deployments yet
- Cleaner to fix migration in place rather than add migration 0012

**Down Migration:**
- Create corresponding .down.sql that drops the unique index
- Enables rollback during development if needed
- Standard migration practice for reversibility

---

## Appendix: Current Flow vs New Flow

**Current Flow (Phases 1-6):**
1. External token with scope_access_request:<uuid> arrives
2. Token exchange happens immediately (no pre-validation)
3. access_request_id claim extracted and injected as header
4. Toolset middleware validates access request (status, app, user, instance)
5. Toolset execution proceeds if validation passes

**New Flow (Phase 4b):**
1. External token with scope_access_request:<uuid> arrives
2. **PRE-VALIDATION:** Look up scope in DB, verify status/app/user
3. If valid: Token exchange proceeds
4. **POST-VERIFICATION:** Verify access_request_id claim matches record.id
5. If match: Inject header
6. Toolset middleware validates (status, app, user, instance) - same as before
7. Toolset execution proceeds

**Key Difference:**
- New flow fails fast if scope is invalid (before KC call)
- New flow verifies KC returned correct claim (after exchange)
- Toolset middleware validation unchanged (still needed for instance authorization)
