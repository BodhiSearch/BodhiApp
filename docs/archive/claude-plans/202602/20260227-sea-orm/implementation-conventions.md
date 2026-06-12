# SeaORM Migration: Implementation Conventions

**Context**: This document captures the conventions established during the SeaORM migration. It serves as a reference for understanding the patterns used in the codebase.

**Reference**: See `migration-decisions.md` for design decisions (PK strategy, timestamp normalization, CITEXT, etc.)

---

## 1. Entity Struct Pattern

### Decision Criteria

Use **Pattern A (Direct alias)** when:
- Domain struct fields match DB columns 1:1
- No encrypted columns to hide
- Struct already has or can have Serialize/Deserialize/ToSchema derives

```rust
// entities/access_request.rs
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema, DeriveEntityModel)]
#[sea_orm(table_name = "access_requests")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    // ... all fields
}

pub type UserAccessRequest = Model;  // Domain type alias
```

Use **Pattern B (Separate domain struct)** when:
- DB has encrypted_*, salt_*, nonce_* columns that must be hidden from domain API
- Field transformation needed between DB and domain representation

```rust
// entities/toolset.rs
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "toolsets")]
pub struct Model {
    // ALL DB columns including encrypted_api_key, salt, nonce
}

// objs.rs (or same file)
pub struct ToolsetRow {
    // Domain fields INCLUDING encrypted_api_key, salt, nonce
    // (kept because service layer needs them for encryption/decryption)
}

impl From<Model> for ToolsetRow { ... }
```

### Entities needing Pattern B (per migration-decisions.md)
- `toolsets` (has encrypted_api_key/salt/nonce) -- DONE
- `api_model_aliases` (has encrypted_api_key) -- already done in prototype
- `mcp_auth_headers` (will have encrypted columns) -- Iteration 6
- `mcp_oauth_configs` (will have encrypted columns) -- Iteration 6
- `mcp_oauth_tokens` (will have encrypted columns) -- Iteration 6
- `apps` (has encrypted client_secret) -- Iteration 8

### All other entities use Pattern A.

---

## 2. Enum DeriveValueType Pattern

For domain enums used as DB column values, add `DeriveValueType` + `sea_orm(value_type = "String")`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, EnumString, strum::Display, PartialEq, ToSchema,
         sea_orm::DeriveValueType)]
#[sea_orm(value_type = "String")]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum SomeStatus {
    Active,
    Inactive,
}
```

Already done: `DownloadStatus`, `ApiFormat`, `UserAccessRequestStatus`, `TokenStatus`.
Remaining: `AppAccessRequestStatus` (new enum, Iteration 5), `FlowType` (new enum, Iteration 5).

---

## 3. Shared Test Infrastructure

### rstest Fixture Pattern

Implemented in `crates/services/src/test_utils/sea.rs`:

```rust
use crate::db::{sea_migrations::Migrator, DefaultDbService, TimeService};
use crate::test_utils::FrozenTimeService;
use rstest::fixture;
use sea_orm::Database;
use sea_orm_migration::MigratorTrait;
use std::sync::Arc;
use tempfile::TempDir;
use chrono::{DateTime, Utc};

pub struct SeaTestContext {
    pub _temp_dir: Option<TempDir>,
    pub service: DefaultDbService,
    pub now: DateTime<Utc>,
}

/// rstest fixture for dual-DB SeaORM testing.
/// Usage: #[values("sqlite", "postgres")] db_type: &str
#[fixture]
pub async fn sea_context(#[default("sqlite")] db_type: &str) -> SeaTestContext {
    // Load .env.test for PG URL
    let env_path = std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/.env.test"));
    if env_path.exists() {
        let _ = dotenv::from_filename(&env_path).ok();
    }

    let time_service = FrozenTimeService::default();
    let now = time_service.utc_now();
    let encryption_key = /* 32-byte key */ ;

    match db_type {
        "sqlite" => {
            let temp_dir = TempDir::new().unwrap();
            let url = format!("sqlite:{}?mode=rwc", temp_dir.path().join("test.db").display());
            let db = Database::connect(&url).await.unwrap();
            Migrator::fresh(&db, None).await.unwrap();  // Clean schema via fresh()
            let service = DefaultDbService::new(db, Arc::new(time_service), encryption_key);
            SeaTestContext { _temp_dir: Some(temp_dir), service, now }
        }
        "postgres" => {
            let pg_url = std::env::var("INTEG_TEST_APP_DB_PG_URL")
                .expect("INTEG_TEST_APP_DB_PG_URL must be set (run `make test.deps.up`)");
            let db = Database::connect(&pg_url).await.unwrap();
            Migrator::fresh(&db, None).await.unwrap();  // Clean schema via fresh()
            let service = DefaultDbService::new(db, Arc::new(time_service), encryption_key);
            SeaTestContext { _temp_dir: None, service, now }
        }
        other => panic!("Unknown db_type: {other}. Use 'sqlite' or 'postgres'."),
    }
}
```

### Test File Pattern (new iterations)

```rust
use crate::test_utils::{setup_env, sea_context, SeaTestContext};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;
use anyhow_trace::anyhow_trace;

