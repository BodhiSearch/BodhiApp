# MCP Auth Schema Redesign: Unified Config Base + Instance-Level Credentials

## Context

The current MCP auth schema has a fundamental design flaw: header-based auth stores both **keys AND values** at the server config level (`mcp_auth_headers`). This means:
- Admin provides the full credential during server setup — no user-level credential separation
- No support for query param authentication
- No uniform storage between OAuth tokens and header/query credentials

**Goal**: Restructure so that:
1. **Server level** (admin): defines WHAT auth is needed (key names for header/query, OAuth app config)
2. **Instance level** (user): provides the actual credential VALUES
3. All runtime credentials (OAuth access tokens + manual header/query values) are stored uniformly in one table
4. Query param auth is supported alongside header auth

**Data loss is acceptable** — no production data to migrate. SQL migration is still required for schema consistency.

---

## Key Design Decisions (from interview)

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Config table structure | Unified base table `mcp_auth_configs` | Same table count (7), real FK from mcps, no polymorphism, better extensibility |
| Credential storage | `mcp_auth_credentials` at instance level | Uniform for ALL auth types (OAuth tokens + manual values) |
| OAuth tokens | Per-instance (not per user+config) | Clean isolation, instance deletion cascades cleanly |
| FK from mcps to config | `mcps.auth_config_id` → `mcp_auth_configs.id` | Real FK constraint, single target table |
| McpAuthType enum | Keep `Header` variant in code | UI shows "Header / Query Params"; DB value stays `"header"` |
| Validation | Lenient — partial/empty values OK | Runtime errors surface missing values; user can update instance later |
| Auth config API | Unified endpoint (discriminated union) | Both types are "configs" — one defines keys, other defines OAuth app settings |
| Instance credentials | Embedded in McpRequest body | Single atomic POST creates instance + stores credentials |
| Instance-to-value coupling | Loosely coupled | Instance credential rows store param_type+key+value independently, no FK to server key defs |
| Multiple configs per server | Yes | Same pattern as OAuth — server can have multiple auth profiles |
| Admin tool preview | None at server level | Tools fetched only through instances after user provides credentials |

---

## Final Schema

### Server Level (admin-configured)

```
mcp_servers (UNCHANGED)

mcp_auth_configs (NEW — unified base for ALL auth config types)
├── id (PK, ULID)
├── tenant_id
├── mcp_server_id (FK → mcp_servers, CASCADE)
├── config_type (header | oauth)
├── name
├── created_by
├── created_at, updated_at

mcp_auth_config_params (NEW — header/query key definitions, child of base)
├── id (PK, ULID)
├── tenant_id
├── auth_config_id (FK → mcp_auth_configs, CASCADE)
├── param_type (header | query)
├── param_key
├── created_at, updated_at

mcp_oauth_config_details (REPLACES mcp_oauth_configs — OAuth-specific, child of base)
├── auth_config_id (PK + FK → mcp_auth_configs, 1:1, CASCADE)
├── tenant_id
├── registration_type (pre_registered | dynamic_registration)
├── client_id
├── encrypted_client_secret, client_secret_salt, client_secret_nonce
├── authorization_endpoint, token_endpoint
├── registration_endpoint
├── encrypted_registration_access_token, registration_access_token_salt, registration_access_token_nonce
├── client_id_issued_at, token_endpoint_auth_method, scopes
├── created_at, updated_at
```

### Instance Level (user-created)

```
mcps (MODIFIED)
├── (all existing fields)
├── auth_type: McpAuthType (public | header | oauth) — KEEP, unchanged
├── auth_config_id (nullable FK → mcp_auth_configs.id) — REPLACES auth_uuid
└── (auth_uuid column DROPPED)

mcp_auth_credentials (NEW — uniform credential storage for ALL auth types)
├── id (PK, ULID)
├── tenant_id
├── mcp_id (FK → mcps, CASCADE)
├── param_type (header | query)
├── param_key
├── encrypted_value, value_salt, value_nonce
├── created_at, updated_at

mcp_oauth_tokens (MODIFIED — per-instance, not per user+config)
├── id (PK, ULID)
├── tenant_id
├── mcp_id (FK → mcps, CASCADE)  ← NEW: links to instance
├── auth_config_id (FK → mcp_auth_configs)  ← denormalized for refresh performance
├── user_id  ← keep for RLS + query convenience
├── encrypted_refresh_token, refresh_token_salt, refresh_token_nonce
├── scopes_granted, expires_at
├── created_at, updated_at
└── (encrypted_access_token REMOVED — access token now in mcp_auth_credentials)
```

