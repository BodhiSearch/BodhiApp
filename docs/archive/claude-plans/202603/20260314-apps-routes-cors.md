# Per-Group CORS + /apps/ API Prefix for External Apps

## Context

Session-only APIs share a permissive CorsLayer with all routes. The initial CORS split (commit 955fb5a85) created `overlapping_session_apis` to handle 3 paths with both session and non-session methods. This plan resolves the tech debt by creating dedicated `/bodhi/v1/apps/...` endpoints for external OAuth apps, eliminating all path overlaps and the `overlapping_session_apis` group entirely.

**Outcome**: All session-only endpoints have restrictive CORS (no cross-origin access). All external-app endpoints live under `/bodhi/v1/apps/...` with permissive CORS. Zero path overlaps between CORS groups.

## External App API Surface (all under /apps/)

```
GET  /bodhi/v1/apps/toolsets                              → list toolsets
POST /bodhi/v1/apps/toolsets/{id}/tools/{tool_name}/execute → execute tool
GET  /bodhi/v1/apps/mcps                                  → list MCPs
GET  /bodhi/v1/apps/mcps/{id}                             → show MCP details
POST /bodhi/v1/apps/mcps/{id}/tools/refresh               → refresh tools
POST /bodhi/v1/apps/mcps/{id}/tools/{tool_name}/execute   → execute tool
```

Each uses the same handler as the original endpoint (handlers already check AuthContext for session vs external app).

## Files to Modify

| File | Change |
|------|--------|
| `crates/routes_app/src/shared/openapi.rs` | New constants, new tag, register new handlers |
| `crates/routes_app/src/shared/constants.rs` | New `API_TAG_APPS` constant |
| `crates/routes_app/src/toolsets/routes_toolsets.rs` | Thin wrapper handlers for /apps/ toolset endpoints |
| `crates/routes_app/src/mcps/routes_mcps.rs` | Thin wrapper handlers for /apps/ MCP endpoints |
| `crates/routes_app/src/routes.rs` | Route restructuring (see Step 4) |
| `crates/routes_app/src/test_cors.rs` | Update CORS tests |
| `crates/routes_app/TECHDEBT.md` | Remove resolved overlapping CORS section |
| `crates/lib_bodhiserver_napi/tests-js/specs/toolsets/toolsets-auth-restrictions.spec.mjs` | Use /apps/ paths |
| `crates/lib_bodhiserver_napi/tests-js/specs/mcps/mcps-auth-restrictions.spec.mjs` | Use /apps/ paths |
| `crates/lib_bodhiserver_napi/tests-js/specs/mcps/mcps-header-auth.spec.mjs` | Use /apps/ paths |
| `crates/lib_bodhiserver_napi/tests-js/specs/mcps/mcps-oauth-auth.spec.mjs` | Use /apps/ paths |
| `crates/lib_bodhiserver_napi/tests-js/specs/mcps/mcps-oauth-dcr.spec.mjs` | Use /apps/ paths |

## Implementation Steps

### Step 1: New endpoint constants (`openapi.rs`)

```rust
make_ui_endpoint!(ENDPOINT_APPS_TOOLSETS, "apps/toolsets");
make_ui_endpoint!(ENDPOINT_APPS_MCPS, "apps/mcps");
```

### Step 2: New API tag (`constants.rs`)

```rust
pub const API_TAG_APPS: &str = "apps";
```

Add to `BodhiOpenAPIDoc` tags:
```rust
(name = API_TAG_APPS, description = "External app API endpoints (OAuth token required)")
```

### Step 3: Thin wrapper handlers

**In `routes_toolsets.rs`** — 2 wrappers:

```rust
/// List toolsets accessible to the authenticated external app
#[utoipa::path(
  get,
  path = ENDPOINT_APPS_TOOLSETS,
  tag = API_TAG_APPS,
  operation_id = "appsListToolsets",
  responses((status = 200, body = ListToolsetsResponse)),
  security(("oauth2_token_exchange" = []))
)]
pub async fn apps_toolsets_index(auth_scope: AuthScope) -> Result<Json<ListToolsetsResponse>, ApiError> {
    toolsets_index(auth_scope).await
}

/// Execute a tool on a toolset via external app
#[utoipa::path(
  post,
  path = "/bodhi/v1/apps/toolsets/{id}/tools/{tool_name}/execute",
  tag = API_TAG_APPS,
  operation_id = "appsExecuteToolset",
  responses((status = 200, body = ToolsetExecutionResponse)),
  security(("oauth2_token_exchange" = []))
)]
pub async fn apps_toolsets_execute(
    auth_scope: AuthScope,
    path: Path<(String, String)>,
    json: Json<ExecuteToolsetRequest>,
) -> Result<Json<ToolsetExecutionResponse>, ApiError> {
    toolsets_execute(auth_scope, path, json).await
}
```

**In `routes_mcps.rs`** — 4 wrappers:

```rust
pub async fn apps_mcps_index(...) -> ... { mcps_index(...).await }
pub async fn apps_mcps_show(...) -> ... { mcps_show(...).await }
pub async fn apps_mcps_refresh_tools(...) -> ... { mcps_refresh_tools(...).await }
pub async fn apps_mcps_execute_tool(...) -> ... { mcps_execute_tool(...).await }
```

Each with `#[utoipa::path]` using `/bodhi/v1/apps/mcps/...` paths, `tag = API_TAG_APPS`.

Register all 6 new handlers in `BodhiOpenAPIDoc` `paths(...)`.

### Step 4: Route restructuring (`routes.rs`)

**Remove these groups:**
- `overlapping_session_apis` — deleted entirely
- `user_oauth_apis` — deleted entirely

**Move routes to `user_session_apis`:**
- `POST ENDPOINT_TOOLSETS` (toolsets_create) — from overlapping_session_apis
- `POST ENDPOINT_MCPS` (mcps_create) — from overlapping_session_apis
- `PUT/DELETE ENDPOINT_MCPS/{id}` (mcps_update, mcps_destroy) — from overlapping_session_apis
- `GET ENDPOINT_TOOLSETS` (toolsets_index) — from user_oauth_apis
- `GET ENDPOINT_MCPS` (mcps_index) — from user_oauth_apis
- `GET ENDPOINT_MCPS/{id}` (mcps_show) — from mcp_exec_apis

**Create new `apps_apis` group:**

```rust
// Apps list endpoints (handler filters by access_request internally)
let apps_list_apis = Router::new()
    .route(ENDPOINT_APPS_TOOLSETS, get(apps_toolsets_index))
    .route(ENDPOINT_APPS_MCPS, get(apps_mcps_index));

// Apps toolset exec (with ToolsetAccessRequestValidator)
let toolset_validator: Arc<dyn AccessRequestValidator> = Arc::new(ToolsetAccessRequestValidator);
let apps_toolset_exec = Router::new()
    .route(
      &format!("{ENDPOINT_APPS_TOOLSETS}/{{id}}/tools/{{tool_name}}/execute"),
      post(apps_toolsets_execute),
    )
    .route_layer(from_fn_with_state(state.clone(), move |state, req, next| {
        let v = toolset_validator.clone();
        access_request_auth_middleware(v, state, req, next)
    }));

// Apps MCP show + exec (with McpAccessRequestValidator)
let mcp_validator: Arc<dyn AccessRequestValidator> = Arc::new(McpAccessRequestValidator);
let apps_mcp_exec = Router::new()
    .route(&format!("{ENDPOINT_APPS_MCPS}/{{id}}"), get(apps_mcps_show))
    .route(&format!("{ENDPOINT_APPS_MCPS}/{{id}}/tools/refresh"), post(apps_mcps_refresh_tools))
    .route(
      &format!("{ENDPOINT_APPS_MCPS}/{{id}}/tools/{{tool_name}}/execute"),
      post(apps_mcps_execute_tool),
    )
    .route_layer(from_fn_with_state(state.clone(), move |state, req, next| {
        let v = mcp_validator.clone();
        access_request_auth_middleware(v, state, req, next)
    }));

// Combine all apps APIs with OAuth-accepting auth
let apps_apis = Router::new()
    .merge(apps_list_apis)
    .merge(apps_toolset_exec)
    .merge(apps_mcp_exec)
    .route_layer(from_fn_with_state(state.clone(),
      move |state, req, next| api_auth_middleware(ResourceRole::User, None, Some(UserScope::User), state, req, next),
    ));
```

