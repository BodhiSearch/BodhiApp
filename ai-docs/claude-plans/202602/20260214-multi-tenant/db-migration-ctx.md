# Database Migration Context

## Current State

### SQLx Configuration
- **Version**: sqlx 0.8.6
- **Features**: `["chrono", "runtime-tokio", "sqlite"]`
- **No compile-time checks** - all queries are runtime strings via `sqlx::query()` / `sqlx::query_as()`
- **Migration**: `sqlx::migrate!()` macro pointing to `crates/services/migrations/`

### Current Tables (9 migrations, 0001-0009)

| Table | Has user_id | Needs org_id | Notes |
|-------|-------------|--------------|-------|
| download_requests | NO | YES | Model download tracking |
| access_requests | YES (requester) | YES | Legacy user onboarding |
| api_tokens | YES | YES | Per-user auth tokens |
| api_model_aliases | NO | YES | Remote API configs, UNIQUE(prefix) |
| model_metadata | NO | YES | GGUF + API model metadata |
| app_toolset_configs | NO | YES | Global toolset enable/disable, UNIQUE(toolset_type) |
| toolsets | YES | YES | Per-user toolset instances, UNIQUE(user_id, slug) |
| user_aliases | NO (global) | YES | UNIQUE(alias) - needs UNIQUE(org_id, alias) |
| app_access_requests | YES | YES | OAuth app consent flow |
| tower_sessions | YES (custom col) | YES | HTTP session storage |

### Unique Constraints That Change
```sql
-- Current → Multi-tenant
api_model_aliases.prefix UNIQUE → UNIQUE(org_id, prefix)
app_toolset_configs.toolset_type UNIQUE → UNIQUE(org_id, toolset_type)
user_aliases.alias UNIQUE → UNIQUE(org_id, alias)
toolsets UNIQUE(user_id, slug) → UNIQUE(org_id, user_id, slug)
```

### New Table: organizations
```sql
CREATE TABLE organizations (
  id TEXT PRIMARY KEY,           -- UUID
  slug TEXT NOT NULL UNIQUE,     -- URL-safe org identifier (subdomain)
  display_name TEXT NOT NULL,
  kc_client_id TEXT NOT NULL,    -- Keycloak client ID
  client_secret TEXT NOT NULL,   -- Encrypted OAuth client secret
  encryption_key TEXT NOT NULL,  -- Per-org encryption key for secrets
  status TEXT NOT NULL CHECK (status IN ('active', 'suspended', 'deleted')),
  created_at TEXT NOT NULL,      -- ISO 8601
  updated_at TEXT NOT NULL       -- ISO 8601
);
```

This table replaces AppRegInfo (currently in encrypted secrets.yaml file).

---

## sqlx::Any Migration Plan

### Current: SqlitePool
```rust
// crates/services/src/db/service.rs
pub struct SqliteDbService {
  pool: SqlitePool,
  time_service: Arc<dyn TimeService>,
  encryption_key: Vec<u8>,
}
```

### Target: AnyPool
```rust
pub struct DbServiceImpl {
  pool: AnyPool,               // sqlx::Any - works with SQLite and PostgreSQL
  time_service: Arc<dyn TimeService>,
  encryption_key: Vec<u8>,     // Platform-level key (per-org keys from organizations table)
}
```

### Key Changes for sqlx::Any
1. **Cargo.toml features**: Add `"any"`, `"postgres"` to sqlx features
2. **Connection URL**: `sqlite:path` or `postgres://user:pass@host/db`
3. **Parameter binding**: Both use `$1, $2` style with `sqlx::query()` runtime API
4. **Type mapping differences**:
   - Boolean: SQLite=INTEGER(0/1), PostgreSQL=BOOLEAN → use INTEGER for compat
   - JSON: SQLite=TEXT, PostgreSQL=TEXT (not JSONB for compat)
   - Timestamps: Both support TEXT ISO 8601 format
