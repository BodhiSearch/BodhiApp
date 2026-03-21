# 🚀 Next Session Kickoff - Access Request Phase 3 Implementation

## Quick Start Prompt

```
I need to implement Phase 3 (API Endpoints) of the Access Request feature.

Context:
- Phases 0-1-2 are COMPLETE (database schema, domain objects, service layer)
- Phase 3 was previously started but has been reverted - start fresh
- Service layer is fully functional with AccessRequestService, repository, business logic, and Keycloak integration
- All services code compiles successfully (cargo check -p services passes)

Task:
Implement Phase 3 from the plan: API endpoint handlers, error types, router integration, OpenAPI docs, and comprehensive tests.

Location: crates/routes_app/src/routes_apps/ (needs to be created)

Process:
1. Create routes_apps module structure with handlers and error types
2. Implement handlers incrementally (POST then GET)
3. Add router integration and OpenAPI documentation
4. Write comprehensive tests following canonical patterns
5. Run `cargo check -p routes_app` after each step
6. Focus on both success paths and error scenarios

Key files to reference:
- Service layer: crates/services/src/access_request_service/
- Test patterns: .claude/skills/test-routes-app/
- Similar handlers: crates/routes_app/src/routes_users/
- Plan document: ai-docs/claude-plans/sharded-dazzling-orbit.md (Phase 3 section)
- Quick ref: ai-docs/claude-plans/20260210-access-request/quick-ref.md
```

## What's Been Done (Quick Summary)

### ✅ Complete (Phases 0-1-2)
1. **Database**: Migrations, schema with UUID PK, indexes
2. **Domain Objects**: All enums, structs, DTOs in objs and services crates
3. **Service Layer**: AccessRequestService trait + implementation, repository, error handling
4. **KC Integration**: register_access_request_consent() method in AuthService
5. **AppService Integration**: Service added to AppService registry and AppServiceBuilder
6. **Cleanup**: Old code completely removed (clean cutover)

### ⬜ Pending (Phase 3)
- **Entire routes_app implementation**: Handlers, error types, router integration, OpenAPI docs, and tests

## What to Do Next

### Step 1: Create Module Structure

```bash
mkdir -p crates/routes_app/src/routes_apps
touch crates/routes_app/src/routes_apps/mod.rs
touch crates/routes_app/src/routes_apps/error.rs
touch crates/routes_app/src/routes_apps/access_request.rs
```

Add to `crates/routes_app/src/lib.rs`:
```rust
mod routes_apps;
pub use routes_apps::*;
```

### Step 2: Define Error Types

Create domain-specific error enum in `error.rs`:
```rust
use objs::{AppError, ErrorType};
use services::AccessRequestError;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum AppAccessRequestError {
  #[error("Invalid flow type '{0}'. Expected 'redirect' or 'popup'.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidFlowType(String),

  #[error("Redirect URI is required for redirect flow.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  MissingRedirectUri,

  #[error("Invalid tools format: {0}")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidToolsFormat(String),

  #[error(transparent)]
  #[error_meta(args_delegate = false)]
  AccessRequestError(#[from] AccessRequestError),
}
```

### Step 3: Implement Handlers (One at a Time)

1. **POST handler** - Create draft access request
2. **GET handler** - Poll access request status
3. Run `cargo check -p routes_app` after each

### Step 4: Add Router Integration

Update `crates/routes_app/src/routes.rs`:
- Import handlers and endpoint constants
- Add route registrations
- Run `cargo check -p routes_app`

### Step 5: Add OpenAPI Documentation

Update `crates/routes_app/src/shared/openapi.rs`:
- Import DTOs, handlers, and domain objects
- Add schema registrations
- Add path registrations
- Run `cargo check -p routes_app`

### Step 6: Implement Tests

Create `crates/routes_app/src/routes_apps/tests/` directory with comprehensive tests:
1. Success paths for both handlers
2. Validation error scenarios
3. Service error handling
4. Run `cargo test -p routes_app` after each test

### Step 7: Verify Complete Implementation

```bash
cargo check -p routes_app
cargo test -p routes_app routes_apps::tests
make test.backend
```

## Quick Reference Links

- **Full Handoff Doc**: `ai-docs/claude-plans/20260210-access-request/phase-3-handoff.md`
- **Quick Ref**: `ai-docs/claude-plans/20260210-access-request/quick-ref.md`
- **Checklist**: `ai-docs/claude-plans/20260210-access-request/checklist.md`
- **Original Plan**: `ai-docs/claude-plans/sharded-dazzling-orbit.md`
- **Test Skill**: `.claude/skills/test-routes-app/`

## Key Implementation Notes

### Error Code Pattern
```rust
assert_eq!("app_access_request_error-invalid_flow_type", err.code());
```

### DateTime Conversion Pattern
```rust
DateTime::from_timestamp(row.created_at, 0)
  .ok_or_else(|| AppAccessRequestError::InvalidToolsFormat("Invalid timestamp".to_string()))?
```

### Mock Setup Pattern
```rust
let mut mock_service = MockAccessRequestService::new();
mock_service
  .expect_create_draft()
  .returning(|app_id, flow, uri, tools| {
    Ok(AppAccessRequestRow {
      id: "550e8400-...".to_string(),
      // ... other fields
    })
  });
```

## Success Criteria

- ✅ All 8 tests pass
- ✅ `cargo check -p routes_app` succeeds
- ✅ `cargo test -p routes_app` succeeds
- ✅ Both success and error paths covered
- ✅ Error codes properly asserted

## Estimated Time

- 30-45 minutes for all 8 tests
- ~5 minutes per test (setup, implement, verify)

## Commands to Use

```bash
# Create test structure
mkdir -p crates/routes_app/src/routes_apps/tests

# Watch mode during development
cargo watch -x 'test -p routes_app routes_apps::tests::access_request_test'

# Run all tests
cargo test -p routes_app routes_apps::tests::access_request_test

# Run specific test
cargo test -p routes_app test_create_draft_success -- --nocapture

# Check compilation
cargo check -p routes_app
```

## If You Get Stuck

1. **Check test patterns**: `.claude/skills/test-routes-app/SKILL.md`
2. **Check similar tests**: `crates/routes_app/src/routes_users/tests/access_request_test.rs`
3. **Check handlers**: `crates/routes_app/src/routes_apps/access_request.rs`
4. **Check quick ref**: `ai-docs/claude-plans/20260210-access-request/quick-ref.md`

## Important Reminders

- Use `#[rstest]` + `#[tokio::test]` + `#[anyhow_trace]` for async tests
- Return `-> anyhow::Result<()>` from tests
- Use `assert_eq!(expected, actual)` convention
- Assert error codes with `.code()`, never error messages
- Import `pretty_assertions::assert_eq` in test module
- Mock services at the service layer, not repository layer
- One test at a time, verify compilation after each

---

**Ready to go! Start with the kickoff prompt above and work through the test list incrementally.** 🚀
