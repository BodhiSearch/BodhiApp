# /review

Perform a thorough code review of staged files or recent commits, validating conventions, test coverage, security, and cross-layer consistency. Produces per-crate review files and a consolidated index optimized for Claude Code consumption.

## Usage
```
/review                          # Review currently staged files
/review HEAD~1                   # Review the last commit
/review HEAD~3..HEAD             # Review last 3 commits
/review abc1234                  # Review a specific commit
/review abc1234 def5678          # Review specific commits
/review --output-dir <path>      # Custom output directory (default: ai-docs/claude-plans/reviews)
```

## Inputs

```yaml
required: none (defaults to staged files)
optional:
  - ref: git ref, commit SHA, or range (default: staged files via `git diff --cached`)
  - output_dir: output directory for review files (default: ai-docs/claude-plans/reviews)
```

## Instructions

### Step 0: Determine Scope

Parse `$ARGUMENTS` to determine what to review:
- No arguments → review currently staged files (`git diff --cached --name-only`)
- Git ref/SHA → review that commit (`git diff <ref>~1..<ref> --name-only`)
- Range (e.g. `HEAD~3..HEAD`) → review that range (`git diff <range> --name-only`)
- `--output-dir <path>` → use custom output directory

If no files are found in the diff, stop and inform the user.

Classify changed files into layers:
- **objs**: `crates/objs/src/**`
- **services**: `crates/services/src/**`
- **server_core**: `crates/server_core/src/**`
- **auth_middleware**: `crates/auth_middleware/src/**`
- **routes_app**: `crates/routes_app/src/**`
- **server_app**: `crates/server_app/src/**`
- **ui**: `crates/bodhi/src/**`
- **e2e**: `crates/lib_bodhiserver_napi/tests-js/**`
- **migrations**: `crates/services/migrations/**`
- **ts-client**: `ts-client/**`

Only review layers that have changed files. Skip unchanged layers entirely.

### Step 1: Load Context

For each affected crate, load its CLAUDE.md and PACKAGE.md:

| Crate | CLAUDE.md | PACKAGE.md |
|---|---|---|
| objs | `crates/objs/CLAUDE.md` | `crates/objs/PACKAGE.md` |
| services | `crates/services/CLAUDE.md` | `crates/services/PACKAGE.md` |
| server_core | `crates/server_core/CLAUDE.md` | `crates/server_core/PACKAGE.md` |
| auth_middleware | `crates/auth_middleware/CLAUDE.md` | `crates/auth_middleware/PACKAGE.md` |
| routes_app | `crates/routes_app/CLAUDE.md` | `crates/routes_app/PACKAGE.md` |
| server_app | `crates/server_app/CLAUDE.md` | `crates/server_app/PACKAGE.md` |
| ui (bodhi/src) | `crates/bodhi/src/CLAUDE.md` | `crates/bodhi/src/PACKAGE.md` |
| e2e (tests-js) | `crates/lib_bodhiserver_napi/tests-js/CLAUDE.md` | -- |

Also load `tests-js/E2E.md` if E2E tests are affected.

### Step 2: Launch Parallel Review Agents

Launch one sub-agent per affected layer using the Task tool. All agents run in parallel (this is read-only review). Each agent writes its findings to a separate file.

**Agent prompt template** (customize per layer):

```
You are reviewing code changes in the {crate_name} crate for convention adherence, test coverage, and correctness.

## Context
{Insert crate CLAUDE.md and PACKAGE.md contents}

## Changed Files
{List of changed files in this crate from the diff}

## Review Checklist

{Layer-specific checklist from below}

## Output Format

Write your findings to: {output_dir}/{crate}-review.md

Use this structure:
```markdown
# {Crate Name} Review

## Files Reviewed
- `relative/path/file.rs` (N lines) - purpose

## Findings

### Finding N: {Title}
- **Priority**: Critical / Important / Nice-to-have
- **File**: relative/path/file.rs
- **Location**: method_name or struct_name
- **Issue**: What's wrong
- **Recommendation**: What to do
- **Rationale**: Why it matters

## Summary
- Total findings: N (Critical: N, Important: N, Nice-to-have: N)
```

If no issues found, write a clean report noting files were reviewed with no findings.
```

### Layer-Specific Checklists

#### Backend Crates (objs, services, server_core, auth_middleware, routes_app, server_app)

**Convention Checks:**
- [ ] Serde conventions: `skip_serializing_if`, `default`, rename strategies match crate patterns
- [ ] Error types: `ErrorMeta` derive with correct code naming (`{enum_snake}_{variant_snake}`)
- [ ] `impl_error_from!` macro for external error conversions
- [ ] No direct `Utc::now()` calls (use `TimeService`)
- [ ] No `use super::*` in `#[cfg(test)]` modules
- [ ] `assert_eq!(expected, actual)` convention (expected first)
- [ ] Error chain: service error → AppError → ApiError → HTTP response
- [ ] Secret fields not exposed in API responses (use boolean indicators)
- [ ] OpenAPI utoipa annotations on new endpoints (7-step checklist from routes_app/CLAUDE.md)
- [ ] AuthContext handler patterns for protected endpoints
- [ ] Validation using constants from objs (MAX_MCP_SLUG_LEN, etc.)

**Security Checks:**
- [ ] No secrets in logs (tokens, passwords, API keys)
- [ ] Encrypted storage for sensitive fields (AES-256-GCM)
- [ ] CSRF protection (state parameter) for OAuth flows
- [ ] PKCE for OAuth authorization code flows
- [ ] Redirect URI exact match validation
- [ ] Ownership checks on user-scoped resources (`created_by` validation)
- [ ] Role/scope authorization on endpoints

