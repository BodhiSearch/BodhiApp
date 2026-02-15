# Phase 3: Org Threading (Organizations Table + org_id + Auth + DataService)

## Goal
Add the organizations table, thread org_id through all repository traits and service calls, refactor auth middleware to use per-org credentials, remove SecretService, and refactor DataService.

This is the largest phase - the core of multi-tenancy.

## Prerequisites
- Phase 2 complete (DbServiceImpl with AnyPool)

---

## Step 1: Organizations Table + Domain Objects

### New Migration (SQLite)
```
crates/services/migrations/
  0010_organizations.up.sql
  0010_organizations.down.sql
```

```sql
-- 0010_organizations.up.sql
CREATE TABLE organizations (
  id TEXT PRIMARY KEY,
  slug TEXT NOT NULL UNIQUE,
  display_name TEXT NOT NULL,
  kc_client_id TEXT NOT NULL,
  client_secret TEXT NOT NULL,    -- Encrypted with master key
  encryption_key TEXT NOT NULL,   -- Per-org key for encrypting org secrets
  status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'suspended', 'deleted')),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX idx_organizations_slug ON organizations(slug);
```

### New Migration (PostgreSQL)
Add same table to `migrations_pg/`.

### Domain Objects
```rust
// crates/objs/src/org.rs (or crates/services/src/objs.rs)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
  pub id: String,
  pub slug: String,
  pub display_name: String,
  pub kc_client_id: String,
  pub client_secret: String,     // Encrypted
  pub encryption_key: String,    // Per-org
  pub status: OrgStatus,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrgStatus {
  Active,
  Suspended,
  Deleted,
}

#[derive(Debug, Clone)]
pub struct OrgContext {
  pub org_id: String,
  pub org_slug: String,
  pub kc_client_id: String,
  pub client_secret: String,     // Decrypted
  pub encryption_key: Vec<u8>,   // Decrypted per-org key
  pub status: OrgStatus,
}
```

### OrgRepository Trait
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

### Update DbService Super-Trait
```rust
pub trait DbService:
  ModelRepository + AccessRepository + AccessRequestRepository
  + TokenRepository + ToolsetRepository + UserAliasRepository
  + OrgRepository    // NEW
  + DbCore
  + Send + Sync + Debug
{}
```

---

## Step 2: Add org_id to All Tables

### New Migration (SQLite)
```
crates/services/migrations/
  0011_add_org_id.up.sql
  0011_add_org_id.down.sql
```

```sql
-- 0011_add_org_id.up.sql

-- Add org_id to all existing tables
ALTER TABLE download_requests ADD COLUMN org_id TEXT NOT NULL DEFAULT 'default';
ALTER TABLE access_requests ADD COLUMN org_id TEXT NOT NULL DEFAULT 'default';
ALTER TABLE api_tokens ADD COLUMN org_id TEXT NOT NULL DEFAULT 'default';
ALTER TABLE api_model_aliases ADD COLUMN org_id TEXT NOT NULL DEFAULT 'default';
ALTER TABLE model_metadata ADD COLUMN org_id TEXT NOT NULL DEFAULT 'default';
ALTER TABLE app_toolset_configs ADD COLUMN org_id TEXT NOT NULL DEFAULT 'default';
ALTER TABLE toolsets ADD COLUMN org_id TEXT NOT NULL DEFAULT 'default';
ALTER TABLE user_aliases ADD COLUMN org_id TEXT NOT NULL DEFAULT 'default';
ALTER TABLE app_access_requests ADD COLUMN org_id TEXT NOT NULL DEFAULT 'default';

-- Create indexes for org_id queries
CREATE INDEX idx_download_requests_org ON download_requests(org_id);
CREATE INDEX idx_api_tokens_org ON api_tokens(org_id);
CREATE INDEX idx_api_model_aliases_org ON api_model_aliases(org_id);
CREATE INDEX idx_toolsets_org ON toolsets(org_id);
CREATE INDEX idx_user_aliases_org ON user_aliases(org_id);
CREATE INDEX idx_app_access_requests_org ON app_access_requests(org_id);

-- Note: SQLite doesn't support DROP CONSTRAINT / recreate with new UNIQUE.
-- For SQLite, unique constraints change requires table recreation.
-- For simplicity, handle uniqueness at application level or via new indexes:
CREATE UNIQUE INDEX idx_api_model_aliases_org_prefix ON api_model_aliases(org_id, prefix);
CREATE UNIQUE INDEX idx_app_toolset_configs_org_type ON app_toolset_configs(org_id, toolset_type);
CREATE UNIQUE INDEX idx_user_aliases_org_alias ON user_aliases(org_id, alias);
CREATE UNIQUE INDEX idx_toolsets_org_user_slug ON toolsets(org_id, user_id, slug);
```

