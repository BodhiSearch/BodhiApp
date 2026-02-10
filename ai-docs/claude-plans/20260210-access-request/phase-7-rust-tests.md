# Phase 7: Rust Unit & API Tests

## Purpose

Implement comprehensive test coverage for all backend changes: service layer, route handlers, and auth middleware.

## Dependencies

- **Phase 2**: Service layer implemented
- **Phase 3**: API endpoints implemented
- **Phase 4**: Auth middleware updated

## Testing Strategy

Follow existing patterns from:
- **test-services skill**: For service layer tests
- **test-routes-app skill**: For route handler tests
- Existing middleware test patterns

## Test Coverage Areas

### 7a. Service Layer Tests

**File**: `crates/services/src/db/access_request_repository.rs` (test module)

Tests for `AccessRequestRepository`:
- `test_create_app_access_request` — insert new draft
- `test_get_app_access_request` — fetch by id
- `test_get_app_access_request_not_found` — 404 case
- `test_update_app_access_request_approval` — update to approved status
- `test_update_app_access_request_denial` — update to denied status
- `test_draft_expiry_logic` — created 10min ago → treat as expired (application logic)
- `test_json_serialization` — tools_requested and tools_approved serialize correctly

**File**: `crates/services/src/auth_service.rs` (test module)

Tests for `register_access_request_consent`:
- `test_register_consent_success_201` — First registration returns 201 Created
- `test_register_consent_idempotent_200` — Retry with same UUID returns 200 OK
- `test_register_consent_conflict_409` — Different context with same UUID returns 409
- `test_register_consent_invalid_token_401` — Invalid user token returns 401
- `test_register_consent_validation_error_400` — Missing fields return 400
- `test_register_consent_not_from_resource_client_401` — Token not from resource client

Use **mockito** for KC HTTP endpoint mocking (see existing auth_service tests).

**Patterns**:
- DB tests: Use in-memory SQLite (see existing repository tests)
- HTTP mocking: Use mockito for KC endpoint (see test-services skill)
- Fixtures: Set up test data with `TimeService` for timestamps (see MEMORY.md)

### 7b. Route Handler Tests

**File**: `crates/routes_app/src/routes_auth/request_access.rs` (test module)

Tests for `POST /apps/request-access`:
- `test_request_access_success_popup` — Valid popup flow request
- `test_request_access_success_redirect` — Valid redirect flow request
- `test_request_access_invalid_flow_type` — Returns 400
- `test_request_access_missing_redirect_uri` — Redirect flow without URI returns 400
- `test_request_access_empty_tools` — Empty tools list returns 400
- `test_request_access_app_not_found` — Invalid app_client_id returns 400

Tests for `GET /apps/request-access/:id`:
- `test_get_access_request_success_draft` — Returns draft status
- `test_get_access_request_success_approved` — Returns approved with scopes
- `test_get_access_request_not_found` — Returns 404
- `test_get_access_request_expired` — Expired draft returns status "expired"

**File**: `crates/routes_app/src/routes_auth/access_request_review.rs` (test module)

Tests for `GET /apps/access-request/:id/review`:
- `test_review_success` — Returns enriched tool info with user instances
- `test_review_unauthenticated` — Returns 401
- `test_review_not_found` — Returns 404
- `test_review_expired` — Returns 400 or shows expired status

Tests for `POST /apps/access-request/:id/approve`:
- `test_approve_success` — Valid approval with tool instances
- `test_approve_unauthenticated` — Returns 401
- `test_approve_expired` — Returns 400
- `test_approve_invalid_tool_instance` — Instance not owned by user, returns 400
- `test_approve_tool_without_api_key` — Instance missing API key, returns 400
- `test_approve_kc_conflict_409` — KC returns 409, surfaces error

Tests for `POST /apps/access-request/:id/deny`:
- `test_deny_success` — Valid denial
- `test_deny_unauthenticated` — Returns 401
- `test_deny_expired` — Returns 400

**Patterns**:
- Follow **test-routes-app skill** for canonical patterns
- Mock services (AuthService, DbService, ToolService)
- Use test fixtures for setup
- Test both success and error paths

### 7c. Auth Middleware Tests

**File**: `crates/auth_middleware/src/auth_middleware.rs` (test module)