### Dropped Tables
- `mcp_auth_headers` → replaced by `mcp_auth_configs` + `mcp_auth_config_params`
- `mcp_oauth_configs` → replaced by `mcp_auth_configs` + `mcp_oauth_config_details`

---

## Flows

### Header/Query Auth Flow
```
1. Admin: POST /mcps/auth-configs { type: "header", mcp_server_id, name, entries: [{param_type, param_key}] }
   → creates mcp_auth_configs row (config_type=header)
   → creates mcp_auth_config_params rows (key definitions only)

2. User: POST /mcps { mcp_server_id, auth_type: "header", auth_config_id, credentials: [{param_type, param_key, value}] }
   → creates mcps row with auth_config_id
   → creates mcp_auth_credentials rows (encrypted values)

3. Runtime: fetch mcp_auth_credentials for instance → build headers + query params → send request
```

### OAuth Flow
```
1. Admin: POST /mcps/auth-configs { type: "oauth", mcp_server_id, name, client_id, endpoints... }
   → creates mcp_auth_configs row (config_type=oauth)
   → creates mcp_oauth_config_details row

2. User: POST /mcps { mcp_server_id, auth_type: "oauth", auth_config_id }
   → creates mcps row with auth_config_id (no credentials yet)

3. User: POST /mcps/oauth/login { mcp_id, redirect_uri }
   → generates authorization URL using config from instance's auth_config_id

4. User: POST /mcps/oauth/exchange { mcp_id, code, redirect_uri, code_verifier }
   → exchanges code for token
   → stores refresh token in mcp_oauth_tokens (linked to instance)
   → stores access token in mcp_auth_credentials (param_type=header, param_key=Authorization, value=Bearer {token})

5. Runtime: fetch mcp_auth_credentials for instance → build headers → send request (uniform with header/query!)

6. Token refresh: update mcp_oauth_tokens (new refresh token) + update mcp_auth_credentials (new access token)
```

---

## Implementation Plan — Sequential Sub-Agents by Crate

Each sub-agent works on one crate group, runs gate checks, commits, and returns a summary to the main agent. The main agent passes context to the next sub-agent.

**Execution order** (upstream → downstream per dependency chain):

```
Sub-agent 1: mcp_client (most upstream)
     ↓ summary
Sub-agent 2: services (migration + entities + types + repo + service)
     ↓ summary
Sub-agent 3: routes_app (handlers + DTOs)
     ↓ summary
Sub-agent 4: openapi + ts-client (regeneration)
     ↓ summary
Sub-agent 5: bodhi frontend (React components + tests)
     ↓ summary
Sub-agent 6: lib_bodhiserver_napi (E2E tests)
```

---

### Sub-agent 1: `mcp_client` crate

**Scope**: Add query param support to MCP client transport layer.

**Reference**: `git stash show -p stash@{0}` — look at `crates/mcp_client/src/lib.rs` changes.

**Tasks**:
- New `McpAuthParams { headers: Vec<(String, String)>, query_params: Vec<(String, String)> }` with `Default` impl
- New `McpHttpTransport` trait (internal, mockable via `#[mockall::automock]`):
  - `fetch_tools(url: &str, default_headers: HeaderMap) → Vec<McpTool>`
  - `call_tool(url: &str, default_headers: HeaderMap, tool_name, args) → Value`
- `ReqwestMcpTransport` production impl (wraps existing reqwest + rmcp logic)
- Refactor `McpClient` trait public API:
  - `fetch_tools(url: &str, auth_params: Option<McpAuthParams>) → Vec<McpTool>`
  - `call_tool(url: &str, tool_name, args, auth_params: Option<McpAuthParams>) → Value`
- `DefaultMcpClient` adapter struct (holds `Arc<dyn McpHttpTransport>`):
  - `prepare_auth(url: &str, auth_params: Option<McpAuthParams>) → (String, HeaderMap)`
  - Headers → `reqwest::HeaderMap` (graceful handling of invalid header names/values)
  - Query params → append via `url::Url::query_pairs_mut()`
- Add `url` crate to `Cargo.toml` dependencies
- Update existing tests, add tests for `prepare_auth` with headers, query params, and mixed

