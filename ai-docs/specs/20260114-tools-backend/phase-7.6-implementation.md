---
name: Phase 7.6 Tool OAuth Fix
status: ✅ COMPLETE
completed: 2026-01-16
overview: Fix Phase 7.5 incorrect Keycloak integration and implement proper external app tool access flow with scope-based authorization, including token exchange scope preservation, app-client tool configuration caching, and updated Keycloak contract. Keycloak extension already implemented and deployed to main-id.getbodhi.app.
todos:
  - id: phase76-keycloak-spec
    content: Update 09-keycloak-extension-contract.md - finalize with implementation details (400 for app not found, status to Implemented)
    status: completed
  - id: phase76-spec
    content: Create spec file 05.6-external-app-tool-access.md and update overview/phases docs
    status: completed
  - id: phase76-mark-incorrect
    content: Review specs (05.5, 05, 08-phases) and strikethrough incorrect Keycloak integration info, add inline corrections or note to see 05.6
    status: completed
  - id: phase76-remove-kc
    content: Remove enable_tool_scope/disable_tool_scope from AuthService, update tool_service to remove Keycloak calls
    status: completed
  - id: phase76-db
    content: Add app_client_tool_configs table migration and DbService CRUD
    status: completed
  - id: phase76-token-exchange
    content: Update token_service to preserve scope_tool-* through exchange
    status: completed
  - id: phase76-header
    content: Add KEY_HEADER_BODHIAPP_TOOL_SCOPES and KEY_HEADER_BODHIAPP_AZP, inject after token exchange
    status: completed
  - id: phase76-request-access
    content: Update /apps/request-access endpoint with caching and new response format
    status: completed
  - id: phase76-middleware
    content: Rewrite tool_auth_middleware with full OAuth authorization logic
    status: completed
  - id: phase76-tests
    content: Add/update tests for all new functionality
    status: completed
---

# Phase 7.6: External App Tool Access and OAuth Scope Fix

## Problem Statement

Phase 7.5 implemented incorrect Keycloak integration:

- Incorrectly added/removed client scopes on the resource-client (BodhiApp) instead of app-client
- Did not preserve `scope_tool-*` through token exchange
- Did not support external app tool authorization via OAuth scopes

## Keycloak Implementation Status

**Keycloak extension has been implemented and deployed** to `main-id.getbodhi.app`.

Key implementation details from Keycloak:

- Response returns 400 (not 404) when app-client not found
- Attributes use dot notation: `bodhi.tools`, `bodhi.config_version`
- Additional `bodhi.client_type` validation (resource/app) - implementation detail
- Tool management endpoints added for developer console (not used by BodhiApp)

## Corrected Architecture

```mermaid
sequenceDiagram
    participant App as ExternalApp
    participant Bodhi as BodhiApp
    participant KC as Keycloak
    participant DB as SQLite

    Note over App,KC: One-time Setup
    App->>Bodhi: POST /apps/request-access
    Note right of App: { app_client_id, version? }
    
    alt version matches DB cache
        Bodhi->>DB: Lookup cached config
        DB-->>Bodhi: { scope, tools, version }
    else no cache or version mismatch
        Bodhi->>KC: POST /resources/request-access
        Note right of Bodhi: { app_client_id }
        KC-->>Bodhi: { scope, tools, app_client_config_version }
        Bodhi->>DB: Store/update config
    end
    
    Bodhi-->>App: { scope, tools, app_client_config_version }

    Note over App,KC: OAuth Flow
    App->>KC: Auth request with scope_resource + scope_tool-*
    KC-->>App: Token with azp=app-client, aud=resource-client
    
    Note over App,Bodhi: Tool Execution
    App->>Bodhi: POST /tools/{id}/execute + Bearer token
    Bodhi->>Bodhi: Validate aud, exchange token
    Note right of Bodhi: Preserve scope_tool-* in exchange
    Bodhi->>Bodhi: Check app_tool_configs (admin enabled)
    Bodhi->>Bodhi: Check app_client_tool_configs (app registered)
    Bodhi->>Bodhi: Check scope_tool-* in token
    Bodhi->>Bodhi: Check user_tool_configs (has API key)
    Bodhi-->>App: Tool result
```

## Changes Required

### 1. Remove Incorrect Keycloak Integration

**Files to modify:**

