# Plan: Update test-routes-app Skill After Auth Test Migration

## Context

The auth test reorganization (commits `7e8432b8..86b14871`) moved all auth tests from standalone integration test files (`tests/*_auth_test.rs`) into their respective module test files (`src/*/tests/*_test.rs`). The `test-routes-app` skill documentation still describes the old two-tier pattern (router-level vs handler-level) and references deleted files. The skill needs to reflect the new unified test pattern.

**test-services skill**: No changes needed -- it covers a different crate and was unaffected by the migration.

## User Decisions
- Replace old pattern completely (no deprecation markers)
- Merge router-level + handler-level into unified pattern
- Prefer module-level imports (don't mention function-level imports)
- Cartesian product (`#[values]` roles x endpoints) for allow tests
- Teach safe endpoint identification pattern (not per-module lists)

## Changes

### 1. `SKILL.md` -- Major rewrite

**File**: `.claude/skills/test-routes-app/SKILL.md`

**A. Replace "Two Test Patterns" with unified pattern**

Remove the current sections "1. Router-Level Tests" and "2. Handler-Level Tests". Replace with a single unified pattern showing:
- Same annotations for all tests: `#[rstest]` + `#[tokio::test]` + `#[anyhow_trace]`
- Return type: `anyhow::Result<()>` with `?` propagation
- Two fixture setups as variations:
  - `build_test_router()` -- for auth tier tests and integration-style tests with real middleware
  - `AppServiceStubBuilder` -- for isolated handler tests with mock expectations

**B. Replace "When to Use Which" table**

New table reflecting unified pattern with two fixture approaches:

| Scenario | Fixture |
|----------|---------|
| Auth tier verification (401/403) | `build_test_router()` |
| Endpoint reachability with correct role | `build_test_router()` |
| Business logic with mock expectations | `AppServiceStubBuilder` |
| Error path testing | `AppServiceStubBuilder` |
| SSE streaming | `AppServiceStubBuilder` |

**C. Update "Core Rules"**

Merge into single annotation set:
1. All tests: `#[rstest]` + `#[tokio::test]` + `#[anyhow_trace]`. Add `#[awt]` ONLY with `#[future]` params.
2. Return: `-> anyhow::Result<()>`, use `?` not `.unwrap()`
3. Remove the separate router-level annotation rule (no `#[anyhow_trace]`, `.unwrap()`)

**D. Update "Standard Imports"**

Remove the separate "Router-level tests" and "Handler-level tests" import blocks. Replace with a single block of typical module-level imports. Include `tower::ServiceExt`, `StatusCode`, test utility imports at module level.

**E. Update "Auth Tier Reference"**

Keep the table but verify it's accurate. Minor: confirm endpoint paths match current routes.

**F. Update location reference (line 22)**

Change:
```
Located in: `crates/routes_app/tests/*_auth_test.rs`
```
To:
```
Located in: `crates/routes_app/src/<module>/tests/*_test.rs` (inline with handler tests)
```

### 2. `advanced.md` -- Major rewrite of auth sections

**File**: `.claude/skills/test-routes-app/advanced.md`

**A. Rewrite "Auth Test Organization" section**

Remove the old file listing showing `tests/routes_*_auth_test.rs`. Replace with:
- Auth tests are now inline in module test files at `src/<module>/tests/*_test.rs`
- Each module file contains both handler tests AND auth tests
- Auth tests placed at the bottom of the file after handler tests

**B. Rewrite "Auth Test Template"**

Replace the three-category template with updated patterns:

1. **Unauthenticated rejection (401)** -- `#[case]` per endpoint, `#[anyhow_trace]`, `anyhow::Result<()>`
2. **Insufficient role rejection (403)** -- `#[values]` cartesian product (roles x endpoints) instead of `for` loops
3. **Authorized access (200/OK)** -- `#[values]` cartesian product (eligible roles x safe endpoints)

Key changes from old template:
- `#[anyhow_trace]` added to all auth tests
- `anyhow::Result<()>` return type with `?` propagation
- `#[values]` replaces `for` loops and `router.clone()`
- `assert_eq!` with specific status codes (not `assert_ne!`)

**C. Add "Safe Endpoint Identification Pattern" section**

New section teaching how to identify safe endpoints for "allow" tests:
- **Safe**: Endpoints using real services in `build_test_router()` -- `DbService` (real SQLite), `DataService` (real file-based), `SessionService` (real SQLite)
- **Unsafe**: Endpoints calling `MockAuthService`, `MockToolService`, or `MockSharedContext` -- these panic without expectations
- **Rule of thumb**: GET list endpoints typically safe (return empty list/OK). Mutating endpoints or those calling external services typically unsafe.
- Add comment in test explaining why certain endpoints are excluded from allow tests

**D. Update "Test File Organization" section**

Remove:
```
- **Router-level auth tests**: `crates/routes_app/tests/<module>_auth_test.rs`
```

Replace with:
```
- **Auth tests**: Inline in `src/<module>/tests/*_test.rs` (bottom of file, after handler tests)
```

**E. Update "Coverage Commands"**

Remove:
```bash
cargo test -p routes_app --test routes_oai_auth_test  # Specific auth test file
```

Replace with:
```bash
cargo test -p routes_app --lib -- routes_oai  # Module filter (includes auth + handler tests)
```

**F. Update "Skipping Allowed Tests" section**

Update to reflect new pattern -- still relevant but use `#[values]` context instead of loops.

### 3. `requests.md` -- Minor update

**File**: `.claude/skills/test-routes-app/requests.md`

- Remove function-level import references (line 5: `Import: use routes_app::test_utils::...`)
- Show imports at module level in examples
- Update the "Multiple requests with same router" pattern to show `#[values]` alternative

### 4. `fixtures.md` -- Minor update

**File**: `.claude/skills/test-routes-app/fixtures.md`

- Clarify `build_test_router()` usage: "Used for both auth tier tests and integration-style handler tests"
- Ensure examples show `?` propagation instead of `.unwrap()`

### 5. `assertions.md` -- No changes

Already uses the correct patterns (`assert_eq!(expected, actual)`, `?` propagation).

## Verification

```bash
# Verify skill files are valid markdown (no broken links)
ls .claude/skills/test-routes-app/

# Verify no references to deleted test file paths
grep -r "tests/.*_auth_test" .claude/skills/test-routes-app/

# Verify no .unwrap() in auth test templates
grep -n "\.unwrap()" .claude/skills/test-routes-app/SKILL.md

# Expected: zero matches for deleted paths, zero unwrap in auth templates
```