**Gate check**:
```bash
cargo check -p mcp_client && cargo test -p mcp_client
```

**Commit**: `refactor(mcp_client): add McpAuthParams with query param support and transport layer split`

**Summary to pass downstream**: McpAuthParams struct location, McpClient trait signature changes, DefaultMcpClient usage pattern.

---

### Sub-agent 2: `services` crate

**Scope**: Migration, entities, domain types, repository, service layer. This is the largest sub-agent.

**Reference**: `git stash show -p stash@{0}` for patterns; BUT the key difference is values move to instance level (not server level as in stash).

#### 2a. Migration

**File**: `crates/services/src/db/sea_migrations/m20250101_000016_mcp_auth_redesign.rs`

> **NOTE**: Schema-only migration. NO data migration. Drop/create/alter tables and columns only. Data loss is acceptable.

- Drop tables (with CASCADE): `mcp_auth_headers`, `mcp_oauth_configs`, `mcp_oauth_tokens`
- Create `mcp_auth_configs` (base table): id, tenant_id, mcp_server_id FK (CASCADE), config_type, name, created_by, created_at, updated_at
- Create `mcp_auth_config_params`: id, tenant_id, auth_config_id FK (CASCADE), param_type, param_key, created_at, updated_at
- Create `mcp_oauth_config_details`: auth_config_id (PK + FK → mcp_auth_configs, CASCADE), tenant_id, registration_type, client_id, encrypted_client_secret + salt + nonce, authorization_endpoint, token_endpoint, registration_endpoint, encrypted_registration_access_token + salt + nonce, client_id_issued_at, token_endpoint_auth_method, scopes, created_at, updated_at
- Create `mcp_auth_credentials`: id, tenant_id, mcp_id FK (CASCADE), param_type, param_key, encrypted_value, value_salt, value_nonce, created_at, updated_at
- Recreate `mcp_oauth_tokens`: id, tenant_id, mcp_id FK (CASCADE), auth_config_id FK, user_id, encrypted_refresh_token + salt + nonce, scopes_granted, expires_at, created_at, updated_at
- Alter `mcps`: drop `auth_uuid`, add `auth_config_id` (nullable FK → mcp_auth_configs)
- Add RLS policies (PostgreSQL) for all new tables
- Add indexes on FK columns
- Unique constraints: `(tenant_id, auth_config_id, param_type, param_key)` on `mcp_auth_config_params`; `(tenant_id, mcp_id, param_type, param_key)` on `mcp_auth_credentials`
- Register in `mod.rs`

#### 2b. Entity files (in `crates/services/src/mcps/`)

- NEW `mcp_auth_config_entity.rs`: `McpAuthConfigEntity` model + `McpAuthConfigView` partial model
- NEW `mcp_auth_config_param_entity.rs`: `McpAuthConfigParamEntity` model
- NEW `mcp_auth_credential_entity.rs`: `McpAuthCredentialEntity` model + `McpAuthCredentialView` (masks encrypted_value → has_value: bool)
- NEW `mcp_oauth_config_detail_entity.rs`: `McpOAuthConfigDetailEntity` model + `McpOAuthConfigDetailView` (masks secrets)
- MODIFY `mcp_oauth_token_entity.rs`: add `mcp_id`, remove `encrypted_access_token` + salt + nonce fields
- MODIFY `mcp_entity.rs`: `auth_uuid` → `auth_config_id` column
- DELETE `mcp_auth_header_entity.rs`
- Update `mod.rs` exports

Follow existing `McpOAuthConfigView` partial model pattern for masking secrets.

#### 2c. Domain types (`mcp_objs.rs`)

- New enum `McpAuthParamType { Header, Query }` with serde, Display, EnumString, DeriveValueType
- New enum `McpAuthConfigType { Header, Oauth }` with same derives
- New struct `McpAuthConfigParam { id, param_type: McpAuthParamType, param_key }` (key definition response)
- New struct `McpAuthConfigParamInput { param_type: McpAuthParamType, param_key }` (key definition input)
- New struct `McpAuthCredential { id, param_type, param_key, has_value: bool }` (masked response)
- New struct `McpAuthCredentialInput { param_type, param_key, value }` (create/update input)
- Updated `CreateMcpAuthConfigRequest::Header { name, entries: Vec<McpAuthConfigParamInput> }` (keys only, NO values)
- Updated `McpAuthConfigResponse::Header { id, name, mcp_server_id, entries: Vec<McpAuthConfigParam> }`
- Updated `McpAuthConfigResponse::Oauth` — loaded from base + details
- Updated `McpRequest`: add `credentials: Option<Vec<McpAuthCredentialInput>>`, rename `auth_uuid` → `auth_config_id`
- Updated `Mcp` struct: rename `auth_uuid` → `auth_config_id`
- Updated `McpOAuthToken`: add `mcp_id`, remove access_token fields
- Add From impls: entity views → domain types