### PostgreSQL Migration
Same additions in `migrations_pg/`. PostgreSQL supports `ADD COLUMN IF NOT EXISTS` and proper constraint management.

---

## Step 3: Update Repository Trait Signatures

Add `org_id: &str` as first parameter to all org-scoped methods.

### TokenRepository
```rust
#[async_trait]
pub trait TokenRepository: Send + Sync {
  async fn create_api_token(&self, org_id: &str, token: &ApiToken) -> Result<(), DbError>;
  async fn list_api_tokens(&self, org_id: &str, user_id: &str) -> Result<Vec<ApiToken>, DbError>;
  async fn get_api_token_by_id(&self, org_id: &str, id: &str) -> Result<Option<ApiToken>, DbError>;
  async fn get_api_token_by_prefix(&self, org_id: &str, prefix: &str) -> Result<Option<ApiToken>, DbError>;
  async fn update_api_token(&self, org_id: &str, id: &str, updates: &TokenUpdate) -> Result<(), DbError>;
}
```

### Similar changes for:
- **AccessRepository** - all methods get `org_id`
- **AccessRequestRepository** - all methods get `org_id`
- **ModelRepository** - all methods get `org_id`
- **ToolsetRepository** - all methods get `org_id`
- **UserAliasRepository** - all methods get `org_id`

### DbCore stays unchanged (org-agnostic):
```rust
pub trait DbCore: Send + Sync {
  async fn migrate(&self) -> Result<(), DbError>;
  fn now(&self) -> DateTime<Utc>;
  fn encryption_key(&self) -> &[u8];
}
```

---

## Step 4: Update DbServiceImpl Queries

Every SQL query that touches org-scoped tables adds `org_id` to:
- INSERT: `org_id` as a column value
- SELECT: `WHERE org_id = $N` in WHERE clause
- UPDATE: `WHERE org_id = $N AND ...` in WHERE clause
- DELETE: `WHERE org_id = $N AND ...` in WHERE clause

Example:
```rust
async fn list_api_tokens(&self, org_id: &str, user_id: &str) -> Result<Vec<ApiToken>, DbError> {
  let rows = sqlx::query_as(
    "SELECT id, org_id, user_id, token_prefix, name, scopes, status, created_at, updated_at
     FROM api_tokens WHERE org_id = $1 AND user_id = $2 ORDER BY created_at DESC"
  )
  .bind(org_id)
  .bind(user_id)
  .fetch_all(&self.pool)
  .await?;
  Ok(rows)
}
```

---

## Step 5: Org Resolution Middleware

### New File: crates/auth_middleware/src/org_middleware.rs
```rust
pub async fn org_resolution_middleware(
  State(state): State<Arc<dyn RouterState>>,
  mut req: Request,
  next: Next,
) -> Result<Response, ApiError> {
  // 1. Try X-BodhiApp-Org header (multi-tenant: from Traefik)
  let org_slug = req.headers()
    .get("X-BodhiApp-Org")
    .and_then(|v| v.to_str().ok())
    .map(String::from);

  let org_slug = match org_slug {
    Some(slug) => slug,
    None => {
      // Single-tenant: load the only org from DB
      // (Could also be a config default)
      let orgs = state.app_service().db_service().list_orgs().await?;
      orgs.first()
        .map(|o| o.slug.clone())
        .ok_or(ApiError::NoOrgConfigured)?
    }
  };

  // 2. Resolve OrgContext from cache or DB
  let org_context = resolve_org_context(&org_slug, &state).await?;

  // 3. Validate status
  if org_context.status != OrgStatus::Active {
    return Err(ApiError::OrgSuspended(org_slug));
  }

  // 4. Inject into request
  req.extensions_mut().insert(org_context.clone());
  req.headers_mut().insert(
    HeaderName::from_static("x-bodhiapp-org-id"),
    HeaderValue::from_str(&org_context.org_id)?,
  );

  Ok(next.run(req).await)
}
```

