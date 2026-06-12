# MCP OAuth Feature - Code Review Plan (Review Only)

## Context

The MCP OAuth 2.1 authentication feature has been implemented across two commits (`2a7eb2dc` unified OAuth system, `f2c434e9` header auth extraction). This is a **pre-merge review-only iteration** -- identify all issues, write detailed reports, and produce an index file. No code changes. The index file will be fed back to Claude Code in a subsequent iteration to pick/filter issues and apply fixes using the layered approach.

**Key constraints gathered from Q&A:**
- No backward compatibility required
- OAuth 2.1 core security compliance required (PKCE implemented, state CSRF implemented)
- Critical path test coverage only (not exhaustive)
- Flag files >500 lines only (useMcps.ts at 521 lines: let it pass)
- Extract fixtures to test-utils proactively (recommend in report)
- Full error chain validation (service -> API -> UI)
- Config naming: `header-default` / `oauth-default` with smart auto-update
- Multiple auth configs per MCP server
- Token refresh: both proactive (60s buffer) and reactive (on 401)
- Frontend constructs authorization URL and handles redirect
- AS metadata endpoint (`discover-as`) may be unused by frontend - verify
- State parameter has no TTL - flag as security gap
- PBKDF2 uses 1000 iterations - flag as nice-to-have
- Per-config Mutex for token refresh - flag as nice-to-have
- Auto-DCR: silent fallback on new page, error with retry on view page
- Discovery errors return 500

## Output Structure

All review outputs to:
```
ai-docs/claude-plans/20260220-mcp-auth/reviews/
  objs-review.md
  services-review.md
  routes_app-review.md
  backend-tests-review.md
  ui-implementation-review.md
  ui-tests-review.md
  e2e-tests-review.md
  index.md                    # Consolidated index of all findings, optimized for Claude Code
```

## Phase 1: Parallel Sub-Agent Reviews (5 agents, read-only)

All agents run in parallel since this is read-only review. Each agent reads the relevant crate's CLAUDE.md/PACKAGE.md, examines the code from commits 2a7eb2dc + f2c434e9, and writes a findings report.

Each report follows a consistent structure:
```markdown
# {Layer} Review

## Files Reviewed
- file_path (line_count lines) - purpose

## Findings

### Finding N: {Title}
- **Priority**: Critical / Important / Nice-to-have
- **File**: path/to/file.rs
- **Location**: method_name or struct_name or line range
- **Issue**: Description of the problem
- **Recommendation**: What should be done
- **Rationale**: Why this matters
```

---

### Agent 1: Backend Implementation (objs + services + routes_app)

**Scope**: `crates/objs/src/`, `crates/services/src/`, `crates/routes_app/src/` -- non-test source files
**Load context**: Each crate's CLAUDE.md + PACKAGE.md, plus `mcp-oauth-review-prompt-ctx.md`
**Output**: Three files: `objs-review.md`, `services-review.md`, `routes_app-review.md`

**objs checklist:**
- [ ] OAuth types: serde conventions (skip_serializing_if, default, kebab-case rename)
- [ ] Secret fields: not exposed in API responses (only boolean indicators)
- [ ] Discriminated union: `CreateMcpAuthConfigRequest` / `McpAuthConfigResponse` with `tag = "type"`
- [ ] McpAuthType enum: Public/Header/Oauth serialization
- [ ] Error enums: ErrorMeta derive with proper code naming
- [ ] Validation: field validations (URL format, string lengths)

**services checklist:**
- [ ] Service traits: McpService auth config operations
- [ ] No direct `Utc::now()` (use TimeService)
- [ ] `impl_error_from!` macro usage
- [ ] DB migrations: 0011 (headers), 0012 (OAuth) schema correctness
- [ ] AES-256-GCM encryption in `encryption.rs`
- [ ] PBKDF2 iterations (1000 - flag as nice-to-have)
- [ ] Token refresh: proactive + reactive with Mutex guard
- [ ] Discovery, DCR, token exchange implementations
- [ ] Ownership checks on get/delete
- [ ] INFO logging for OAuth operations

**routes_app checklist:**
- [ ] Error enums with errmeta_derive
- [ ] OpenAPI utoipa registration (7-step checklist)
- [ ] AuthContext handler patterns
- [ ] State CSRF: generation, storage, validation -- **flag no TTL as security gap**
- [ ] PKCE: code_verifier, code_challenge S256, session storage, one-time use
- [ ] Redirect URI: exact match validation
- [ ] Discovery endpoints: both AS and MCP handlers present
- [ ] DCR, login, token exchange handlers
- [ ] Error chain: service -> AppError -> ApiError -> HTTP (500 for discovery)

---

### Agent 2: Backend Tests

**Scope**: Test files in `objs`, `services`, `routes_app` from commits 2a7eb2dc + f2c434e9
**Load context**: Each crate's CLAUDE.md
**Output**: `backend-tests-review.md`

**Checklist:**
- [ ] Critical path coverage (flag missing):
  - MCP metadata discovery (success + failure)
  - AS metadata discovery (tested? used?)
  - DCR success + failure
  - Auth code flow with state validation
  - Invalid state (CSRF prevention)
  - Token exchange success (with PKCE) + failures
  - Token refresh proactive + reactive
  - Auth config CRUD (header + OAuth pre-reg + OAuth DCR)
  - Ownership check enforcement
  - Encryption roundtrip