#### 2d. Repository layer (`mcp_auth_repository.rs`)

Update trait + impl:
- CRUD for `mcp_auth_configs` (base table)
- CRUD for `mcp_auth_config_params` (key definitions)
- CRUD for `mcp_oauth_config_details` (replaces mcp_oauth_configs methods, encryption for client_secret)
- CRUD for `mcp_auth_credentials` (instance-level, encryption using existing `encrypt_api_key`/`decrypt_api_key`)
- `get_decrypted_credentials(tenant_id, mcp_id) → Option<McpAuthParams>` (decrypt all credential entries, separate into headers + query_params)
- Updated `mcp_oauth_tokens` methods: per-instance (mcp_id), no access_token storage
- `get_decrypted_refresh_token(tenant_id, mcp_id)` for token refresh
- DELETE old methods: `create_mcp_auth_header`, `get_decrypted_auth_header`, `update_mcp_auth_header`, etc.
- Use `begin_tenant_txn(tenant_id)` for mutating operations (RLS on PostgreSQL)

#### 2e. Service layer (`mcp_service.rs` + `auth_scoped.rs`)

**McpService trait**:
- `create_auth_config(tenant_id, mcp_server_id, created_by, request) → McpAuthConfigResponse`
  - Creates base `mcp_auth_configs` row + type-specific child (params or OAuth details)
- `list_auth_configs(tenant_id, mcp_server_id) → Vec<McpAuthConfigResponse>`
  - Query base table, load type-specific details per config_type
- `get_auth_config(tenant_id, id) → Option<McpAuthConfigResponse>`
- `delete_auth_config(tenant_id, id)` — CASCADE handles children
- Updated `create()` for instances: accepts `credentials`, stores in `mcp_auth_credentials`
- Updated `update()`: supports credential updates (replace-all pattern)
- `resolve_auth_params_for_mcp(tenant_id, mcp_row) → Option<McpAuthParams>`
  - **Uniform**: always reads `mcp_auth_credentials` for the instance. No branching on auth_type!
- Updated `exchange_oauth_token`: dual write — `mcp_oauth_tokens` (refresh token, expiry) + `mcp_auth_credentials` (access token as `Authorization: Bearer {token}`)
- Updated token refresh: dual update within transaction
- Updated `store_oauth_token`: same dual-write pattern

**AuthScopedMcpService**:
- Updated method signatures
- OAuth flow methods accept `mcp_id` parameter
- Inject tenant_id + user_id from AuthContext as before

#### 2f. Tests

- Update all `test_mcp_*.rs` files in `crates/services/src/mcps/`
- Update test helpers: `make_auth_config_row`, `make_auth_config_param_row`, `make_auth_credential_row`, etc.
- Test repository CRUD, encryption/decryption, isolation
- Test service create/list/delete auth configs, instance with credentials, OAuth dual-write

**Gate check**:
```bash
cargo check -p services && cargo test -p services
```

**Commit**: `refactor(services): unified auth config base table + instance-level credentials`

**Summary to pass downstream**: New domain types (McpAuthConfigResponse, McpAuthCredentialInput, McpAuthParamType, McpAuthConfigType), updated McpService trait signatures, updated McpRequest shape, McpAuthParams from mcp_client, auth_config_id replacing auth_uuid.

---

### Sub-agent 3: `routes_app` crate

**Scope**: Route handlers, DTOs, API schemas, error types.

**Tasks**:

**`routes_mcps_auth.rs`**:
- Updated `mcp_auth_configs_create`: uses `CreateMcpAuthConfigRequest` discriminated union (type=header → key defs, type=oauth → OAuth config)
- Updated `mcp_auth_configs_index`: list by server, returns `Vec<McpAuthConfigResponse>`
- Updated `mcp_auth_configs_show`: returns `McpAuthConfigResponse`
- Updated `mcp_auth_configs_delete`: deletes from base table (CASCADE handles children)

