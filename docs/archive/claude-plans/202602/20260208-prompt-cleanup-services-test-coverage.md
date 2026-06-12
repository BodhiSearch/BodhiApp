# Services Crate Test Revamp: Uniformity + Coverage Enhancement

## Context

The `services` crate (283 tests, 54.14% line coverage) contains 13 primary services organized across authentication, model management, AI/tools, infrastructure, concurrency, and configuration. Tests evolved organically — some modules are well-tested (concurrency_service 99.56%, token 97.67%, ai_api_service 95.16%) while others have significant gaps (exa_service 3.88%, auth_service 43.76%, queue_service 45.69%, db/service.rs 52.30%, keyring_service 26.53%).

This plan standardizes all service tests to a canonical pattern AND fills coverage gaps per-module in one pass. Each module is handled by a dedicated sub-agent that reads the source, reads the tests, standardizes the pattern, fills gaps, and verifies.

## Decisions

| Decision | Choice |
|----------|--------|
| Test annotations | `#[rstest]` + `#[tokio::test]` + `#[anyhow_trace]` on all async tests; `#[rstest]` on sync tests |
| `#[awt]` | Only when `#[future]` fixture params used |
| Assertions | `pretty_assertions::assert_eq` everywhere, `assert_eq!(expected, actual)` order |
| Error handling | `?` not `.unwrap()`; `.expect("msg")` only in closures/Option chains |
| Test naming | `test_<function>_<scenario>` |
| Module naming | `mod tests` |
| Return type | `-> anyhow::Result<()>` on all tests |
| Mock patterns | Keep `mockall` for service traits, `mockito` for HTTP APIs |
| DB fixtures | `TestDbService` with temp dirs, rstest async fixtures |
| Coverage target | 80%+ per service file (except `bin/setup_client.rs` and `test_utils/`) |
| Migration approach | Sub-agent per module: standardize + fill gaps in single pass |
| Documentation | Update CLAUDE.md and PACKAGE.md (crate-level only, NOT ai-docs/) |
| Skill | Create `.claude/skills/test-services/` with progressive disclosure |

## Canonical Test Pattern (Services)

```rust
// Inline test module in service file:
#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;
  use anyhow_trace::anyhow_trace;
  use rstest::rstest;

  // For services with DB dependencies:
  use crate::test_utils::{test_db_service, TestDbService};
  // For services needing mocks:
  use mockall::predicate::*;
  // For HTTP API services:
  use mockito::Server;

  #[rstest]
  #[tokio::test]
  #[anyhow_trace]
  async fn test_function_name_success() -> anyhow::Result<()> {
    // Arrange
    let mut server = Server::new_async().await;
    let mock = server.mock("GET", "/endpoint")
      .with_status(200)
      .with_body(r#"{"result": "ok"}"#)
      .create_async().await;

    let service = MyService::new(&server.url());

    // Act
    let result = service.do_thing().await?;

    // Assert
    assert_eq!(expected, result);
    mock.assert_async().await;
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  #[anyhow_trace]
  async fn test_function_name_error_scenario() -> anyhow::Result<()> {
    // ... setup that triggers error ...
    let result = service.do_thing().await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    // Check error variant, not message text
    assert!(matches!(err, MyError::NotFound { .. }));
    Ok(())
  }
}
```

## Execution Phases (Sub-Agent Based)

Each phase is a sub-agent that:
1. Reads the source file(s) and existing tests
2. Identifies current coverage gaps from the baseline table below
3. Standardizes existing tests to canonical pattern
4. Adds new tests to fill the most impactful coverage gaps
5. Runs `cargo test -p services -- <module_filter>` to verify
6. Returns: changes summary, new test count, coverage-relevant additions

### Coverage Baseline (services/ source files only)