- [crates/services/src/auth_service.rs](crates/services/src/auth_service.rs): Remove `enable_tool_scope()` and `disable_tool_scope()` methods from trait and implementation
- [crates/services/src/tool_service.rs](crates/services/src/tool_service.rs): Remove `set_app_tool_enabled()` Keycloak calls, keep only DB update
- [crates/routes_app/src/routes_tools.rs](crates/routes_app/src/routes_tools.rs): Update `enable_app_tool`/`disable_app_tool` to not call Keycloak

### 2. New Database Table: app_client_tool_configs

**File:** [crates/services/migrations/0008_app_client_tool_configs.up.sql](crates/services/migrations/0008_app_client_tool_configs.up.sql)

```sql
CREATE TABLE IF NOT EXISTS app_client_tool_configs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    app_client_id TEXT NOT NULL UNIQUE,
    config_version TEXT NOT NULL,
    tools_json TEXT NOT NULL,  -- JSON array: [{"tool_id":"...","tool_scope":"..."}]
    resource_scope TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_app_client_tool_configs_client_id ON app_client_tool_configs(app_client_id);
```

### 3. Update Token Exchange to Preserve Tool Scopes

**File:** [crates/auth_middleware/src/token_service.rs](crates/auth_middleware/src/token_service.rs)

Current code filters only `scope_user_*`:

```rust
let mut scopes: Vec<&str> = claims
  .scope
  .split_whitespace()
  .filter(|s| s.starts_with("scope_user_"))  // FIX: Also keep scope_tool-*
  .collect();
```

Change to:

```rust
let mut scopes: Vec<&str> = claims
  .scope
  .split_whitespace()
  .filter(|s| s.starts_with("scope_user_") || s.starts_with("scope_tool-"))
  .collect();
```

### 4. Add Tool Scopes Header

**File:** [crates/auth_middleware/src/lib.rs](crates/auth_middleware/src/lib.rs)

Add new header constant:

```rust
pub const KEY_HEADER_BODHIAPP_TOOL_SCOPES: &str = "X-BodhiApp-Tool-Scopes";
```

**File:** [crates/auth_middleware/src/api_auth.rs](crates/auth_middleware/src/api_auth.rs) (or equivalent)

After token exchange, extract and inject tool scopes:

```rust
// Extract tool scopes from exchanged token
let tool_scopes: Vec<&str> = scope_claims.scope
  .split_whitespace()
  .filter(|s| s.starts_with("scope_tool-"))
  .collect();

if !tool_scopes.is_empty() {
  req.headers_mut().insert(
    KEY_HEADER_BODHIAPP_TOOL_SCOPES,
    tool_scopes.join(" ").parse().unwrap()  // Space-separated (matches JWT scope format)
  );
}
```

### 5. Update /apps/request-access Endpoint

**File:** [crates/routes_app/src/routes_login.rs](crates/routes_app/src/routes_login.rs)

Update request/response types:

```rust
#[derive(Deserialize)]
pub struct AppAccessRequest {
  pub app_client_id: String,
  pub version: Option<String>,  // NEW: cache key
}

#[derive(Serialize)]
pub struct AppAccessResponse {
  pub scope: String,
  pub tools: Vec<AppClientTool>,  // NEW
  pub app_client_config_version: String,  // NEW
}

#[derive(Serialize, Deserialize)]
pub struct AppClientTool {
  pub tool_id: String,
  pub tool_scope: String,
}
```

Update handler logic:

1. If `version` provided, check DB cache
2. If cache hit (version matches), return cached data
3. Otherwise, call Keycloak `/resources/request-access`
4. Store response in `app_client_tool_configs` table
5. Return response to caller

### 6. Update AuthService for New Keycloak Contract

**File:** [crates/services/src/auth_service.rs](crates/services/src/auth_service.rs)

Update `request_access` method response type:

```rust
#[derive(Deserialize)]
pub struct KeycloakRequestAccessResponse {
  pub scope: String,
  pub tools: Vec<AppClientTool>,
  pub app_client_config_version: String,
}
```

### 7. Update tool_auth_middleware

**File:** [crates/auth_middleware/src/tool_auth_middleware.rs](crates/auth_middleware/src/tool_auth_middleware.rs)

New authorization logic:

```rust
async fn _impl(...) -> Result<Response, ToolAuthError> {
  let headers = req.headers();
  let user_id = extract_user_id(headers)?;
  
  // Determine auth type
  let is_session_auth = headers.contains_key(KEY_HEADER_BODHIAPP_ROLE);
  let is_oauth_auth = headers.contains_key(KEY_HEADER_BODHIAPP_SCOPE) 
                      && !is_session_auth;

  // 1. Check app-level enabled (both auth types)
  if !tool_service.is_tool_enabled_for_app(&tool_id).await? {
    return Err(ToolError::ToolAppDisabled.into());
  }

  if is_oauth_auth {
    // 2. Check app-client registered for tool
    let azp = headers.get(KEY_HEADER_BODHIAPP_AZP)?;  // Need to add this header
    if !tool_service.is_app_client_registered_for_tool(azp, &tool_id).await? {
      return Err(ToolError::AppClientNotRegistered.into());
    }
    
    // 3. Check scope_tool-* in token
    let tool_scopes = headers.get(KEY_HEADER_BODHIAPP_TOOL_SCOPES)?;
    let required_scope = ToolScope::scope_for_tool_id(&tool_id)?;
    if !tool_scopes.contains(required_scope) {
      return Err(ToolError::MissingToolScope.into());
    }
  }

  // 4. Check user has tool configured (API key required for execution)
  if !tool_service.is_tool_available_for_user(user_id, &tool_id).await? {
    return Err(ToolError::ToolNotConfigured.into());
  }

  Ok(next.run(req).await)
}
```

### 8. Update Keycloak Contract Spec

**File:** [ai-docs/specs/20260114-tools-backend/09-keycloak-extension-contract.md](ai-docs/specs/20260114-tools-backend/09-keycloak-extension-contract.md)

Remove the `POST/DELETE /resources/tools` endpoints (incorrect).

Update `/resources/request-access` contract:

- Request: `{ app_client_id: string }`
- Response: `{ scope: string, tools: [{tool_id, tool_scope}], app_client_config_version: string }`
- Keycloak reads `bodhi.tools` and `bodhi.config_version` attributes from app-client
- Returns 400 (not 404) when app-client not found
- Adds `scope_resource-xyz` to app-client's optional scopes (existing behavior)
- Tool scopes (`scope_tool-*`) already configured on app-client by developer portal

### 9. New ToolService Methods

**File:** [crates/services/src/tool_service.rs](crates/services/src/tool_service.rs)

```rust
// Add to trait
async fn is_app_client_registered_for_tool(&self, app_client_id: &str, tool_id: &str) 
  -> Result<bool, ToolError>;
async fn get_app_client_tool_config(&self, app_client_id: &str) 
  -> Result<Option<AppClientToolConfig>, ToolError>;
async fn upsert_app_client_tool_config(&self, config: &AppClientToolConfig) 
  -> Result<AppClientToolConfig, ToolError>;
```

## Summary of Auth Flow

| Auth Type | Check 1 | Check 2 | Check 3 | Check 4 |

|-----------|---------|---------|---------|---------|

| Session/First-party | app_tool_configs (admin enabled) | - | - | user_tool_configs (enabled + API key) |

| External OAuth | app_tool_configs (admin enabled) | app_client_tool_configs (registered) | scope_tool-* in token | user_tool_configs (API key) |

## Files Summary

**New files:**

- `crates/services/migrations/0008_app_client_tool_configs.up.sql`
- `crates/services/migrations/0008_app_client_tool_configs.down.sql`

**Modified files:**

- `crates/services/src/auth_service.rs` - Remove tool scope methods, update request_access response
- `crates/services/src/tool_service.rs` - Add app-client methods, remove Keycloak calls
- `crates/services/src/db/service.rs` - Add CRUD for app_client_tool_configs
- `crates/services/src/db/objs.rs` - Add AppClientToolConfigRow
- `crates/auth_middleware/src/token_service.rs` - Preserve scope_tool-* in exchange
- `crates/auth_middleware/src/lib.rs` - Add KEY_HEADER_BODHIAPP_TOOL_SCOPES
- `crates/auth_middleware/src/tool_auth_middleware.rs` - Full auth logic rewrite
- `crates/routes_app/src/routes_login.rs` - Update /apps/request-access endpoint with caching
- `ai-docs/specs/20260114-tools-backend/09-keycloak-extension-contract.md` - Finalize with implementation details