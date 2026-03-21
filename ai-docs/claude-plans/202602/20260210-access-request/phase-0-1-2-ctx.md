# Phase 0-1-2 Context: Access Request Implementation

## User Requirements Summary

### Authentication & Token Management
- **User token source**: ACCESS_TOKEN from user session (existing infrastructure)
- **Review page auth**: Protected route requiring login session with redirect flow
  - Not logged in → redirect to login with return URL stored in browser session
  - After login at `/ui/auth/callback`, check session for redirect URL
  - Use redirect URL instead of default `/ui/chat` from backend

### Error Handling & Retry Logic
- **409 Conflict from KC**: Mark status as `'failed'` with error message, app client polls and creates new draft request
- **Network errors**: Keep status as `'draft'`, user can retry approval
- **200 OK (idempotent retry)**: Treat as success, proceed normally

### URL & Routing
- **Review URL format**: `{frontend_url}/ui/apps/request-access/review?id={id}`
  - Frontend is compiled Next.js app, cannot have dynamic path params
  - Use `/ui/` prefix following API pattern `/bodhi/v1/`
  - Query param instead of path param
- **Frontend URL source**: Environment variable `FRONTEND_URL`
- **Polling endpoint**: `GET /apps/request-access?id={id}` (query param, not path param)

### Flow Behavior
- **Redirect flow**: After approval/denial, HTTP redirect to `redirect_uri`. Opener queries `/apps/request-access?id={id}` for status
- **Popup flow**: Popup window closes on approval/denial. Opener has callback for window close event, queries `/apps/request-access?id={id}` for status

### Service & Data Design
- **Service architecture**: New `AccessRequestService` with trait + `DefaultAccessRequestService`
- **Service dependencies**: Inject `DbService`, `AuthService`, `ToolService` (all three)
- **Service methods**: All async, return `Result<AppAccessRequestRow>`
- **UUID generation**: Service layer generates UUID on draft creation
- **Tool type validation**: Skip validation, accept any string (validate at approval time if needed)
- **Consent description**: Auto-generate from approved tool instance names (bullet list format)

### Database & Migrations
- **Migration split**:
  - `0010_app_client_toolset_configs_drop.{up,down}.sql` - drops old table
  - `0011_app_access_requests.{up,down}.sql` - creates new table
- **Status values**: `'draft' | 'approved' | 'denied' | 'failed'` (added `'failed'` status)
- **Indexing**: Only `status` and `app_client_id` indexes (no additional indexes)
- **Expiry handling**: Check `expires_at` at read time, return 410 Gone if expired
- **Migration execution**: Auto-runs via `sqlx::migrate!` at startup

### Scope & Response Handling
- **Two-phase polling**:
  - POST creates draft, returns review URL with empty/placeholder scopes
  - After user approval, KC registration happens
  - App polls `GET /apps/request-access?id={id}` until status=approved and scopes populated
- **Polling response**: Full `AppAccessRequestRow` data (all fields)

### Testing Strategy
- **HTTP mocking**: Mock `AuthService` itself, not HTTP endpoints (no mockito for KC)
- **Test timing**: Implement tests alongside service code (TDD approach)
- **Repository tests**: In-memory SQLite database
- **Service return types**: Return `Result<AppAccessRequestRow>` with updated data

### Code Removal
- **Migration strategy**: Clean cutover, remove old code immediately
- **Safety checks**: Grep for method names across all crates before deletion, check test files explicitly

---

## Codebase Patterns Reference

### Migration Patterns

**Next available number**: `0010` for old table drop, `0011` for new table creation

**Naming pattern**: `NNNN_description-with-hyphens.{up,down}.sql`

**JSON column storage**:
```sql
tools_requested TEXT NOT NULL DEFAULT '[]',  -- JSON array
tools_approved TEXT DEFAULT NULL,            -- JSON array (nullable until approved)
```

**Timestamp storage**:
```sql
created_at INTEGER NOT NULL,  -- Unix timestamp as i64
updated_at INTEGER NOT NULL,
expires_at INTEGER NOT NULL,
```

**Conversion in Rust**:
```rust
// Store: DateTime<Utc> -> i64
.bind(request.created_at.timestamp())

// Read: i64 -> DateTime<Utc>
created_at: DateTime::<Utc>::from_timestamp(created_at, 0).unwrap_or_default()
```

### Domain Object Patterns

**Enum with serialization** (from `resource_role.rs`):
```rust
#[derive(
  Debug, Clone, Copy, PartialEq, Eq,
  strum::Display, strum::EnumIter,
  Serialize, Deserialize, ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum AppAccessRequestStatus {
  Draft,
  Approved,
  Denied,
  Failed,  // New status
}

impl FromStr for AppAccessRequestStatus {
  type Err = AccessRequestError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "draft" => Ok(Self::Draft),
      "approved" => Ok(Self::Approved),
      "denied" => Ok(Self::Denied),
      "failed" => Ok(Self::Failed),
      _ => Err(AccessRequestError::InvalidStatus(s.to_string())),
    }
  }
}
```

