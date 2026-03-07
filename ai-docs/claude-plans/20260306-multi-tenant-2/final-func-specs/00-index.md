# Multi-Tenant Functional Specification — Index

> **Scope**: System overview, deployment modes, environment configuration, route map, feature matrix
> **Related specs**: [Auth Flows](01-auth-flows.md) · [Tenant Management](02-tenant-management.md) · [Info Endpoint](03-info-endpoint.md) · [API Tokens](04-api-tokens.md) · [External OAuth](05-external-oauth.md) · [Feature Gating](06-feature-gating.md)
> **Decisions**: D23 (unified code path), D67 (deployment field), D91 (single realm)

---

## Purpose & Audience

This is the canonical functional specification for BodhiApp's multi-tenant feature. Consumers are AI coding assistants and developers who need to understand multi-tenant behavior for implementing features, fixing bugs, and writing E2E tests.

**Key principle**: Standalone mode IS multi-tenant with exactly 1 tenant. The codebase uses a unified code path (D23) — middleware, token resolution, and session management work identically in both modes. Differences are in setup flows, service construction, and feature availability.

---

## Deployment Modes

| Aspect | Standalone | Multi-Tenant |
|--------|-----------|-------------|
| `BODHI_DEPLOYMENT` | `"standalone"` (default) | `"multi_tenant"` |
| Tenants | Exactly 1 | 1+ per user (CREATE 1, MEMBER of many) (D65) |
| Login | Single-phase: resource OAuth | Two-phase: dashboard OAuth → resource OAuth |
| Setup | Multi-step wizard (`POST /setup`) | Tenant registration form (`POST /tenants`) |
| Tenant status after creation | `ResourceAdmin` → `Ready` (after first login) | `Ready` immediately (D79) |
| LLM features | Active (local GGUF models) | Disabled (API models only) |
| Dashboard endpoints | Return `not_multi_tenant` error (D101) | Active |
| Client ID format | `bodhi-resource-<UUID>` | `bodhi-tenant-<UUID>` (D82) |
| Auth issuer | Keycloak (single realm) | Same Keycloak, same realm (D91) |

---

## Environment Variables

| Variable | Required In | Source | Purpose |
|----------|-----------|--------|---------|
| `BODHI_DEPLOYMENT` | Both | Setting (DB or env) | `"standalone"` (default) or `"multi_tenant"` |
| `BODHI_MULTITENANT_CLIENT_ID` | Multi-tenant | Setting (DB or env) | Dashboard Keycloak client ID |
| `BODHI_MULTITENANT_CLIENT_SECRET` | Multi-tenant | Env only (D98) | Dashboard Keycloak client secret |
| `BODHI_AUTH_ISSUER` | Both | Setting | Keycloak issuer URL |
| `BODHI_PUBLIC_SCHEME` | Both | Setting | URL scheme for `public_server_url()` |
| `BODHI_PUBLIC_HOST` | Both | Setting | Hostname for `public_server_url()` |
| `BODHI_PUBLIC_PORT` | Both | Setting | Port for `public_server_url()` |

**Test-only variables:**

| Variable | Purpose |
|----------|---------|
| `INTEG_TEST_MULTI_TENANT_CLIENT_ID` | Dashboard client for integration tests |
| `INTEG_TEST_MULTI_TENANT_CLIENT_SECRET` | Dashboard client secret for tests |

> `BODHI_APP_URL` was planned (D90) but never created. `public_server_url()` is used instead.

---

## AppStatus Overview

### Standalone

```
[No tenants in DB] ──POST /setup──▶ [ResourceAdmin] ──first login──▶ [Ready]
         Setup                                                          Ready
```

### Multi-Tenant

```
[No dashboard token] ──dashboard OAuth──▶ [Dashboard token, no active tenant]
    TenantSelection                              │
                                    ┌────────────┼────────────┐
                                    ▼            ▼            ▼
                               0 tenants    1 tenant     N tenants
                                Setup     (auto-OAuth)  TenantSelection
                                  │            │            │
                              register     resource      select +
                              + OAuth       OAuth         OAuth
                                  │            │            │
                                  └────────────┴────────────┘
                                              ▼
                                           [Ready]
```

---

## Route Map

