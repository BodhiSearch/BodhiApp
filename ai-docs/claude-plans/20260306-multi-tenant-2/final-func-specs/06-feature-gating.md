# Multi-Tenant Functional Specification — Feature Gating

> **Scope**: LLM feature availability, navigation visibility, service construction, chat accessibility, error behavior
> **Related specs**: [Index](00-index.md) · [Info Endpoint](03-info-endpoint.md) · [Tenant Management](02-tenant-management.md)
> **Decisions**: D23, D67, D79, D101

---

## Route Gating Strategy

**All routes are registered in both deployment modes.** There is no conditional route registration based on deployment mode (D23 — unified code path). Instead:

- **Multi-tenant management endpoints** (`/tenants/*`, `/auth/dashboard/*`) check `is_multi_tenant()` at handler entry and return an error in standalone mode (D101)
- **LLM-dependent services** return errors when operations require local GGUF models in multi-tenant mode
- **Setup routes** (`POST /setup`) are gated by `AppStatus` (only works when status is `Setup`)

### Multi-Tenant Endpoint Guard

All dashboard auth and tenant management handlers start with:

```rust
if !settings.is_multi_tenant().await {
  return Err(DashboardAuthRouteError::NotMultiTenant)?;
}
```

**Error response (500):**
```json
{
  "error": {
    "message": "Multi-tenant mode is not enabled",
    "type": "not_multi_tenant",
    "code": "dashboard_auth_route_error-not_multi_tenant"
  }
}
```

**Guarded endpoints:**
- `POST /bodhi/v1/auth/dashboard/initiate`
- `POST /bodhi/v1/auth/dashboard/callback`
- `GET /bodhi/v1/tenants`
- `POST /bodhi/v1/tenants`
- `POST /bodhi/v1/tenants/{client_id}/activate`

---

## Service Construction {#service-behavior}

Services are constructed differently based on deployment mode at startup:

| Service | Standalone | Multi-Tenant |
|---------|-----------|-------------|
| `InferenceService` | `SingleInstanceInferenceService` — manages local llama.cpp process | `SharedInferenceService` — no local LLM |
| `HubService` | Active — downloads GGUF models from hub | Disabled — no model file management |
| `DataService` | `DefaultDataService` — full model aliases + GGUF metadata | `MultiTenantDataService` — API models only |

### Implications for Routes {#llm-features}

| Route Category | Standalone | Multi-Tenant |
|---------------|-----------|-------------|
| Model alias CRUD (`/bodhi/v1/models`) | Full (GGUF + API) | API models only |
| Model files (`/bodhi/v1/modelfiles`) | Returns GGUF files | Returns empty / error |
| Model pull (`/bodhi/v1/modelfiles/pull`) | Downloads from hub | Returns error |
| Chat completions (`/v1/chat/completions`) | Local GGUF + API | API models only |
| API models (`/bodhi/v1/api-models`) | Full | Full |
| Settings update | All settings editable | LLM settings restricted |

### Settings Restrictions in Multi-Tenant

LLM-specific settings cannot be modified via API in multi-tenant mode:

```rust
const LLM_SETTINGS: &[&str] = &["BODHI_EXEC_VARIANT", "BODHI_KEEP_ALIVE_SECS"];

// PUT /bodhi/v1/settings/{key}
if settings.is_multi_tenant().await && LLM_SETTINGS.contains(&key.as_str()) {
  return Err(SettingsRouteError::Unsupported)?;
}
```

---

## Chat Accessibility {#chat}

### Desired End-State

Chat is **accessible in multi-tenant mode** using API models (external API configurations like OpenAI, Anthropic, etc.):

| Model Type | Standalone | Multi-Tenant |
|-----------|-----------|-------------|
| Local GGUF | Available | Not available |
| API models (OpenAI, etc.) | Available | Available |

### How It Works

1. User configures API model providers via `/bodhi/v1/api-models` (e.g., OpenAI API key)
2. API model configs create model aliases that point to external APIs
3. Chat completions (`/v1/chat/completions`) route requests to the configured API
4. No local llama.cpp process needed — requests proxied directly to external API

---

## Navigation Visibility (Desired End-State) {#nav-visibility}

Frontend navigation should be filtered based on `deployment` from `/info` (D67):

| Navigation Item | Standalone | Multi-Tenant |
|----------------|-----------|-------------|
| Chat | Show | Show |
| API Models | Show | Show |
| API Tokens | Show | Show |
| MCPs | Show | Show |
| Toolsets | Show | Show |
| Settings | Show | Show |
| Model Files | Show | **Hide** |
| Downloads | Show | **Hide** |

> **TECHDEBT** [F7]: Navigation visibility filtering not yet implemented. LLM-specific nav items (Model Files, Downloads) should be hidden when `deployment === 'multi_tenant'`. Implementation location: `use-navigation.tsx`, filter `defaultNavigationItems` based on `AppInfo.deployment`. See [TECHDEBT.md](../TECHDEBT.md).

---

## Setup Flow Differences

### Standalone Setup

Multi-step wizard when `AppStatus == Setup`:

```
1. GET /info → status: "setup"
2. /ui/setup → setup wizard form (server name, min 10 chars)
3. POST /bodhi/v1/setup { name, description? }
   → SPI creates client, BodhiApp creates tenant (ResourceAdmin)
4. POST /auth/initiate { client_id }
5. POST /auth/callback → make_resource_admin + set_client_ready
6. GET /info → status: "ready"
```

### Multi-Tenant Setup

No wizard — tenant created `Ready` immediately (D79):

```
1. GET /info → status: "setup" (dashboard session, 0 tenants)
2. /ui/setup/tenants → registration form (name, description)
3. POST /bodhi/v1/tenants { name, description }
   → SPI creates client, BodhiApp creates tenant (Ready)
4. Auto: POST /auth/initiate { client_id }
5. POST /auth/callback → sets active_client_id
6. GET /info → status: "ready"
```

Key differences:
- No `ResourceAdmin` intermediate state in multi-tenant
- No `make_resource_admin` call (roles assigned via SPI groups)
- Tenant immediately `Ready` — user can start using the app right away

---

## Error Behavior

### OpenAI-Compatible Error Format

All errors across the application use OpenAI-compatible format:

```json
{
  "error": {
    "message": "Human-readable error message",
    "type": "error_type",
    "code": "error_enum-variant_name",
    "param": "field_name"
  }
}
```

### Feature-Gating Errors

| Scenario | Error Code | HTTP Status |
|----------|-----------|-------------|
| Dashboard endpoint in standalone | `dashboard_auth_route_error-not_multi_tenant` | 500 |
| Tenant endpoint in standalone | `dashboard_auth_route_error-not_multi_tenant` | 500 |
| LLM settings update in multi-tenant | `settings_route_error-unsupported` | 400 |
| Model pull in multi-tenant | Service-level error | 500 |
| Model file listing in multi-tenant | Empty result or service error | 200/500 |
| SPI proxy failure | `dashboard_auth_route_error-spi_request_failed` | 500 |

---

## TECHDEBT

> **TECHDEBT** [F7]: Navigation visibility — LLM-specific nav items (Model Files, Downloads) should be hidden in multi-tenant mode based on `deployment` from `/info`. Not yet implemented in `use-navigation.tsx`. See [TECHDEBT.md](../TECHDEBT.md).

> **TECHDEBT** [F8]: Service construction — LLM-specific routes (model pull, model files) and `llama_server_proc` listener should be conditionally skipped in multi-tenant mode. Currently all routes are registered but services return errors. See [TECHDEBT.md](../TECHDEBT.md).
