# 01 — BodhiApp codebase map (for the tunnel feature)

Findings from exploring the Rust workspace. These are the concrete files/patterns the implementation will touch. Paths are relative to the repo root.

## 1. Desktop vs container distinction (the feature-flag gate)

BodhiApp already has the exact distinction this feature needs.

- Compile-time: the `bodhi` crate has a **`native` Cargo feature** (`crates/bodhi/src-tauri/Cargo.toml`).
  - `crates/bodhi/src-tauri/src/lib.rs` selects `native_init` (with `native`) vs `server_init` (without).
  - `crates/bodhi/src-tauri/src/app.rs` re-exports the matching initializer; the `Serve` CLI subcommand exists only without `native`.
- Runtime: `AppType` enum in `crates/services/src/settings/setting_objs.rs` (lines ~21–33): `Native` | `Container`.
  - `crates/bodhi/src-tauri/src/native_init.rs` sets `APP_TYPE = AppType::Native`.
  - `crates/bodhi/src-tauri/src/server_init.rs` sets `APP_TYPE = AppType::Container`.
  - It becomes the system setting `BODHI_APP_TYPE` in `crates/lib_bodhiserver/src/app_dirs_builder.rs` (`build_system_settings`).
- Accessor: `SettingService::app_type()` and **`SettingService::is_native()`** in `crates/services/src/settings/setting_service.rs` (lines ~126–133 and ~189–191).

> Recommended gate for "enabled on desktop, disabled on non-desktop by default": use `is_native()` at runtime (or `AppType::Native`), possibly overridden by an explicit setting. Note `native` Cargo feature is a *compile-time* split (a single binary is either native or server), so it can gate code inclusion, but the runtime `AppType` is what should drive default on/off.

## 2. Settings / env config

- Central keys + defaults: `crates/services/src/settings/constants.rs`.
  - Relevant existing keys: `BODHI_HOST`, `BODHI_PORT`, `BODHI_SCHEME`, `BODHI_PUBLIC_HOST`, `BODHI_PUBLIC_PORT`, `BODHI_PUBLIC_SCHEME`, `BODHI_CANONICAL_REDIRECT`, `BODHI_APP_TYPE`, `BODHI_AUTH_URL`, `BODHI_AUTH_REALM`, `BODHI_ENCRYPTION_KEY`.
  - `LOOPBACK_HOSTS = ["localhost", "127.0.0.1", "0.0.0.0"]`.
- Resolution precedence (System → CommandLine → Environment → Database → SettingsFile → Default): `crates/services/src/settings/default_service.rs` `get_setting_value_with_source()` (lines ~361–384). `.env` in `$BODHI_HOME` is loaded in `from_parts`.
- Public URL helpers in `crates/services/src/settings/setting_service.rs`:
  - `public_scheme()`, `public_host()`, `public_port()`, `public_server_url()` (lines ~355–433).
  - `get_public_host_explicit()` (~390) — true only when `BODHI_PUBLIC_HOST` is explicitly set or RunPod active.
  - `login_callback_url()` / `dashboard_callback_url()` (~463/467) built from `public_server_url()`.
  - RunPod synthesis (`on_runpod_enabled()`, ~516) already produces an automatic public URL — the closest existing analog to this feature.
- New tunnel settings would follow this same pattern: define `BODHI_TUNNEL_*` constants, add accessors on `SettingService`, and (if user-editable) add to `SETTING_VARS`.

## 3. Feature flags

There is **no runtime product-feature-flag system** — "feature flags" today are **Cargo features**:

- `bodhi`: `native`, `production`, `test-utils`.
- `lib_bodhiserver`: `embed-ui`, `test-utils`.
- Others: `test-utils` throughout.

So "enable/disable the tunnel feature" can be implemented either as a Cargo feature (compile-time) or, more naturally for a user-facing toggle, as a settings/env flag plus the `AppType`/`is_native()` gate. The existing `native` split is the precedent to follow.

## 4. HTTP server / route registration (where tunnel endpoints go)

- Framework: **axum** (`crates/routes_app/Cargo.toml`), router in `crates/routes_app/src/routes.rs` `build_routes()` (lines ~103–648).
- Endpoint constants via `make_ui_endpoint!` in `crates/routes_app/src/shared/openapi.rs` (prefix `/bodhi/v1/`).
- Role-scoped sub-routers merged in `build_routes()`:
  - `public_apis` (no auth), `optional_auth`, `guest_endpoints`, `user_apis`, `user_session_apis`, `apps_apis`, `power_user_apis`, `admin_session_apis`, `manager_session_apis`.
- Handler pattern: `#[utoipa::path(...)] async fn handler(auth_scope: AuthScope, ...) -> Result<T, BodhiErrorResponse>`; `AuthScope` extractor in `crates/routes_app/src/shared/auth_scope_extractor.rs`.
- Middleware: `crates/routes_app/src/middleware/auth/auth_middleware.rs`, `middleware/apis/api_middleware.rs` (`api_auth_middleware` with role/scope), `middleware/redirects/canonical_url_middleware.rs`.
- Server bind/listen: `crates/server_app/src/server.rs` (`Server::start_new`, `TcpListener::bind`), orchestrated by `crates/server_app/src/serve.rs` (`ServeCommand::get_server_handle` / `aexecute`).