**Test Coverage Checks:**
- [ ] Happy path tested for new functionality
- [ ] Main error scenarios covered (validation errors, not-found, unauthorized)
- [ ] New tests use rstest patterns (#[case::], #[values]) where appropriate
- [ ] Test fixtures: identify reusable patterns for extraction to test-utils
- [ ] Duplicate test detection (identical logic, copy-paste patterns)
- [ ] No inline timeouts in tests
- [ ] Tests provide maintenance value (no trivial constructor/derive tests)

**DB Migration Checks (if applicable):**
- [ ] Migration has both .up.sql and .down.sql
- [ ] Appropriate indexes for query patterns
- [ ] Foreign key constraints with proper ON DELETE behavior
- [ ] Column types and nullability match domain requirements

#### Frontend (crates/bodhi/src/)

**Convention Checks:**
- [ ] All API types imported from `@bodhiapp/ts-client` (no hand-rolled types)
- [ ] Types re-exported from hooks for consumer convenience
- [ ] API interactions in custom hooks using react-query/useMutation
- [ ] Error handling in hooks, not components
- [ ] Component files under 500 lines (flag only if significantly over)
- [ ] `data-testid` attributes on interactive elements for testing
- [ ] Absolute imports with `@/` prefix
- [ ] react-hook-form + zod for form validation
- [ ] Proper cleanup of side effects (useEffect cleanup, sessionStorage)

**Test Checks:**
- [ ] Critical user flows have test coverage
- [ ] MSW handlers for all new/modified API endpoints
- [ ] Handler response types match @bodhiapp/ts-client
- [ ] `getByTestId` preferred over CSS selectors
- [ ] No inline timeouts in component tests
- [ ] Proper async handling (waitFor, findBy*)

#### E2E Tests (tests-js/)

**Convention Checks:**
- [ ] User journey approach (multiple scenarios per test)
- [ ] `test.step()` for logical grouping
- [ ] Page Object Model with BasePage
- [ ] Static factory methods on fixtures
- [ ] Proper flow: setup → action → verification → cleanup
- [ ] Shared vs dedicated server used appropriately
- [ ] No excessive waits or hardcoded timeouts (except ChatPage)
- [ ] Known quirks handled (SPA nav, KC session, toast)

### Step 3: Cross-Cutting Analysis

After all agents complete, perform these checks across layers:

1. **Type Consistency**: If both backend and frontend changed, verify types align:
   - objs Rust types ↔ OpenAPI spec ↔ ts-client ↔ frontend hooks
   - Check if `make build.ts-client` would produce changes

2. **Error Chain**: If error types changed, trace through all layers:
   - Service error → AppError → ApiError → HTTP status → frontend error handling → UI display

3. **Test Coverage Gaps**: Aggregate missing coverage from all layer reviews

4. **Dead Code**: Check if removed/renamed items are still referenced elsewhere

5. **Documentation**: Check if CLAUDE.md/PACKAGE.md need updates for modified crates

### Step 4: Generate Index File

Write `{output_dir}/index.md` -- consolidated index optimized for Claude Code:

```markdown
# Code Review Index

## Review Scope
- **Ref**: {git ref or "staged files"}
- **Date**: {current date}
- **Files Changed**: N files across M crates
- **Crates Affected**: {list}

## Summary
- Total findings: N
- Critical: N | Important: N | Nice-to-have: N

## Critical Issues (Blocks Merge)
| # | Crate | File | Location | Issue | Fix Description | Report |
|---|-------|------|----------|-------|-----------------|--------|

## Important Issues (Should Fix)
| # | Crate | File | Location | Issue | Fix Description | Report |
|---|-------|------|----------|-------|-----------------|--------|

## Nice-to-Have (Future)
| # | Crate | File | Location | Issue | Fix Description | Report |
|---|-------|------|----------|-------|-----------------|--------|

## Missing Test Coverage
| # | Crate | What's Missing | Priority | Report |
|---|-------|----------------|----------|--------|

## Fix Order (Layered)
When applying fixes, follow this order:
1. objs issues → verify: `cargo test -p objs`
2. services issues → verify: `cargo test -p objs -p services`
3. server_core issues → verify: `cargo test -p server_core`
4. auth_middleware issues → verify: `cargo test -p auth_middleware`
5. routes_app issues → verify: `cargo test -p objs -p services -p routes_app`
6. Full backend: `make test.backend`
7. Regenerate ts-client: `make build.ts-client`
8. Frontend issues → verify: `cd crates/bodhi && npm test`
9. E2E issues → verify: `make build.ui-rebuild && make test.napi`
10. Documentation updates

## Reports Generated
- {list of report files with paths}
```

### Step 5: Present Summary

After generating all files, present a concise summary to the user:
- Number of files reviewed
- Finding counts by priority
- Critical issues (if any) with one-line descriptions
- Location of review files

## Important Notes

- This is **review only** -- do NOT modify any source code
- Do NOT recommend fixing trivial style issues that formatters handle (rustfmt, prettier)
- Do NOT flag existing code that wasn't changed in the diff
- Focus findings on code that was actually added or modified
- For test coverage: only flag missing tests for **new** functionality, not pre-existing gaps
- Use method/struct names in Location field, not line numbers (line numbers go stale)
- Each finding must be actionable -- if there's nothing to do, don't report it