| File | Lines | Cover | Priority |
|------|-------|-------|----------|
| `ai_api_service.rs` | 186 | 95.16% | Low |
| `app_service.rs` | 44 | 0.00% | Skip (trait impls) |
| `auth_service.rs` | 425 | 43.76% | HIGH |
| `cache_service.rs` | 23 | 95.65% | Low |
| `concurrency_service.rs` | 451 | 99.56% | Low |
| `data_service.rs` | 241 | 82.99% | Medium |
| `db/encryption.rs` | 67 | 92.54% | Low |
| `db/error.rs` | 18 | 44.44% | Medium |
| `db/objs.rs` | 65 | 23.08% | Medium |
| `db/service.rs` | 1543 | 52.30% | HIGH |
| `db/sqlite_pool.rs` | 21 | 100.00% | Low |
| `db/time_service.rs` | 18 | 5.56% | Medium |
| `env_wrapper.rs` | 63 | 85.71% | Low |
| `exa_service.rs` | 206 | 3.88% | HIGH |
| `hub_service.rs` | 361 | 76.73% | Medium |
| `keyring_service.rs` | 49 | 26.53% | HIGH |
| `macros.rs` | 6 | 50.00% | Skip |
| `objs.rs` | 3 | 0.00% | Skip |
| `progress_tracking.rs` | 126 | 82.54% | Low |
| `queue_service.rs` | 383 | 45.69% | HIGH |
| `secret_service.rs` | 141 | 88.65% | Low |
| `service_ext.rs` | 38 | 84.21% | Low |
| `session_service.rs` | 446 | 87.67% | Low |
| `setting_service/service.rs` | 661 | 66.26% | Medium |
| `token.rs` | 172 | 97.67% | Low |
| `tool_service/service.rs` | 690 | 57.83% | HIGH |

---

### Phase 1: `auth_service` (43.76% -> target 80%+)

**Sub-agent scope**: `crates/services/src/auth_service.rs`

Current state: 10 tests, 425 lines, 43.76% coverage.

Tasks:
1. Standardize existing 10 tests (annotations, naming, assertions)
2. Identify uncovered branches:
   - OAuth2 PKCE flow error paths
   - Token exchange error handling
   - Keycloak callback scenarios
   - User registration flows
3. Add tests for critical uncovered paths (targeting 80%+ coverage)
4. Verify: `cargo test -p services -- auth_service`

---

### Phase 2: `exa_service` (3.88% -> target 80%+)

**Sub-agent scope**: `crates/services/src/exa_service.rs`

Current state: 7 tests, 206 lines, 3.88% coverage.

Tasks:
1. Standardize existing 7 tests
2. Identify why coverage is so low (likely mocking issues or tests not exercising actual code paths)
3. Add tests using `mockito` for all 4 API methods: `search()`, `find_similar()`, `contents()`, `answer()`
4. Cover error paths (HTTP errors, JSON parse errors, missing fields)
5. Verify: `cargo test -p services -- exa_service`

---

### Phase 3: `db/service.rs` (52.30% -> target 75%+)

**Sub-agent scope**: `crates/services/src/db/service.rs` + `crates/services/src/db/tests.rs`

Current state: 43 tests, 1543 lines, 52.30% coverage.

Tasks:
1. Standardize existing 43 tests
2. Focus on uncovered repository operations:
   - `access_repository` operations (create, update, list, paginate)
   - `token_repository` operations (CRUD, pagination, expiry)
   - `model_repository` operations (metadata extraction queue)
   - `toolset_repository` operations
3. Cover error paths for each repository
4. Verify: `cargo test -p services -- db`

Also standardize `db/error.rs` (44.44%), `db/objs.rs` (23.08%), `db/time_service.rs` (5.56%) — add tests for Display impls, From conversions, TimeService trait.

---

### Phase 4: `queue_service` (45.69% -> target 75%+)

**Sub-agent scope**: `crates/services/src/queue_service.rs`

Current state: 4 tests, 383 lines, 45.69% coverage.

Tasks:
1. Standardize existing 4 tests
2. Add tests for:
   - Queue lifecycle (enqueue, dequeue, process)
   - Concurrent queue access
   - Error handling during processing
   - Queue drain and shutdown
3. Verify: `cargo test -p services -- queue_service`

---

### Phase 5: `keyring_service` (26.53% -> target 70%+)

**Sub-agent scope**: `crates/services/src/keyring_service.rs`

Current state: 1 test, 49 lines, 26.53% coverage.

Tasks:
1. Standardize existing test
2. Add tests for:
   - Store/retrieve/delete operations
   - Error handling (keyring not available, entry not found)
   - Use `KeyringStoreStub` from test_utils for platform-independent testing
3. Verify: `cargo test -p services -- keyring_service`

---

### Phase 6: `tool_service` (57.83% -> target 75%+)

**Sub-agent scope**: `crates/services/src/tool_service/service.rs` + `crates/services/src/tool_service/tests.rs`

Current state: 42 tests, 690 lines, 57.83% coverage.