Tunnel management endpoints (enable/disable/status/URL) belong under `admin_session_apis` (admin session role), following the settings-route pattern in `crates/routes_app/src/settings/routes_settings.rs`.

## 5. LLM/AI API serving (what gets exposed)

- OpenAI-compatible: `crates/routes_app/src/oai/` — `/v1/models`, `/v1/chat/completions`, `/v1/embeddings`, `/v1/responses`.
- Anthropic: `crates/routes_app/src/anthropic/` — `/v1/messages`, `/anthropic/v1/*`.
- Gemini: `crates/routes_app/src/gemini/` — `/v1beta/*`.
- **SSE streaming** is handled via `crates/server_core/src/fwd_sse.rs` (`RawSSE`, `fwd_sse()`). Streaming responses are `text/event-stream` — this is exactly what **quick tunnels buffer/drop**.

## 6. Keycloak integration (for the redirect-URI management endpoint)

- `KeycloakAuthService` in `crates/services/src/auth/auth_service.rs` (impl `AuthService`).
  - Base URLs: `auth_api_url()` → `{auth_url}/realms/{realm}/bodhi` (custom Bodhi SPI); token URL for OIDC.
  - `register_client(...)` (~311), `create_tenant(...)` (~837), `get_app_client_info()` (~760) reads `redirect_uris` (`AppClientInfo` ~163).
  - **`forward_request()` (~791–835)** is the generic SPI forwarder — the low-level pattern to reuse for a new SPI call.
- **There is no Rust client for the standard Keycloak Admin REST API** (`/admin/realms/...`). The only Admin REST usage is a JS test util: `crates/lib_bodhiserver/tests-js/utils/auth-server-client.mjs` `addRedirectUri()` (~466–516), which does `GET /admin/realms/{realm}/clients?clientId=...` then `PUT /admin/realms/{realm}/clients/{id}`.
- Redirect URIs are only **set at creation time** today (`setup_create` in `crates/routes_app/src/setup/routes_setup.rs` ~132–168; `tenants_create` in `crates/routes_app/src/tenants/routes_tenants.rs` ~111–112). There is **no** add/remove/update redirect-URI method or route in the Rust backend.
- Session auth (client_id + client_secret) lives in `crates/routes_app/src/middleware/token_service/token_service.rs` and `crates/routes_app/src/auth/routes_auth.rs`.

### Two ways to implement redirect-URI add/remove/update

1. **Extend the Bodhi SPI** (consistent with existing code): add a new method on `AuthService` (e.g. `update_redirect_uris`) that POSTs to a new Bodhi SPI endpoint (e.g. `/realms/{realm}/bodhi/resources/redirect-uris`), and implement the endpoint in the external Keycloak SPI (out of this repo). Reuse `forward_request()`.
2. **Use the standard Keycloak Admin REST API** from Rust: add a small admin client (or extend `KeycloakAuthService`) to call `PUT /admin/realms/{realm}/clients/{id}` with the merged `redirectUris` list. This requires an admin-capable token (realm admin service account), which the current app does not obtain — the JS test util logs in via `admin-cli` + password grant.

The user's stated plan ("update the keycloak, have an endpoint to add/remove/redirect a URL for a given client") aligns with **option 1** (extend the SPI). Option 2 avoids an external SPI change but introduces a new auth/privilege surface.

## 7. Existing tunneling/proxy code

- **None.** No ngrok/tailscale/cloudflared integration exists. The only tunnel-like behavior is:
  - RunPod automatic public host synthesis (`setting_service.rs`).
  - UI dev proxy (`crates/routes_app/src/routes_proxy.rs`), MCP proxy (`crates/routes_app/src/mcps/mcp_proxy.rs`), llama.cpp proxy (`crates/llama_server_proc/src/server.rs`) — all internal reverse proxies, not public tunnels.

So this is a greenfield module.

## 8. Where the new code most likely lands

- New crate or module (e.g. `crates/tunnel/` or a module under `services`) for:
  - `cloudflared` binary download/versioning (per-OS, mirror GitHub releases).
  - Process manager (spawn, PID tracking, watchdog, restart, stop).
  - Quick-tunnel URL parsing (stdout/log regex for `https://*.trycloudflare.com`).
  - Named-tunnel management via `cloudflare` crate (optional phase 2).
- `services` crate: new `TunnelService` trait + `DefaultTunnelService`, wired into `AppServiceBuilder` (`crates/lib_bodhiserver/src/app_service_builder.rs`).
- `routes_app`: new `tunnel` route module + `ENDPOINT_*` constants, registered under `admin_session_apis` in `build_routes()`.
- `constants.rs` / `setting_service.rs`: `BODHI_TUNNEL_ENABLED`, `BODHI_TUNNEL_MODE`, `BODHI_CLOUDFLARE_API_TOKEN`, etc., gated by `is_native()`.
- Keycloak: new `AuthService` method + external SPI endpoint for redirect-URI management.
