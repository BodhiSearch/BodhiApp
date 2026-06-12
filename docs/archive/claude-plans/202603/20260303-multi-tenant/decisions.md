# Multi-Tenant Architectural Decisions

> **Purpose**: Complete record of all architectural decisions for BodhiApp multi-tenancy.
> Each decision includes the question asked, options considered, chosen approach, and rationale.
> Cross-references research docs where applicable.
>
> **Created**: 2026-03-03

---

## D1: Rename `apps` table to `tenants`

**Question**: The current `apps` table stores a single OAuth client registration. How should it evolve for multi-tenancy?

**Options considered**:
1. Evolve apps table in place (add columns, keep name)
2. Create separate `tenants` table alongside `apps`
3. Rename `apps` → `tenants` with schema evolution

**Decision**: Rename `apps` → `tenants`. Add `id` (ULID) as new PK. `client_id` becomes unique index.

**Rationale**: Domain terminology clarity. "app" refers to the global BodhiApp deployment. "tenant" refers to a scoped resource/customer. Using "app" for both the deployment and tenant identity creates confusion in code, docs, and conversations. Early rename prevents ambiguity spreading through the codebase.

**Note**: Initially chose to keep the name `apps` and use `app_id` as FK. Reversed after realizing the confusion during the planning conversation itself — "app_toolset_configs" with an "app_id" column creates ambiguity about which "app" is meant.

**Migration approach**: Modify existing CREATE TABLE migration in place (no production deployments exist).

---

## D2: ULID for tenant ID format

**Question**: What ID format for the tenants.id PK? Research doc suggested UUID.

**Options**: ULID (codebase convention) vs UUID v4 (industry standard)

**Decision**: ULID

**Rationale**: Every other table in the codebase uses ULID (via `services::ulid_new()`). Consistency reduces cognitive load. ULIDs are sortable and URL-safe. The research doc's UUID recommendation was for a TypeScript stack; BodhiApp's Rust stack has established ULID as convention.

---

## D3: `tenant_id` on EVERY data table

**Question**: Should child tables (e.g., `mcp_oauth_tokens` which FK to `mcp_oauth_configs` which FK to `mcp_servers`) also get `tenant_id`, or can they inherit from parent?

**Options**: Every table vs only root/parent tables

**Decision**: Every table gets `tenant_id` FK.

**Rationale**: Industry standard (Nile, Supabase, Citus all recommend this). Benefits:
- RLS policies per table without JOINs
- Defense-in-depth — corrupt FK can't leak data across tenants
- Simpler query patterns — no need to JOIN through parent to verify tenant
- Independent indexing per table
- Audit trail — every record independently traceable

Tradeoff is storage overhead (one extra column per row) and consistency enforcement (child's tenant_id must match parent's). Storage cost is negligible; consistency enforced by app-layer + FK constraints.

---

## D4: Unified schema for both deployment modes

**Question**: Should standalone (SQLite) and multi-tenant (PostgreSQL) share the same schema?

**Options**: Unified schema vs conditional schema

**Decision**: Unified schema. Standalone has one tenant row with all data referencing that tenant_id.

**Rationale**:
- One set of SeaORM entities, one query pattern, one code path
- Migration from standalone → multi is trivial (just add more tenant rows)
- Tests work identically in both modes
- No conditional query builders or dual entity definitions

---

## D5: `BODHI_DEPLOYMENT` setting values

**Question**: What values and semantics?

**Decision**: `standalone` (default) and `multi`.

**Semantics of `multi`**:
- Multi-tenant capable
- No local LLM inference (llama.cpp routes disabled)
- Stateless app instances (future: Redis cache, PostgreSQL required)
- Multiple tenant rows in DB

---

## D6: `BODHI_MULTITENANT_CLIENT_ID` env var

**Question**: How does the platform/account client work for multi-tenant initial auth?

**Options**: Hardcoded convention vs configurable env var vs derived from existing config

**Decision**: Configurable via `BODHI_MULTITENANT_CLIENT_ID` env var.

**Validation rules**:
- If `BODHI_DEPLOYMENT=standalone`: setting this is an error
- If `BODHI_DEPLOYMENT=multi` and NOT set: error on startup
- Added to `SettingService` trait with proper validation

**Rationale**: Different Keycloak deployments may use different client names. Env var allows flexibility without code changes.

---

## D7: App-layer filtering + PG RLS defense-in-depth

**Question**: How to enforce tenant isolation given SeaORM (Rust) doesn't have built-in RLS support?

**Options**: App-layer only, App-layer + RLS, RLS-first

**Decision**: App-layer primary (auth-scoped services add `tenant_id` filter), PG RLS as defense-in-depth.

**Rationale**:
- Auth-scoped services already scope by `user_id` — extending to `(tenant_id, user_id)` is natural
- App-layer works with both SQLite (standalone) and PostgreSQL (multi)
- RLS requires PG-specific setup (roles, policies, SET LOCAL in transactions)
- SeaORM can execute raw SQL in transactions for SET LOCAL
- Layered approach: even if app-layer has a bug, RLS catches it

**Implementation**:
- Phase 4: App-layer filtering in auth-scoped services
- Phase 6: PG RLS policies as defense-in-depth

---

## D8: `tenant_id` in AuthContext

**Question**: Where should tenant_id resolution (client_id → tenant_id) happen?

**Options**: In AuthContext (middleware), in AuthScopedAppService (extractor), lazily in services

**Decision**: Add `tenant_id: Option<String>` to all AuthContext variants. Middleware resolves during auth.

