# Fix: MCP Header/Query Auth Credential Input Fields

## Context

When creating an MCP instance with header/query auth, the form only shows a read-only summary of key names (e.g., "Keys: query:tavilyApiKey") but provides **no input fields** for entering actual credential values. This means:
- "Fetch Tools" can't authenticate (backend ignores `auth_config_id` for preview)
- "Create MCP" sends no `credentials` array — the instance is created without credentials
- Tool execution fails because no auth params are stored

The `McpRequest` already has a `credentials: Option<Vec<McpAuthParamInput>>` field that the backend handles correctly for storing encrypted credentials — the form just never populates it.

## Changes

### Layer 1: Services — `fetch_tools_for_server` signature change

**`crates/services/src/mcps/mcp_service.rs`**
- Replace `auth_header_key: Option<String>, auth_header_value: Option<String>` with `credentials: Option<Vec<McpAuthParamInput>>` in the trait and impl
- In impl: build `McpAuthParams` from credentials array (split by `param_type` into headers vs query_params)

**`crates/services/src/mcps/auth_scoped.rs`**
- Update `fetch_tools_for_server` signature to match

### Layer 2: Routes — DTO + handler

**`crates/routes_app/src/mcps/mcps_api_schemas.rs`**
- Add `credentials: Option<Vec<McpAuthParamInput>>` to `FetchMcpToolsRequest`
- Import `McpAuthParamInput` from services

**`crates/routes_app/src/mcps/routes_mcps.rs`**
- Update `mcps_fetch_tools` handler: prefer `request.credentials`, fall back to legacy `McpAuth::Header` → single-credential conversion for backward compat

**`crates/routes_app/src/mcps/test_mcps.rs`**
- Update mock expectations for new `fetch_tools_for_server` signature

### Layer 3: Regenerate OpenAPI + TS client

### Layer 4: Frontend store

**`crates/bodhi/src/stores/mcpFormStore.ts`**
- Add `credentialValues: Record<string, string>` state
- Add `setCredentialValue(key, value)`, `clearCredentialValues()`
- Include in `saveToSession`/`restoreFromSession`, `disconnect()`, `reset()`

### Layer 5: Frontend form

**`crates/bodhi/src/app/ui/mcps/new/page.tsx`**
- **Replace** the read-only `auth-config-header-summary` div (lines 655-672) with credential input fields: for each `entry` in the selected config's `entries`, render a password input with toggle
- `handleAuthConfigChange`: call `store.clearCredentialValues()` on config change
- `handleFetchTools`: build `credentials` array from `store.credentialValues` + config entries, pass in mutation
- `onSubmit`: include `credentials` array in the payload for header auth type
- Session restore: restore `credentialValues` from session state

### Layer 6: Frontend tests

**`crates/bodhi/src/app/ui/mcps/new/page.test.tsx`**
- Update header auth tests to verify credential inputs render
- Verify credentials are included in fetch tools and create requests

### Layer 7: E2E tests

**`crates/lib_bodhiserver_napi/tests-js/specs/mcps/mcps-header-auth.spec.mjs`**
- Replace `page.evaluate(fetch(...))` API bypass with form-based flow: select config in dropdown → fill credential inputs → fetch tools → create MCP
- Add McpsPage helpers: `fillCredentialValue(paramKey, value)`, `expectCredentialField(paramKey)`

## Verification

1. `cargo check -p services -p routes_app -p server_app`
2. `cargo test -p services --lib -- mcps`
3. `cargo test -p routes_app --lib -- mcps`
4. `cargo run --package xtask openapi && make build.ts-client`
5. `cd crates/bodhi && npm test`
6. `make build.ui-rebuild`
7. `cd crates/lib_bodhiserver_napi && npx playwright test --project standalone -- tests-js/specs/mcps/mcps-header-auth.spec.mjs`