**`routes_mcps.rs`**:
- Updated `mcps_create`: extract `credentials: Option<Vec<McpAuthCredentialInput>>` from `McpRequest`, pass to service
- Updated `mcps_update`: support credential updates
- Updated response: `auth_uuid` → `auth_config_id` in Mcp response

**`routes_mcps_oauth.rs`**:
- Updated `mcp_oauth_login`: accept `mcp_id` in request body, resolve config from instance's `auth_config_id`
- Updated `mcp_oauth_exchange`: accept `mcp_id`, service handles dual-write (token + credential)
- Updated request/response DTOs

**`mcps_api_schemas.rs`**:
- Updated `OAuthLoginRequest`: add `mcp_id`
- Updated `OAuthTokenExchangeRequest`: add `mcp_id`
- Register new types: `McpAuthParamType`, `McpAuthConfigType`, `McpAuthConfigParam`, `McpAuthCredentialInput`, `McpAuthCredential`
- Update OpenAPI schema registrations

**`error.rs`**:
- New error variants if needed for auth config not found, credential validation

**Tests**: Update all `test_*.rs` files in `crates/routes_app/src/mcps/`

**Gate check**:
```bash
cargo check -p routes_app && cargo test -p routes_app
```

**Commit**: `refactor(routes_app): update MCP auth routes for unified config + instance credentials`

**Summary to pass downstream**: API shape changes, new request/response types for OpenAPI generation.

---

### Sub-agent 4: OpenAPI + TypeScript Client

**Scope**: Regenerate API spec and TypeScript types.

**Tasks**:
```bash
cargo run --package xtask openapi           # regenerate openapi.json
cd ts-client && npm install && npm run generate   # regenerate TypeScript types
```
- Verify generated types include new types: `McpAuthParamType`, `McpAuthConfigType`, `McpAuthConfigParam`, `McpAuthCredentialInput`, `McpAuthCredential`
- Verify `McpRequest` has `auth_config_id` and `credentials` fields
- Verify `McpAuthConfigResponse` variants are correct

**Gate check**:
```bash
make ci.ts-client-check
```

**Commit**: `chore: regenerate OpenAPI spec and TypeScript client for MCP auth redesign`

**Summary to pass downstream**: TypeScript type names and shapes from `@bodhiapp/ts-client` for frontend components.

---

### Sub-agent 5: Frontend (`bodhi` crate)

**Scope**: React components and component tests.

**Reference**: `git stash show -p stash@{0}` for frontend change patterns (AuthConfigForm, mcpUtils, etc.)

**Tasks**:

**`AuthConfigForm.tsx`** (server config — key definitions only):
- Multi-entry form: each entry has param_type selector (Header/Query) + param_key input
- NO value input — this is admin config, not user credentials
- Add/remove entry buttons
- Uses `McpAuthConfigParamInput` type from ts-client

**Instance creation/edit pages**:
- Credential value input form
- Fetch key definitions from selected auth_config → show param_type + param_key as labels
- User provides values for each key via `PasswordInput`
- Uses `McpAuthCredentialInput` type from ts-client
- `auth_uuid` references → `auth_config_id`

**`mcpUtils.ts`**:
- `authConfigTypeLabel("header")` → "Header / Query Params"
- Update `authConfigDetail` to show param entries

**`test-utils/msw-v2/handlers/mcps.ts`**:
- Update mock handlers: auth config create/list expects new shapes
- Instance create expects `credentials` array
- OAuth flow expects `mcp_id`

**Component tests**: Update for new form shapes and prop changes.

**Gate check**:
```bash
cd crates/bodhi && npm install && npm test
```

**Commit**: `refactor(bodhi): update MCP auth UI for key definitions + instance credentials`

**Summary to pass downstream**: UI changes for E2E test validation.

---

### Sub-agent 6: E2E Tests (`lib_bodhiserver_napi`)

**Scope**: Mock MCP auth servers + Playwright E2E tests.

**Reference**: `git stash show -p stash@{0}` for `test-mcp-auth-server/` and `mcps-header-auth.spec.mjs`.

**Tasks**:

