# Service Layer Context

## Current Service Architecture

### AppService Registry (15 services)
```rust
pub trait AppService: Debug + Send + Sync {
  fn setting_service(&self) -> Arc<dyn SettingService>;
  fn data_service(&self) -> Arc<dyn DataService>;
  fn hub_service(&self) -> Arc<dyn HubService>;
  fn auth_service(&self) -> Arc<dyn AuthService>;
  fn db_service(&self) -> Arc<dyn DbService>;
  fn session_service(&self) -> Arc<dyn SessionService>;
  fn secret_service(&self) -> Arc<dyn SecretService>;       // REMOVING
  fn cache_service(&self) -> Arc<dyn CacheService>;
  fn time_service(&self) -> Arc<dyn TimeService>;
  fn ai_api_service(&self) -> Arc<dyn AiApiService>;
  fn concurrency_service(&self) -> Arc<dyn ConcurrencyService>;
  fn queue_producer(&self) -> Arc<dyn QueueProducer>;
  fn tool_service(&self) -> Arc<dyn ToolService>;
  fn network_service(&self) -> Arc<dyn NetworkService>;
  fn access_request_service(&self) -> Arc<dyn AccessRequestService>;
}
```

### Service Initialization (AppServiceBuilder)
Location: `crates/lib_bodhiserver/src/app_service_builder.rs`

Order:
1. TimeService → DefaultTimeService
2. Encryption key → from env var or platform keyring
3. SecretService → AES-GCM encrypted file (REMOVING)
4. DbService → SqliteDbService with pool + migrations
5. SessionService → SqliteSessionService with separate pool
6. HubService → HfHubService (HuggingFace)
7. DataService → LocalDataService
8. CacheService → MokaCacheService (in-memory)
9. AuthService → KeycloakAuthService
10. AiApiService → API alias management
11. ConcurrencyService → LocalConcurrencyService
12. ToolService → function calling
13. AccessRequestService → workflow management
14. NetworkService → HTTP client
15. QueueProducer + RefreshWorker → background tasks

---

## Multi-Tenant Service Changes

### Services Being Modified

#### DbService
- **Change**: SqliteDbService → DbServiceImpl (sqlx::Any)
- **Pool**: AnyPool (connects to SQLite or PostgreSQL based on URL)
- **All repository methods**: Add `org_id: &str` first parameter
- **New**: OrgRepository trait for organizations table
- **Encryption**: Per-org encryption key from OrgContext, not global

#### SessionService
- **Change**: SqliteSessionService → configurable backend
- **Multi-tenant**: PostgresStore via tower-sessions-sqlx-store
- **Single-tenant**: SqliteStore (unchanged)
- **Config**: `SESSION_DB_URL` env var determines backend
- **Org-scoping**: Sessions naturally org-scoped via cookie subdomain isolation

#### CacheService
- **Change**: MokaCacheService → generic trait with typed methods
- **Current trait**: `get(&str) -> Option<String>`, `set(&str, &str)`, `remove(&str)`
- **New trait**: Generic `get<T>(key)`, `set<T>(key, val, ttl)`, `invalidate(key)`
- **Multi-tenant impl**: Redis-backed with event-driven invalidation
- **Single-tenant impl**: In-memory (moka or similar)
- **Key use case**: Org config caching

#### AuthService
- **Change**: No longer uses global client_id/client_secret
- **Methods receive**: client_id and client_secret as parameters (from OrgContext)
- **Currently**: Some methods read from SettingService
- **Target**: All auth methods are org-agnostic, credentials passed in

#### DataService
- **Change**: Rethink as LocalModelService
- **Keep**: Local model management (GGUF, downloads, user aliases for local models)
- **In hosted mode**: Most methods return UnsupportedOperation
- **AiApi model aliases**: Already handled by AiApiService

#### AiApiService
- **Change**: Add org_id to alias lookups
- **Currently**: Queries api_model_aliases without org scope
- **Target**: All queries include org_id filter

### Services Being Removed

#### SecretService
- **Reason**: All secrets move to organizations table or env vars
- **Consumers to update**: auth_middleware, token_service, login handlers, setup routes, access_request_service
- **Replacement**: OrgContext (from cache/DB) for per-org secrets, env vars for platform secrets

### Services Unchanged

#### SettingService
- **Status**: Platform-level only, no per-org settings
- **Auth URLs**: Stay global (single KC realm)
- **Changes**: Remove `secrets_path()`, `encryption_key()` methods (moving to env/DB)

#### TimeService
- **Status**: No changes needed

#### HubService
- **Status**: Used only in single-tenant (local LLM mode)
- **In hosted mode**: Not called (DataService returns UnsupportedOperation)

#### ConcurrencyService
- **Status**: LocalConcurrencyService may need distributed impl for multi-tenant
- **Current**: In-process mutex for token refresh locking
- **Multi-tenant**: Need distributed lock (Redis-based) for token refresh across instances
- **Note**: This is related to CacheService Redis infrastructure

#### ToolService
- **Change**: Toolset configs become org-scoped
- **app_toolset_configs**: Queries include org_id filter
- **User toolsets**: Queries include org_id filter

#### NetworkService
- **Status**: Stateless HTTP client, no changes needed

#### QueueProducer
- **Status**: Background model metadata extraction
- **In hosted mode**: May not be needed (no local models)
- **Keep for now**: Returns UnsupportedOperation in hosted mode

---

## Org ID Propagation Pattern

### Request Lifecycle
```
1. Traefik injects X-BodhiApp-Org: <slug>
2. Org resolution middleware:
   - Read org slug from header
   - Look up OrgContext from CacheService
   - Cache miss → query organizations table
   - Inject Extension<OrgContext> into request
3. Auth middleware:
   - Read OrgContext from Extension
   - Use org credentials for token validation/refresh
   - Inject X-BodhiApp-Org-Id header
4. Route handler:
   - Extract OrgContext or ExtractOrgId
   - Pass org_id explicitly to service methods
5. Service method:
   - Pass org_id to repository methods
6. Repository method:
   - Include org_id in SQL WHERE clause
```

### Handler Pattern
```rust
pub async fn list_toolsets_handler(
  ExtractOrgId(org_id): ExtractOrgId,
  ExtractUserId(user_id): ExtractUserId,
  State(state): State<Arc<dyn RouterState>>,
) -> Result<impl IntoResponse, ApiError> {
  let toolsets = state.app_service()
    .db_service()
    .list_toolsets(&org_id, &user_id)
    .await?;
  Ok(Json(toolsets))
}
```

---

## AppServiceBuilder Changes

### Multi-Tenant Initialization
```rust
// Pseudo-code for new initialization flow
AppServiceBuilder::new(setting_service)
  .with_db_url(DATABASE_URL)           // PostgreSQL or SQLite
  .with_session_db_url(SESSION_DB_URL) // Optional separate session DB
  .with_cache_backend(CacheBackend::Redis(REDIS_URL))  // or InMemory
  .build()
  .await?;
```

### Service Dependency Changes
```
Before:
  SecretService → encrypted file
  DbService → SqlitePool
  SessionService → SqliteStore
  AuthService → reads from SettingService + SecretService

After:
  DbService → AnyPool (SQLite or PostgreSQL)
  SessionService → PostgresStore or SqliteStore
  CacheService → Redis or Moka (generic trait)
  AuthService → receives credentials via OrgContext (no SecretService)
  OrgContext → resolved from CacheService → backed by organizations table
```
