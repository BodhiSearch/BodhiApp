# Prompt: routes_app Test Uniformity & Consistency Analysis

## Context

The `crates/routes_app/` test suite has grown organically and now needs a holistic review for consistency and uniformity across common testing concerns. Tests were recently migrated from standalone auth test files to inline module tests, and we want to ensure consistent patterns across all modules.

## Goal

Explore all tests in `crates/routes_app/` and identify opportunities for uniformity in:

1. **Authorization testing patterns** - role-based access control (401/403/200 tests)
2. **Test utilities** - reusable helpers for common scenarios
3. **Test file organization** - naming conventions, module structure
4. **rstest usage** - leveraging `#[values]` for role/endpoint combinations
5. **Code organization** - `mod.rs` usage, re-exports vs implementation

## Background: Current Canonical Pattern

Our canonical test pattern (documented in `.claude/skills/test-routes-app/`) expects:

- **Unified annotations**: `#[rstest]` + `#[tokio::test]` + `#[anyhow_trace]`, return `anyhow::Result<()>`
- **Auth tier tests**: Use `build_test_router()` with `#[values]` cartesian product for role × endpoint combinations
- **Cartesian product pattern** for 403 tests:
  ```rust
  #[rstest]
  #[tokio::test]
  #[anyhow_trace]
  async fn test_endpoints_reject_insufficient_role(
    #[values("resource_user", "resource_power_user")] role: &str,
    #[values(("GET", "/path/a"), ("POST", "/path/b"))] endpoint: (&str, &str),
  ) -> anyhow::Result<()> {
    // Test logic using cartesian product of all combinations
  }
  ```
- **Test file location**: Auth tests inline in `src/<module>/tests/*_test.rs` alongside handler tests

## Known Inconsistencies (Examples, Not Exhaustive)

### Example 1: Inconsistent Role Testing Patterns

Some modules use `#[values]` cartesian product for role-based tests:
```rust
// Good: Cartesian product
#[values("resource_user", "resource_power_user")] role: &str,
#[values(("GET", "/path/a"), ("POST", "/path/b"))] endpoint: (&str, &str),
```

Others use manual loops or individual test cases:
```rust
// Inconsistent: Manual iteration
for (method, path) in endpoints {
  let response = router.clone().oneshot(...).await?;
}
```

### Example 2: Inconsistent Test Utilities

Common scenarios like "test all endpoints allow admin" or "test all endpoints disallow non-admin" are often duplicated across modules instead of using shared utilities. We may need test helpers like:
- `test_endpoints_allow_role(router, role, endpoints)` - verify role and above can access
- `test_endpoints_reject_role(router, role, endpoints)` - verify below-role gets 403

### Example 3: Module Organization

Some modules use `mod.rs` for actual implementation, others use it only for re-exports. We need consistency:
- **Preferred**: `mod.rs` only for re-exports, implementation in dedicated files
- **Avoid**: Business logic in `mod.rs`

## Your Task

**Phase 1: Exploration**

1. Survey all test files in `crates/routes_app/src/*/tests/`
2. Identify patterns and anti-patterns in:
   - Authorization test structure (401/403/200 tests)
   - Use of `#[values]` vs manual iteration
   - Test naming conventions
   - Module organization (`mod.rs` usage)
   - Common test utilities that could be extracted
3. Note modules that follow canonical patterns vs those that don't
4. Identify opportunities for new test utilities to reduce duplication

**Phase 2: Planning**

Create a plan that addresses:

1. **Test utility extraction** - What common helpers should exist in `test_utils` to reduce duplication?
   - Example: `test_endpoints_allow_admin(router, endpoints)` helper
   - Example: `test_endpoints_reject_below_role(router, min_role, endpoints)` helper

2. **Uniformity migrations** - Which modules need updates to use:
   - `#[values]` cartesian product pattern for role-based tests
   - Consistent naming: `test_<tier>_endpoints_allow_<role>`, `test_<tier>_endpoints_reject_insufficient_role`
   - Consistent annotation pattern: `#[rstest]` + `#[tokio::test]` + `#[anyhow_trace]`

3. **Module organization** - Identify `mod.rs` files with implementation code that should be refactored

4. **Prioritization** - Suggest order of changes:
   - High impact, low risk changes first (test utilities)
   - Module-by-module migration strategy
   - Breaking changes last (if any)

## Key Principles

- **Uniformity over cleverness** - Consistent patterns are more valuable than optimized one-offs
- **Cartesian product for roles** - Use `#[values]` for hierarchical role testing (user < power_user < manager < admin)
- **Shared utilities for common concerns** - Authorization, authentication should have reusable helpers
- **Test file naming** - Clear, predictable conventions
- **Separation of concerns** - `mod.rs` for re-exports, implementation in dedicated files

## Constraints

- Don't break existing tests - migrations should preserve coverage
- Leverage existing `test_utils` module in `routes_app`
- Follow patterns in `.claude/skills/test-routes-app/` skill documentation
- Consider maintainability - new developers should understand the pattern quickly

## Expected Deliverables

1. **Analysis document** - Summary of current state:
   - Modules following canonical patterns
   - Modules with inconsistencies
   - Common duplication patterns
   - Opportunities for new utilities

2. **Migration plan** - Phased approach:
   - New test utilities to create
   - Module-by-module migration order
   - Breaking changes (if any)
   - Verification strategy

3. **Examples** - Show before/after for:
   - Role-based auth test using new utilities
   - Module organization improvement
   - Cartesian product migration

## Notes

- This is an exploratory task - you should discover the specific inconsistencies, not just apply a checklist
- Focus on high-leverage changes that improve consistency across many modules
- Consider the cost/benefit of each proposed change
- Reference the canonical patterns in `.claude/skills/test-routes-app/` but don't be overly rigid
- Propose new patterns if they improve clarity and maintainability

## Success Criteria

After implementing your plan:
- All modules follow consistent role-based authorization testing patterns
- Common scenarios use shared test utilities (DRY principle)
- Test file organization is predictable across modules
- New developers can quickly understand and replicate patterns
- `#[values]` cartesian product used consistently for hierarchical role testing
- `mod.rs` files follow consistent re-export-only pattern