**Updated router composition:**

```rust
// Session-protected (restrictive CORS)
let session_protected = Router::new()
    .merge(user_session_apis)           // expanded with list/create/show routes
    .merge(power_user_session_apis)
    .merge(admin_session_apis)
    .merge(manager_session_apis)
    .route_layer(from_fn_with_state(state.clone(), auth_middleware))
    .layer(restrictive_cors());

// API-protected (permissive CORS)
let api_protected = Router::new()
    .merge(user_apis)                   // OpenAI/Ollama compatible
    .merge(power_user_apis)             // model management
    .merge(toolset_exec_apis)           // original exec (dual-purpose, no CORS issue)
    .merge(mcp_exec_apis)              // original exec minus GET /mcps/{id}
    .merge(apps_apis)                   // NEW: all /apps/ endpoints
    .route_layer(from_fn_with_state(state.clone(), auth_middleware))
    .layer(permissive_cors());

// Public + optional auth (permissive CORS)
let public_router = Router::new()
    .merge(public_apis)
    .merge(optional_auth)
    .layer(permissive_cors());

// Final router — NO global CorsLayer
let router = Router::<Arc<dyn AppService>>::new()
    .merge(public_router)
    .merge(session_protected)
    .merge(api_protected)
    .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi))
    .with_state(state);
```

**No cross-group path overlaps**: All `/bodhi/v1/toolsets` and `/bodhi/v1/mcps` CRUD methods are in `session_protected`. All `/bodhi/v1/apps/...` paths are in `api_protected`. Exec paths (`/toolsets/{id}/tools/.../execute`, `/mcps/{id}/tools/...`) are unique to `api_protected`.

### Step 5: Update CORS tests (`test_cors.rs`)

Update test cases:
- Remove overlapping path tests (POST /toolsets permissive, POST /mcps permissive)
- Add: POST /toolsets → restrictive (now session-only)
- Add: POST /mcps → restrictive (now session-only)
- Add: GET /apps/toolsets → permissive
- Add: GET /apps/mcps → permissive
- Add: POST /apps/toolsets/{id}/tools/{tool}/execute → permissive

### Step 6: Update E2E tests (5 files)

All external app REST client calls update from original paths to `/apps/` paths:

| File | Change |
|------|--------|
| `toolsets-auth-restrictions.spec.mjs` | `GET /toolsets` → `GET /apps/toolsets`; `POST .../execute` → `POST /apps/.../execute` |
| `mcps-auth-restrictions.spec.mjs` | `GET /mcps` → `GET /apps/mcps`; `GET /mcps/{id}` → `GET /apps/mcps/{id}`; exec paths → `/apps/...` |
| `mcps-header-auth.spec.mjs` | Same pattern: list, show, refresh, execute → `/apps/` |
| `mcps-oauth-auth.spec.mjs` | `GET /mcps/{id}` → `GET /apps/mcps/{id}`; execute → `/apps/...` |
| `mcps-oauth-dcr.spec.mjs` | `GET /mcps/{id}` → `GET /apps/mcps/{id}`; execute → `/apps/...` |

### Step 7: Update TECHDEBT.md

Remove the "Overlapping CORS path structure" section (resolved by this change).

### Step 8: Regenerate OpenAPI + TypeScript client

```bash
cargo run --package xtask openapi && cd ts-client && npm run generate
```

## Verification

1. `cargo check -p routes_app` — compilation
2. `cargo test -p routes_app` — all routes_app tests including CORS tests
3. `make test.backend` — full backend regression
4. `make build.ui-rebuild` — rebuild embedded UI for E2E
5. E2E tests: `cd crates/lib_bodhiserver_napi && npm run test:playwright` — verify external app tests pass with /apps/ paths
6. Manual curl:
   - `curl -X OPTIONS -H "Origin: https://evil.com" -H "Access-Control-Request-Method: POST" http://localhost:1135/bodhi/v1/toolsets -v` → no ACAO header
   - `curl -X OPTIONS -H "Origin: https://example.com" -H "Access-Control-Request-Method: GET" http://localhost:1135/bodhi/v1/apps/toolsets -v` → `Access-Control-Allow-Origin: *`
