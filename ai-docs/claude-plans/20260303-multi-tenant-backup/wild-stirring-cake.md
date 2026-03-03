# Plan: Thorough Review of Multi-Tenant Layer 1 Commit (HEAD)

## Context

The HEAD commit (`80e09c70c`) is a squashed mega-commit containing ~30 sub-commits that implement:
1. **Multi-tenant DB infrastructure**: begin_tenant_txn(), RLS policies, tenant_id threading through all layers
2. **CRUD refactor**: Form->Request renames, ValidatedJson, 2.5-type pattern, entity aliases
3. **auth_middleware crate merge**: Deleted crate, moved into routes_app/middleware/
4. **InferenceService extraction**: Removed RouterState/SharedContext, lifecycle via InferenceService trait
5. **Domain service patterns**: ApiModel, Download, Toolset, MCP, Token, UserAlias all refactored

**Scope**: 351 files, 20,579 insertions, 11,606 deletions across 8 crates + frontend + ts-client.

**Goal**: Produce domain-by-domain review reports optimized for Claude Code to consume and fix issues, making the code production-ready.

**Prior review**: SUMMARY.md documents 21 fixed issues + 11 remaining known/accepted issues. We treat the prior review as superseded — this is a fresh review on the final squashed code.

---

## Review Strategy

### Phase A: Sequential Warm-up (2 domains)

Run sequentially to calibrate review depth and discover cross-cutting patterns to feed into parallel agents.

**A1: Settings domain** (~7 files)
- `crates/services/src/settings/` (3 files: constants.rs, setting_objs.rs, settings_service.rs)
- `crates/routes_app/src/settings/` (4 files: routes_settings.rs, mod.rs, tests)
- Focus: Type moves (UpdateSettingRequest, constants), ValidatedJson adoption, tenant_id handling

**A2: Tokens domain** (~11 files)
- `crates/services/src/tokens/` (7 files: error.rs, token_objs.rs, token_service.rs, mod.rs, tests)
- `crates/routes_app/src/tokens/` (4 files: routes_tokens.rs, error.rs, tests)
- Focus: CreateTokenForm->Request, Entity->Response pattern, token format change (bodhiapp_<random>.<client_id>), auth-scoped delegation

After each warm-up agent completes, extract:
- Cross-cutting patterns to check in remaining domains
- Any calibration notes on review depth/format

### Phase B: Parallel domain reviews (10 agents)

All run in parallel, each writing to their own review file.