Tasks:
1. Standardize existing 42 tests
2. Identify uncovered branches in toolset management:
   - Tool registration/deregistration edge cases
   - App client toolset configuration
   - Error paths in tool execution
3. Verify: `cargo test -p services -- tool_service`

---

### Phase 7: `setting_service` (66.26% -> target 80%+)

**Sub-agent scope**: `crates/services/src/setting_service/service.rs` + `crates/services/src/setting_service/tests.rs`

Current state: 50 tests, 661 lines, 66.26% coverage.

Tasks:
1. Standardize existing 50 tests
2. Add tests for uncovered settings operations:
   - Settings change notification paths
   - Environment variable override interactions
   - Precedence rule edge cases
3. Verify: `cargo test -p services -- setting_service`

---

### Phase 8: Medium Priority Modules

**Sub-agent scope**: Multiple files, handled in one pass.

Modules:
- `data_service.rs` (82.99% -> 90%+): Standardize 24 tests, add edge case coverage
- `hub_service.rs` (76.73% -> 85%+): Standardize 53 tests, fill uncovered branches
- `progress_tracking.rs` (82.54%): Standardize 2 tests, minor additions
- `service_ext.rs` (84.21%): Standardize 1 test, minor additions

Verify: `cargo test -p services -- data_service && cargo test -p services -- hub_service`

---

### Phase 9: Low Priority Modules (Standardize Only)

**Sub-agent scope**: Files already at 85%+ coverage — standardize test pattern only.

Modules:
- `ai_api_service.rs` (95.16%): Standardize 17 tests
- `cache_service.rs` (95.65%): Standardize 1 test
- `concurrency_service.rs` (99.56%): Standardize 6 tests
- `env_wrapper.rs` (85.71%): Standardize 6 tests
- `secret_service.rs` (88.65%): Standardize 4 tests
- `session_service.rs` (87.67%): Standardize 5 tests
- `token.rs` (97.67%): Standardize 7 tests

Verify: `cargo test -p services`

---

### Phase 10: Full Suite Verification

Run: `cargo test -p services`

Confirm: all tests pass, no regressions.

---

### Phase 11: Update Documentation

Update crate-level documentation (NOT ai-docs/):

1. `crates/services/CLAUDE.md` — Add testing conventions section covering:
   - Canonical test pattern
   - Test naming conventions
   - Coverage expectations per module
   - Mock vs real service usage guidelines

2. `crates/services/PACKAGE.md` — Update test section if module structure changed

---

### Phase 12: Create Local Skill

Create `.claude/skills/test-services/` with progressive disclosure:

```
.claude/skills/test-services/
├── SKILL.md          # Quick reference + core rules
├── db-testing.md     # DB fixtures, TestDbService, migrations
├── api-testing.md    # mockito patterns, HTTP service testing
├── mock-patterns.md  # mockall setup, service trait mocking
└── advanced.md       # Concurrency, queue, progress tracking tests
```

The skill should:
- Cover all 13 service modules with module-specific guidance
- Include module-specific coverage expectations
- Show the canonical pattern with service-appropriate variations
- Include migration checklist similar to routes_app skill
- Reference `test_utils/` modules and their usage
- Include "When NOT to use this skill" section

---

## Critical Files Reference

| File | Role |
|------|------|
| `crates/services/src/test_utils/app.rs` | AppServiceStubBuilder |
| `crates/services/src/test_utils/db.rs` | TestDbService, temp DB setup |
| `crates/services/src/test_utils/auth.rs` | MockAuthService, RSA keys |
| `crates/services/src/test_utils/hf.rs` | OfflineHubService |
| `crates/services/src/test_utils/secret.rs` | SecretServiceStub, KeyringStoreStub |
| `crates/services/src/test_utils/session.rs` | Session mocks |
| `crates/services/src/test_utils/envs.rs` | Environment mocking |
| `crates/services/src/test_utils/objs.rs` | Domain object builders |
| `crates/services/Cargo.toml` | Dependencies, test-utils feature |
| `crates/services/CLAUDE.md` | Crate documentation |
| `crates/services/PACKAGE.md` | Implementation guide |

## Out of Scope

- `bin/setup_client.rs` (CLI binary, not a service)
- `app_service.rs` (trait definitions + AsRef impls, no logic to test)
- `macros.rs` (procedural macro, tested transitively)
- `objs.rs` (re-exports, no logic)
- `test_utils/` files (test infrastructure, not production code)
- Frontend/UI tests
- Integration tests in `routes_app` or `routes_all`
