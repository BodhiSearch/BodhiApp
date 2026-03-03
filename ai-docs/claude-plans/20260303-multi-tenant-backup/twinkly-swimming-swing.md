# Plan: OpenAPI Comment Cleanup + Frontend Type Migration + Naming Audit

## Context

The backend request/response types were renamed and unified (tracked in HEAD commit). The generated `openapi-schema.ts` reflects these changes but the frontend code and tests still reference old type names. Additionally, verbose internal implementation notes leaked into the public OpenAPI spec via `///` doc comments. A full naming convention audit was performed and disposition agreed with user.

**Goals:**
1. Clean up Rust `///` doc comments — move internal notes to `//`, keep user-facing summaries as `///`
2. Rename `CreateAuthConfigBody` → `CreateAuthConfig`
3. Add convention comments to schema files that don't follow `*Form` convention
4. Regenerate openapi.json and ts-client types
5. Migrate frontend code and tests to use new type names/shapes
6. Create TECHDEBT files for deferred work
7. Gate: `npm run test:all` passes

---

## Phase 1: Rust Comment Cleanup

Convert internal implementation notes from `///` to `//`. Keep only concise user-facing summaries as `///`.

### 1a. Service layer structs

| File | Structs | What to change |
|------|---------|----------------|
| `crates/services/src/models/model_objs.rs` | `ApiModelForm` (~L1096) | Remove "Used as `Json<ApiModelForm>` in handlers for both create and update (PUT)" from `///` → `//` |
| `crates/services/src/models/model_objs.rs` | `ApiModelOutput` (~L1131) | Remove "returned as `Json<ApiModelOutput>` from handlers" from `///` → `//` |
| `crates/services/src/models/model_objs.rs` | `UserAliasForm` (~L593) | Remove "Used as `Json<UserAliasForm>` in handlers..." from `///` → `//` |
| `crates/services/src/models/model_objs.rs` | `ApiKeyUpdate` | Remove "Uses tagged enum for JSON..." from `///` → `//` |
| `crates/services/src/mcps/mcp_objs.rs` | `McpForm` (~L398) | Remove "Used as `Json<McpForm>` in handlers..." from `///` → `//` |
| `crates/services/src/mcps/mcp_objs.rs` | `McpServerForm` (~L435) | Remove "Used as `Json<McpServerForm>` in handlers..." from `///` → `//` |
| `crates/services/src/toolsets/toolset_objs.rs` | `ToolsetForm` (~L114) | Remove "Used as `Json<ToolsetForm>` in handlers..." from `///` → `//` |
| `crates/services/src/tokens/token_objs.rs` | `CreateTokenForm`, `UpdateTokenForm`, `TokenCreated`, `TokenDetail` | Same pattern |
| `crates/services/src/models/download_service.rs` | `DownloadRequestForm` | Same pattern |
| `crates/services/src/users/access_request_entity.rs` | `UserAccessRequest` output type | Remove "User access request output type for API responses" if it's internal |

### 1b. Route handler doc comments

| File | Location | What to change |
|------|----------|----------------|
| `crates/routes_app/src/api_models/routes_api_models.rs` | `testApiModel` handler (~L181-187) | "CRUD uniformity exception..." `///` → `//` |
| `crates/routes_app/src/api_models/routes_api_models.rs` | `fetchApiModels` handler (~L263-269) | "CRUD uniformity exception..." `///` → `//` |

### 1c. Rename: `CreateAuthConfigBody` → `CreateAuthConfig`

| File | Change |
|------|--------|
| `crates/routes_app/src/mcps/mcps_api_schemas.rs` | Rename struct `CreateAuthConfigBody` → `CreateAuthConfig` |
| `crates/routes_app/src/shared/openapi.rs` | Update schema registration |
| All files referencing `CreateAuthConfigBody` | Update imports and usage |

### 1d. Add convention comments to schema files

Add a comment at the top of these files explaining that types here don't use services/DB persistence, so they don't follow the `<Domain>Form` / `<Domain>Output` convention:

| File | Comment to add |
|------|----------------|
| `crates/routes_app/src/mcps/mcps_api_schemas.rs` | `// NOTE: Types in this file are utility/action DTOs that don't use services/DB for persistence. They don't follow the <Domain>Form/<Domain>Output naming convention used by CRUD entities.` |
| `crates/routes_app/src/api_models/api_models_api_schemas.rs` | Same convention comment |
| `crates/routes_app/src/models/models_api_schemas.rs` | Same convention comment |
| `crates/routes_app/src/toolsets/toolsets_api_schemas.rs` | Same convention comment (for `ExecuteToolsetRequest`, `ToolsetExecutionResponse`) |

**Verify:** `cargo check -p services && cargo check -p routes_app`

---

## Phase 2: Regenerate OpenAPI + ts-client

```bash
cargo run --package xtask openapi
make build.ts-client
```

Confirm the regenerated `openapi-schema.ts`:
- No longer has verbose internal notes in descriptions
- `CreateAuthConfigBody` → `CreateAuthConfig` reflected
- All HEAD changes still present

---

## Phase 3: Frontend Type Migration

All paths relative to `crates/bodhi/src/`.

### 3a. Type name renames (imports + usage)