**Database row type** (in `services/src/db/objs.rs`):
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct AppAccessRequestRow {
  pub id: String,              // UUID (access_request_id)
  pub app_client_id: String,
  pub flow_type: String,       // "redirect" | "popup"
  pub redirect_uri: Option<String>,
  pub status: String,          // "draft" | "approved" | "denied" | "failed"
  pub tools_requested: String, // JSON array
  pub tools_approved: Option<String>, // JSON array
  pub user_id: Option<String>,
  pub resource_scope: Option<String>,
  pub access_request_scope: Option<String>,
  pub error_message: Option<String>,  // New: store error details for 'failed' status
  pub expires_at: i64,         // Unix timestamp
  pub created_at: i64,
  pub updated_at: i64,
}
```

### Service Layer Patterns

**Service trait definition**:
```rust
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait]
pub trait AccessRequestService: Send + Sync + std::fmt::Debug {
  async fn create_draft(
    &self,
    app_client_id: String,
    flow_type: String,
    redirect_uri: Option<String>,
    tools_requested: Vec<String>,
  ) -> Result<AppAccessRequestRow, AccessRequestError>;

  async fn get_request(&self, id: &str) -> Result<Option<AppAccessRequestRow>, AccessRequestError>;

  async fn approve_request(
    &self,
    id: &str,
    user_id: &str,
    user_token: &str,
    tools_approved: Vec<String>,
  ) -> Result<AppAccessRequestRow, AccessRequestError>;

  async fn deny_request(
    &self,
    id: &str,
    user_id: &str,
  ) -> Result<AppAccessRequestRow, AccessRequestError>;
}
```

**Default implementation**:
```rust
#[derive(Debug)]
pub struct DefaultAccessRequestService {
  db_service: Arc<dyn DbService>,
  auth_service: Arc<dyn AuthService>,
  tool_service: Arc<dyn ToolService>,
  time_service: Arc<dyn TimeService>,
  frontend_url: String,  // From FRONTEND_URL env var
}

