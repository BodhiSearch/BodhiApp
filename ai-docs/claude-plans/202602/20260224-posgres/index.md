---
name: Session PG Migration Plan
overview: Create a layered, crate-by-crate functional specification for re-implementing session PostgreSQL migration on the main branch, based on work done in the multi-tenant worktree. Includes auth refactor archive, and 6 phase files following crate dependency order.
todos:
  - id: readme
    content: Write README.md index for ai-docs/claude-plans/20260224-postgres/
    status: completed
  - id: auth-refactors
    content: Write auth-refactors.md archive document (ExternalApp role optional + ResourceScope move)
    status: completed
  - id: phase-01-services
    content: "Write 01-services.md: session store abstraction, PG backend, config, rename, docker-compose, tests"
    status: completed
  - id: phase-02-lib-bodhiserver
    content: "Write 02-lib-bodhiserver.md: AppServiceBuilder session backend selection"
    status: completed
  - id: phase-03-auth-middleware
    content: "Write 03-auth-middleware.md: import updates for DefaultSessionService"
    status: completed
  - id: phase-04-routes-app
    content: "Write 04-routes-app.md: import updates + test utility changes"
    status: completed
  - id: phase-05-server-app
    content: "Write 05-server-app.md: live server test utility updates"
    status: completed
  - id: phase-06-lib-bodhiserver-napi
    content: "Write 06-lib-bodhiserver-napi.md: NAPI config export"
    status: completed
isProject: false
---

# Session PostgreSQL Migration -- Re-implementation Plan

## Context

The `multi-tenant` worktree implemented session PostgreSQL migration (Phase 1 of the multi-tenant plan) but main has diverged too far to merge. This plan reverse-engineers the functional requirements from the worktree's committed and uncommitted changes, creating an exploratory (not prescriptive) specification for re-implementation on main.

Key references from the worktree:

- [20260214-multi-tenant/README.md](ai-docs/claude-plans/20260214-multi-tenant/README.md) -- overall multi-tenant vision
- [20260214-multi-tenant/phase-1-session-pg.md](ai-docs/claude-plans/20260214-multi-tenant/phase-1-session-pg.md) -- original Phase 1 spec
- [20260210-access-request/20260217-scope-role-optional.md](ai-docs/claude-plans/20260210-access-request/20260217-scope-role-optional.md) -- auth refactor (completed in worktree)
- Agent transcript: [Session PG Migration](a612e30c-83e0-46c1-a23f-176198546aae)

## What Was Implemented in the Worktree

### Committed (19 commits)

1. E2E test infrastructure cleanup (shared Playwright webServer, OAuth test app rewrite, ExternalTokenSimulator)
2. Auth middleware consolidation (header-based extractors removed, authentication context consolidated)
3. **ExternalApp role made optional** + **ResourceScope moved from objs to auth_middleware** (2 commits)
4. AppRegInfo-to-org-table planning (plan only, not implemented)

### Uncommitted (git diff)

1. **Session service folder module restructure**: `session_service.rs` deleted, replaced with `session_service/` folder (mod.rs, error.rs, sqlite.rs, postgres.rs, service.rs, tests.rs)
2. **SessionStoreBackend enum**: `SqliteSessionStore` + `PgSessionStore` with `DefaultSessionService` wrapper
3. **BODHI_SESSION_DB_URL**: Added to `SettingService` with `session_db_url()` method
4. **AppServiceBuilder**: Updated to select session backend based on URL scheme
5. **Rename**: `SqliteSessionService` -> `DefaultSessionService` across ~15 files
6. **Dependencies**: `postgres` + `any` features added to sqlx and tower-sessions-sqlx-store
7. **Docker compose**: Test PostgreSQL instance
8. **NAPI config**: `BODHI_SESSION_DB_URL` exported
9. **Playwright**: `pg-chromium` project + `start-pg-server.mjs`
10. **Makefile**: `test.deps.up`, `test.deps.down`, `test.session.all`, `test.e2e.pg` targets

## Plan Structure

Output to `ai-docs/claude-plans/20260224-postgres/`:

- `README.md` -- index and overview (this plan becomes its basis)
- `auth-refactors.md` -- archive of auth refactors (nice-to-have, not in implementation flow)
- `01-services.md` -- services crate: session store abstraction + PG backend + config
- `02-lib-bodhiserver.md` -- lib_bodhiserver crate: AppServiceBuilder changes
- `03-auth-middleware.md` -- auth_middleware crate: import updates
- `04-routes-app.md` -- routes_app crate: import updates + test utility changes
- `05-server-app.md` -- server_app crate: live server test utility updates
- `06-lib-bodhiserver-napi.md` -- lib_bodhiserver_napi: NAPI config export + docker-compose

## Key Functional Requirements

### Deployment Mode

- `BODHI_DEPLOYMENT=standalone` (default): SQLite session backend, existing behavior preserved
- `BODHI_DEPLOYMENT=cluster`: requires `BODHI_SESSION_DB_URL` pointing to PostgreSQL
- Both backends always compiled -- runtime selection, no Cargo feature flags

### Session Store

- `DefaultSessionService` supports both SQLite and PostgreSQL backends
- Suggested module structure: folder with separate files per backend (implementer may adjust)
- Separate URLs: `BODHI_SESSION_DB_URL` for sessions, `BODHI_DATABASE_URL` reserved for future app DB

### Testing

- Suggested: rstest `#[values("sqlite","postgres")]` parameterized tests (implementer may adjust)
- `docker-compose-test-deps.yml` for PostgreSQL test instance
- Makefile targets for test dependency lifecycle

### Crate Dependency Order

```
services -> lib_bodhiserver -> auth_middleware -> routes_app -> server_app -> lib_bodhiserver_napi
```

### Validation at Each Phase

- Per-crate: `cargo test -p <crate>`
- Cumulative: `cargo test -p <all-changed-crates-so-far>`
- Full backend: `make test.backend`
- Format: `cargo fmt`

