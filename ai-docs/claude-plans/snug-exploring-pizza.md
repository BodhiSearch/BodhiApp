# Plan: Consolidate Multi-Tenant Stage 2 Docs into Functional Specs

## Context

28 planning/kickoff/implementation docs in `ai-docs/claude-plans/20260306-multi-tenant-2-backup/` document the multi-tenant stage 2 work across commits HEAD (`6cc6e9b44`) and HEAD~1 (`018e13dba`). These docs are a mix of kickoff prompts, implementation plans, code reviews, and decisions — scattered across chronological files. We need to consolidate them into **domain-wise functional specs** that serve as permanent documentation for AI coding assistants.

**Goal**: Create sequential, evolving functional specs in `consolidated-docs/` that replace the 28 source docs.

## Output Structure

8 consolidated spec files in `ai-docs/claude-plans/20260306-multi-tenant-2-backup/consolidated-docs/`:

| # | File | Domain | Purpose |
|---|------|--------|---------|
| 01 | `01-deployment-modes-and-status.md` | Deployment modes, AppStatus | Foundation: standalone vs multi-tenant, state machine |
| 02 | `02-auth-sessions-middleware.md` | Auth, sessions, middleware | Gateway: JWT resolution, session keys, token lifecycle |
| 03 | `03-tenant-management-and-spi.md` | Tenant CRUD, membership, SPI | Core: tenant lifecycle, Keycloak integration |
| 04 | `04-database-migrations-entities.md` | Migrations, entities, schema | Infrastructure: data model, migration restructuring |
| 05 | `05-data-isolation-rls.md` | RLS, isolation | Cross-cutting: tenant-scoped data access |
| 06 | `06-frontend-ui.md` | UI components, hooks | Consumer: multi-tenant UI flows |
| 07 | `07-testing-infrastructure.md` | E2E, integration, unit tests | Verification: test infrastructure and coverage |
| 08 | `08-decisions-index.md` | All decisions D21-D106 | Reference: decision lookup table |

## Spec Template

Each spec follows:
```
# <Title>
## Overview (2-3 sentences)
## Functional Behavior (the "what" — user/API perspective, endpoint contracts)
## Architecture & Data Model (the "how" — types, services, data flow)
## Technical Implementation (concise — key files with paths, no code duplication)
## Decisions (table of relevant decisions with status)
## Known Gaps & TECHDEBT
```

## Execution Strategy — Sub-Agent Topology

### Phase A: Read Git Diffs (preparation)
Save `git diff HEAD~2..HEAD~1` and `git diff HEAD~1..HEAD` to temp files for sub-agents to reference.

### Phase B: Wave 1 — Independent Foundation (3 agents in parallel)

**Agent 1: `01-deployment-modes-and-status.md`**
- Source docs: `multi-tenant-flow-ctx.md`, `kickoff-bodhi-backend.md` (Section 2), `20260309-mt-arch-refactor.md` (DeploymentMode), `decisions.md`
- Git diff files: `crates/services/src/settings/`, `crates/routes_app/src/setup/`, `crates/services/src/auth/auth_context.rs`
- Focus: DeploymentMode enum, AppStatus variants, /info endpoint behavior

**Agent 2: `04-database-migrations-entities.md`**
- Source docs: `20260307-backend-impl.md` (Phase 1), `20260309-mt-arch-refactor.md` (Section 6), `20260309-create-tenant-membership.md`, `20260309-mt-repo-test-audit.md`
- Git diff files: `crates/services/src/db/sea_migrations/`, `crates/services/src/tenants/tenant_entity.rs`, `crates/services/src/tenants/tenant_user_entity.rs`, entity files
- Focus: Migration restructuring, tenants_users table, entity cleanup (Row→Entity), new_ulid()

**Agent 3: `08-decisions-index.md`**
- Source docs: `decisions.md`, `multi-tenant-flow-ctx.md` (Decision Index), `decision-organization-feature-deferred.md`
- No git diffs needed — pure reference extraction
- Focus: All decisions D21-D106 organized by domain with status

### Phase C: Wave 2 — Auth & RLS (2 agents in parallel, receive Wave 1 outputs)

**Agent 4: `02-auth-sessions-middleware.md`** (receives 01 output as context)
- Source docs: `kickoff.md`, `20260306-middleware-multi-tenant.md`, `multi-tenant-flow-ctx.md` (Sessions + Middleware), `kickoff-bodhi-backend.md` (Sections 3,6,7,8), `20260308-pre-e2e-fixes.md`, `20260309-mt-arch-refactor.md` (Sections 2-4,11-12)
- Git diff files: `crates/routes_app/src/middleware/auth/`, `crates/routes_app/src/middleware/token_service/`, `crates/routes_app/src/auth/`, `crates/services/src/auth/auth_context.rs`, `crates/services/src/auth/auth_service.rs`
- Focus: AuthContext variants, middleware flow, session namespacing, token refresh, auth endpoints