**Rationale**:
- Single resolution point — tenant_id computed once, reused everywhere
- AuthContext already carries `client_id` — adding `tenant_id` is consistent
- Enables `require_tenant_id()` pattern matching `require_user_id()`
- Cached lookup (tenants table mapping is near-static)

**Flow**: JWT arrives → extract `azp` → lookup `tenants` table → get `tenant_id` ULID → inject into AuthContext

---

## D9: Settings table stays global permanently

**Question**: Should the settings table get tenant_id for per-tenant settings?

**Options**: Add tenant_id now (future-proof), keep global (add later), keep global permanently

**Decision**: Keep global permanently. Create separate `tenant_settings` table if per-tenant settings needed.

**Rationale**:
- ALL current settings are infrastructure/global (see `settings-analysis.md`)
- In multi-tenant mode, LLM-related settings are dead code
- Only 2 editable settings today: BODHI_EXEC_VARIANT, BODHI_KEEP_ALIVE_SECS
- Mixing global and per-tenant in same table adds complexity (nullable tenant_id, composite lookups)
- Dedicated `tenant_settings` table is cleaner when needed — clear separation of concerns

---

## D10: Conditional route registration + feature flag for LLM

**Question**: How to disable LLM features in multi-tenant mode?

**Decision**: Both conditional route registration AND service-level guards.

**Route level**: In `build_routes()`, skip LLM route groups when `is_multi_tenant() == true`.
**Service level**: DataService, HubService, QueueProducer check deployment mode and return 501 Not Implemented.

**Rationale**: Belt and suspenders. Routes not being registered is the primary control. Service-level guards protect against any code path that might bypass route-level gating (e.g., internal service calls, background jobs).

---

## D11: Keep `AppStatus` enum name

**Question**: Should AppStatus be renamed to TenantStatus?

**Decision**: Keep `AppStatus` as-is.

**Rationale**: Widely used across the codebase (routes, middleware, frontend API contract). Renaming creates large blast radius for minimal benefit. The enum values (Setup, Ready, ResourceAdmin) still make semantic sense for tenants. Rename other types based on how widely they're used.

---

## D12: Always store encrypted_client_secret

**Question**: Does every tenant need its own client_secret?

**Decision**: Yes. Every tenant row stores `encrypted_client_secret`.

**Rationale**: Needed for server-side token exchange with Keycloak in both modes. Standalone needs it for its single client. Multi-tenant needs it per tenant for token refresh and exchange operations.

---

## D13: Session-based tenant routing, no slug column

**Question**: Should we add `slug` and `tier` columns to tenants table?

**Decision**: Defer both. Session-based routing (active tenant stored in cookie), not URL-path routing.

**Rationale**: The research doc recommended path-based routing (`app.getbodhi.ai/org/{slug}/dashboard`). After discussion, session-based routing was preferred — user selects a tenant, it's stored in browser cookie, and all subsequent requests use that tenant's auth token. No slug needed for routing. Tier is a business concern for later.

---

## D14: Modify existing CREATE TABLE migrations in place

**Question**: How to handle schema migration for tenant_id addition?

**Decision**: Modify existing migration files directly. No new ALTER TABLE migrations.

**Rationale**: No production deployments exist. Modifying existing CREATE TABLE statements produces a clean schema from day one. No backfill needed. Simplifies migration chain.

---

## D15: External provisioning for multi-tenant tenants

**Question**: How are new tenants created in multi-tenant mode?

**Decision**: External provisioning (scripts, migration, Keycloak admin). No admin API in this plan.

**Rationale**: Focus on making the app multi-tenant ready and ensuring standalone works. Tenant creation flow requirements not yet finalized. Will add API endpoint in a follow-up milestone.

---

## D16: Backend-only scope

**Decision**: This implementation plan covers schema, services, middleware, routes only. Frontend changes tracked in `frontend-tasks.md`.

---

## D17: Defer cache externalization

**Decision**: Keep in-memory MokaCacheService. Redis deferred.

**Rationale**: Multi-tenant doesn't immediately require statelessness if sticky sessions are used (Cloudflare). Redis adds infrastructure complexity. Defer until true horizontal scaling is needed.

---

## D18: Generate tenant_id during setup for standalone

**Question**: When is the default tenant created in standalone mode?

**Options**: During setup, migration seed, first boot

**Decision**: During setup. The existing `setup_create()` flow generates the tenant row with a ULID `id`.

**Rationale**: Clean lifecycle — no data exists before setup completes. Setup already handles OAuth client registration; it now also generates the tenant identity.

---

## D19: Foundation-up phasing strategy

**Decision**: Phase 1 (tenants table + deployment mode) → Phase 2 (AuthContext + tenant_id) → Phase 3 (all tables get tenant_id) → Phase 4 (service scoping) → Phase 5 (feature gating) → Phase 6 (PG RLS).

**Rationale**: Each phase builds on the previous. Foundation (tenants table) must exist before AuthContext can resolve tenant_id. AuthContext must carry tenant_id before services can scope by it. Services must scope before routes can be gated. All app-layer changes must work before RLS is added.

---

## D20: Remove seed logic (toolset_configs seeding)

**Question**: The current `seed_toolset_configs` method creates initial `app_toolset_configs` records. With tenant_id, seeding would need a tenant_id parameter.

**Decision**: Remove seeding logic entirely. It was temporary. Mark any tests that fail from removal as `#[ignore]` for later attention.

**Rationale**: Seeding was a convenience during initial development. With multi-tenancy, seeded data must belong to a tenant. Rather than adding complexity to seeding, remove it. Toolset configs should be created on-demand by tenant admins or during tenant provisioning.
