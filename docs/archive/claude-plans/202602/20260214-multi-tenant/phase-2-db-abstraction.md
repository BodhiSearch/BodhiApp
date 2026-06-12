# Phase 2: Database Abstraction (sqlx::Any)

## Goal
Migrate from `SqlitePool`-only to `sqlx::Any` (`AnyPool`) so the same DbService implementation works with both SQLite and PostgreSQL.

## Prerequisites
- Phase 1 complete (sqlx postgres + any features already added to Cargo.toml)

---

## Step 1: Enable sqlx::Any Runtime Installation

### Main Binary Startup
```rust
// In main.rs or serve.rs, before any database connection:
sqlx::any::install_default_drivers();
```

This registers SQLite and PostgreSQL drivers for the Any pool.

---

## Step 2: Rename SqliteDbService → DbServiceImpl

### Change struct and pool type
```rust
// crates/services/src/db/service.rs

// Before
pub struct SqliteDbService {
  pool: SqlitePool,
  time_service: Arc<dyn TimeService>,
  encryption_key: Vec<u8>,
}

// After
pub struct DbServiceImpl {
  pool: AnyPool,
  time_service: Arc<dyn TimeService>,
  encryption_key: Vec<u8>,
}
```

### Update constructor
```rust
impl DbServiceImpl {
  pub async fn new(database_url: &str, time_service: Arc<dyn TimeService>, encryption_key: Vec<u8>) -> Result<Self> {
    let pool = AnyPool::connect(database_url).await?;
    Ok(Self { pool, time_service, encryption_key })
  }

  // Or accept pre-connected pool
  pub fn with_pool(pool: AnyPool, time_service: Arc<dyn TimeService>, encryption_key: Vec<u8>) -> Self {
    Self { pool, time_service, encryption_key }
  }
}
```

---

## Step 3: Migrate SQL Queries to sqlx::Any Compatible

### Key Differences Between SQLite and PostgreSQL

| Feature | SQLite | PostgreSQL | Strategy |
|---------|--------|------------|----------|
| Parameter binding | `?1, ?2` (positional) | `$1, $2` | sqlx::Any handles this automatically |
| Boolean | INTEGER (0/1) | BOOLEAN | Use INTEGER, PG casts automatically |
| JSON | TEXT | TEXT (not JSONB) | Keep as TEXT for compatibility |
| Auto-increment | INTEGER PRIMARY KEY | SERIAL | Already using TEXT UUIDs |
| UPSERT | INSERT OR REPLACE | INSERT...ON CONFLICT | Need dialect-specific variants |
| String concat | `\|\|` | `\|\|` | Same |
| LIKE | Case-insensitive by default | Case-sensitive | Use ILIKE for PG or LOWER() |
| NOW() | Not available | NOW() | Already using TimeService |

### Query Migration Approach

Most queries work as-is with `sqlx::query()` on AnyPool. The runtime driver handles parameter binding differences.

**Queries needing attention**:

1. **UPSERT patterns**:
```rust
// Before (SQLite-specific)
sqlx::query("INSERT OR REPLACE INTO ...")

// After (compatible)
// Option A: Delete + Insert (both DBs)
// Option B: Runtime dialect check
if self.is_postgres() {
  sqlx::query("INSERT INTO ... ON CONFLICT (key) DO UPDATE SET ...")
} else {
  sqlx::query("INSERT OR REPLACE INTO ...")
}
```

2. **COLLATE NOCASE** (SQLite-specific):
```sql
-- Before
UNIQUE(user_id, slug COLLATE NOCASE)

-- After (PostgreSQL uses citext or LOWER())
-- Handle in migration: use LOWER(slug) for both
UNIQUE(user_id, LOWER(slug))
```

3. **Type conversions**:
```rust
// sqlx::Any returns AnyRow, need to handle type differences
// Use sqlx::FromRow derive or manual row mapping
let row: AnyRow = sqlx::query("SELECT ...").fetch_one(&self.pool).await?;
let id: String = row.get("id");       // Works for both
let status: String = row.get("status"); // Works for both
let enabled: bool = row.get("enabled"); // SQLite returns i32, PG returns bool
// May need: let enabled: i32 = row.get("enabled"); enabled != 0
```

### Helper Method for Dialect Detection
```rust
impl DbServiceImpl {
  fn is_postgres(&self) -> bool {
    // Check pool backend type
    // sqlx::Any provides this via the connection URL or pool metadata
    self.pool.connect_options().kind() == AnyKind::Postgres
  }
}
```

---

## Step 4: Migrate Each Repository Trait Implementation

### Pattern for each method:
```rust
// Before
#[async_trait]
impl TokenRepository for SqliteDbService {
  async fn create_api_token(&self, token: &ApiToken) -> Result<(), DbError> {
    sqlx::query(
      "INSERT INTO api_tokens (id, user_id, token_prefix, ...) VALUES (?, ?, ?, ...)"
    )
    .bind(&token.id)
    .bind(&token.user_id)
    .execute(&self.pool)
    .await?;
    Ok(())
  }
}

// After
#[async_trait]
impl TokenRepository for DbServiceImpl {
  async fn create_api_token(&self, token: &ApiToken) -> Result<(), DbError> {
    sqlx::query(
      "INSERT INTO api_tokens (id, user_id, token_prefix, ...) VALUES ($1, $2, $3, ...)"
    )
    .bind(&token.id)
    .bind(&token.user_id)
    .execute(&self.pool)
    .await?;
    Ok(())
  }
}
```