#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_something(
    _setup_env: (),
    #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
    let ctx = sea_context(db_type).await;
    // Seed data specific to this test
    // ... test logic ...
    Ok(())
}
```

### Test File Organization

All 9 repository test files use `sea_context` fixture. Test files are in `crates/services/src/db/test_*_repository.rs` (no `_sea` suffix).

---

## 4. SeaORM Migration File Pattern

### Naming Convention
`m20250101_000NNN_<table_name>.rs` where NNN follows the sequence in `sea_migrations/mod.rs`.

### Structure
```rust
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum TableName {
    Table,
    Id,
    // ... columns
    CreatedAt,
    UpdatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create()
                .table(TableName::Table)
                .if_not_exists()  // Required during transition (both sqlx + sea-orm run)
                .col(string(TableName::Id).primary_key())
                .col(timestamp_with_time_zone(TableName::CreatedAt))
                .col(timestamp_with_time_zone(TableName::UpdatedAt))
                // ... other columns
                .to_owned(),
        ).await?;

        // Add indexes
        manager.create_index(
            Index::create()
                .if_not_exists()
                .name("idx_tablename_column")
                .table(TableName::Table)
                .col(TableName::Column)
                .to_owned(),
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop in reverse order (indexes, then dependent tables, then main table)
        manager.drop_table(Table::drop().table(TableName::Table).to_owned()).await?;
        Ok(())
    }
}
```

### Key Rules
- **All tables use `.if_not_exists()`** during transition (sqlx migrations may have already created the table)
- **All PKs are String (ULID)**: `.col(string(Id).primary_key())`
- **All timestamps are timezone-aware**: `.col(timestamp_with_time_zone(CreatedAt))`
- **JSON columns**: Use `.col(json_binary(ColumnName))` for cross-DB JSON support
- **CITEXT columns (PG)**: Use raw SQL conditionally via `manager.get_database_backend()` where needed
- **FK constraints**: Add where relationships exist

---

## 5. Service Implementation Pattern (service_*.rs)

### Structure
```rust
use crate::db::{entities::entity_name, DefaultDbService, SomeRepository};
use sea_orm::prelude::*;

#[async_trait::async_trait]
impl SomeRepository for DefaultDbService {
    async fn create_thing(&self, thing: &DomainType) -> Result<(), DbError> {
        let now = self.time_service.utc_now();
        let id = ulid::Ulid::new().to_string();

        let active = entity_name::ActiveModel {
            id: Set(id),
            // ... Set all fields explicitly (no NotSet for inserts)
            created_at: Set(now),
            updated_at: Set(now),
        };
        entity_name::Entity::insert(active).exec(&self.db).await.map_err(DbError::from)?;
        Ok(())
    }

    async fn update_thing(&self, id: &str, changes: &Changes) -> Result<(), DbError> {
        let now = self.time_service.utc_now();

        // For partial updates: Default + set only changed fields
        let mut active: entity_name::ActiveModel = Default::default();
        active.id = Set(id.to_string());
        active.changed_field = Set(new_value);
        active.updated_at = Set(now);
        entity_name::Entity::update(active).exec(&self.db).await.map_err(DbError::from)?;
        Ok(())
    }

    async fn get_thing(&self, id: &str) -> Result<Option<DomainType>, DbError> {
        entity_name::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(DbError::from)
    }
}
```

### Error Handling
- **Default**: `.map_err(DbError::from)?` for generic sea_orm::DbErr wrapping
- **Match specific variants** only when business logic requires it:
  ```rust
  match result {
      Err(DbErr::RecordNotFound(_)) => Ok(None),  // Expected: return None
      Err(e) => Err(DbError::from(e)),             // Unexpected: propagate
      Ok(model) => Ok(Some(model.into())),
  }
  ```
- Use hybrid approach: generic wrapping by default, specific matching only for upsert-on-conflict, unique constraint violations, etc.

### Encryption Pattern (Pattern B entities)
```rust
// Create: encrypt before insert
let (encrypted, salt, nonce) = if let Some(ref key) = plaintext_key {
    encrypt_api_key(&self.encryption_key, key)?
} else {
    (None, None, None)
};
active.encrypted_api_key = Set(encrypted);
active.salt = Set(salt);
active.nonce = Set(nonce);