- [ ] rstest patterns in new tests (#[case::], #[values])
- [ ] Flag existing test rstest opportunities (don't recommend conversion)
- [ ] Duplicate detection (identical logic, similar patterns, copy-paste)
- [ ] Fixture extraction opportunities (reusable patterns for test-utils)
- [ ] Mock patterns for HTTP servers

---

### Agent 3: Frontend Implementation

**Scope**: `crates/bodhi/src/` -- OAuth-related .tsx/.ts source files (not tests)
**Load context**: `crates/bodhi/src/CLAUDE.md`
**Output**: `ui-implementation-review.md`

**Files:**
- `app/ui/mcp-servers/new/page.tsx` (320 lines)
- `app/ui/mcp-servers/view/page.tsx` (357 lines)
- `app/ui/mcp-servers/components/AuthConfigForm.tsx` (337 lines)
- `hooks/useMcps.ts` (521 lines - let pass on size)
- `stores/mcpFormStore.ts`
- `app/ui/mcps/oauth/callback/page.tsx`

**Checklist:**
- [ ] Types: all from @bodhiapp/ts-client, no hand-rolled API types
- [ ] Hooks: react-query/useMutation, error handling in hooks not components
- [ ] Auto-DCR behavior (new page: silent fallback; view page: error + retry)
- [ ] Config name auto-update: `header-default` <-> `oauth-default`
- [ ] OAuth callback: state validation, code exchange, token storage, redirect
- [ ] mcpFormStore: sessionStorage persistence, cleanup, no state leakage
- [ ] **AS metadata**: verify if `useOAuthDiscoverAs()` called from any component -- if unused, recommend removal
- [ ] React best practices, accessibility
- [ ] Code duplication between new/view pages
- [ ] Auto-DCR behavior documented in CLAUDE.md for `ui/mcp-servers/`

---

### Agent 4: Frontend Tests

**Scope**: Test files in `crates/bodhi/src/` for OAuth/MCP
**Load context**: `crates/bodhi/src/CLAUDE.md`
**Output**: `ui-tests-review.md`

**Files:**
- `app/ui/mcp-servers/new/page.test.tsx`
- `app/ui/mcp-servers/view/page.test.tsx` (if exists)
- `app/ui/mcps/oauth/callback/page.test.tsx` (if exists)
- `test-utils/msw-v2/handlers/mcps.ts`

**Checklist:**
- [ ] Critical flow coverage (flag missing):
  - Auto-DCR success + silent fallback
  - Config name auto-update on type switch
  - Config name preserved on custom edit
  - OAuth callback flow
  - Auth config CRUD on view page
- [ ] MSW handlers: all endpoints, registration order, response types match ts-client, error handlers
- [ ] Test quality: data-testid, getByTestId, async handling, no inline timeouts

---

### Agent 5: E2E Tests

**Scope**: `crates/lib_bodhiserver_napi/tests-js/` OAuth-related specs
**Load context**: `tests-js/CLAUDE.md`, `tests-js/E2E.md`
**Output**: `e2e-tests-review.md`

**Files:**
- `specs/mcps/mcps-oauth-auth.spec.mjs`
- `specs/mcps/mcps-oauth-dcr.spec.mjs`
- `pages/McpsPage.mjs`
- `test-mcp-oauth-server/src/oauth.ts`

**Checklist:**
- [ ] User journey coverage (flag missing):
  - OAuth pre-registered: create -> config -> connect -> authorize -> instance
  - OAuth DCR: discover -> DCR -> config -> connect -> authorize -> instance
  - Tool execution with bearer token
  - Config reuse across instances
  - Error scenarios
  - 3rd-party app access
- [ ] Test structure: user journey, test.step(), proper flow
- [ ] Page Object: BasePage, factory methods, encapsulation
- [ ] Mock OAuth Server: RFC compliance, PKCE S256, state, DCR
- [ ] No excessive waits

---

## Phase 2: Cross-Cutting Analysis (Main Agent)

After all 5 agents complete, main agent reads all reports and performs:

1. **Type Consistency**: objs <-> OpenAPI <-> ts-client <-> frontend types
2. **Error Chain**: Trace OAuth errors end-to-end across all layers
3. **OAuth 2.1 Security Summary**: State (no TTL), PKCE (implemented), redirect URI, token encryption, PBKDF2 iterations, Mutex concurrency
4. **AS Metadata Resolution**: Consolidate findings on `discover-as` usage from Agent 1 (backend) + Agent 3 (frontend)
5. **Documentation Gaps**: CLAUDE.md/PACKAGE.md updates needed for modified crates
6. **Test Coverage Gaps**: Aggregate missing tests from Agents 2, 4, 5

---

## Phase 3: Index File Generation

Write `index.md` -- a consolidated, machine-readable index of all findings optimized for Claude Code consumption. Structure:

```markdown
# MCP OAuth Review - Issue Index

## Summary
- Total findings: N
- Critical: N | Important: N | Nice-to-have: N

## Critical Issues (Blocks Merge)
| # | File | Location | Issue | Fix Description | Report |
|---|------|----------|-------|-----------------|--------|

## Important Issues (Should Fix)
| # | File | Location | Issue | Fix Description | Report |
|---|------|----------|-------|-----------------|--------|

## Nice-to-Have (Future)
| # | File | Location | Issue | Fix Description | Report |
|---|------|----------|-------|-----------------|--------|

## Fix Order (Layered)
When applying fixes, follow this order:
1. objs crate issues -> verify: cargo test -p objs
2. services crate issues -> verify: cargo test -p objs -p services
3. routes_app crate issues -> verify: cargo test -p objs -p services -p routes_app
4. Full backend: make test.backend
5. Regenerate ts-client: make build.ts-client
6. Frontend issues -> verify: cd crates/bodhi && npm test
7. E2E issues -> verify: make build.ui-rebuild && make test.napi
8. Documentation updates
```

The index file is designed to be fed directly into Claude Code with instructions like "fix all Critical issues" or "fix issues #3, #7, #12 from the index" -- each row has enough context for Claude Code to locate and fix the issue without re-reading the full reports.
