# CRUD Uniformity — Functional Plan

**Milestone**: CRUD pattern standardization across all domains
**HEAD commit**: `3d10659e7` — "RLS first cut"
**Date**: 2026-03-04

---

## A. Request Type Consolidation

### Naming Convention
- Each domain has a single `*Request` type for both create and update:
  - `ApiModelRequest`, `UserAliasRequest`, `ToolsetRequest`, `McpRequest`, `McpServerRequest`, `NewDownloadRequest`
  - Exception: Token has `CreateTokenRequest` + `UpdateTokenRequest` (asymmetric CRUD — create returns secret, update does not)

### Location
- Request types defined in `services::*_objs` (not in route handler modules)
- Single request type per domain eliminates redundant `Create*` / `Update*` pairs

---

## B. ValidatedJson Extraction

- `ValidatedJson<T>` extractor for request body deserialization + validation in one step
- Validation happens at route boundary (extractor layer), not in service layer
- Error pipeline: `ValidationRejection` enum -> `IntoResponse` -> `ApiError`

### Key File
- `crates/routes_app/src/shared/validated_json.rs`

---

## C. Entity -> Response Pattern

- Services return Entity types (e.g., `ToolsetEntity`, `ApiModelOutput`, `TokenDetail`)
- Route handlers convert Entity -> Response via `impl From<Entity> for Response`
- Auth-scoped services pass through Entity types (thin wrappers over underlying service)

---

## D. Entity Type Aliases

### New Entity Aliases
| Entity Type | Table / Source |
|---|---|
| `ToolsetEntity` | `toolsets` |
| `McpEntity` | `mcps` |
| `McpWithServerEntity` | `mcps` joined with `mcp_servers` |
| `McpServerEntity` | `mcp_servers` |
| `McpAuthHeaderEntity` | `mcp_auth_headers` |
| `McpOAuthConfigEntity` | `mcp_oauth_configs` |
| `McpOAuthTokenEntity` | `mcp_oauth_tokens` |
| `ApiModelEntity` | `api_model_aliases` |
| `UserAliasEntity` | `user_aliases` |
| `TokenEntity` | `api_tokens` |
| `DownloadRequestEntity` | `download_requests` |
| `UserAccessRequestEntity` | `user_access_requests` |
| `AppAccessRequest` | `app_access_requests` |
| `ModelMetadataEntity` | `model_metadata` |

### Table & Type Renames
- Table rename: `access_requests` -> `user_access_requests`
- `AppAccessRequestRow` -> `AppAccessRequest`
- `DownloadRequest` (type alias) -> `DownloadRequestEntity`

---

## E. Type Consolidation into services

Types consolidated into service-layer `*_objs` modules (some moved from routes_app, some new):

### API Models (`services::models::model_objs`)
- `ApiKey`, `ApiKeyUpdate`, `RawApiKeyUpdate`, `TestCreds`, `TestPromptRequest`, `TestPromptResponse`
- `FetchModelsRequest`, `FetchModelsResponse`
- `ApiModelOutput`, `PaginatedApiModelOutput`, `ApiFormatsResponse`
- `ApiModelRequest`, `UserAliasRequest`

### Models / Aliases (`services::models::model_objs`)
- `UserAliasResponse`, `ModelAliasResponse`, `ApiAliasResponse`, `AliasResponse`
- `CopyAliasRequest`, `RefreshSource`, `RefreshRequest`, `RefreshResponse`
- `PaginatedUserAliasResponse`, `PaginatedAliasResponse`

### Tokens (`services::tokens::token_objs`)
- `CreateTokenRequest`, `UpdateTokenRequest`
- `TokenCreated`, `TokenDetail`, `PaginatedTokenResponse`

### Toolsets (`services::toolsets::toolset_objs`)
- `ToolsetRequest` — single request type for create + update

### MCPs (`services::mcps::mcp_objs`)
- `McpRequest`, `McpServerRequest` — single request type per entity for create + update

### Downloads (`services::models::download_service`)
- `NewDownloadRequest`, `DownloadRequest` output type

### Users (`services::users::user_objs`)
- `ChangeRoleRequest`, `UserAccessStatusResponse`
- `UserAccessRequest`, `PaginatedUserAccessResponse`

### Settings (`services::settings::setting_objs`)
- `UpdateSettingRequest`, `EDIT_SETTINGS_ALLOWED`, `LLM_SETTINGS` constants

### App Access Requests (`services::app_access_requests::access_request_objs`)
- `CreateAccessRequest`, `ApproveAccessRequest`

---

## F. Column Renames

### MCPs
- `created_by` -> `user_id` on `mcps` and `mcp_oauth_tokens` tables
- Remove `created_by` from `mcp_auth_headers` and `mcp_oauth_configs` tables (not needed)

---

## G. Frontend Type Updates

- Regenerate TypeScript types via `make build.ts-client`
- Frontend uses generated types: `ApiModelRequest`, `has_api_key`, `ApiKeyUpdate`
- MSW handlers and test fixtures updated for consistent type names

### Key Files
- `crates/bodhi/src/schemas/apiModel.ts`
- All route handler files: `crates/routes_app/src/*/routes_*.rs`