---

## Step 6: Update Auth Middleware

### Remove SecretService dependency
```rust
// Before: auth_middleware reads from SecretService
let app_reg_info = secret_service.app_reg_info()?.ok_or(AppRegInfoMissingError)?;

// After: read from OrgContext extension
let org_ctx = req.extensions().get::<OrgContext>().ok_or(ApiError::OrgContextMissing)?;
// Use org_ctx.kc_client_id instead of app_reg_info.client_id
// Use org_ctx.client_secret instead of app_reg_info.client_secret
```

### Update Token Service
- `handle_external_client_token()`: Use org's client_id for audience validation
- `get_valid_session_token()`: Use org's credentials for token refresh
- Remove all `SecretService` usage from token validation

### Update Login Handlers
- `auth_initiate_handler()`: Use OrgContext for OAuth redirect client_id
- `auth_callback_handler()`: Use OrgContext credentials for code exchange
- Remove ResourceAdmin first-login flow (org provisioned externally now)

---

## Step 7: Remove SecretService

### Files to Remove/Modify
1. **Remove**: `crates/services/src/secret_service.rs` (trait + impl)
2. **Remove**: `crates/services/src/service_ext.rs` (SecretServiceExt)
3. **Remove**: `secret_service` from `AppService` trait
4. **Remove**: `secret_service` from `DefaultAppService`
5. **Remove**: `SecretService` from `AppServiceBuilder`
6. **Remove**: Keyring dependencies from Cargo.toml (macos/linux/windows)
7. **Remove**: AES-GCM, PBKDF2 dependencies (encryption now at DB level)
8. **Update**: All consumers (auth_middleware, token_service, login handlers)
9. **Update**: Test utilities (remove `with_app_reg_info()`, `MockSecretService`)

### AppStatus Migration
- `AppStatus` was stored in SecretService
- For multi-tenant: org status in organizations table
- For single-tenant: org status in organizations table (single row)
- Remove `AppStatus` enum or repurpose as `OrgStatus`

---

## Step 8: Update Route Handlers

All route handlers that call org-scoped services add `ExtractOrgId`:

```rust
// Before
pub async fn list_toolsets_handler(
  ExtractUserId(user_id): ExtractUserId,
  State(state): State<Arc<dyn RouterState>>,
) -> Result<impl IntoResponse, ApiError> {
  let toolsets = state.app_service().db_service()
    .list_toolsets(&user_id).await?;
  Ok(Json(toolsets))
}

// After
pub async fn list_toolsets_handler(
  ExtractOrgId(org_id): ExtractOrgId,
  ExtractUserId(user_id): ExtractUserId,
  State(state): State<Arc<dyn RouterState>>,
) -> Result<impl IntoResponse, ApiError> {
  let toolsets = state.app_service().db_service()
    .list_toolsets(&org_id, &user_id).await?;
  Ok(Json(toolsets))
}
```

---

## Step 9: Update Service Layer

Services that call repository methods need org_id passed through:

### ToolService
```rust
// Methods receive org_id, pass to db_service
async fn create_toolset(&self, org_id: &str, user_id: &str, ...) -> Result<...> {
  self.db_service.create_toolset(org_id, &toolset).await?;
}
```

### AiApiService
```rust
// API model alias lookups include org_id
async fn resolve_model(&self, org_id: &str, model_name: &str) -> Result<...> {
  self.db_service.get_api_model_alias_by_prefix(org_id, model_name).await?;
}
```

