# Phase 1: Session PostgreSQL Migration

## Goal
Migrate tower-sessions from SQLite-only to configurable backend (SQLite or PostgreSQL) with a separate connection pool for sessions.

## Prerequisites
- tower-sessions-sqlx-store crate supports PostgreSQL (version 0.15.0 already in workspace)

---

## Step 1: Add PostgreSQL Dependencies

### Cargo.toml Changes

**Workspace Cargo.toml**:
```toml
[workspace.dependencies]
sqlx = { version = "0.8.6", features = ["chrono", "runtime-tokio", "sqlite", "postgres", "any"] }
tower-sessions-sqlx-store = { version = "0.15.0", features = ["sqlite", "postgres"] }
```

**crates/services/Cargo.toml**:
```toml
[dependencies]
sqlx = { workspace = true, features = ["chrono", "runtime-tokio", "sqlite", "postgres", "any"] }
tower-sessions-sqlx-store = { workspace = true, features = ["sqlite", "postgres"] }
```

### Verification
- `cargo check -p services` passes
- Existing tests still pass with `cargo test -p services`

---

## Step 2: Abstract Session Store

### Current: SqliteSessionService
```rust
pub struct SqliteSessionService {
  pub session_store: AppSessionStore,
}

pub struct AppSessionStore {
  inner: SqliteStore,
  pool: Pool<Sqlite>,
}
```

### Target: Configurable SessionService

**Option A**: Use sqlx::Any for session store as well
- tower-sessions-sqlx-store may not support AnyPool directly
- Need to check crate compatibility

**Option B**: Enum-based dispatch
```rust
pub enum SessionStoreBackend {
  Sqlite(SqliteStore),
  Postgres(PostgresStore),
}

pub struct AppSessionStore {
  backend: SessionStoreBackend,
  pool: AnyPool,  // For custom queries (user_id column, etc.)
}
```

**Recommended**: Option B - enum dispatch, since tower-sessions stores have DB-specific types.

### Implementation

```rust
// crates/services/src/session_service.rs

pub struct AppSessionStore {
  backend: SessionStoreBackend,
  pool: AnyPool,
}

enum SessionStoreBackend {
  Sqlite(SqliteStore),
  Postgres(PostgresStore),
}

impl AppSessionStore {
  pub async fn new_sqlite(pool: SqlitePool) -> Result<Self> {
    let store = SqliteStore::new(pool.clone());
    let any_pool = AnyPool::from(pool);
    Ok(Self {
      backend: SessionStoreBackend::Sqlite(store),
      pool: any_pool,
    })
  }

  pub async fn new_postgres(pool: PgPool) -> Result<Self> {
    let store = PostgresStore::new(pool.clone());
    let any_pool = AnyPool::from(pool);
    Ok(Self {
      backend: SessionStoreBackend::Postgres(store),
      pool: any_pool,
    })
  }

  pub async fn migrate(&self) -> Result<()> {
    match &self.backend {
      SessionStoreBackend::Sqlite(store) => store.migrate().await?,
      SessionStoreBackend::Postgres(store) => store.migrate().await?,
    }
    // Add custom user_id column (SQL differs between SQLite and PG)
    self.add_user_id_column().await?;
    Ok(())
  }

  async fn add_user_id_column(&self) -> Result<()> {
    // Check if column exists, add if not
    // SQLite: ALTER TABLE tower_sessions ADD COLUMN user_id TEXT;
    // PostgreSQL: ALTER TABLE tower_sessions ADD COLUMN IF NOT EXISTS user_id TEXT;
    // Both: CREATE INDEX IF NOT EXISTS idx_tower_sessions_user_id ON tower_sessions(user_id);
  }
}

impl SessionStore for AppSessionStore {
  async fn save(&self, record: &Record) -> Result<(), Error> {
    match &self.backend {
      SessionStoreBackend::Sqlite(store) => store.save(record).await?,
      SessionStoreBackend::Postgres(store) => store.save(record).await?,
    }
    // Update user_id column (same for both DBs via AnyPool)
    if let Some(user_id) = record.data.get("user_id").and_then(|v| v.as_str()) {
      sqlx::query("UPDATE tower_sessions SET user_id = $1 WHERE id = $2")
        .bind(user_id)
        .bind(record.id.to_string())
        .execute(&self.pool)
        .await?;
    }
    Ok(())
  }

  // load() and delete() delegate similarly
}
```

---

## Step 3: Configuration

### New Environment Variable
```env
SESSION_DB_URL=postgres://user:pass@host:5432/sessions  # PostgreSQL
SESSION_DB_URL=sqlite:{BODHI_HOME}/session.db            # SQLite (default)
```

### SettingService Addition
```rust
fn session_db_url(&self) -> String {
  // If SESSION_DB_URL set, use it
  // Otherwise, fall back to sqlite:{session_db_path}
}
```

### AppServiceBuilder Changes
```rust
async fn get_or_build_session_service(&mut self) -> Result<Arc<dyn SessionService>> {
  let session_db_url = self.setting_service.session_db_url();

  let session_store = if session_db_url.starts_with("postgres://") {
    let pool = PgPool::connect(&session_db_url).await?;
    AppSessionStore::new_postgres(pool).await?
  } else {
    let pool = SqlitePool::connect(&session_db_url).await?;
    AppSessionStore::new_sqlite(pool).await?
  };

  session_store.migrate().await?;
  Ok(Arc::new(DefaultSessionService::new(session_store)))
}
```

---

## Step 4: Rename SessionService Implementation

```rust
// Before
pub struct SqliteSessionService { ... }

// After
pub struct DefaultSessionService { ... }  // Works with any backend
```

Update all references across crates.

---

## Step 5: Tests

### Existing Tests
- All existing session tests continue to work (SQLite backend)
- No PostgreSQL needed for local testing

### New Tests
```rust
#[tokio::test]
async fn test_session_store_sqlite_backend() {
  let store = AppSessionStore::new_sqlite(sqlite_pool).await.unwrap();
  store.migrate().await.unwrap();
  // ... existing session test patterns
}

#[tokio::test]
async fn test_session_user_id_tracking() {
  // Verify user_id column works with both backends
  let store = AppSessionStore::new_sqlite(sqlite_pool).await.unwrap();
  store.migrate().await.unwrap();

  let mut record = Record::new();
  record.data.insert("user_id".into(), json!("test-user"));
  store.save(&record).await.unwrap();

  let count = store.count_sessions_for_user("test-user").await.unwrap();
  assert_eq!(1, count);
}
```

---

## Deliverable
- tower-sessions works with both SQLite and PostgreSQL
- Configurable via `SESSION_DB_URL` environment variable
- Separate connection pool for sessions
- All existing tests pass
- `SqliteSessionService` renamed to `DefaultSessionService`
- Session-related custom queries (user_id tracking) work on both DBs
