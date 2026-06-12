## Exploration: Migrating BodhiApp Data Layer from sqlx to Diesel ORM

### Objective

Explore the feasibility, benefits, risks, and effort of migrating BodhiApp's data access layer from raw sqlx queries to Diesel ORM. This is an exploratory analysis — research, web search, and investigate independently to form a recommendation.

### Current State

BodhiApp uses sqlx with raw SQL queries for its data layer:
- **9 repository traits** composing a `DbService` super-trait: ModelRepository, AccessRepository, AccessRequestRepository, AppInstanceRepository, TokenRepository, ToolsetRepository, McpRepository, UserAliasRepository, SettingsRepository + DbCore
- **~81 sqlx::query/query_as/query_scalar calls** across 10 repository impl files in `crates/services/src/db/`
- **14 SQLite migration files** in `crates/services/migrations/`
- **SqlitePool** used directly in `SqliteDbService`
- **5 test files** (`test_access_repository.rs`, `test_access_request_repository.rs`, `test_mcp_repository.rs`, `test_model_repository.rs`, `test_token_repository.rs`) with real-SQLite tests
- **TestDbService** wraps SqliteDbService with event broadcasting + FrozenTimeService
- **MockDbService** uses mockall for unit testing across downstream crates (routes_app, auth_middleware, etc.)
- The codebase is fully async (tokio runtime, axum web framework)
- The project has NO production release yet — zero backwards compatibility or data migration concerns

### Target Architecture Vision

1. **Dual-database support**: SQLite for standalone/desktop deployment, PostgreSQL for cluster/multi-tenant deployment. Runtime selection based on `BODHI_DATABASE_URL` environment variable.

2. **Multi-tenant with org_id**: Future plans to add `org_id` column to relevant domain tables for multi-tenancy. Each organization's data isolated via row-level security (RLS) in PostgreSQL. SQLite deployments are single-tenant, no RLS needed.

3. **PostgreSQL RLS**: For cluster deployments, PostgreSQL RLS policies enforce complete data isolation per organization at the database level. This is a security-critical requirement for the multi-tenant architecture.

4. **SQLite lightweight**: Standalone deployments should remain lightweight — no RLS overhead, simple single-file database, minimal configuration.

### Diesel-Specific Investigation Areas

Research these areas thoroughly (web search, check latest docs, check community feedback):

1. **Diesel MultiConnection** (`#[derive(MultiConnection)]`):
   - How does it handle SQLite + PostgreSQL in the same codebase?
   - What SQL types are supported? What needs custom FromSql/ToSql impls?
   - How does it interact with async? (investigate `diesel_async` crate maturity, community adoption, issues)
   - Can we pattern-match on the connection variant for backend-specific operations (like RLS setup)?

2. **Migration story**:
   - How does Diesel handle migrations for multiple database backends?
   - Can the same migration directory serve both SQLite and PG, or do we need separate directories?
   - barrel was removed in Diesel 2.0 — what replaced it? What's the current recommended approach for cross-DB DDL generation?
   - How do we handle PG-specific DDL (like RLS policies, CITEXT, SERIAL) alongside SQLite-specific DDL (AUTOINCREMENT, COLLATE NOCASE)?

3. **Schema generation**:
   - How does `diesel print-schema` work with multiple backends?
   - Can a single `schema.rs` serve both SQLite and PG?
   - How do we handle the type differences (e.g., SQLite uses INTEGER for booleans, PG has native BOOLEAN)?

4. **Query DSL vs raw SQL**:
   - For our 9 repository traits with ~81 queries, what does the Diesel DSL look like?
   - Can we still use raw SQL for complex queries that don't map well to DSL?
   - How do ON CONFLICT/UPSERT, JSON columns, case-insensitive comparisons, pagination work in Diesel DSL across both backends?

5. **Testing story**:
   - How does Diesel testing work with multiple backends?
   - Can we parameterize tests to run against both SQLite and PG?
   - What happens to our mockall-based MockDbService? Does Diesel have its own mocking approach?
   - How does diesel_async interact with tokio::test?

6. **Community health and maturity**:
   - diesel_async: GitHub stars, last update, open issues, community size
   - Diesel 2.x stability, release cadence, breaking change history
   - Real-world projects using Diesel MultiConnection with SQLite+PG
   - Known pain points, gotchas, limitations reported by the community
   - Compare with staying on sqlx (which approach has more community adoption for multi-DB?)

### Codebase References

Read these files to understand the current data layer:
- `crates/services/src/db/service.rs` — DbService trait, SqliteDbService struct, DbCore impl
- `crates/services/src/db/mod.rs` — module structure and exports
- `crates/services/src/db/error.rs` — DbError enum
- `crates/services/src/db/sqlite_pool.rs` — DbPool::connect
- `crates/services/src/db/service_model.rs` — largest repository impl (~750 lines, 14 queries)
- `crates/services/src/db/service_mcp.rs` — complex repository with JOIN queries (~900 lines, 34 queries)
- `crates/services/src/test_utils/db.rs` — TestDbService, FrozenTimeService, MockDbService
- `crates/services/migrations/` — all 14 migration files
- `crates/services/CLAUDE.md` — crate architecture docs
- `crates/services/PACKAGE.md` — implementation details
- `CLAUDE.md` (project root) — overall architecture, crate dependency chain

Also read the session PoC plans for context on the PostgreSQL migration approach that was already proven:
- `ai-docs/claude-plans/20260224-posgres/20260224-session-to-pg.plan.md` — the session migration plan
- `ai-docs/claude-plans/20260224-posgres/reviews/services-review.md` — review findings
- `ai-docs/claude-plans/20260224-posgres/TECHDEBT.md` — deferred items

### Deliverable

Produce a detailed analysis document covering:
1. **Feasibility assessment** — Can Diesel serve BodhiApp's dual-DB + multi-tenant + RLS requirements?
2. **Effort estimate** — What's the migration effort broken into phases?
3. **Risk analysis** — What are the technical risks, unknowns, and potential blockers?
4. **Comparison matrix** — Diesel ORM vs staying with sqlx (custom Migrator + AnyPool) across: migration management, query portability, multi-tenant/RLS support, async story, testing, community health, long-term maintainability
5. **Recommendation** — Go/no-go with clear reasoning
6. **If go: phased migration plan** — How to incrementally migrate from sqlx to Diesel without a big-bang rewrite

Output this to `ai-docs/claude-plans/20260224-posgres/diesel-exploration.md`