5. **Auto-increment**: SQLite=INTEGER PRIMARY KEY, PostgreSQL=SERIAL → use TEXT UUIDs (already the pattern)
6. **UPSERT**: SQLite=`INSERT OR REPLACE`, PostgreSQL=`INSERT...ON CONFLICT` → need dialect check

### Migration File Strategy
- **SQLite migrations**: Continue in `crates/services/migrations/` (existing path)
- **PostgreSQL migrations**: New directory `crates/services/migrations_pg/`
- **Organizations table**: In both migration directories
- **org_id column**: Added to all existing tables in new migration files

### PostgreSQL-Specific Schema Additions
```sql
-- Row-Level Security (defense-in-depth)
ALTER TABLE toolsets ENABLE ROW LEVEL SECURITY;
CREATE POLICY toolsets_org_isolation ON toolsets
  USING (org_id = current_setting('app.current_org_id'));

-- Similar RLS for all org-scoped tables
```

---

## Session Database Migration

### Current
- tower-sessions 0.14.0 with tower-sessions-sqlx-store 0.15.0 (SQLite feature)
- Separate database file: `{BODHI_HOME}/session.db`
- Custom `user_id` column added via AppSessionStore.migrate()
- Cookie: `bodhiapp_session_id`, SameSite::Strict

### Target
- tower-sessions-sqlx-store with PostgreSQL feature
- **Separate PgPool** for sessions (configurable `SESSION_DB_URL`)
- Add `org_id` column to tower_sessions table
- Session cookie scoped per subdomain (natural browser behavior)

### Session Store Changes
```rust
// Current
pub struct AppSessionStore {
  inner: SqliteStore,
  pool: Pool<Sqlite>,
}

// Target (multi-tenant)
pub struct AppSessionStore {
  inner: PostgresStore,          // or SqliteStore based on config
  pool: AnyPool,                 // Shared pool reference
}
```

### Configuration
```env
# Multi-tenant PostgreSQL sessions
SESSION_DB_URL=postgres://user:pass@session-host:5432/sessions
SESSION_DB_SCHEMA=sessions

# Single-tenant SQLite sessions (default)
# Uses BODHI_HOME/session.db
```

---

## Repository Trait Changes

### org_id Parameter Addition
Every org-scoped repository method gets `org_id: &str` as first parameter:

```rust
// Before
async fn create_api_token(&self, token: &ApiToken) -> Result<(), DbError>;
async fn list_api_tokens(&self, user_id: &str) -> Result<Vec<ApiToken>, DbError>;

// After
async fn create_api_token(&self, org_id: &str, token: &ApiToken) -> Result<(), DbError>;
async fn list_api_tokens(&self, org_id: &str, user_id: &str) -> Result<Vec<ApiToken>, DbError>;
```

### Affected Repository Traits
- **AccessRepository**: All methods get org_id
- **AccessRequestRepository**: All methods get org_id
- **TokenRepository**: All methods get org_id
- **ModelRepository**: All methods get org_id
- **ToolsetRepository**: All methods get org_id
- **UserAliasRepository**: All methods get org_id
- **DbCore**: `migrate()` and `encryption_key()` stay org-agnostic

### New Repository: OrgRepository
```rust
#[async_trait]
pub trait OrgRepository: Send + Sync {
  async fn get_org_by_slug(&self, slug: &str) -> Result<Option<Organization>, DbError>;
  async fn get_org_by_id(&self, id: &str) -> Result<Option<Organization>, DbError>;
  async fn list_orgs(&self) -> Result<Vec<Organization>, DbError>;
  async fn create_org(&self, org: &Organization) -> Result<(), DbError>;
  async fn update_org_status(&self, id: &str, status: &str) -> Result<(), DbError>;
}
```

### DbService Super-Trait Update
```rust
pub trait DbService:
  ModelRepository
  + AccessRepository
  + AccessRequestRepository
  + TokenRepository
  + ToolsetRepository
  + UserAliasRepository
  + OrgRepository          // NEW
  + DbCore
  + Send + Sync + Debug
{}
```
