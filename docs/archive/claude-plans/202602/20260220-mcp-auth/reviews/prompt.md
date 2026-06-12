  # MCP OAuth Feature - Fix Issues from Code Review
                                                                                                                        
  ## Background                                                   

  A comprehensive code review was performed on the MCP OAuth 2.1 authentication feature (commits `2a7eb2dc` and
  `f2c434e9`). Five parallel review agents analyzed the codebase across all layers:

  1. **Backend implementation** (objs, services, routes_app source code)
  2. **Backend tests** (objs, services, routes_app test files)
  3. **Frontend implementation** (React components, hooks, stores)
  4. **Frontend tests** (component tests, MSW handlers)
  5. **E2E tests** (Playwright specs, page objects, mock OAuth server)

  The consolidated index is at `ai-docs/claude-plans/20260220-mcp-auth/reviews/index.md`. Individual reports are in the
  same directory.

  ## Review Results Summary

  - **3 Critical** issues (must fix)
  - **19 Important** issues (should fix)
  - **20 Nice-to-have** issues (future improvement)
  - **1 False positive retracted** (services Finding 3 -- `resolve_oauth_token` correctly uses token ID, not config ID)

  ## Your Task

  ### Step 1: Read the index
  Read `ai-docs/claude-plans/20260220-mcp-auth/reviews/index.md` to understand all findings.

  ### Step 2: Plan fixes for Critical issues
  All 3 Critical issues **must be fixed**. Present a plan (using EnterPlanMode) for fixing them. The Critical issues
  are:

  - **C1**: OAuth login/token exchange handlers have ZERO test coverage (backend-tests)
  - **C2**: Token refresh logic (`resolve_oauth_token`) has ZERO test coverage (backend-tests)
  - **C3**: Inline `setTimeout` violations in 2 frontend component tests

  ### Step 3: Present Important and Nice-to-have issues to user
  After planning Critical fixes, use the **AskUserQuestion** tool to present the Important and Nice-to-have issues to
  the user, asking which ones they want fixed in this iteration. Group them logically (e.g., "Security fixes", "Test
  coverage gaps", "Frontend deduplication", "Dead code cleanup") so the user can select clusters.

  ### Step 4: Fix issues using layered development methodology
  Follow the project's layered development methodology (documented in root `CLAUDE.md`):

  1. **objs** crate first → `cargo test -p objs`
  2. **services** crate → `cargo test -p objs -p services`
  3. **routes_app** crate → `cargo test -p objs -p services -p routes_app`
  4. Full backend: `make test.backend`
  5. Regenerate ts-client: `make build.ts-client` (only if types changed)
  6. Frontend fixes → `cd crates/bodhi && npm test`
  7. E2E fixes → `make build.ui-rebuild && make test.napi`

  When working on fixes, read the **detailed report** for the relevant layer to understand the full context of each
  finding before making changes. The index has short descriptions; the individual reports have file paths, line numbers,
   code analysis, and specific recommendations.

  ## Key Constraints (from original review)

  - No backward compatibility required
  - OAuth 2.1 core security compliance required
  - Critical path test coverage only (not exhaustive)
  - Files >500 lines: flag only (useMcps.ts at 521 lines is allowed)
  - `assert_eq!(expected, actual)` convention for Rust tests
  - No inline timeouts in frontend component tests
  - Use `data-testid` with `getByTestId` for UI test selectors
  - For time handling, use `TimeService` not `Utc::now()` directly

  ## Report Files

  - `ai-docs/claude-plans/20260220-mcp-auth/reviews/index.md` — Start here
  - `ai-docs/claude-plans/20260220-mcp-auth/reviews/objs-review.md`
  - `ai-docs/claude-plans/20260220-mcp-auth/reviews/services-review.md`
  - `ai-docs/claude-plans/20260220-mcp-auth/reviews/routes_app-review.md`
  - `ai-docs/claude-plans/20260220-mcp-auth/reviews/backend-tests-review.md`
  - `ai-docs/claude-plans/20260220-mcp-auth/reviews/ui-implementation-review.md`
  - `ai-docs/claude-plans/20260220-mcp-auth/reviews/ui-tests-review.md`
  - `ai-docs/claude-plans/20260220-mcp-auth/reviews/e2e-tests-review.md`
