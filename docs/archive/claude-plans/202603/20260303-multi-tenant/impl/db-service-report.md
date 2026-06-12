# Direct DbService Access Report

## Summary

Found **19 direct `auth_scope.db()` calls** in route handlers across 4 files, plus **2 auth-scoped service methods** that bypass their domain service and call `db_service()` directly.

## Route Handlers with Direct DB Access

### 1. `routes_api_models.rs` — API Models (8+ calls, no domain service exists)

| Handler | DB Methods Called |
|---------|-----------------|
| `api_models_index` | `list_api_model_aliases()` |
| `api_models_show` | `get_api_model_alias()`, `get_api_key_for_alias()` |
| `api_models_create` | `create_api_model_alias()` |
| `api_models_update` | `get_api_model_alias()`, `update_api_model_alias()`, `get_api_key_for_alias()` |
| `api_models_destroy` | `get_api_model_alias()`, `delete_api_model_alias()` |
| `api_models_test` | `get_api_model_alias()`, `get_api_key_for_alias()` |
| `api_models_fetch_models` | `get_api_model_alias()`, `get_api_key_for_alias()` |
| `api_models_sync` | `get_api_model_alias()`, `get_api_key_for_alias()`, `update_api_model_cache()` |
| `spawn_cache_refresh` (bg) | `get_api_model_alias()`, `get_api_key_for_alias()`, `update_api_model_cache()` |

**Missing service**: No `ApiModelService` or `AuthScopedApiModelService` exists. All business logic (ID generation, timestamp management, cache refresh spawning) lives in route handlers.

### 2. `routes_users_access_request.rs` — Access Requests (6 calls, service exists but not used here)

| Handler | DB Methods Called |
|---------|-----------------|
| `users_request_access` | `get_pending_request()`, `insert_pending_request()` |
| `users_request_status` | `get_pending_request()` |
| `users_access_requests_pending` | `list_pending_requests()` |
| `users_access_requests_index` | `list_all_requests()` |
| `users_access_request_approve` | `get_request_by_id()`, `update_request_status()` |
| `users_access_request_reject` | `update_request_status()` |

**Note**: `AccessRequestService` trait exists in services but handlers bypass it.

### 3. `routes_models_pull.rs` — Download Requests (4 calls, no domain service exists)

| Handler | DB Methods Called |
|---------|-----------------|
| `models_pull_index` | `list_download_requests()` |
| `models_pull_create` | `find_download_request_by_repo_filename()`, `create_download_request()` |
| `models_pull_show` | `get_download_request()` |
| `update_download_status` (bg) | `get_download_request()`, `update_download_request()` |

**Missing service**: No `DownloadService` exists.

### 4. `routes_dev.rs` — Dev endpoint (1 call, intentionally direct)

| Handler | DB Methods Called |
|---------|-----------------|
| `dev_db_reset_handler` | `reset_all_tables()` |

**Acceptable**: Dev-only endpoint, no domain service needed.

## Auth-Scoped Services with Direct DB Access

### 1. `auth_scoped_data.rs:112` — `update_alias()`

Calls `app_service.db_service().update_user_alias()` directly instead of `app_service.data_service().update_alias()`. All other methods in this file delegate to `data_service()`.

### 2. `auth_scoped_mcps.rs:205` — `delete_oauth_token()`

Calls `app_service.db_service().delete_mcp_oauth_token()` directly instead of `app_service.mcp_service().delete_oauth_token()`. All other MCP methods delegate to `mcp_service()`.

## Domains Missing a Service Layer

| Domain | Has Service? | Current Pattern |
|--------|:---:|---|
| API Models | No | Handler → DbService directly |
| Download Requests | No | Handler → DbService directly |
| Access Requests | Partial | Service exists but handlers bypass it |
| MCP OAuth Tokens | Partial | One method bypasses McpService |
| User Aliases | Partial | One method bypasses DataService |
| Tokens | Yes | Handler → AuthScopedTokenService → TokenService → DbService |
| MCPs (instances) | Yes | Handler → AuthScopedMcpService → McpService → DbService |
| Toolsets | Yes | Handler → AuthScopedToolService → ToolService → DbService |
| Settings | Yes | Handler → SettingService → DbService |

## Recommendation

Create domain services for API Models and Download Requests. Fix the 2 bypass instances in auth-scoped services. This centralizes business logic (ID generation, timestamps, validation, side effects) out of route handlers.