Tests for token claim extraction:
- `test_access_request_id_claim_extracted` — Token with claim → header injected
- `test_access_request_id_claim_missing` — Token without claim → no header
- `test_session_auth_no_claim` — Session auth doesn't check for claim

**File**: `crates/auth_middleware/src/toolset_auth_middleware.rs` (test module)

Tests for access request validation:
- `test_oauth_valid_access_request` — Valid access_request_id → allowed
- `test_oauth_expired_access_request` — Expired → denied
- `test_oauth_wrong_user` — User mismatch → denied
- `test_oauth_missing_tool_instance` — Requested instance not in approved list → denied
- `test_oauth_access_request_not_found` — access_request_id not in DB → denied
- `test_session_auth_unchanged` — Session auth flow works without access_request_id

**Patterns**:
- Mock `AccessRequestRepository` for DB access
- Mock token decoding for claim extraction
- Test both OAuth and session flows

## Files to Create/Modify

| Area | File | Test Focus |
|------|------|-----------|
| Service | `crates/services/src/db/access_request_repository.rs` | CRUD operations, JSON serialization |
| Service | `crates/services/src/auth_service.rs` | KC consent registration, error handling |
| Routes | `crates/routes_app/src/routes_auth/request_access.rs` | POST /apps/request-access, GET by id |
| Routes | `crates/routes_app/src/routes_auth/access_request_review.rs` | Review, approve, deny endpoints |
| Middleware | `crates/auth_middleware/src/auth_middleware.rs` | Claim extraction, header injection |
| Middleware | `crates/auth_middleware/src/toolset_auth_middleware.rs` | Access request validation |

## Research Questions

1. **Test fixtures**: How are existing tests setting up DB fixtures? (See test-services skill)
2. **Mock services**: What's the pattern for mocking services in route tests? (See test-routes-app skill)
3. **Session auth mocking**: How do we mock authenticated sessions in tests? (Check existing session tests)
4. **Time mocking**: How do we control time in tests for expiry logic? (Check `TimeService` mock)
5. **Mockito patterns**: What's the existing pattern for mockito HTTP mocks? (Check auth_service tests)
6. **Error assertions**: How do we assert on specific error variants? (See test-routes-app skill)

## Acceptance Criteria

### Service Layer Tests
- [ ] All repository CRUD operations tested
- [ ] JSON serialization/deserialization tested
- [ ] Draft expiry logic tested
- [ ] KC consent registration tested (201, 200, 409, 401, 400 responses)
- [ ] Mock tests for AuthService method
- [ ] `cargo test -p services` passes

### Route Handler Tests
- [ ] All endpoint handlers tested (success paths)
- [ ] Error paths tested (validation, auth, not found, expired)
- [ ] Request/response serialization tested
- [ ] Session auth tested for protected endpoints
- [ ] Service mocking works correctly
- [ ] `cargo test -p routes_app` passes

### Middleware Tests
- [ ] Token claim extraction tested
- [ ] Header injection tested
- [ ] Access request validation tested (all scenarios)
- [ ] Session auth flow tested (unchanged)
- [ ] `cargo test -p auth_middleware` passes

### Overall
- [ ] `make test.backend` passes
- [ ] No test uses `if`/`else` logic (see CLAUDE.md)
- [ ] Tests follow `assert_eq!(expected, actual)` convention
- [ ] No trivial tests (constructors, derives)
- [ ] Clear test names describing scenario
- [ ] Good coverage of error paths

## Notes for Sub-Agent

- **Use skills**: Invoke test-services and test-routes-app skills for guidance
- **Follow patterns**: Study existing tests before writing new ones
- **Don't test trivial code**: Skip testing derives, simple constructors (see CLAUDE.md)
- **Deterministic tests**: No random values, no try-catch in tests
- **Error logging**: Use console.log only for error scenarios in tests
- **Avoid timeouts**: No inline timeouts in tests
- **Mock time**: Use TimeService mock for timestamp control (see MEMORY.md)
- **Test coverage**: Run `make test.coverage` after to check coverage

## Verification

```bash
# Run service tests
cargo test -p services

# Run routes tests
cargo test -p routes_app

# Run middleware tests
cargo test -p auth_middleware

# Run all backend tests
make test.backend

# Check coverage
make test.coverage
```

## Next Phase

Phase 8 will implement frontend component tests for the review page.
