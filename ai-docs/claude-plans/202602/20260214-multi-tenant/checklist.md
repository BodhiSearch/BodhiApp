# Multi-Tenancy Implementation Checklist

## Phase 1: Session PostgreSQL Migration
- [ ] Add `postgres`, `any` features to sqlx in workspace Cargo.toml
- [ ] Add `postgres` feature to tower-sessions-sqlx-store
- [ ] Create `SessionStoreBackend` enum (Sqlite/Postgres)
- [ ] Refactor `AppSessionStore` to use enum dispatch
- [ ] Add `new_sqlite()` and `new_postgres()` constructors
- [ ] Implement `SessionStore` trait delegation for both backends
- [ ] Add `SESSION_DB_URL` to SettingService
- [ ] Update `AppServiceBuilder` to select session backend based on URL
- [ ] Rename `SqliteSessionService` → `DefaultSessionService`
- [ ] Handle `user_id` column migration for both backends
- [ ] Verify existing session tests pass
- [ ] Add session backend selection tests

## Phase 2: DB Abstraction (sqlx::Any)
- [ ] Install sqlx::Any default drivers at startup
- [ ] Rename `SqliteDbService` → `DbServiceImpl`
- [ ] Change pool type from `SqlitePool` → `AnyPool`
- [ ] Update constructor to accept database URL
- [ ] Migrate SQL queries to sqlx::Any compatible syntax
- [ ] Handle dialect differences (UPSERT, COLLATE, types)
- [ ] Add `is_postgres()` helper method
- [ ] Create PostgreSQL migration directory (`migrations_pg/`)
- [ ] Create PostgreSQL schema (all tables)
- [ ] Implement runtime migration selection
- [ ] Add `DATABASE_URL` to SettingService
- [ ] Update `AppServiceBuilder` for configurable DB URL
- [ ] Update `TestDbService` to use `DbServiceImpl` with AnyPool
- [ ] Update NAPI bindings for AnyPool
- [ ] Verify all existing tests pass
- [ ] Verify `cargo check` for all crates

## Phase 3: Org Threading
### 3a: Organizations table
- [ ] Create `Organization` domain struct
- [ ] Create `OrgContext` struct
- [ ] Create `OrgStatus` enum
- [ ] Create `OrgRepository` trait
- [ ] Add `OrgRepository` to `DbService` super-trait
- [ ] Write SQLite migration: `0010_organizations.{up,down}.sql`
- [ ] Write PostgreSQL migration for organizations
- [ ] Implement `OrgRepository` in `DbServiceImpl`

### 3b: org_id in all tables
- [ ] Write SQLite migration: `0011_add_org_id.{up,down}.sql`
- [ ] Write PostgreSQL migration for org_id columns
- [ ] Add org_id indexes
- [ ] Update unique constraints (org-scoped)

### 3c: Repository trait signatures
- [ ] Add `org_id: &str` to `TokenRepository` methods
- [ ] Add `org_id: &str` to `AccessRepository` methods
- [ ] Add `org_id: &str` to `AccessRequestRepository` methods
- [ ] Add `org_id: &str` to `ModelRepository` methods
- [ ] Add `org_id: &str` to `ToolsetRepository` methods
- [ ] Add `org_id: &str` to `UserAliasRepository` methods

### 3d: DbServiceImpl query updates
- [ ] Update all INSERT queries with org_id
- [ ] Update all SELECT queries with org_id WHERE clause
- [ ] Update all UPDATE queries with org_id WHERE clause
- [ ] Update all DELETE queries with org_id WHERE clause

### 3e: Middleware
- [ ] Create `org_resolution_middleware`
- [ ] Create `ExtractOrgId` extractor
- [ ] Create `ExtractOrgContext` extractor
- [ ] Add org_resolution_middleware to route composition
- [ ] Update `auth_middleware` to use `OrgContext` instead of `SecretService`
- [ ] Update `token_service` to use `OrgContext`
- [ ] Handle single-tenant org resolution (DB lookup, single row)

