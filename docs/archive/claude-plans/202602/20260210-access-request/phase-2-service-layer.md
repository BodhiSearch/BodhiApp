# Phase 2: Service Layer — DB Repository + AuthService Extension

## Purpose

Implement database access layer for access requests and extend AuthService to call Keycloak's consent registration SPI.

## Dependencies

- **Phase 1**: Database migration and domain objects created

## Key Changes

### 1. Access Request Repository

**Create**: `crates/services/src/db/access_request_repository.rs`

```rust
#[async_trait]
pub trait AccessRequestRepository {
    async fn create_app_access_request(&self, row: &AppAccessRequestRow) -> Result<()>;
    async fn get_app_access_request(&self, id: &str) -> Result<Option<AppAccessRequestRow>>;
    async fn update_app_access_request_approval(
        &self,
        id: &str,
        user_id: &str,
        tools_approved: &str,       // JSON
        resource_scope: &str,        // KC-returned scope
        access_request_scope: &str,  // KC-returned scope
    ) -> Result<()>;
    async fn update_app_access_request_denial(
        &self,
        id: &str,
        user_id: &str,
    ) -> Result<()>;
}
```

**Implement in**: `crates/services/src/db/service.rs`

Add repository trait to `DbService` and implement in `DefaultDbService` following existing patterns (see `toolset_repository.rs`, `user_repository.rs`).

### 2. AuthService Extension

**Modify**: `crates/services/src/auth_service.rs`

Add method to `AuthService` trait:
```rust
async fn register_access_request_consent(
    &self,
    user_token: &str,           // User's access token from resource client session
    app_client_id: &str,
    access_request_id: &str,
    description: &str,
) -> Result<RegisterAccessRequestResponse>;

pub struct RegisterAccessRequestResponse {
    pub scope: String,                  // "scope_resource-xyz789abc"
    pub access_request_id: String,      // echo back
    pub access_request_scope: String,   // "scope_access_request:<uuid>"
}
```

**Implementation** (in `KeycloakAuthService`):
- Call `POST {auth_url}/realms/{realm}/bodhi/users/request-access`
- Authorization: `Bearer {user_token}` (NOT service account token)
- Request body: `{ app_client_id, access_request_id, description }`
- Parse response:
  - 201 Created → first registration
  - 200 OK → idempotent retry
  - 409 Conflict → UUID collision (abort, regenerate UUID) — surface as error
  - 401 → user token invalid/expired (surface as auth error)
  - 400 → validation failure (surface as bad request)

### 3. Remove Old Code

**Modify**: `crates/services/src/auth_service.rs`
- Remove `request_access()` method from trait and implementation
- Remove `RequestAccessRequest`/`RequestAccessResponse` structs

**Modify**: `crates/services/src/tool_service/service.rs`
- Remove `is_app_client_registered_for_toolset()` method

**Modify**: `crates/services/src/db/toolset_repository.rs`
- Remove `get_app_client_toolset_config()` method
- Remove `upsert_app_client_toolset_config()` method

**Modify**: `crates/services/src/db/objs.rs`
- Remove `AppClientToolsetConfigRow` struct

**Modify**: `crates/services/src/lib.rs`
- Remove exports for deleted types

## Files to Create/Modify

| File | Action | Description |
|------|--------|-------------|
| `crates/services/src/db/access_request_repository.rs` | Create | New repository trait + impl |
| `crates/services/src/db/service.rs` | Modify | Add repository to DbService |
| `crates/services/src/db/mod.rs` | Modify | Register new module |
| `crates/services/src/auth_service.rs` | Modify | Add consent registration, remove old |
| `crates/services/src/tool_service/service.rs` | Modify | Remove old registration check |
| `crates/services/src/db/toolset_repository.rs` | Modify | Remove old app_client methods |
| `crates/services/src/db/objs.rs` | Modify | Remove old row types |
| `crates/services/src/lib.rs` | Modify | Update exports |

## Research Questions

1. **SQL patterns**: How do existing repositories handle JSON columns? (Check `toolset_repository.rs`)
2. **Error handling**: What error types should we return? (Check existing repository error patterns)
3. **Timestamps**: Should we use `TimeService` for `created_at`/`updated_at`? (See MEMORY.md note)
4. **Transaction handling**: Do we need transactions for multi-step updates? (Check existing patterns)
5. **HTTP client**: How does `KeycloakAuthService` make HTTP calls? (Check existing methods like token exchange)
6. **Mock layer**: How are AuthService methods mocked? (Check `crates/services/src/auth_service.rs` for mockall patterns)

## Acceptance Criteria

### Repository Layer
- [ ] `AccessRequestRepository` trait defined with all CRUD methods
- [ ] Implementation in `DefaultDbService` follows existing patterns
- [ ] Queries use parameterized statements (no SQL injection risk)
- [ ] JSON serialization/deserialization works for `tools_requested`/`tools_approved`
- [ ] Timestamps handled consistently with other repositories

### AuthService Extension
- [ ] `register_access_request_consent` method added to trait
- [ ] Implementation calls correct KC endpoint with user token
- [ ] Request body matches KC spec
- [ ] Response parsing handles 201/200/409/401/400 correctly
- [ ] 409 Conflict surfaces as distinct error (for UUID regeneration)
- [ ] Mock implementation added for testing

### Code Removal
- [ ] Old `request_access` method removed from AuthService
- [ ] Old `is_app_client_registered_for_toolset` removed from ToolService
- [ ] Old `app_client_toolset_configs` repository methods removed
- [ ] Old row types and exports cleaned up
- [ ] All references to old types removed (search codebase)

### Testing
- [ ] Unit tests for repository CRUD operations
- [ ] Test JSON serialization/deserialization
- [ ] Mock test for `register_access_request_consent` (using mockito)
- [ ] Test error handling (401, 400, 409 responses)
- [ ] `cargo test -p services` passes

## Notes for Sub-Agent

- **Follow existing patterns**: Look at `toolset_repository.rs` and `user_repository.rs` for SQL patterns
- **TimeService**: Pass timestamps via constructors (see MEMORY.md)
- **Mockito**: Use mockito for KC HTTP endpoint mocking (see existing auth_service tests)
- **Error types**: Check if we need new error variants or can reuse existing ones
- **SQL placeholders**: Use `?` for SQLite parameterized queries
- **JSON**: Use `serde_json` for serialization — check existing examples
- **HTTP client**: `KeycloakAuthService` likely uses `reqwest` — follow existing patterns

## Next Phase

Phase 3 will use these services to implement the API endpoints.