Note: sqlx::Any with runtime queries uses `$1, $2` syntax for both SQLite and PostgreSQL.

### Migration Order (by complexity, simplest first):
1. **DbCore** - migrate(), now(), encryption_key() (2-3 methods)
2. **TokenRepository** - Simple CRUD (5 methods)
3. **AccessRepository** - Simple CRUD (6 methods)
4. **UserAliasRepository** - CRUD with unique constraints (6 methods)
5. **ToolsetRepository** - CRUD + app configs (10+ methods)
6. **AccessRequestRepository** - CRUD + status updates (6 methods)
7. **ModelRepository** - Most complex, downloads + aliases + metadata (15+ methods)

---

## Step 5: PostgreSQL Migration Files

### New Directory
```
crates/services/migrations_pg/
  0001_create_tables.up.sql    -- All tables in one migration (clean slate)
  0001_create_tables.down.sql
```

### PostgreSQL Schema (all tables)
```sql
-- 0001_create_tables.up.sql

CREATE TABLE download_requests (
  id TEXT PRIMARY KEY,
  repo TEXT NOT NULL,
  filename TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('pending', 'downloading', 'completed', 'error')),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE access_requests (
  id SERIAL PRIMARY KEY,
  username TEXT NOT NULL,
  user_id TEXT NOT NULL,
  email TEXT,
  reviewer TEXT,
  status TEXT NOT NULL CHECK (status IN ('pending', 'approved', 'rejected')),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

-- ... all other tables matching SQLite schema but with PG-compatible types
-- Key differences:
-- INTEGER PRIMARY KEY → SERIAL PRIMARY KEY (for access_requests, model_metadata)
-- COLLATE NOCASE → case-insensitive handling via LOWER()
```

### Migration Selection at Runtime
```rust
impl DbServiceImpl {
  pub async fn migrate(&self) -> Result<()> {
    if self.is_postgres() {
      sqlx::migrate!("./migrations_pg").run(&self.pool).await?;
    } else {
      sqlx::migrate!("./migrations").run(&self.pool).await?;
    }
    Ok(())
  }
}
```

**Note**: `sqlx::migrate!()` is a compile-time macro. For runtime selection, use `sqlx::migrate::Migrator::new()` with runtime path resolution.

---

## Step 6: Update AppServiceBuilder

### Database Connection
```rust
// Before
let app_db_pool = DbPool::connect(&format!("sqlite:{}", self.setting_service.app_db_path().display())).await?;
let db_service = SqliteDbService::new(app_db_pool, time_service, encryption_key);

// After
let database_url = self.setting_service.database_url();
// database_url is either:
//   "sqlite:{BODHI_HOME}/app.db" (single-tenant)
//   "postgres://user:pass@host:5432/bodhi" (multi-tenant)
let db_service = DbServiceImpl::new(&database_url, time_service, encryption_key).await?;
db_service.migrate().await?;
```

### SettingService Addition
```rust
fn database_url(&self) -> String {
  // Check DATABASE_URL env var first
  // Fall back to sqlite:{app_db_path}
  self.get_setting("DATABASE_URL")
    .unwrap_or_else(|| format!("sqlite:{}", self.app_db_path().display()))
}
```

---

## Step 7: Update Test Infrastructure

### TestDbService
```rust
pub struct TestDbService {
  _temp_dir: Arc<TempDir>,
  inner: DbServiceImpl,          // Was SqliteDbService
  event_sender: Sender<String>,
  now: DateTime<Utc>,
  encryption_key: Vec<u8>,
}

impl TestDbService {
  pub async fn new() -> Self {
    sqlx::any::install_default_drivers();
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let url = format!("sqlite:{}", db_path.display());
    let pool = AnyPool::connect(&url).await.unwrap();
    let time_service = Arc::new(FrozenTimeService::new(now));
    let inner = DbServiceImpl::with_pool(pool, time_service, encryption_key);
    inner.migrate().await.unwrap();
    // ...
  }
}
```

### MockDbService
- No changes needed - mockall mock doesn't depend on pool type

---

## Step 8: NAPI Bindings Update

### lib_bodhiserver_napi Changes
- Accept `DATABASE_URL` in initialization config
- Create AnyPool instead of SqlitePool
- Pass to DbServiceImpl

---

## Deliverable
- `SqliteDbService` renamed to `DbServiceImpl` using `AnyPool`
- All repository methods work with both SQLite and PostgreSQL
- PostgreSQL migration files created
- Runtime migration selection (SQLite vs PG based on URL)
- `DATABASE_URL` env var for PostgreSQL connection
- All existing tests pass (using SQLite via AnyPool)
- NAPI bindings updated
- `AppServiceBuilder` uses configurable database URL

## Testing Checklist
- [ ] All existing `cargo test -p services` pass
- [ ] All existing `cargo test` pass
- [ ] TestDbService works with AnyPool + SQLite
- [ ] MockDbService still generates correctly
- [ ] NAPI tests pass
- [ ] `cargo check` passes for all crates