### 3f: Remove SecretService
- [ ] Remove `SecretService` trait
- [ ] Remove `DefaultSecretService` implementation
- [ ] Remove `SecretServiceExt` trait
- [ ] Remove from `AppService` trait
- [ ] Remove from `DefaultAppService`
- [ ] Remove from `AppServiceBuilder`
- [ ] Remove keyring dependencies
- [ ] Remove AES-GCM/PBKDF2 deps (if not used elsewhere)
- [ ] Update all consumers to use OrgContext

### 3g: Auth flow updates
- [ ] Update `auth_initiate_handler` to use OrgContext client_id
- [ ] Update `auth_callback_handler` to use OrgContext credentials
- [ ] Remove ResourceAdmin first-login flow (or adapt for org setup)
- [ ] Update `AppStatus` handling (merge into OrgStatus or remove)

### 3h: Service layer updates
- [ ] Update `ToolService` with org_id parameters
- [ ] Update `AiApiService` with org_id parameters
- [ ] Update `AccessRequestService` with org_id parameters
- [ ] Refactor `DataService` for hosted mode (UnsupportedOperation)

### 3i: Route handler updates
- [ ] Add `ExtractOrgId` to all org-scoped handlers
- [ ] Pass org_id to service calls
- [ ] Add `GET /api/orgs/current` endpoint
- [ ] Add `GET /api/orgs/user-memberships` endpoint

### 3j: Tests
- [ ] Update all test call sites with org_id parameter
- [ ] Update TestDbService with org_id support
- [ ] Update MockDbService mock definitions
- [ ] Add org isolation tests (data not visible cross-org)
- [ ] Add middleware tests (org resolution, suspension)
- [ ] Add auth flow tests with OrgContext
- [ ] Verify all `cargo test` pass

## Phase 4: CacheService + Redis
- [ ] Define new generic `CacheService` trait (typed get/set/invalidate)
- [ ] Define `CacheError` type
- [ ] Refactor `MokaCacheService` to new generic trait
- [ ] Add Redis dependency to workspace Cargo.toml
- [ ] Implement `RedisCacheService`
- [ ] Implement `RedisConcurrencyService` (distributed locks)
- [ ] Add `REDIS_URL` to SettingService
- [ ] Update `AppServiceBuilder` cache/concurrency selection
- [ ] Integrate org config caching in org_resolution_middleware
- [ ] Migrate existing token cache to new interface
- [ ] Add cache invalidation via Redis Pub/Sub
- [ ] Update MockCacheService for new trait
- [ ] Verify all tests pass

## Phase 5: Docker & Deployment
- [ ] Add `libpq5` to runtime stage of all Dockerfiles
- [ ] Create `docker-compose.multi-tenant.yml`
- [ ] Configure Traefik wildcard subdomain routing
- [ ] Implement app-level org extraction from Host header
- [ ] Add `BODHI_ORG_DOMAIN` env var
- [ ] Update health endpoint for multi-backend checks
- [ ] Create `.env.hosted.template`
- [ ] Add Makefile targets for hosted Docker
- [ ] Test docker-compose stack locally
- [ ] Verify horizontal scaling (3 replicas)

## Phase 6: Frontend
- [ ] Create `useCurrentOrg` hook
- [ ] Create `useUserOrgs` hook
- [ ] Create `OrgContext` provider
- [ ] Create `OrgSwitcher` component
- [ ] Integrate OrgSwitcher in layout
- [ ] Create `useAppMode` hook
- [ ] Add `GET /bodhi/v1/app/info` endpoint
- [ ] Conditionally hide local LLM features in hosted mode
- [ ] Add org error handling (not found, suspended)
- [ ] Write OrgSwitcher component tests
- [ ] Write mode detection tests
- [ ] `make build.ui` succeeds
- [ ] `npm test` passes