// Read: decrypt after select
let decrypted = decrypt_api_key(&self.encryption_key, &model.encrypted_api_key, &model.salt, &model.nonce)?;
```

---

## 6. ULID Generation

All new records use ULID for primary keys:
```rust
let id = ulid::Ulid::new().to_string();
```

- 26-char Crockford Base32, lexicographically sortable, time-ordered
- Stored as TEXT in both SQLite and PostgreSQL
- Generated in service layer (not DB default)

---

## 7. Dual-DB Test Conventions

### Test Annotation Stack
```rust
#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_name(
    _setup_env: (),
    #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> { ... }
```

### Rules
- `#[values("sqlite", "postgres")]` for db_type (NOT `#[case]` -- keep `#[case]` free for test variations)
- `#[serial(pg_app)]` on ALL dual-DB tests (both SQLite and PG variants serialize)
- `_setup_env: ()` for environment setup
- PostgreSQL is **mandatory** -- tests panic if PG unavailable (no skip behavior)
- Use `Migrator::fresh()` for schema setup (replaces manual DROP CASCADE)
- Test file naming: `test_<repo>_repository.rs`

### Assertion Style
```rust
use pretty_assertions::assert_eq;
assert_eq!(expected, actual);  // expected first (JUnit convention)
```

---

## 8. Remaining Iteration Entity Decisions

Quick reference from migration-decisions.md:

| Iteration | Entity/Table | Pattern | Key Changes |
|-----------|-------------|---------|-------------|
| 4 | user_aliases | A (direct) | Rename columns: request_params_json -> request_params, context_params_json -> context_params. Add DeriveValueType to Repo. Add FromJsonQueryResult to OAIRequestParams. JSON columns use json_binary(). |
| 5 | app_access_requests | A (direct) | Create new enums: AppAccessRequestStatus, FlowType with DeriveValueType. Timestamps i64 -> DateTime<Utc>. |
| 6 | mcp_servers | A (direct) | Timestamps i64 -> DateTime<Utc>. url column CITEXT/COLLATE NOCASE. |
| 6 | mcps | A (direct) | Relation BelongsTo McpServer. slug CITEXT/COLLATE NOCASE. Remove McpWithServerRow, use tuple return. |
| 6 | mcp_auth_headers | B (separate) | REFACTOR: hide encrypted columns from domain struct. |
| 6 | mcp_oauth_configs | B (separate) | REFACTOR: hide encrypted columns from domain struct. |
| 6 | mcp_oauth_tokens | B (separate) | REFACTOR: hide encrypted columns from domain struct. |
| 7 | settings | A (direct) | Natural key (key column as PK), no ULID. Timestamps i64 -> DateTime<Utc>. |
| 8 | apps | B (separate) | Hide encrypted client_secret. Timestamps i64 -> DateTime<Utc>. |

---

## 9. Checklist for Adding New Entities

When adding new database entities, follow this checklist:

1. [ ] Add DeriveValueType to any new enums in objs
2. [ ] Create entity file in `entities/` (Pattern A or B)
3. [ ] Create SeaORM migration in `sea_migrations/`
4. [ ] Register migration in `sea_migrations/mod.rs`
5. [ ] Define repository trait in `*_repository.rs`
6. [ ] Implement repository for DefaultDbService in `service_*.rs`
7. [ ] Update routes_app for any type changes
8. [ ] Create dual-DB tests using `sea_context` fixture in `test_*_repository.rs`
9. [ ] Run `cargo test -p services` (all tests pass on both DBs)
10. [ ] Run `cargo test -p routes_app` (all downstream tests pass)
11. [ ] Run `cargo fmt --all`

---

## 10. Things NOT to Do

- Do NOT create separate entity structs when fields match 1:1 (use Pattern A)
- Do NOT use `Utc::now()` directly (use `self.time_service.utc_now()`)
- Do NOT use `#[case]` for db_type parameterization (use `#[values]`)
- Do NOT add `use super::*` in test modules
- Do NOT add inline timeouts in tests
- Do NOT plan for backwards compatibility (no production release exists)
- Do NOT create new MockDbService usage (use real DB in new tests)
- Do NOT use `NotSet` for insert operations (set all fields explicitly)
