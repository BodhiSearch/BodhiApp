# Dual-DB E2E Testing -- Design Decisions & Context

This document captures decisions from the planning Q&A for dual-DB E2E testing. It serves as authoritative context for sub-agents executing the plan.

## Background

BodhiApp E2E tests (Playwright) currently run against SQLite only. After the full SeaORM migration completes, we add PostgreSQL E2E testing using Playwright's `--project` feature.

**Prerequisite**: Full SeaORM migration (`.cursor/plans/seaorm_full_migration_6cebce11.plan.md`) iterations 0-9 complete.

---

## BODHI_APP_DB_URL Strategy

**Decision: Full parity with BODHI_SESSION_DB_URL.**

- New constant `BODHI_APP_DB_URL` in `constants.rs`, added to `SETTING_VARS`
- Default: `sqlite:{bodhi_home}/bodhi.sqlite` in `default_service.rs`
- New trait method `app_db_url()` in `SettingService`
- URL-scheme autodetection (postgres:// vs sqlite:) for connection branching
- In bootstrap (`app_service_builder.rs`): read from env var directly (SettingService depends on DbService -- circular dep)

---

## Port Strategy

**Decision: Two shared servers running simultaneously on different ports.**

| DB Backend | Port  |
|------------|-------|
| SQLite     | 51135 |
| PostgreSQL | 41135 |

- Playwright starts ALL webServers globally before any project runs
- Both servers always running; tests select based on project name
- Dedicated servers continue to use random ports (20000-30000 range) but receive DB URLs based on project

---

## Playwright Project Configuration

**Decision: Two projects (`sqlite`, `postgres`), sequential execution.**

- `workers: 1`, `fullyParallel: false` preserved
- Both projects share same test specs
- Projects run sequentially (Playwright default for multiple projects)
- `@scheduled` tag exclusion applies to both projects

---

## Test Scope

**Decision: ALL tests run in both projects (shared + dedicated server tests).**

- No selective tagging or partial project coverage
- Maximum coverage -- every user journey validated against both DBs
- This doubles E2E execution time (acceptable tradeoff for correctness)

---

## Docker Postgres Instances

**Decision: Reuse existing instances from `docker-compose-test-deps.yml`.**

| Service           | Port  | Database        |
|-------------------|-------|-----------------|
| bodhi_session_db  | 54320 | bodhi_sessions  |
| bodhi_app_db      | 64320 | bodhi_app       |

- Same instances used by backend rstest dual-DB tests
- Acceptable: backend and E2E tests unlikely to run concurrently
- Credentials: `bodhi_test:bodhi_test`

---

## Environment Variable Flow

**Decision: .env.test has E2E-specific PG URLs. Playwright project config maps them.**

```
# In .env.test
E2E_PG_APP_DB_URL=postgres://bodhi_test:bodhi_test@localhost:64320/bodhi_app
E2E_PG_SESSION_DB_URL=postgres://bodhi_test:bodhi_test@localhost:54320/bodhi_sessions
```

Flow:
1. `playwright.config.mjs` loads `.env.test` via dotenv
2. Postgres webServer entry reads `process.env.E2E_PG_APP_DB_URL` and maps to `BODHI_APP_DB_URL`
3. `start-shared-server.mjs` passes `BODHI_APP_DB_URL`/`BODHI_SESSION_DB_URL` from `process.env` to NAPI server
4. SQLite webServer has no DB env vars (uses built-in defaults)

For dedicated servers: test reads `testInfo.project.name` → calls `getDbConfigForProject()` → passes DB URLs to `createServerManager()`.

---

## Shared Server URL

**Decision: Fixture-provided value (not utility function import).**

- Add `sharedServerUrl` fixture to `fixtures.mjs`
- Resolves based on `testInfo.project.name` → port mapping
- All 20 spec files migrate from `import { SHARED_SERVER_URL }` to destructuring `{ sharedServerUrl }` from test args
- Consistent with project convention for test-context-aware values

---

## Database Reset Strategy

**Decision: Project-aware fixture reset. TRUNCATE CASCADE on Postgres (all tables EXCEPT apps).**

- `autoResetDb` fixture reads `testInfo.project.name` to determine which server to reset
- Resets only the matching server (not both)
- `reset_all_tables()` on `DefaultDbService`:
  - Postgres: `TRUNCATE TABLE <tables> CASCADE` (excludes `apps`)
  - SQLite: `DELETE FROM <table>` for each table (excludes `apps`)
- After truncation: re-seeds toolset configs

---

## Apps Table Handling

**Decision: Test fixtures re-seed apps record (not server-side auto-reseed).**

- `apps` table excluded from `reset_all_tables()` truncation
- If `TRUNCATE CASCADE` reaches `apps` via FK relationships → test fixtures call API to re-create app instance
- More explicit than server-side re-seed; fixtures control what app state tests expect

---

## Dedicated Server DB Config Propagation

**Decision: Tests read project name from Playwright context, pick right env vars from .env.test.**

- In `test.beforeAll`: use `test.info().project.name` to get project
- Call `getDbConfigForProject(projectName)` to get DB URLs
- Pass as `envVars` in `createServerManager()` config
- `createServerManager()` / `bodhi-app-server.mjs` propagates `envVars` to NAPI `createTestServer()`

---

## Shared Utility

**Decision: `tests-js/utils/db-config.mjs` provides `getDbConfigForProject()` and `getSharedServerUrl()`.**

- Used by: `fixtures.mjs`, `start-shared-server.mjs` (indirectly via env vars), dedicated server specs
- Maps project name → `{ appDbUrl, sessionDbUrl, sharedServerPort }`
- Single source of truth for port numbers and env var names

---

## Session DB

**Decision: Session DB stays on sqlx (out of scope for SeaORM migration).**

- `BODHI_SESSION_DB_URL` already supports both SQLite and Postgres
- Session service uses `is_postgres_url()` to branch between `connect_sqlite()` and `connect_postgres()`
- E2E tests pass `E2E_PG_SESSION_DB_URL` for the postgres project
- No additional work needed for session DB dual-DB support

---

## Migration Ordering

This work executes AFTER the full SeaORM migration. Specifically:
1. SeaORM migration iterations 0-9 complete (all tables on SeaORM, DefaultDbService only)
2. This plan: add `BODHI_APP_DB_URL` + E2E infrastructure
3. Future: migrate session service from sqlx to SeaORM (separate initiative)