**B1: DB Core + Migrations** (~20 files)
- `crates/services/src/db/` (db_core.rs, default_service.rs, error.rs, service.rs, sea_migrations/*, test_rls.rs)
- Focus: begin_tenant_txn() impl, RLS policy SQL audit (USING/WITH CHECK), SQLite fallback, composite unique indexes, migration correctness

**B2: Tenants** (~8 files)
- `crates/services/src/tenants/` (all files — entirely new domain)
- Focus: TenantService trait design, get_tenant_by_client_id(), repository patterns, test coverage

**B3: Models / ApiModels** (~30 files)
- `crates/services/src/models/` (21 files) + `crates/routes_app/src/api_models/` (9 files) + `crates/routes_app/src/models/` (7 files)
- Focus: ApiModelService + AuthScopedApiModelService, type placement (ApiModelRequest, ApiModelOutput), ValidatedJson, user_id threading, DataService changes

**B4: MCPs** (~28 files)
- `crates/services/src/mcps/` (19 files) + `crates/routes_app/src/mcps/` (9 files)
- Focus: McpForm->McpRequest, Row->Entity aliases, McpService trait changes, auth config tenant scoping, ValidatedJson

**B5: Toolsets** (~13 files)
- `crates/services/src/toolsets/` (9 files) + `crates/routes_app/src/toolsets/` (4 files)
- Focus: ToolsetForm->ToolsetRequest, Entity->Response in handlers, ToolService returns ToolsetEntity, validation moved to ValidatedJson

**B6: Users + User Access Requests** (~14 files)
- `crates/services/src/users/` (5 files) + `crates/routes_app/src/users/` (9 files)
- Focus: Type moves (ChangeRoleRequest, UserAccessStatusResponse, etc.), AuthScopedUserAccessRequestService, tenant_id injection
- Note: "user access requests" = users requesting access to the app (pending/approved/denied)

**B7: Apps + App Access Requests** (~7 files)
- `crates/services/src/apps/` (3 files) + `crates/services/src/app_access_requests/` (7 files) + `crates/routes_app/src/apps/` (4 files)
- Focus: CreateAccessRequest/ApproveAccessRequest type moves, handler delegation to auth-scoped services
- Note: "app access requests" = external apps requesting access, reviewed/approved by users

**B8: Middleware** (~24 files in routes_app/src/middleware/)
- All files under `crates/routes_app/src/middleware/` (merged from deleted auth_middleware crate)
- Focus: **Security-focused** — auth flow correctness, token validation (new bodhiapp_<random>.<client_id> format), session handling, tenant_id extraction, no auth bypass risks, old backward-compat removal

**B9: InferenceService + server_core** (~12 files)
- `crates/services/src/inference/` (4 files) + `crates/server_core/` (8 files)
- Focus: InferenceService trait design, lifecycle methods (stop, set_variant, set_keep_alive, is_loaded), StandaloneInferenceService/MultitenantInferenceService, keep-alive timer internalization, VariantChangeListener rewiring

**B10: Frontend + TS types + Cross-cutting** (~40 files)
- `crates/bodhi/src/` (34 files) + `ts-client/` (2 files) + `openapi.json` + `crates/lib_bodhiserver/` (6 files) + `crates/lib_bodhiserver_napi/` (5 files) + `crates/server_app/` (9 files)
- Focus: Type renames in frontend (Form->Request), MSW handler updates, hook changes, ts-client regeneration consistency, bootstrap wiring (lib_bodhiserver, NAPI), server_app route registration

### Phase C: Consolidation (1 agent)

After all B-phase agents complete, launch a consolidation agent that:
1. Reads all 12 review files (A1, A2, B1-B10)
2. Deduplicates cross-cutting findings
3. Assigns final priority (P0-P3) to each finding
4. Writes `summary.md` with consolidated index following the sample format from 20260301-reorg-routes/reviews/index.md

---

## Output Structure

```
ai-docs/claude-plans/20260303-multi-tenant/reviews/
  settings.md          (A1)
  tokens.md            (A2)
  db-core.md           (B1)
  tenants.md           (B2)
  models-api-models.md (B3)
  mcps.md              (B4)
  toolsets.md          (B5)
  users.md             (B6)
  apps-access.md       (B7)
  middleware.md         (B8)
  inference.md         (B9)
  frontend-cross.md    (B10)
  summary.md           (C)
```

## Review Format (per domain file)

```markdown
# {Domain} Review

## Scope
- Files reviewed: N
- Lines changed: +N / -N

## Findings

| # | Priority | File | Location | Issue | Recommendation (with rationale) |
|---|----------|------|----------|-------|---------------------------------|

## Summary
- Total: N (P0: N, P1: N, P2: N, P3: N)
```

Priority levels:
- **P0**: Correctness/security bug — blocks production readiness
- **P1**: Convention violation or design issue — should fix
- **P2**: Improvement opportunity — fix in follow-up
- **P3**: Cosmetic/minor — optional cleanup

## Review Checklist (all agents)

Each agent checks for:
1. **Multi-tenant correctness**: tenant_id properly threaded, not hardcoded "", RLS-safe
2. **CRUD pattern compliance**: ValidatedJson, Request types, Entity->Response, service-layer validation
3. **Auth-scoped service usage**: Handlers use AuthScope, not raw State/Extension
4. **Error handling**: Correct ErrorType, proper From chains, no .expect()/.unwrap() in non-test code
5. **Type placement**: Request/Response in services, presentation-only in routes_app
6. **Test coverage**: New functionality has tests, test data uses TEST_TENANT_ID/TEST_USER_ID
7. **Security**: No auth bypass, secrets not exposed, ownership checks present
8. **Dead code**: Removed types not still referenced, no orphaned imports

## Execution Flow

```
[A1: Settings] ──sequential──> [A2: Tokens] ──extract patterns──>
                                                                  \
                                                                   [B1..B10 in parallel]
                                                                  /
                                                          ──all complete──> [C: Consolidation]
```

## Verification

After reviews are generated:
1. Read `summary.md` to confirm all domains covered
2. Verify finding counts are consistent across domain files and summary
3. Present summary to user with P0/P1 highlights

---

## Phase D: Fix Implementation Kick-off Prompt

After review is complete, the following prompt should be fed into a new Claude Code session to implement all fixes. It uses sequential specialized sub-agents going upstream-to-downstream with gate checks.

### Kick-off Prompt

```
You are implementing fixes from code review reports for BodhiApp's multi-tenant commit.

## Review Reports Location
All review files are at: `ai-docs/claude-plans/20260303-multi-tenant/reviews/`
- `summary.md` — consolidated index with all findings by priority
- Domain files: settings.md, tokens.md, db-core.md, tenants.md, models-api-models.md, mcps.md, toolsets.md, users.md, apps-access.md, middleware.md, inference.md, frontend-cross.md

## Instructions

1. Read `summary.md` first to understand all P0 and P1 findings
2. Read each domain review file referenced by P0/P1 findings
3. Implement fixes using sequential specialized sub-agents, one per crate layer, upstream to downstream
4. Each sub-agent must pass its gate check before the next starts

## Fix Order (upstream to downstream)

### Step 1: services crate fixes
Launch a sub-agent to fix all P0/P1 findings in `crates/services/src/`:
- DB core / migrations / RLS issues (db-core.md)
- Tenants issues (tenants.md)
- Domain service issues: models, mcps, toolsets, tokens, users, app_access_requests
- inference module issues (inference.md)

**Gate check**: `cargo test -p services`

### Step 2: server_core crate fixes
Launch a sub-agent to fix all P0/P1 findings in `crates/server_core/src/`:
- InferenceService impl issues (inference.md)
- Any wiring issues

**Gate check**: `cargo test -p server_core`

### Step 3: routes_app crate fixes
Launch a sub-agent to fix all P0/P1 findings in `crates/routes_app/src/`:
- Middleware security issues (middleware.md)
- Route handler issues across all domains
- Error type issues
- ValidatedJson / auth-scope issues

**Gate check**: `cargo test -p routes_app`

### Step 4: server_app + lib_bodhiserver fixes
Launch a sub-agent for remaining backend crate fixes.

**Gate check**: `cargo test -p server_app -p lib_bodhiserver`

### Step 5: Full backend validation
Run: `make test.backend`
If failures, fix them before proceeding.

### Step 6: Regenerate TypeScript types
Run: `cargo run --package xtask openapi && cd ts-client && npm run generate`
If types changed, verify the diffs make sense.

### Step 7: Frontend fixes
Launch a sub-agent to fix all P0/P1 findings in `crates/bodhi/src/`:
- Type renames, hook updates, MSW handler fixes

**Gate check**: `cd crates/bodhi && npm run build && npm run test:all`

### Step 8: Rebuild NAPI + UI
Run: `make build.ui-rebuild`

### Step 9: E2E tests
Run: `cd crates/lib_bodhiserver_napi && npm run test:playwright:sqlite`
If failures, fix and re-run.

### Step 10: P2 fixes (if time permits)
After all P0/P1 fixes pass end-to-end, optionally address P2 findings using the same upstream-to-downstream order.

## Important
- Do NOT fix P3 issues unless explicitly asked
- Each sub-agent should read the relevant review file(s) and the specific findings it needs to fix
- After each gate check failure, diagnose and fix before proceeding — do not skip gates
- Commit after each successful gate check with message: `fix(review): {domain} — {brief description}`
```
