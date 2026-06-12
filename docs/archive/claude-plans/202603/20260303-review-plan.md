# Plan: Multi-Tenant RLS Review & Report

## Context

Commit `9223adb5d` ("RLS first cut") introduces multi-tenancy infrastructure: renames AppInstance to Tenant, adds `tenant_id` column to 14 tables, installs PostgreSQL RLS policies, and threads `tenant_id` through AuthContext. However, only tokens and user access requests are fully tenant-scoped at the application layer. All other domains (MCP, toolsets, models, user aliases) hardcode `tenant_id: String::new()` and don't filter by tenant_id in queries. RLS policies are installed but never activated at runtime (`SET LOCAL app.current_tenant_id` is never called).

**Goal**: Produce thorough review reports that identify every tenant isolation gap. Reports will be reviewed by the user, then fed to Claude Code for fixes in a separate session. No code changes in this plan.

## Decisions (from user interview)

1. **Scope**: Migrate ALL domains to pass tenant_id through AuthScope (not deferred)
2. **RLS activation**: Flag as critical gap - evaluate concurrent multi-tenant compatibility
3. **Test fixtures**: Shared fixture helper for toolset seeding; truncate tenants table in reset-db
4. **Anonymous tenant_id**: Carry tenant_id on Anonymous when available (matching client_id behavior)

---

## Phase 1: Generate Reviews (6 sub-agents in 2 batches)

Output directory: `ai-docs/claude-plans/20260303-multi-tenant/reviews/`

### Batch 1 (3 agents in parallel)

**Agent 1: `core-infra-review.md`** - Tenants + RLS + DB + AuthContext + AuthScope
- Files: `crates/services/src/tenants/`, `crates/services/src/db/sea_migrations/m20250101_000014_rls.rs`, `crates/services/src/db/default_service.rs`, `crates/services/src/db/service.rs`, `crates/services/src/db/test_rls.rs`, `crates/services/src/auth/auth_context.rs`, `crates/services/src/app_service/auth_scoped.rs`
- Focus: RLS concurrent safety (SET LOCAL + connection pooling), `get_standalone_app()` singleton assumption for multi-tenant, `reset_all_tables` tenant truncation gap, migration in-place vs additive

**Agent 2: `auth-middleware-review.md`** - Client_id/tenant_id extraction + token service
- Files: `crates/auth_middleware/src/auth_middleware/middleware.rs`, `tests.rs`, `crates/auth_middleware/src/token_service/service.rs`, `tests.rs`, `crates/auth_middleware/src/api_auth_middleware.rs`, `access_request_auth_middleware/tests.rs`, `utils.rs`, `tests/test_live_auth_middleware.rs`
- Focus: `.expect()` panics in middleware, Anonymous tenant_id=None (should carry), bearer token cross-tenant risk, redundant `get_standalone_app()` calls per request

**Agent 3: `tokens-mcps-toolsets-review.md`** - Tokens + MCPs + Toolsets vertical
- Files: `crates/services/src/tokens/` (all), `crates/services/src/mcps/` (all), `crates/services/src/toolsets/` (all), `crates/services/src/app_service/auth_scoped_tokens.rs`, `crates/routes_app/src/tokens/`, `crates/routes_app/src/mcps/test_oauth_utils.rs`
- Focus: Tokens as reference impl (verify correct). MCPs: every `String::new()`, missing filters, service gaps. Toolsets: seeding removal, `String::new()`, missing filters, `app_toolset_configs` gaps.

### Batch 2 (3 agents in parallel)

**Agent 4: `models-users-review.md`** - Models + Users + App Access Requests vertical
- Files: `crates/services/src/models/` (all), `crates/services/src/users/` (all), `crates/services/src/app_access_requests/` (all), `crates/routes_app/src/models/`, `crates/routes_app/src/users/`, `crates/routes_app/src/apps/`
- Focus: download/alias/metadata repos ALL lack tenant_id filtering, `check_prefix_exists` cross-tenant, access_repository partial migration, `String::new()` in app_access_requests

**Agent 5: `routes-composition-review.md`** - Routes + settings + setup + auth
- Files: `crates/routes_app/src/routes.rs`, `routes_dev.rs`, `settings/`, `setup/`, `auth/`, `shared/auth_scope_extractor.rs`, `test_utils/router.rs`
- Focus: Multi-tenant route gating, settings LLM blocking, setup tenant creation flow, auth flow tenant_id threading

**Agent 6: `app-layer-test-infra-review.md`** - App bootstrap + NAPI + test infrastructure
- Files: `crates/lib_bodhiserver/`, `crates/lib_bodhiserver_napi/`, `crates/server_app/tests/utils/`, `crates/services/src/test_utils/`, `crates/services/src/settings/`, `crates/services/src/lib.rs`
- Focus: `Tenant.id = String::new()` in NAPI, TEST_TENANT_ID consistency, AppServiceStub tenant support, test fixture gaps for toolset seeding

---

## Phase 2: Consolidation (main agent)

After all 6 agents complete:
1. Read all review files
2. Deduplicate findings across agents
3. Normalize priorities:
   - **P0 (Critical)**: tenant_id not passed where required for data isolation, potential cross-tenant data leak, RLS not activated
   - **P1 (High)**: `String::new()` placeholder insertions, missing repository filters, `.expect()` panics
   - **P2 (Medium)**: Naming inconsistencies, redundant DB calls, test gaps, stale references
   - **P3 (Low)**: Minor cosmetic, pre-existing issues amplified
4. Generate `index.md` with summary tables, fix iteration order, cross-cutting concerns, report links

---

## Review Report Format

Each report uses the format from the sample at `ai-docs/claude-plans/20260301-reorg-routes/reviews/`:
- Only findings (no positives/compliance)
- Priority table with file, location, issue, recommendation
- Recommendations are moderately prescriptive: reference relevant files/methods, but expect the implementor to explore and propose final fixes
- Each finding includes enough context for Claude Code to locate and understand the issue

---

## Proposed Fix Plan (to be refined after reviews)

Reviews will inform a layered fix plan following the crate dependency chain. No implementation in this session -- the fix plan will be a separate deliverable in `index.md` for future execution:

1. **Core**: RLS activation, Anonymous tenant_id, middleware .expect() fix
2. **Services upstream**: Migrate all domain services to accept/filter tenant_id
3. **Routes downstream**: Update handlers to use auth-scoped tenant_id
4. **Test infra**: Shared toolset fixture, tenants table truncation, per-domain isolation tests
5. **Validation**: Full backend test suite