| Old Name | New Name | Files |
|----------|----------|-------|
| `CreateAliasRequest` | `UserAliasForm` | `schemas/alias.ts` (L1, 42), `hooks/useModels.ts` (L4, 55), `hooks/useModels.test.ts` (L3, 348) |
| `UpdateAliasRequest` | `UserAliasForm` | `schemas/alias.ts` (L56), `hooks/useModels.ts` (L11, 75, 77), `hooks/useModels.test.ts` (L4, 420) |

Most other old type names (`CreateApiModelRequest`, `McpResponse`, `ApiTokenDetail`, etc.) were **not found** in frontend — consumed indirectly through ts-client, will auto-resolve after regeneration.

### 3b. Field removals — `created_by` / `updated_by`

Remove from mock data:

| File | Fields to remove |
|------|-----------------|
| `test-utils/msw-v2/handlers/mcps.ts` | `created_by`, `updated_by` (~L47-48, 85, 118, 135, 155) |
| `test-utils/msw-v2/handlers/toolsets.ts` | `updated_by` (~L46, 129, 147) |
| `app/ui/setup/toolsets/page.test.tsx` | `updated_by` (~L80, 107, 138) |
| `app/ui/mcp-servers/new/page.test.tsx` | `created_by`, `updated_by` |
| `app/ui/toolsets/page.test.tsx` | `updated_by` |
| `app/ui/toolsets/edit/page.test.tsx` | `updated_by` |

### 3c. Field change — `api_key_masked` → `has_api_key`

Already done in frontend. No changes needed.

### 3d. Download type changes

- `downloaded_bytes`: now required (was optional). Frontend uses truthiness checks — safe, no changes needed.
- `started_at`: now optional (was required). Check `app/ui/pull/page.tsx` table columns for `started_at` display — add `?.` or `?? ''` if accessing directly.

### 3e. `McpServerResponse` intersection type

TypeScript intersection types allow flat field access. No structural changes needed. Verify type-check passes.

### 3f. `OAuthTokenResponse.created_by` → `user_id`

Search frontend for display of `created_by` from OAuth token responses, update to `user_id`.

### 3g. `CreateAuthConfigBody` → `CreateAuthConfig`

Search frontend for any usage of `CreateAuthConfigBody` and update to `CreateAuthConfig`.

---

## Phase 4: Create TECHDEBT Files

### `crates/bodhi/TECHDEBT.md`
- Frontend create/update forms should be unified to match single `*Form` API types (e.g., `ApiModelForm` for both create and update)

### `crates/routes_app/TECHDEBT.md`
- Migrate settings request/response types (`UpdateSettingRequest`, `SetupRequest`, `SetupResponse`) to services crate and rename per convention
- Migrate user management request/response types (`ChangeRoleRequest`, `ApproveUserAccessRequest`, `UserAccessStatusResponse`, `UserResponse`, `UserAliasResponse`, `ModelAliasResponse`, `ApiAliasResponse`) to services crate and rename per convention

---

## Known Exceptions (not violations — do not migrate)

| Type | Reason |
|------|--------|
| `CreateChatCompletionRequest`, `CreateEmbeddingRequest`, `ListModelResponse` | OpenAI-compatible API spec names |
| `ChatRequest`, `ShowRequest`, `ShowResponse`, `ModelsResponse` | Ollama-compatible API spec names |
| `ToolsetTypeRequest`, `McpServerRequest`, `McpInstance`, `ToolsetInstance` | Access-request domain objects ("request for access", not HTTP request) |
| `PingResponse`, `RedirectResponse` | Public anonymous API types — no migration needed |
| `McpServerResponse` | Wraps `McpServer` + computed counts — `Response` distinguishes from base |
| `ToolsetResponse` | Wraps `Toolset` + config — same pattern |
| `AppToolsetConfig.updated_by`, `McpServer.created_by/updated_by` | Admin-managed audit fields, intentional |
| MCP OAuth types (`OAuthLoginRequest`, etc.) | Utility/action DTOs, not DB-persisted — don't follow `*Form` convention |
| Action types (`TestPromptRequest`, `McpExecuteRequest`, etc.) | Utility/action DTOs, not DB-persisted — don't follow `*Form` convention |

---

## Verification

```bash
# 1. Rust compilation
cargo check -p services
cargo check -p routes_app

# 2. Regenerate
cargo run --package xtask openapi
make build.ts-client

# 3. Frontend type-check + tests (final gate)
cd crates/bodhi && npm run test:all
```

---

## Execution Order

### Step 1: Rust Changes (Sub-agent: worktree isolation)
**Agent prompt**: Implement Phase 1 (1a–1d) — comment cleanup, `CreateAuthConfigBody` → `CreateAuthConfig` rename, convention comments.
**Gate**: `cargo check -p services && cargo check -p routes_app` must pass.

### Step 2: Regenerate (Main agent)
Run Phase 2 — `cargo run --package xtask openapi && make build.ts-client`.
**Gate**: Verify openapi-schema.ts diff shows cleaned comments and `CreateAuthConfig` rename.

### Step 3: Frontend Migration (Sub-agent: worktree isolation)
**Agent prompt**: Implement Phase 3 (3a–3g) — type renames, field removals, download type fixes, OAuth field rename.
**Gate**: `cd crates/bodhi && npm run test:all` must pass.

### Step 4: TECHDEBT Files (Main agent)
Create Phase 4 TECHDEBT files.

### Step 5: Final Verification
`cd crates/bodhi && npm run test:all` — final gate.