**Agent 5: `05-data-isolation-rls.md`** (receives 04 output as context)
- Source docs: `20260306-services-multi-tenant-isolation-test.md`, `routes_app-isolation-test.md`, `kickoff-multi-tenant-routes-app-test.md`, `20260309-mt-repo-test-audit.md`
- Git diff files: `crates/services/src/db/test_rls.rs`, migration files (RLS policy sections), isolation test files
- Focus: RLS policy distribution, begin_tenant_txn(), isolation test patterns

### Phase D: Wave 3 — Tenant & Frontend (2 agents in parallel, receive Wave 2 outputs)

**Agent 6: `03-tenant-management-and-spi.md`** (receives 01+02 output as context)
- Source docs: `kickoff-keycloak-spi.md`, `multi-tenant-flow-ctx.md` (Tenant sections), `kickoff-bodhi-backend.md` (Sections 4-5,9), `20260309-mt-arch-refactor.md` (Sections 7-10,13-14), `20260309-create-tenant-membership.md`, `decision-organization-feature-deferred.md`
- Git diff files: `crates/routes_app/src/tenants/`, `crates/services/src/tenants/`, `crates/services/src/users/`
- Focus: Tenant CRUD, membership, SPI proxy, two-phase login, AuthScopedTenantService

**Agent 7: `06-frontend-ui.md`** (receives 01+02 output as context)
- Source docs: `kickoff-bodhi-frontend.md`, `20260308-frontend-impl.md`, `kickoff-ui-test-multi-tenant.md`, `multi-tenant-flow-ctx.md` (Frontend Flow)
- Git diff files: `crates/bodhi/src/` (all changed UI files)
- Focus: Login page refactor, dashboard callback, tenant selector, hooks, AppInitializer

### Phase E: Wave 4 — Testing (1 agent, receives all previous outputs)

**Agent 8: `07-testing-infrastructure.md`** (receives all outputs as context)
- Source docs: `20260308-rs-integration-test.md`, `kickoff-integ-test-multi-tenant.md`, `kickoff-e2e-multi-tenant.md`, `kickoff-e2e-standalone-fixes.md`, `kickoff-e2e-multi-tenant-coverage.md`, `20260306-multi-tenant-e2e-pg.md`, `20260307-e2e-test-failure.md`, `TECHDEBT.md`
- Git diff files: `crates/lib_bodhiserver_napi/` (E2E files), test files across all crates
- Focus: Test levels, infrastructure, MultiTenantSession rstest, E2E Playwright, known gaps

### Phase F: Wave 5 — Summarizer (1 agent, receives all 8 docs)

**Summarizer Agent**: Reads all 8 consolidated docs, removes duplication between docs, fixes cross-references, resolves anomalies, ensures sequential narrative coherence. Produces final versions of all 8 files.

## Source Doc → Consolidated Spec Mapping

| Source Doc | Maps To |
|-----------|---------|
| kickoff.md | 02 |
| multi-tenant-flow-ctx.md | 01, 02, 03, 06 |
| decisions.md | 08 (primary), referenced by all |
| decision-organization-feature-deferred.md | 03, 08 |
| TECHDEBT.md | 07 |
| kickoff-bodhi-backend.md | 01, 02, 03 |
| kickoff-bodhi-frontend.md | 06 |
| kickoff-keycloak-spi.md | 03 |
| kickoff-e2e-multi-tenant.md | 07 |
| kickoff-e2e-multi-tenant-coverage.md | 07 |
| kickoff-e2e-standalone-fixes.md | 07 |
| kickoff-integ-test-multi-tenant.md | 07 |
| kickoff-multi-tenant-routes-app-test.md | 05 |
| kickoff-pre-e2e-fixes.md | 02 |
| kickoff-ui-test-multi-tenant.md | 06, 07 |
| 20260306-middleware-multi-tenant.md | 02 |
| 20260306-multi-tenant-e2e-pg.md | 07 |
| 20260306-services-multi-tenant-isolation-test.md | 05 |
| 20260307-backend-impl.md | 02, 03, 04 |
| 20260307-e2e-test-failure.md | 07 |
| 20260308-frontend-impl.md | 06 |
| 20260308-pre-e2e-fixes.md | 02 |
| 20260308-rs-integration-test.md | 07 |
| 20260309-create-tenant-membership.md | 03, 04 |
| 20260309-mt-arch-refactor.md | 01, 02, 03, 04 |
| 20260309-mt-repo-test-audit.md | 04, 05, 07 |
| 20260309-mt-review.md | (process doc — findings absorbed into refactor, not a spec) |
| routes_app-isolation-test.md | 05 |

## Verification

After all agents complete:
1. All 8 files exist in `consolidated-docs/`
2. Every source doc's functional content is captured in at least one consolidated spec
3. No duplicated content across specs (summarizer's job)
4. Cross-references between specs are consistent
5. Technical sections reference actual file paths that exist in the codebase
6. Decisions in specs match 08-decisions-index.md
