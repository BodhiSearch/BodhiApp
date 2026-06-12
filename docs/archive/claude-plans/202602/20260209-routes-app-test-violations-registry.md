# routes_app Test Uniformity - Violations Registry

## Purpose
This document catalogs all acceptable violations from the canonical auth test patterns discovered during the routes_app test uniformity effort (Phases 0-4). These violations are intentional and documented to avoid confusion during future maintenance.

## Violation Categories

### Category 1: MockService Dependencies (No Allow Tests)
**Pattern**: Endpoints use MockToolService, MockAuthService, or MockSharedContext, preventing allow tests without complex expectations.

**Rationale**: These services panic without explicit expectations. Adding allow tests would require setting up mock expectations for every tested role, which provides no additional value beyond the 401 test.

**Coverage strategy**: 401 test proves auth layer works. Allow tests omitted.

#### Violations:
1. **routes_toolsets** (all CRUD endpoints)
   - File: `crates/routes_app/src/routes_toolsets/tests.rs`
   - Endpoints:
     - GET /toolsets
     - GET /toolsets/{toolset_id}
     - POST /toolsets
     - PUT /toolsets/{toolset_id}
     - DELETE /toolsets/{toolset_id}
   - Service: MockToolService
   - Test coverage: 401 ✅, 403 ✅, Allow ❌ (documented)

2. **routes_users/management** (admin user management)
   - File: `crates/routes_app/src/routes_users/tests/management_test.rs`
   - Endpoints:
     - GET /users (list users)
     - POST /users/{user_id}/role (change role)
     - DELETE /users/{user_id} (remove user)
   - Service: MockAuthService
   - Test coverage: 401 ✅, 403 ✅, Allow ❌ (documented)

3. **routes_users/access_request** (POST endpoints)
   - File: `crates/routes_app/src/routes_users/tests/access_request_test.rs`
   - Endpoints:
     - POST /user/request-access (request access)
     - POST /users/pending-requests/{request_id}/approve (approve request)
     - POST /users/pending-requests/{request_id}/reject (reject request)
   - Service: MockAuthService
   - Test coverage: 401 ✅, 403 ✅ (where applicable), Allow ❌ (documented)
   - Note: GET /users/pending-requests has allow test (uses AppServiceStubBuilder)

### Category 2: Streaming Endpoints (MockSharedContext for SSE)
**Pattern**: Endpoints use `MockSharedContext.expect_forward_request()` for Server-Sent Events (SSE) streaming responses.

**Rationale**: Streaming tests require complex mock setup to simulate SSE chunk delivery. Migrating to `build_test_router()` would require restructuring the entire test to use real services with actual LLM processes, which is out of scope for auth uniformity.

**Coverage strategy**: 401 test exists separately. Allow tests omitted.

#### Violations:
1. **routes_oai/chat_test.rs** (OpenAI chat/embeddings)
   - File: `crates/routes_app/src/routes_oai/tests/chat_test.rs`
   - Endpoints:
     - POST /v1/chat/completions
     - POST /v1/embeddings
   - Mock: MockSharedContext.expect_forward_request()
   - Test coverage: 401 ✅ (in models_test.rs), Allow ❌ (documented)
   - Documentation: Lines 1-7 (violation header comment)

2. **routes_ollama/handlers_test.rs** (Ollama chat)
   - File: `crates/routes_app/src/routes_ollama/tests/handlers_test.rs`
   - Endpoint:
     - POST /api/chat
   - Mock: MockSharedContext (SSE streaming)
   - Test coverage: 401 ✅, Allow ❌ (documented)
   - Documentation: Lines 2-8 (violation header comment)

### Category 3: Focused Handler Test Helpers
**Pattern**: Handler tests use focused helper functions (e.g., `app()`) instead of `build_test_router()`.

**Rationale**: If auth tier tests already use `build_test_router()`, handler tests can keep their focused helpers. This is acceptable when auth coverage is complete.

#### Violations:
1. **routes_api_token/tests.rs** (handler tests)
   - File: `crates/routes_app/src/routes_api_token/tests.rs`
   - Helper: `app()` function
   - Auth tests: Use `build_test_router()` ✅
   - Handler tests: Use focused `app()` helper ✅
   - Status: Acceptable - auth coverage is separate and complete

2. **routes_ollama/handlers_test.rs** (inner mod test)
   - File: `crates/routes_app/src/routes_ollama/tests/handlers_test.rs`
   - Pattern: Inner `mod test` with `router_state_stub` fixture
   - Auth tests: Use `build_test_router()` ✅
   - Handler tests: Use `router_state_stub` ✅
   - Status: Acceptable - auth coverage is separate and complete
   - Documentation: Lines 2-8 (violation header comment)

## Coverage Matrix

| Module | 401 Test | 403 Test | Allow Test | Violations |
|--------|----------|----------|------------|------------|
| routes_settings | ✅ | N/A (Public) | N/A (Public) | None |
| routes_models | ✅ | N/A (User) | ✅ | None |
| routes_toolsets | ✅ | ✅ | ❌ | MockToolService |
| routes_api_token | ✅ | ✅ | ✅ | Handler helper (acceptable) |
| routes_users/info | ✅ | N/A (User) | ✅ | None |
| routes_users/management | ✅ | ✅ | ❌ | MockAuthService |
| routes_users/access_request | ✅ | ✅ | Partial | MockAuthService (POST endpoints) |
| routes_api_models | ✅ | ✅ | ✅ | None |
| routes_oai/models | ✅ | N/A (User) | ✅ | None |
| routes_oai/chat | ✅ | N/A (User) | ❌ | MockSharedContext (streaming) |
| routes_ollama | ✅ | N/A (User) | Partial | MockSharedContext (chat streaming) |

## Documentation Pattern

All violations are documented with header comments in the test files:

```rust
// ============================================================================
// VIOLATION DOCUMENTATION:
// [Description of violation]
// [Affected endpoints]
// [Rationale]
// [Coverage strategy]
// ============================================================================
```

## Maintenance Guidelines

1. **Do not remove violation comments** - They provide context for future developers
2. **Do not attempt to add allow tests for documented violations** - The patterns prevent it
3. **If migrating away from mocks** - Update this registry and remove violation comments
4. **If adding new endpoints** - Follow canonical patterns unless a documented violation applies

## References

- **Canonical patterns**: See phase summaries in `ai-docs/claude-plans/`
- **Phase 0-4 summaries**: Test infrastructure and module-specific implementations
- **Critical Pattern Rules**: Documented in each phase summary