**`test-mcp-auth-server/`** (NEW — from stash):
- Express app + MCP SDK mock server
- Accepts CLI args: `--header KEY=VALUE`, `--query KEY=VALUE`, `--port PORT`
- Validates configured auth params against incoming requests
- Returns 401 if validation fails
- Tools: `echo` (text echo), `get_auth_info` (returns received auth params)

**`playwright.config.mjs`**:
- Add 3 webServer entries:
  1. `test-mcp-auth-header` (port 55176): validates `Authorization: Bearer test-header-key`
  2. `test-mcp-auth-query` (port 55177): validates `api_key=test-query-key`
  3. `test-mcp-auth-mixed` (port 55178): validates 2 headers + 2 query params

**`mcpFixtures.mjs`**: Export URLs, expected keys/values for test servers.

**`mcps-header-auth.spec.mjs`** (rewrite):
- Test 1: Single header auth — create auth config (key def), create instance (with value), fetch tools
- Test 2: Single query param auth — same flow with param_type=query
- Test 3: Mixed auth — 2 headers + 2 query params, verify all sent correctly

**Gate check**:
```bash
make build.ui-rebuild && make test.napi
```

**Commit**: `test(e2e): add MCP auth header/query/mixed E2E tests with local mock servers`

---

### Final Gate: Full Test Suite

After all sub-agents complete:
```bash
make test
```

---

## Critical Files by Sub-Agent

### Sub-agent 1 (mcp_client)
| File | Change |
|------|--------|
| `crates/mcp_client/src/lib.rs` | McpAuthParams, McpHttpTransport, DefaultMcpClient |
| `crates/mcp_client/Cargo.toml` | Add `url` dependency |

### Sub-agent 2 (services)
| File | Change |
|------|--------|
| `db/sea_migrations/m20250101_000016_*.rs` | NEW migration |
| `db/sea_migrations/mod.rs` | Register migration |
| `mcps/mcp_auth_config_entity.rs` | NEW entity |
| `mcps/mcp_auth_config_param_entity.rs` | NEW entity |
| `mcps/mcp_auth_credential_entity.rs` | NEW entity |
| `mcps/mcp_oauth_config_detail_entity.rs` | NEW entity (replaces mcp_oauth_config_entity.rs) |
| `mcps/mcp_oauth_token_entity.rs` | MODIFY (add mcp_id, remove access_token) |
| `mcps/mcp_entity.rs` | MODIFY (auth_uuid → auth_config_id) |
| `mcps/mcp_auth_header_entity.rs` | DELETE |
| `mcps/mcp_objs.rs` | Major type updates |
| `mcps/mcp_service.rs` | Major trait/impl updates |
| `mcps/mcp_auth_repository.rs` | Major trait/impl updates |
| `mcps/mcp_instance_repository.rs` | Update for auth_config_id |
| `mcps/auth_scoped.rs` | Updated wrappers |
| `mcps/error.rs` | New error variants if needed |
| `mcps/mod.rs` | Updated exports |
| `mcps/test_*.rs` | Updated tests |

### Sub-agent 3 (routes_app)
| File | Change |
|------|--------|
| `mcps/routes_mcps_auth.rs` | Updated handlers |
| `mcps/routes_mcps.rs` | Credential handling in create/update |
| `mcps/routes_mcps_oauth.rs` | mcp_id in OAuth flow |
| `mcps/mcps_api_schemas.rs` | Updated DTOs |
| `mcps/error.rs` | New error variants |
| `mcps/test_*.rs` | Updated tests |

### Sub-agent 4 (openapi + ts-client)
| File | Change |
|------|--------|
| `openapi.json` | Regenerated |
| `ts-client/src/types/types.gen.ts` | Regenerated |

### Sub-agent 5 (bodhi frontend)
| File | Change |
|------|--------|
| `src/app/ui/mcp-servers/components/AuthConfigForm.tsx` | Key-only form |
| `src/app/ui/mcp-servers/components/mcpUtils.ts` | Label updates |
| MCP instance creation/edit pages | Credential value input |
| `test-utils/msw-v2/handlers/mcps.ts` | Updated mocks |
| Component test files | Updated for new shapes |

### Sub-agent 6 (E2E tests)
| File | Change |
|------|--------|
| `test-mcp-auth-server/` | NEW mock server directory |
| `playwright.config.mjs` | New webServer entries |
| `tests-js/specs/mcps/mcps-header-auth.spec.mjs` | Rewritten tests |
| `tests-js/fixtures/mcpFixtures.mjs` | New constants |