### AccessRequestService
```rust
// Access requests are org-scoped
async fn create(&self, org_id: &str, ...) -> Result<...> {
  self.db_service.create_access_request(org_id, &request).await?;
}
```

---

## Step 10: DataService Refactor

### Current DataService
Handles: local model files (GGUF), user aliases, model listing

### Changes
- Rename to `LocalModelService` (or keep as `DataService` with reduced scope)
- Local model methods: return `UnsupportedOperation` when `BODHI_MULTI_TENANT=true`
- User alias methods: add org_id parameter
- Keep for single-tenant; in multi-tenant, most methods are no-ops

```rust
pub trait DataService: Send + Sync + Debug {
  // Local model methods (UnsupportedOperation in multi-tenant)
  async fn download_model(&self, ...) -> Result<...>;
  async fn list_local_models(&self) -> Result<Vec<Model>>;

  // User alias methods (org-scoped)
  async fn create_alias(&self, org_id: &str, ...) -> Result<...>;
  async fn list_aliases(&self, org_id: &str) -> Result<Vec<UserAlias>>;
}
```

---

## Step 11: Update All Tests

### Mechanical Changes
Every test calling repository methods adds org_id parameter:
```rust
// Before
db.create_api_token(&token).await.unwrap();

// After
db.create_api_token("test-org-id", &token).await.unwrap();
```

### New Org Isolation Tests
```rust
#[tokio::test]
async fn test_tokens_isolated_between_orgs() {
  let db = TestDbService::new().await;
  db.create_org(&test_org_alpha()).await.unwrap();
  db.create_org(&test_org_beta()).await.unwrap();

  db.create_api_token("alpha", &token_a).await.unwrap();
  db.create_api_token("beta", &token_b).await.unwrap();

  let alpha_tokens = db.list_api_tokens("alpha", "user-1").await.unwrap();
  assert_eq!(1, alpha_tokens.len());

  let beta_tokens = db.list_api_tokens("beta", "user-1").await.unwrap();
  assert_eq!(1, beta_tokens.len());
}
```

### Auth Middleware Tests
- Test org resolution from header
- Test org resolution fallback (single-tenant)
- Test suspended org rejection
- Test OrgContext injection

---

## Step 12: New API Endpoints

### GET /api/orgs/current
```rust
pub async fn get_current_org(
  ExtractOrgContext(org): ExtractOrgContext,
) -> Result<impl IntoResponse, ApiError> {
  Ok(Json(OrgInfo {
    org_id: org.org_id,
    slug: org.org_slug,
    display_name: org.display_name,
    status: org.status,
  }))
}
```

### GET /api/orgs/user-memberships
```rust
pub async fn get_user_memberships(
  ExtractUserId(user_id): ExtractUserId,
  ExtractToken(token): ExtractToken,
  State(state): State<Arc<dyn RouterState>>,
) -> Result<impl IntoResponse, ApiError> {
  // Call KC Organizations API to get user's org memberships
  let memberships = state.app_service().auth_service()
    .get_user_organizations(&token).await?;
  Ok(Json(memberships))
}
```

---

## Deliverable
- Organizations table in both SQLite and PostgreSQL
- org_id column in all existing tables
- All repository traits + impls updated with org_id parameter
- Org resolution middleware
- Auth middleware uses OrgContext (no more SecretService)
- SecretService completely removed
- Login flow uses per-org KC client credentials
- DataService refactored for multi-tenant mode
- New extractors: ExtractOrgId, ExtractOrgContext
- New endpoints: /api/orgs/current, /api/orgs/user-memberships
- All existing tests updated and passing
- New org isolation tests

## Testing Checklist
- [ ] All `cargo test` pass with org_id parameters
- [ ] Org isolation: data from org-alpha not visible to org-beta
- [ ] Auth middleware works with OrgContext
- [ ] Login flow works with per-org credentials
- [ ] Single-tenant mode works (single org row, no Traefik header)
- [ ] SecretService fully removed, no compilation references
- [ ] NAPI tests pass