impl DefaultAccessRequestService {
  pub fn new(
    db_service: Arc<dyn DbService>,
    auth_service: Arc<dyn AuthService>,
    tool_service: Arc<dyn ToolService>,
    time_service: Arc<dyn TimeService>,
    frontend_url: String,
  ) -> Self {
    Self { db_service, auth_service, tool_service, time_service, frontend_url }
  }
}
```

### Repository Patterns

**Repository trait**:
```rust
#[async_trait::async_trait]
pub trait AccessRequestRepository: Send + Sync {
  async fn create(&self, row: &AppAccessRequestRow) -> Result<AppAccessRequestRow, DbError>;
  async fn get(&self, id: &str) -> Result<Option<AppAccessRequestRow>, DbError>;
  async fn update_approval(
    &self,
    id: &str,
    user_id: &str,
    tools_approved: &str,
    resource_scope: &str,
    access_request_scope: &str,
  ) -> Result<AppAccessRequestRow, DbError>;
  async fn update_denial(
    &self,
    id: &str,
    user_id: &str,
  ) -> Result<AppAccessRequestRow, DbError>;
  async fn update_failure(
    &self,
    id: &str,
    error_message: &str,
  ) -> Result<AppAccessRequestRow, DbError>;
}
```

**Implementation pattern** (in `db/service.rs`):
```rust
impl AccessRequestRepository for SqliteDbService {
  async fn create(&self, row: &AppAccessRequestRow) -> Result<AppAccessRequestRow, DbError> {
    let result = query_as::<_, (String, String, String, Option<String>, String, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, i64, i64, i64)>(
      "INSERT INTO app_access_requests
        (id, app_client_id, flow_type, redirect_uri, status, tools_requested,
         tools_approved, user_id, resource_scope, access_request_scope, error_message,
         expires_at, created_at, updated_at)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
       RETURNING id, app_client_id, flow_type, redirect_uri, status, tools_requested,
                 tools_approved, user_id, resource_scope, access_request_scope, error_message,
                 expires_at, created_at, updated_at"
    )
    .bind(&row.id)
    .bind(&row.app_client_id)
    .bind(&row.flow_type)
    .bind(&row.redirect_uri)
    .bind(&row.status)
    .bind(&row.tools_requested)
    .bind(&row.tools_approved)
    .bind(&row.user_id)
    .bind(&row.resource_scope)
    .bind(&row.access_request_scope)
    .bind(&row.error_message)
    .bind(row.expires_at)
    .bind(row.created_at)
    .bind(row.updated_at)
    .fetch_one(&self.pool)
    .await?;

    Ok(AppAccessRequestRow {
      id: result.0,
      app_client_id: result.1,
      // ... map all 14 fields
    })
  }
}
```

### Error Handling Patterns

**Service error enum**:
```rust
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum AccessRequestError {
  #[error("Access request '{0}' not found.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  NotFound(String),

  #[error("Access request '{0}' has expired.")]
  #[error_meta(error_type = ErrorType::Gone)]
  Expired(String),

  #[error("Access request '{0}' has already been processed.")]
  #[error_meta(error_type = ErrorType::Conflict)]
  AlreadyProcessed(String),

  #[error("Invalid status '{0}' for access request.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidStatus(String),

  #[error("KC registration failed: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  KcRegistrationFailed(String),

  #[error(transparent)]
  DbError(#[from] DbError),

  #[error(transparent)]
  AuthError(#[from] AuthServiceError),

  #[error(transparent)]
  ToolError(#[from] ToolsetError),
}

type Result<T> = std::result::Result<T, AccessRequestError>;
```

### Testing Patterns

**Test fixture usage**:
```rust
#[rstest]
#[awt]
#[anyhow_trace]
#[tokio::test]
async fn test_create_draft_request(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let now = db_service.now();  // Use fixture's time service
  let row = AppAccessRequestRow {
    id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
    app_client_id: "app-abc123".to_string(),
    flow_type: "redirect".to_string(),
    redirect_uri: Some("https://example.com/callback".to_string()),
    status: "draft".to_string(),
    tools_requested: r#"[{"tool_type":"builtin-exa-search"}]"#.to_string(),
    tools_approved: None,
    user_id: None,
    resource_scope: None,
    access_request_scope: None,
    error_message: None,
    expires_at: (now + Duration::minutes(10)).timestamp(),
    created_at: now.timestamp(),
    updated_at: now.timestamp(),
  };

  let result = db_service.create_app_access_request(&row).await?;
  assert_eq!(result.id, row.id);
  assert_eq!(result.status, "draft");
  Ok(())
}
```

---

## KC SPI Integration Details

### Endpoint
`POST {auth_url}/realms/{realm}/bodhi/users/request-access`

### Authentication
Bearer token = user's ACCESS_TOKEN from resource client session (NOT service account token)

### Request Body
```json
{
  "app_client_id": "app-abc123def456",
  "access_request_id": "550e8400-e29b-41d4-a716-446655440000",
  "description": "- Exa Web Search\n- OpenAI GPT-4"
}
```

### Response (201 Created - first registration)
```json
{
  "scope": "scope_resource-xyz789abc",
  "access_request_id": "550e8400-e29b-41d4-a716-446655440000",
  "access_request_scope": "scope_access_request:550e8400-e29b-41d4-a716-446655440000"
}
```

### Response (200 OK - idempotent retry)
Same as 201, treat as success

### Error Responses
- **409 Conflict**: UUID collision (different context) → Mark status as `'failed'`, store error message
- **401 Unauthorized**: Invalid/expired user token → Bubble up as auth error
- **400 Bad Request**: Validation failure → Bubble up as bad request error

### AuthService Method
```rust
async fn register_access_request_consent(
  &self,
  user_token: &str,
  app_client_id: &str,
  access_request_id: &str,
  description: &str,
) -> Result<RegisterAccessRequestResponse, AuthServiceError>;

pub struct RegisterAccessRequestResponse {
  pub scope: String,
  pub access_request_id: String,
  pub access_request_scope: String,
}
```

---

## File Paths Reference

### Migrations
- `crates/services/migrations/0010_app_client_toolset_configs_drop.up.sql`
- `crates/services/migrations/0010_app_client_toolset_configs_drop.down.sql`
- `crates/services/migrations/0011_app_access_requests.up.sql`
- `crates/services/migrations/0011_app_access_requests.down.sql`

### Domain Objects
- `crates/objs/src/access_request.rs` - Enums and types
- `crates/objs/src/lib.rs` - Re-exports

### Database Layer
- `crates/services/src/db/access_request_repository.rs` - Repository trait + impl
- `crates/services/src/db/objs.rs` - Row struct
- `crates/services/src/db/service.rs` - Add trait to DbService
- `crates/services/src/db/mod.rs` - Register module

### Service Layer
- `crates/services/src/access_request_service.rs` - New service trait + impl
- `crates/services/src/auth_service.rs` - Add KC registration method
- `crates/services/src/lib.rs` - Export new service

### App Service
- `crates/services/src/app_service.rs` - Add AccessRequestService getter

### Tests
- `crates/services/src/db/tests/access_request_tests.rs` - Repository tests
- `crates/services/src/tests/access_request_service_tests.rs` - Service tests

### Code to Remove (Phase 2)
- `crates/services/src/auth_service.rs`:
  - Remove `request_access()` method
  - Remove `RequestAccessRequest`/`RequestAccessResponse` structs
- `crates/services/src/tool_service/service.rs`:
  - Remove `is_app_client_registered_for_toolset()` method
- `crates/services/src/db/toolset_repository.rs`:
  - Remove `get_app_client_toolset_config()` method
  - Remove `upsert_app_client_toolset_config()` method
- `crates/services/src/db/objs.rs`:
  - Remove `AppClientToolsetConfigRow` struct
- `crates/services/src/lib.rs`:
  - Remove exports for deleted types