All routes are prefixed with `/bodhi/v1/` unless noted. Auth levels: **Public** (no auth), **Optional** (auth attempted but not required), **Session** (browser session required), **Bearer** (API token or OAuth token), **Any** (session or bearer).

### System

| Method | Path | Auth | Modes | Purpose | Spec |
|--------|------|------|-------|---------|------|
| GET | `/ping` | Public | Both | Health ping | — |
| GET | `/health` | Public | Both | Health check | — |

### Authentication

| Method | Path | Auth | Modes | Purpose | Spec |
|--------|------|------|-------|---------|------|
| POST | `/bodhi/v1/auth/initiate` | Optional | Both | Start resource OAuth flow | [01](01-auth-flows.md#resource-oauth) |
| POST | `/bodhi/v1/auth/callback` | Optional | Both | Complete resource OAuth flow | [01](01-auth-flows.md#resource-oauth-callback) |
| POST | `/bodhi/v1/logout` | Public | Both | Logout (clear session) | [01](01-auth-flows.md#logout) |
| POST | `/bodhi/v1/auth/dashboard/initiate` | Optional | Multi | Start dashboard OAuth flow | [01](01-auth-flows.md#dashboard-oauth) |
| POST | `/bodhi/v1/auth/dashboard/callback` | Optional | Multi | Complete dashboard OAuth flow | [01](01-auth-flows.md#dashboard-oauth-callback) |

### Setup & Info

| Method | Path | Auth | Modes | Purpose | Spec |
|--------|------|------|-------|---------|------|
| GET | `/bodhi/v1/info` | Optional | Both | App status, deployment, client_id | [03](03-info-endpoint.md#info) |
| POST | `/bodhi/v1/setup` | Public | Standalone | Initial standalone setup | [02](02-tenant-management.md#standalone-creation) |
| GET | `/bodhi/v1/user` | Optional | Both | Current user info + dashboard state | [03](03-info-endpoint.md#user-info) |

### Tenant Management

| Method | Path | Auth | Modes | Purpose | Spec |
|--------|------|------|-------|---------|------|
| GET | `/bodhi/v1/tenants` | Optional | Multi | List user's tenants | [02](02-tenant-management.md#tenant-listing) |
| POST | `/bodhi/v1/tenants` | Optional | Multi | Create new tenant | [02](02-tenant-management.md#multi-tenant-creation) |
| POST | `/bodhi/v1/tenants/{client_id}/activate` | Optional | Multi | Instant tenant switch | [02](02-tenant-management.md#tenant-switching) |

### API Tokens

| Method | Path | Auth | Modes | Purpose | Spec |
|--------|------|------|-------|---------|------|
| POST | `/bodhi/v1/tokens` | Session (PowerUser+) | Both | Create API token | [04](04-api-tokens.md#creation) |
| GET | `/bodhi/v1/tokens` | Session (PowerUser+) | Both | List tokens | [04](04-api-tokens.md#crud) |
| PUT | `/bodhi/v1/tokens/{id}` | Session (PowerUser+) | Both | Update token | [04](04-api-tokens.md#crud) |

### External App Access

| Method | Path | Auth | Modes | Purpose | Spec |
|--------|------|------|-------|---------|------|
| POST | `/bodhi/v1/apps/request-access` | Public | Both | Create access request | [05](05-external-oauth.md#access-request-creation) |
| GET | `/bodhi/v1/apps/access-requests/{id}` | Public | Both | Poll request status | [05](05-external-oauth.md#polling) |
| GET | `/bodhi/v1/access-requests/{id}/review` | Session (User+) | Both | Review access request | [05](05-external-oauth.md#review-flow) |
| PUT | `/bodhi/v1/access-requests/{id}/approve` | Session (User+) | Both | Approve request | [05](05-external-oauth.md#approval) |
| POST | `/bodhi/v1/access-requests/{id}/deny` | Session (User+) | Both | Deny request | [05](05-external-oauth.md#denial) |

### User Management

| Method | Path | Auth | Modes | Purpose | Spec |
|--------|------|------|-------|---------|------|
| GET | `/bodhi/v1/users` | Session (Manager+) | Both | List users | — |
| PUT | `/bodhi/v1/users/{user_id}/role` | Session (Manager+) | Both | Change user role | — |
| DELETE | `/bodhi/v1/users/{user_id}` | Session (Admin) | Both | Remove user | — |

### User Access Requests

| Method | Path | Auth | Modes | Purpose | Spec |
|--------|------|------|-------|---------|------|
| GET | `/bodhi/v1/access-requests` | Session (Manager+) | Both | List all access requests | — |
| GET | `/bodhi/v1/access-requests/pending` | Session (Manager+) | Both | List pending requests | — |
| POST | `/bodhi/v1/access-requests/{id}/approve` | Session (Manager+) | Both | Approve request | — |
| POST | `/bodhi/v1/access-requests/{id}/reject` | Session (Manager+) | Both | Reject request | — |
| POST | `/bodhi/v1/user/request-access` | Optional | Both | User requests access to app | — |
| GET | `/bodhi/v1/user/request-status` | Optional | Both | User checks own request status | — |

### Models (GGUF Aliases)

| Method | Path | Auth | Modes | Purpose | Spec |
|--------|------|------|-------|---------|------|
| GET | `/bodhi/v1/models` | Any (User+) | Both | List model aliases | [06](06-feature-gating.md#llm-features) |
| GET | `/bodhi/v1/models/{id}` | Any (User+) | Both | Get model alias | [06](06-feature-gating.md#llm-features) |
| POST | `/bodhi/v1/models` | Any (PowerUser+) | Both | Create model alias | [06](06-feature-gating.md#llm-features) |
| PUT | `/bodhi/v1/models/{id}` | Any (PowerUser+) | Both | Update model alias | — |
| DELETE | `/bodhi/v1/models/{id}` | Any (PowerUser+) | Both | Delete model alias | — |
| POST | `/bodhi/v1/models/{id}/copy` | Any (PowerUser+) | Both | Copy model alias | — |
| GET | `/bodhi/v1/modelfiles` | Any (User+) | Both | List local model files | [06](06-feature-gating.md#llm-features) |
| GET | `/bodhi/v1/modelfiles/pull` | Any (PowerUser+) | Both | List pull jobs | — |
| POST | `/bodhi/v1/modelfiles/pull` | Any (PowerUser+) | Both | Start model pull | — |
| GET | `/bodhi/v1/modelfiles/pull/{id}` | Any (PowerUser+) | Both | Get pull job status | — |
| POST | `/bodhi/v1/models/refresh` | Session (PowerUser+) | Both | Refresh model metadata | — |

### API Models (External API Configurations)

| Method | Path | Auth | Modes | Purpose | Spec |
|--------|------|------|-------|---------|------|
| GET | `/bodhi/v1/api-models` | Session (PowerUser+) | Both | List API model configs | — |
| POST | `/bodhi/v1/api-models` | Session (PowerUser+) | Both | Create API model config | — |
| GET | `/bodhi/v1/api-models/{id}` | Session (PowerUser+) | Both | Get API model config | — |
| PUT | `/bodhi/v1/api-models/{id}` | Session (PowerUser+) | Both | Update API model config | — |
| DELETE | `/bodhi/v1/api-models/{id}` | Session (PowerUser+) | Both | Delete API model config | — |
| POST | `/bodhi/v1/api-models/test` | Session (PowerUser+) | Both | Test API credentials | — |
| POST | `/bodhi/v1/api-models/fetch-models` | Session (PowerUser+) | Both | Fetch models from API | — |
| GET | `/bodhi/v1/api-models/api-formats` | Session (PowerUser+) | Both | Get supported formats | — |
| POST | `/bodhi/v1/api-models/{id}/sync-models` | Session (PowerUser+) | Both | Sync models from API | — |

### OpenAI-Compatible

| Method | Path | Auth | Modes | Purpose | Spec |
|--------|------|------|-------|---------|------|
| GET | `/v1/models` | Any (User+) | Both | List models | — |
| GET | `/v1/models/{id}` | Any (User+) | Both | Get model | — |
| POST | `/v1/chat/completions` | Any (User+) | Both | Chat completion | — |
| POST | `/v1/embeddings` | Any (User+) | Both | Generate embeddings | — |

### Ollama-Compatible

| Method | Path | Auth | Modes | Purpose | Spec |
|--------|------|------|-------|---------|------|
| GET | `/api/tags` | Any (User+) | Both | List models | — |
| POST | `/api/show` | Any (User+) | Both | Show model | — |
| POST | `/api/chat` | Any (User+) | Both | Chat completion | — |

### MCPs (Model Context Protocol)

| Method | Path | Auth | Modes | Purpose | Spec |
|--------|------|------|-------|---------|------|
| GET | `/bodhi/v1/mcps` | Session/OAuth (User+) | Both | List MCPs | — |
| POST | `/bodhi/v1/mcps` | Session (User+) | Both | Create MCP | — |
| GET | `/bodhi/v1/mcps/{id}` | Session/OAuth (User+) | Both | Get MCP | — |
| PUT | `/bodhi/v1/mcps/{id}` | Session (User+) | Both | Update MCP | — |
| DELETE | `/bodhi/v1/mcps/{id}` | Session (User+) | Both | Delete MCP | — |
| POST | `/bodhi/v1/mcps/fetch-tools` | Session (User+) | Both | Fetch tools from server | — |
| POST | `/bodhi/v1/mcps/{id}/tools/refresh` | Session/OAuth (User+) | Both | Refresh tools | — |
| POST | `/bodhi/v1/mcps/{id}/tools/{tool_name}/execute` | Session/OAuth (User+) | Both | Execute MCP tool | — |

### MCP Auth & OAuth

| Method | Path | Auth | Modes | Purpose | Spec |
|--------|------|------|-------|---------|------|
| POST | `/bodhi/v1/mcps/auth-configs` | Session (User+) | Both | Create auth config | — |
| GET | `/bodhi/v1/mcps/auth-configs` | Session (User+) | Both | List auth configs | — |
| GET | `/bodhi/v1/mcps/auth-configs/{id}` | Session (User+) | Both | Get auth config | — |
| DELETE | `/bodhi/v1/mcps/auth-configs/{id}` | Session (User+) | Both | Delete auth config | — |
| POST | `/bodhi/v1/mcps/auth-configs/{id}/login` | Session (User+) | Both | OAuth login | — |
| POST | `/bodhi/v1/mcps/auth-configs/{id}/token` | Session (User+) | Both | Token exchange | — |
| GET | `/bodhi/v1/mcps/oauth-tokens/{token_id}` | Session (User+) | Both | Get OAuth token | — |
| DELETE | `/bodhi/v1/mcps/oauth-tokens/{token_id}` | Session (User+) | Both | Delete OAuth token | — |
| POST | `/bodhi/v1/mcps/oauth/discover-as` | Session (User+) | Both | Discover AS metadata | — |
| POST | `/bodhi/v1/mcps/oauth/discover-mcp` | Session (User+) | Both | Discover MCP metadata | — |
| POST | `/bodhi/v1/mcps/oauth/dynamic-register` | Session (User+) | Both | Dynamic client registration | — |

### MCP Servers

| Method | Path | Auth | Modes | Purpose | Spec |
|--------|------|------|-------|---------|------|
| GET | `/bodhi/v1/mcps/servers` | Session (User+) | Both | List MCP servers | — |
| POST | `/bodhi/v1/mcps/servers` | Session (Admin) | Both | Create MCP server | — |
| GET | `/bodhi/v1/mcps/servers/{id}` | Session (User+) | Both | Get MCP server | — |
| PUT | `/bodhi/v1/mcps/servers/{id}` | Session (Admin) | Both | Update MCP server | — |

### Toolsets

| Method | Path | Auth | Modes | Purpose | Spec |
|--------|------|------|-------|---------|------|
| GET | `/bodhi/v1/toolset_types` | Session (User+) | Both | List toolset types | — |
| PUT | `/bodhi/v1/toolset_types/{type}/app-config` | Session (Admin) | Both | Enable toolset type | — |
| DELETE | `/bodhi/v1/toolset_types/{type}/app-config` | Session (Admin) | Both | Disable toolset type | — |
| GET | `/bodhi/v1/toolsets` | Session/OAuth (User+) | Both | List toolsets | — |
| POST | `/bodhi/v1/toolsets` | Session (User+) | Both | Create toolset | — |
| GET | `/bodhi/v1/toolsets/{id}` | Session/OAuth (User+) | Both | Get toolset | — |
| PUT | `/bodhi/v1/toolsets/{id}` | Session (User+) | Both | Update toolset | — |
| DELETE | `/bodhi/v1/toolsets/{id}` | Session (User+) | Both | Delete toolset | — |
| POST | `/bodhi/v1/toolsets/{id}/tools/{tool_name}/execute` | Session/OAuth (User+) | Both | Execute tool | — |

### Settings

| Method | Path | Auth | Modes | Purpose | Spec |
|--------|------|------|-------|---------|------|
| GET | `/bodhi/v1/settings` | Session (Admin) | Both | List settings | — |
| PUT | `/bodhi/v1/settings/{key}` | Session (Admin) | Both | Update setting | [06](06-feature-gating.md#service-behavior) |
| DELETE | `/bodhi/v1/settings/{key}` | Session (Admin) | Both | Delete setting | — |

### Queue

| Method | Path | Auth | Modes | Purpose | Spec |
|--------|------|------|-------|---------|------|
| GET | `/bodhi/v1/queue` | Session (PowerUser+) | Both | Get queue status | — |

### Dev-Only (non-production)

| Method | Path | Auth | Modes | Purpose | Spec |
|--------|------|------|-------|---------|------|
| GET | `/dev/secrets` | Optional | Both | List secret env vars | — |
| GET | `/dev/envs` | Optional | Both | List all env vars | — |
| POST | `/dev/db-reset` | Optional | Both | Reset database | — |
| POST | `/dev/clients/{client_id}/dag` | Optional | Multi | Enable Direct Access Grants (D106) | — |
| GET | `/dev/tenants/cleanup` | Optional | Multi | List cleanup candidates | — |
| DELETE | `/dev/tenants/cleanup` | Optional | Multi | Cleanup stale tenants (D106) | — |

---

## Feature Matrix

| Feature | Standalone | Multi-Tenant | Spec |
|---------|-----------|-------------|------|
| Chat (OpenAI-compatible) | Local GGUF + API models | API models only | [06](06-feature-gating.md#chat) |
| Model aliases (CRUD) | Active | Active (API models only) | [06](06-feature-gating.md#llm-features) |
| Model files (GGUF) | Active | Disabled | [06](06-feature-gating.md#llm-features) |
| Model downloads | Active | Disabled | [06](06-feature-gating.md#llm-features) |
| API model configs | Active | Active | — |
| API tokens | Active | Active (tenant-scoped) | [04](04-api-tokens.md) |
| MCPs | Active | Active | — |
| Toolsets | Active | Active | — |
| User management | Active | Active | — |
| Settings | All settings | LLM settings restricted | [06](06-feature-gating.md#service-behavior) |
| Dashboard login | Error (D101) | Active | [01](01-auth-flows.md#dashboard-oauth) |
| Tenant management | Error (D101) | Active | [02](02-tenant-management.md) |
| Tenant switching | N/A (1 tenant) | Active | [02](02-tenant-management.md#tenant-switching) |
| External app access | Active | Active | [05](05-external-oauth.md) |
| Setup wizard | Active | N/A (tenant created Ready) | [02](02-tenant-management.md) |

---

## Cross-Reference Index

| Topic | Primary Spec | Related Specs |
|-------|-------------|---------------|
| Login flows | [01-auth-flows](01-auth-flows.md) | [03-info-endpoint](03-info-endpoint.md) |
| Session architecture | [01-auth-flows](01-auth-flows.md) | [04-api-tokens](04-api-tokens.md) |
| Tenant CRUD | [02-tenant-management](02-tenant-management.md) | [01-auth-flows](01-auth-flows.md) |
| AppStatus state machine | [03-info-endpoint](03-info-endpoint.md) | [02-tenant-management](02-tenant-management.md) |
| Token format & scoping | [04-api-tokens](04-api-tokens.md) | [06-feature-gating](06-feature-gating.md) |
| External app auth | [05-external-oauth](05-external-oauth.md) | [04-api-tokens](04-api-tokens.md) |
| Feature availability | [06-feature-gating](06-feature-gating.md) | [03-info-endpoint](03-info-endpoint.md) |

---

## TECHDEBT

> **TECHDEBT** [F7]: Navigation visibility — LLM-specific nav items (Model Files, Downloads) should be hidden in multi-tenant mode based on `deployment` from `/info`. Not yet implemented. See [TECHDEBT.md](../TECHDEBT.md).

> **TECHDEBT** [F8]: Service construction — LLM-specific routes and llama_server_proc listener should be conditionally skipped in multi-tenant mode. Not yet implemented. See [TECHDEBT.md](../TECHDEBT.md).
