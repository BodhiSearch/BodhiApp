# 21 — Codebase map: settings, network helpers, and `/info` (2026-09-15)

Scope: exact file:line findings for wiring `BODHI_TUNNEL_*` settings and extending `/bodhi/v1/info` with a served-origins list. Extends `01-bodhi-app-codebase-map.md` §2 (settings) with the precedence/default mechanics in full, and is new ground versus the other research docs (they don't cover `/info`, CORS/CSP/cookie interaction, or the Vite HMR proxy). Read `01-bodhi-app-codebase-map.md` first — this doc does not repeat its §1/§6/§7/§8 findings.

## 1. Settings layer — precedence, defaults, and the AppType-default question

**Precedence** (`crates/services/src/settings/default_service.rs:361-384`, `get_setting_value_with_source`): System settings (from `Vec<Setting>` baked at boot) → CommandLine → Environment (`env_wrapper.var`) → Database → SettingsFile (`settings.yaml`) → Default (in-memory `HashMap` built once by `build_all_defaults`). `BODHI_APP_TYPE` itself is a **System**-source setting, not a Default (`crates/lib_bodhiserver/src/app_dirs_builder.rs:103-108`, `build_system_settings`), set from `AppOptions.app_type` — `native_init.rs:18` (`const APP_TYPE: AppType = AppType::Native`) vs `server_init.rs:54` (`AppType::Container`).

**`SETTING_VARS`** (`crates/services/src/settings/constants.rs:82-105`) is the list `list()` (`default_service.rs:398-410`) and `is_valid_db_key()` (`default_service.rs:119-121`) iterate — a new `BODHI_TUNNEL_*` key must be added here to appear in `GET /bodhi/v1/settings` and to be writable to the DB at all.

**No existing default depends on `AppType`.** `build_all_defaults()` (`default_service.rs:183-278`) takes `env_type: &EnvType`, `env_wrapper`, `file_defaults`, `bodhi_home` — **not** `AppType`. The only env-conditioned default today is `BODHI_REFERENCE_API_URL` (`default_service.rs:264-275`): `if env_type.is_production() { PROD } else { DEV }` inside the same `ensure_default!` macro pattern (`default_service.rs:195-199`) used for every other default. This is the shape to copy, but for `EnvType` not `AppType`.

**How `EnvType` gets into `build_all_defaults` is the exact precedent to replicate for `AppType`/`is_native`:** `DefaultSettingService::from_parts()` (`default_service.rs:67-117`) pulls `env_type` out of `parts.system_settings` *before* calling `build_all_defaults` (`default_service.rs:94-99`):
```rust
let env_type = parts.system_settings.iter()
  .find(|s| s.key == BODHI_ENV_TYPE)
  .and_then(|s| s.value.as_str().and_then(|v| v.parse::<EnvType>().ok()))
  .unwrap_or_default();
let defaults = build_all_defaults(&env_type, parts.env_wrapper.as_ref(), &parts.file_defaults, &parts.bodhi_home);
```
`parts.system_settings` already contains the `BODHI_APP_TYPE` Setting at this point (built in `app_dirs_builder.rs:103-108` before `BootstrapParts` is constructed), so the same block can extract `is_native: bool` the same way and thread it into `build_all_defaults(env_type, is_native, ...)`, then:
```rust
ensure_default!(BODHI_TUNNEL_ENABLED, Value::Bool(is_native));
```
This is a small, mechanical change (new parameter on a private fn used from exactly one call site) — there is no architectural blocker, just no existing example of a second parameter. Confirm no other caller of `build_all_defaults` exists before changing its signature (`grep -rn "build_all_defaults"` — only `default_service.rs:101` at time of writing).

**Alternative (no signature change):** compute `is_native` from `SettingService::is_native().await` *inside* the `tunnel_enabled()` accessor default arm instead of via `build_all_defaults`, mirroring how `public_scheme()`/`public_host()`/`public_port()`/`get_public_host_explicit()` branch on `on_runpod_enabled().await` **at read time** rather than at defaults-build time (`setting_service.rs:355-405`, e.g. `SettingSource::Default if self.on_runpod_enabled().await => "https".to_string()`). This is actually the *stronger* precedent — RunPod's public-host synthesis is a trait-default method on `SettingService`, not a `build_all_defaults` entry, and it's the closest existing analog to "default depends on runtime state" called out in `01-bodhi-app-codebase-map.md` §2. Recommendation: add `async fn tunnel_enabled(&self) -> bool` as a trait-default method following the `is_secure_transport()`/`on_runpod_enabled()` pattern (`setting_service.rs:368-370`, `516-530`) rather than touching `build_all_defaults`:
```rust
async fn tunnel_enabled(&self) -> bool {
  let (value, source) = self.get_setting_value_with_source(BODHI_TUNNEL_ENABLED).await;
  match source {
    SettingSource::Default => self.is_native().await,
    _ => value.and_then(|v| v.as_bool()).unwrap_or(false),
  }
}
```
This keeps the "default = is_native()" logic co-located with the setting's semantics (like RunPod) instead of a boot-time global, and needs zero signature changes elsewhere.

**Editability is a separate, narrower gate — do not conflate with `SETTING_VARS`.** `EDIT_SETTINGS_ALLOWED` (`constants.rs:77`) is `&[BODHI_EXEC_VARIANT, BODHI_KEEP_ALIVE_SECS]` — only these two keys are writable via `PUT /bodhi/v1/settings/{key}` (enforced in `routes_settings.rs:111` and `:184` for update/delete). Every other `SETTING_VARS` entry (including `BODHI_PUBLIC_HOST`, `BODHI_CANONICAL_REDIRECT`, etc.) is **list-only** through the generic settings API today. The frontend mirrors this exactly: `EDITABLE_KEYS = new Set(['BODHI_EXEC_VARIANT', 'BODHI_KEEP_ALIVE_SECS'])` (`crates/bodhi/src/routes/settings/-components/settingsFormat.tsx:6`), consumed by `SettingRow.tsx:15` to decide whether to render the edit pencil. **Conclusion for tunnel enable/disable: do not add `BODHI_TUNNEL_ENABLED` to `EDIT_SETTINGS_ALLOWED`/`EDITABLE_KEYS`.** Toggling the tunnel has side effects (spawn/stop `cloudflared`, sync Keycloak redirect URIs) that a bare `set_setting_value` PUT must not trigger silently; it needs a dedicated `POST /bodhi/v1/tunnel/enable` / `disable` route (per `01-bodhi-app-codebase-map.md` §4/§8) that calls `settings.set_setting_with_source(BODHI_TUNNEL_ENABLED, ..., SettingSource::Database)` directly (bypassing the `EDIT_SETTINGS_ALLOWED` gate, which is only enforced in `routes_settings.rs`, not in `SettingService` itself) after doing the process/SPI work. `GET /bodhi/v1/settings` still lists `BODHI_TUNNEL_ENABLED` for visibility (it needs to be in `SETTING_VARS`), and the settings UI will show it as a read-only row automatically — `SettingsPageV2.tsx:40-55` already buckets any `SETTING_VARS` key not in the static `SETTINGS_CONFIG` (`crates/bodhi/src/routes/settings/index.tsx:32-113`) into a dynamic "ungrouped" group with no edit affordance, so no frontend change is strictly required to make it visible (a curated group/description is still worth adding for UX).

## 2. `NetworkService` (LAN-IP helper)

`crates/services/src/utils/network_service.rs:1-30` — trait `NetworkService { fn get_server_ip(&self) -> Option<String> }`, one impl `DefaultNetworkService`. Implementation (`:13-29`): binds a UDP socket to `0.0.0.0:0`, calls `.connect("8.8.8.8:80")` (no packet sent — just forces OS route selection), reads `socket.local_addr()`, returns the IP if non-loopback. **IPv4 only** — the hardcoded target `8.8.8.8:80` means an IPv6-only or IPv6-preferring host never gets an IPv6 local address back; there is no dual-stack fallback.

Wiring: `AppService::network_service()` (`crates/services/src/app_service/app_service.rs:37,135-137`) → `AuthScopedAppService::network()` (`crates/services/src/app_service/auth_scoped.rs:102-104`) → built in `crates/lib_bodhiserver/src/app_service_builder.rs:342-344` as `Arc::new(DefaultNetworkService)` (no config, always on). **Only call site today**: `crates/routes_app/src/setup/routes_setup.rs:160` inside `setup_create`, used to add the LAN IP to the OAuth `redirect_uris` list sent to Keycloak at tenant-creation time (single call, not cached/polled). This is the only precedent for LAN-IP-aware behavior in the codebase — the `/info` origins list is new ground.

## 3. `/bodhi/v1/info` (`AppInfo`) — current shape and handler

**Struct** `crates/routes_app/src/setup/setup_api_schemas.rs:21-45`:
```rust
pub struct AppInfo {
  pub version: String,
  pub commit_sha: String,
  pub status: AppStatus,
  pub deployment: DeploymentMode,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub client_id: Option<String>,
  pub url: String,                 // = settings.public_server_url()
  pub reference_api_url: String,
}
```
**Handler** `crates/routes_app/src/setup/routes_setup.rs:30-88` (`setup_show`): computes `status`/`client_id` from `AuthContext` (5-way match, `:34-77`), then builds `AppInfo` at `:79-87` — `url: settings.public_server_url().await` (single value, `setting_service.rs:425-433`: `{scheme}://{host}[:{port}]` from `public_scheme()`/`public_host()`/`public_port()`).

**Auth requirement**: registered in the `optional_auth` group (`crates/routes_app/src/routes.rs:118`, `.route(ENDPOINT_APP_INFO, get(setup_show))`), gated by `optional_auth_middleware` (`routes.rs:156-157`) — falls back to `AuthContext::Anonymous` rather than failing, so `/info` is reachable **unauthenticated**. Any `origins`/`urls` addition is therefore public information by default; if it should reveal the tunnel hostname (a secret-ish value worth not exposing to anonymous scanners) that's a design call to flag, not a given — likely fine since the tunnel hostname is only useful if you already know the domain, but worth an explicit product decision.

**Frontend consumer**: `crates/bodhi/src/hooks/info/useInfo.ts:9-11` (`useGetAppInfo` → `useQuery<AppInfo>(appInfoKeys.all, ENDPOINT_APP_INFO)`). Two real uses of the `url` field found:
- `crates/bodhi/src/components/AppInitializer.tsx` — reads `status`/`deployment`/`client_id` for the setup/login redirect state machine (per `crates/bodhi/src/CLAUDE.md` "App Initialization Flow"), does **not** use `url`.
- `crates/bodhi/src/routes/users/-components/InviteLinkAction.tsx:19` — `` `${appInfo.url}/ui/login/?invite=${appInfo.client_id}` `` — builds a shareable multi-tenant invite link directly from `url`. **This is the field that most wants to become "the tunnel URL when one is active, else public_server_url"** — a stable tunnel hostname is a better invite link than a LAN IP or `localhost`.

### Proposed extension

Keep `url` (back-compat — `InviteLinkAction.tsx` and any external consumer depend on a single string) but change its *value* to prefer an active tunnel origin over the current `public_server_url()`, and add a new list:

```rust
#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum OriginKind { Loopback, Lan, Public, Tunnel }

#[derive(Serialize, Deserialize, ToSchema)]
pub struct OriginInfo {
  pub url: String,        // e.g. "http://127.0.0.1:1135"
  pub kind: OriginKind,
  pub verified: bool,     // tunnel: true only once cloudflared reports connected
}

pub struct AppInfo {
  // ...existing fields unchanged...
  pub origins: Vec<OriginInfo>,
}
```
Data sources per kind, all already resolvable from existing services with no new plumbing beyond the tunnel state itself:
- **loopback**: `services::LOOPBACK_HOSTS` (`crates/services/src/settings/constants.rs:53`, `["localhost","127.0.0.1","0.0.0.0"]`) × `settings.scheme()`/`port()` (not `public_*` — these are the bind-level values, `setting_service.rs:211-241`).
- **lan**: `auth_scope.network().get_server_ip()` (same call already used in `routes_setup.rs:160`) × `settings.scheme()`/`port()`.
- **public**: only when `settings.get_public_host_explicit().await.is_some()` (`setting_service.rs:390-405`) — reuse `public_server_url()` verbatim; this already covers the RunPod case for free (RunPod's synthesized host reads as `Default` source but `get_public_host_explicit()` special-cases it to `Some`, `:392-399`).
- **tunnel**: from the new `TunnelService` (not yet built — see `01-bodhi-app-codebase-map.md` §8) once it reports a connected named tunnel + its public hostname; `verified` distinguishes "configured but cloudflared not yet up" from "confirmed reachable". This is the one kind with no existing data source — it's the new state this feature introduces.

Build this list in `setup_show` (`routes_setup.rs:79-87`) alongside the existing fields; it needs `auth_scope.network()` (already imported via `AuthScope`) and the new tunnel accessor.

## 4. Where a tunnel origin could be rejected or redirected

### 4.1 Canonical redirect — the real risk

`crates/routes_app/src/middleware/redirects/canonical_url_middleware.rs:15-77`. Applied as a **global outer layer** wrapping the whole router including the UI (`crates/routes_app/src/routes.rs:641-647` — `apply_ui_router` runs first at `:639`, then `.layer(session_layer)` at `:642`, then `.layer(canonical_url_middleware)` at `:643-646`, so canonical redirect sees every request to `/ui/*` and every API path alike).

Two early-outs matter:
1. `canonical_redirect_enabled()` — default `true` (`constants.rs:38`, `DEFAULT_CANONICAL_REDIRECT`).
2. **`get_public_host_explicit().await.is_none()` short-circuits the whole middleware** (`canonical_url_middleware.rs:26-29`). Since `get_public_host_explicit()` is `None` unless `BODHI_PUBLIC_HOST` is explicitly set (or RunPod is active) (`setting_service.rs:390-405`), **today, with no `BODHI_PUBLIC_HOST` configured, a tunnel hostname hitting the app is never redirected** — the middleware is a no-op. That's the safe default state.

The risk appears only if an operator (or a future tunnel-setup flow) sets `BODHI_PUBLIC_HOST`/`BODHI_PUBLIC_SCHEME`/`BODHI_PUBLIC_PORT` to something **other** than the tunnel hostname (e.g. a LAN IP, or leaves it pointing at `localhost`): `should_redirect_to_canonical()` (`:107-136`) compares the *request's* scheme/host/port against `public_scheme()`/`public_host()`/`public_port()`, and on any mismatch issues a 301 to `public_server_url()` + the original path (`:58-71`). A browser that reached the app via the tunnel hostname would then be redirected to the *other* configured public host/LAN-IP — likely unreachable from outside the LAN, breaking the tunnel entirely. **Design requirement**: either (a) tunnel-enable flow must set `BODHI_PUBLIC_HOST`/`SCHEME`/`PORT` to the tunnel's own values (making the tunnel host *the* canonical host, which then 301s loopback/LAN visitors — probably undesirable too, since LAN/loopback access should keep working), or (b) `should_redirect_to_canonical` needs a tunnel-aware exemption (skip when `request_host` matches the active tunnel hostname, independent of what `BODHI_PUBLIC_HOST` is set to). Neither exists today — this middleware currently assumes exactly one "canonical" public identity, and the tunnel introduces a second legitimate one (LAN/loopback being the others already implicitly tolerated only because the middleware no-ops without an explicit public host).

`extract_scheme()` (`:81-95`) already honors `x-forwarded-proto`/`x-forwarded-scheme` — relevant if `cloudflared` (or any reverse proxy in front) sets these; Cloudflare Tunnel's local ingress typically connects to the origin over plain HTTP even though the public leg is HTTPS, so whatever local process terminates the tunnel connection needs to set `x-forwarded-proto: https` (or the app needs to trust cloudflared's own `--protocol` framing) for this scheme comparison to line up. UNVERIFIED: whether `cloudflared`'s HTTP-to-origin proxying sets `X-Forwarded-Proto` by default — check `cloudflared` docs/source before relying on it.

### 4.2 CORS — not host-specific, not a blocker

`crates/routes_app/src/routes.rs:84-101`: `permissive_cors()` = `CorsLayer::new().allow_origin(Any)...allow_credentials(false)`; `restrictive_cors()` = `CorsLayer::new()` (all-default = deny, no origin reflected). Neither is an *allowlist* keyed on hostname — permissive allows every origin equally (with credentials disabled, per CORS spec `Access-Control-Allow-Origin: *` + no credentials), restrictive blocks every cross-origin caller equally regardless of who they are. **A tunnel origin is not treated differently from any other origin** by this layer. The dominant flow (browser loads the UI from the tunnel host and calls the API on that same tunnel host) is same-origin and never triggers CORS preflight/allow-origin checks at all — CORS only matters for the OAI-SDK/external-tool case, which permissive groups already allow from any origin.

### 4.3 CSP `connect-src` — self-relative, not host-specific

`crates/routes_app/src/spa_router.rs:16-28` (`build_csp`): `connect-src 'self' <reference_api_origin>`. `'self'` resolves relative to whatever origin the browser loaded the page from — so a page served over the tunnel gets a CSP whose `'self'` *is* the tunnel origin; no rejection possible here regardless of hostname. Only relevant external add would be if a tunnel-management UI needs to `fetch()` a *different* origin (e.g. a Cloudflare API call made client-side, which should not happen — that belongs server-side per the "opaque proxy"/provider-ownership conventions already in this codebase's design preferences).

### 4.4 Session cookie `Secure` flag — a real structural gap, not currently in security.md

`crates/routes_app/src/routes.rs:640,642`:
```rust
let secure_cookie = app_service.setting_service().is_secure_transport().await;
router.layer(app_service.session_service().session_layer(secure_cookie))
```
`is_secure_transport()` = `public_scheme() == "https"` (`setting_service.rs:368-370`). `session_layer(secure: bool)` (`crates/services/src/auth/session_service.rs:183-190`) builds ONE `SessionManagerLayer` with `.with_secure(secure).with_same_site(SameSite::Strict)` — **computed once at router-build time (server boot), applied globally to every request regardless of which Host/scheme it actually arrived on.** `docs/architecture/security.md:183` already documents "Session cookie Secure=false" as an accepted risk with the stated mitigation "Derive from `BODHI_PUBLIC_SCHEME` via `is_secure_transport()`" — i.e. today's accepted answer assumes a single scheme for the whole instance.

The tunnel feature breaks that assumption by design: the same running instance is now reachable over `http://localhost`, `http://<lan-ip>`, **and** `https://<tunnel-host>` simultaneously.
- If `BODHI_PUBLIC_SCHEME` stays `http` (default) while the tunnel is enabled → `secure_cookie=false` globally. Login still *works* over the https tunnel (browsers accept non-Secure cookies over https, Secure only restricts the reverse), but it's a regression versus the documented mitigation: the session cookie is now sendable in the clear over LAN http from the same browser profile.
- If a tunnel-setup flow sets `BODHI_PUBLIC_SCHEME=https` to close that gap → `secure_cookie=true` globally → **breaks session login for loopback/LAN http access on the same instance**, since browsers refuse to send `Secure` cookies over plain http.
There is no per-request/per-origin `Secure` toggle available in this wiring — `tower_sessions::SessionManagerLayer` is configured once, not per-connection. This is new ground for the tunnel design, not something already resolved by the accepted-risk note in `security.md`; flag it as an open design question (candidates: two cookie names/paths per scheme, a custom `Set-Cookie` rewrite based on the actual request scheme before the session layer, or accepting that enabling the tunnel means all origins on that instance are treated as https-only going forward).

### 4.5 Host header — no allowlist/rebinding guard exists

`extract_request_host`/`is_valid_hostname` (`crates/routes_app/src/shared/utils.rs:3-28`) only validate character set (alphanumeric, `.`, `-`) and length (≤253) — used solely to build the `redirect_uris` list in `setup_create` (`routes_setup.rs:150-157`) and similarly in `crates/routes_app/src/auth/routes_auth.rs:92`. There is **no middleware anywhere in `routes_app` that rejects a request based on its `Host` header value** (no DNS-rebinding guard, no explicit allowlist check). A tunnel hostname is never blocked at this layer — confirmed by grep across `middleware/` and `routes.rs`.

## 5. UI dev proxy / Vite HMR — origin sensitivity

**Initial page load through the Rust dev proxy is origin-agnostic** — `crates/routes_app/src/routes_proxy.rs`, `http_proxy()` (`:64-85`) rewrites the outgoing `Host` header to the Vite backend's own authority before forwarding (`:73-76`, "Replace Host header so Vite doesn't treat this as a cross-origin request"). Vite's dev server has its own Host-header validation (`server.allowedHosts`, not configured here) — but since axum always presents Vite with its own `localhost:<port>` Host, **Vite never sees the tunnel/LAN hostname the browser actually used**, so any Host reaching `/ui/*` proxies through fine. Same for the websocket-upgrade path in `ws_proxy()` (`:87-188`) — it writes a raw HTTP/1.1 upgrade request with `Host: {addr}` set to the backend address (`:110`), stripping the client's own `Host` header (`:112-114`).

**But the HMR *client* connection bypasses this proxy entirely and IS origin-sensitive.** `crates/bodhi/vite.config.ts:28-41`:
```ts
server: {
  port: devProxyPort,
  hmr: {
    // Behind the Rust proxy (make app.run.live) only /ui/* is proxied, so HMR must
    // connect directly to Vite's port instead of the browser's page origin.
    clientPort: devProxyPort,
  },
},
```
Vite's HMR client defaults its WebSocket target host to `location.hostname` and only overrides the **port** here — so the browser opens `ws(s)://<page-hostname>:<devProxyPort>/...` **directly to Vite**, not through the axum `/ui/*` proxy (`routes.rs:664-675` only proxies HTTP paths under `/ui`, port 3000 itself is never exposed through the Rust server's port). Over a Cloudflare **named tunnel** — which forwards only the single configured origin/port (the Bodhi server's port, e.g. 1135) — a direct connection to `<tunnel-host>:3000` has nothing listening on the public side (the tunnel doesn't forward an arbitrary port), so **HMR's websocket fails over the tunnel**: initial page load and normal navigation still work (served via the proxied `/ui/*` route), but hot module reload degrades to a broken/reconnecting-forever HMR client, not a silent success. This only affects the `make app.run.live` / `make test.e2e` **live-Vite dev loop**; it does not affect embedded-bundle serving (`spa_router.rs`, which serves static files with no proxy or second port involved) — so it matters only if the tunnel is exercised against a live-Vite dev/E2E setup, not against a production/Tauri/Docker build.

## Sources

Pure codebase research — no external sources. All claims above are `file_path:line_number` citations verified by reading the code at the paths listed (repo root: `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp`), current as of the `main` branch commit at the time of writing (`d4268ddb`).

---

## Follow-up: Finalize AppInfo.origins / tunnel-hostname exposure-to-anonymous-callers decision

**Date:** 2026-09-15. Answers the gap flagged in §3 ("that's a design call to flag, not a given") and the two open items it created downstream: `23-codebase-persistence-routes-and-backend-tests.md` "Open questions for the implementation plan" (`TunnelStatusResponse`/`ProvisionTunnelRequest` wire shape) and `24-codebase-frontend-settings-v2-and-e2e.md`'s "blocked on OpenAPI schema for `TunnelStatus`". This section is the single settled answer; docs 23 and 24 are corrected in place with one-line pointers here (see end of each).

### 1. Decision — tunnel hostname/origin IS included in the anonymous `/bodhi/v1/info` response

**Yes, include it, unauthenticated, exactly as §3's `OriginInfo`/`OriginKind` proposed** (finalized in §6 below). Rationale, in order of weight:

1. **The hostname is already public and already resolvable without the app's help.** A Cloudflare named tunnel's public hostname is a CNAME (or, in Full/Cloudflare-DNS setup, a proxied A/AAAA-equivalent) DNS record on the user's own zone, created by `cloudflared tunnel route dns <tunnel> <hostname>` and served by Cloudflare's public DNS resolvers the moment it's created — this is how tunnel routing works by design, not an implementation detail BodhiApp controls (Cloudflare, "DNS records", `developers.cloudflare.com/cloudflare-one/networks/connectors/cloudflare-tunnel/routing-to-tunnel/dns/`: *"you configure the tunnel to route to a hostname by creating a DNS record that points to your tunnel"*; also indexed by public Certificate Transparency logs the moment Cloudflare issues the edge TLS cert for it). Gating it behind admin auth on `/info` prevents nothing an attacker couldn't already get from `dig <hostname>` or crt.sh — it only degrades the legitimate pre-auth consumer below.
2. **A real pre-auth consumer already depends on `AppInfo.url` being reachable *before* login**: `crates/bodhi/src/routes/users/-components/InviteLinkAction.tsx:19` builds `${appInfo.url}/ui/login/?invite=...` — an invite link handed to someone who has no session yet. If the tunnel is the instance's real public entry point (the whole reason it's enabled), gating tunnel awareness behind admin auth would make `/info` return a `url`/`origins` set that's *wrong for the actual anonymous recipient of the invite link* (a LAN IP or `localhost` that isn't reachable from wherever they are). This consumer only exists because `/info` is unauthenticated today — moving the tunnel field to an admin-only endpoint breaks it, it doesn't protect it.
3. **`/bodhi/v1/info` has no precedent for a mixed public/private field split.** It is registered in the `optional_auth` group and every existing field (`version`, `status`, `deployment`, `url`, `reference_api_url`) is already visible to `AuthContext::Anonymous` (`crates/routes_app/src/routes.rs:118`, `crates/routes_app/src/setup/routes_setup.rs:30-88` — confirmed in §3 above). Splitting one field's visibility by auth state inside an otherwise-flat DTO is a new access-control shape this endpoint doesn't have anywhere else; §6's `AppInfo.origins` avoids inventing it.
4. **What actually stays secret is kept out of scope of this decision already**: `origins`/`OriginInfo` (§6) carries only `url` + `kind` — no `credentials_path`, no token presence, no `auth_mode`, no Cloudflare account/zone IDs. Those live exclusively in `TunnelStatusResponse` behind `GET /bodhi/v1/tunnel`, which `23-codebase-persistence-routes-and-backend-tests.md` already correctly scoped to `session_auth = ["resource_admin"]` (lines ~157-158, ~193-194 in that doc). So the "anonymous exposure" surface is deliberately just a hostname string + an enum tag, never the tunnel's operational/auth state.

**Narrowing versus §3's original proposal**: drop the `verified: bool` field from the *public* `OriginInfo` (kept in §3's draft). Reasoning: a `Tunnel`-kind entry should only ever appear in `AppInfo.origins` once the tunnel is actually `TunnelStatus::Enabled` (i.e., cloudflared reports connected) — a provisioning/error tunnel should simply be **absent** from the anonymous list rather than present-with-`verified:false`. Exposing "a tunnel exists but isn't up yet" to anonymous callers leaks operator activity (someone is mid-setup) for no benefit to any legitimate anonymous consumer — the invite-link use case only needs the URL once it's actually reachable. Internal-only "is it configured but not yet connected" state stays fully expressed in the admin-only `TunnelStatusResponse.status` enum (§6) — no information is lost, it's just not duplicated onto the public endpoint.

### 2. Naming conflict between docs 23 and 24 — resolved in favor of doc 23's shape

Doc 24 (`24-codebase-frontend-settings-v2-and-e2e.md:82,95-99,104,176-177,185`) was written against an assumed schema (`TunnelStatus` as the *response type name*, field `state`, endpoint `ENDPOINT_TUNNEL_STATUS`) that **does not match** doc 23's route skeleton (`23-codebase-persistence-routes-and-backend-tests.md:153-174`), which uses `TunnelStatusResponse` as the response type returned from `GET ENDPOINT_TUNNEL` (no `_STATUS` suffix), with a `status` field. Doc 23's shape is the one to keep, because it mirrors the one existing precedent for exactly this kind of DTO in this codebase: `DownloadStatus` (enum, `crates/services/src/models/model_objs.rs:373-377`, field name `status`) + `DownloadRequest` (the response struct that carries a `status: DownloadStatus` field, `crates/services/src/models/download_service.rs:34-49`) — not a struct literally named `...Status` carrying a `state` field. §6 below finalizes on: enum `TunnelStatus` (the lifecycle enum, `DownloadStatus`-shaped), struct `TunnelStatusResponse` (the DTO, `DownloadRequest`-shaped) with field `pub status: TunnelStatus`, and endpoint constant `ENDPOINT_TUNNEL` (already defined in doc 23 §c) reused for `GET`, exactly as doc 23 already laid out. **Doc 24's hook/mock code (`useTunnelStatus.ts`, `mockTunnelStatus*`, `ENDPOINT_TUNNEL_STATUS`) needs `state` → `status` and `TunnelStatus` (as a type import) → `TunnelStatusResponse` before it's implemented** — flagged at the end of that doc.

Also: doc 23's route skeleton (`23-codebase-persistence-routes-and-backend-tests.md:153`) imports `EnableTunnelRequest` from `services::` but the `tunnel_enable` handler it shows takes no `ValidatedJson` body (`:171`) — that import is dead. **Decision: no request body for `/tunnel/enable` or `/tunnel/disable`; drop `EnableTunnelRequest` entirely**, it has no field to carry (enable/disable act on the already-provisioned singleton row).

### 3. `has_api_token` vs the `has_api_key` masking convention — confirmed, not renamed

Doc 23 flagged `has_api_token: bool` as a placeholder guess against `ApiAliasResponse.has_api_key` (`crates/services/src/models/model_objs.rs:2044-2063`, masks `ApiAlias`'s encrypted API key: the response never carries the ciphertext, only a boolean computed via `.with_has_api_key(has_api_key)` at the service layer, `crates/services/src/models/api_model_service.rs:218-224`). Checking the convention against the `tunnels` table's actual secret columns (`23-codebase-persistence-routes-and-backend-tests.md` §a, the `encrypted_api_token`/`api_token_salt`/`api_token_nonce` triplet): **`has_api_token` is in fact the correct name** — it mirrors `has_api_key` 1:1 (`api_key` ↔ `api_token`, same masking shape, boolean presence flag replacing the encrypted triplet in the response DTO). It was right to flag as unresolved (nothing had confirmed it), but it doesn't need to change. One addition: `credentials_path` (tier-1 `cloudflared` CLI credentials JSON, not app-encrypted but still a local filesystem detail with no frontend use beyond "is one configured") gets the same boolean-presence treatment for consistency — `has_credentials_file: bool` — rather than serializing the raw path. Finalized together in §6.

### 4. §6 — Finalized schema

All four types below go in a new `crates/services/src/tunnels/tunnel_objs.rs` (mirrors `crates/services/src/models/model_objs.rs`'s role for the `tunnels` domain), re-exported from `services::` the same way `TunnelStatusResponse`/`ProvisionTunnelRequest` are already imported in doc 23's route skeleton. `AppInfo`'s two new members (`OriginKind`, `OriginInfo`) stay in `crates/routes_app/src/setup/setup_api_schemas.rs` next to `AppInfo` itself, per §3 above — they are not tunnel-specific, `origins` also carries loopback/LAN/public entries.

```rust
// crates/routes_app/src/setup/setup_api_schemas.rs — extends AppInfo (§3)

/// Kind of network origin the app is reachable on. `Tunnel` is only ever present
/// once the tunnel is confirmed connected (see TunnelStatus::Enabled) — a
/// provisioning/error tunnel is simply absent from this list, not present with
/// a false "verified" flag (§ Follow-up, "Decision" note).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, strum::Display, ToSchema)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum OriginKind {
  Loopback,
  Lan,
  Public,
  Tunnel,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct OriginInfo {
  /// e.g. "http://127.0.0.1:1135", "https://bodhi.example.com"
  pub url: String,
  pub kind: OriginKind,
}

pub struct AppInfo {
  // ...existing fields unchanged (version, commit_sha, status, deployment, client_id, url, reference_api_url)...
  pub origins: Vec<OriginInfo>,
}
```

```rust
// crates/services/src/tunnels/tunnel_objs.rs (new)

/// Tunnel lifecycle state — DownloadStatus-shaped (model_objs.rs:373-377): plain
/// enum, `status` field name on the response DTO, no "state" synonym anywhere.
/// `missing_binary` (cloudflared not on PATH) deliberately does NOT live here —
/// that's `CloudflaredDetection` (23-codebase-....md §c), a separate concern
/// from the persisted tunnel row's lifecycle. A `Disabled` tunnel with no
/// cloudflared installed is still `status: "disabled"`; the frontend calls
/// `GET /bodhi/v1/tunnel/detect` separately to decide whether to show the
/// install nudge before offering to enable.
#[derive(
  Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, strum::Display, ToSchema,
  sea_orm::DeriveValueType,
)]
#[sea_orm(value_type = "String")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum TunnelStatus {
  Disabled,
  Provisioning,
  Enabled,
  Error,
}

/// Which of the three product-decision auth tiers is configured for this
/// instance's tunnel. `None` on TunnelStatusResponse.auth_mode means "never
/// configured" (status == Disabled, nothing provisioned yet).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, strum::Display, ToSchema)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum TunnelAuthMode {
  CloudflaredCli,
  Oauth,
  ApiToken,
}

/// GET /bodhi/v1/tunnel response. Admin-session-only (session_auth=resource_admin,
/// 23-codebase-....md §c) — never reachable from optional_auth/anonymous routes,
/// unlike AppInfo.origins above. Never carries credentials_path, the encrypted
/// token, or its salt/nonce — only presence booleans, same masking shape as
/// ApiAliasResponse.has_api_key (model_objs.rs:2044-2063).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct TunnelStatusResponse {
  pub status: TunnelStatus,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub auth_mode: Option<TunnelAuthMode>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub hostname: Option<String>,
  /// Full public URL, e.g. "https://bodhi.example.com" — set only once status == Enabled.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub public_url: Option<String>,
  /// True once a tier-3 pasted API token is stored (encrypted_api_token IS NOT NULL).
  /// Mirrors ApiAliasResponse.has_api_key — never serializes the token itself.
  pub has_api_token: bool,
  /// True once tier-1 `cloudflared tunnel login` has written a credentials JSON
  /// (credentials_path IS NOT NULL). The path itself is never serialized.
  pub has_credentials_file: bool,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub last_error: Option<String>,
  #[schema(value_type = Option<String>, format = "date-time")]
  pub last_synced_at: Option<chrono::DateTime<chrono::Utc>>,
  #[schema(value_type = String, format = "date-time")]
  pub created_at: chrono::DateTime<chrono::Utc>,
  #[schema(value_type = String, format = "date-time")]
  pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// POST /bodhi/v1/tunnel/provision request body. Issued after tier-1 `cloudflared
/// tunnel login` (POST /tunnel/login, no body) has produced a cert.pem; this step
/// creates the cfd_tunnel + DNS route + local config using that cert. account_id/
/// zone_id are resolved server-side from the authenticated cloudflared session
/// (or from the tier-3 API token's own scope, per 00-consolidated-research.md §1)
/// — never client-supplied, so they are not request fields.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, validator::Validate, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct ProvisionTunnelRequest {
  /// FQDN on the user's own Cloudflare zone, e.g. "bodhi.example.com".
  #[validate(length(min = 1, max = 253, message = "hostname must not be empty"))]
  pub hostname: String,
  /// Optional cfd_tunnel display name; defaults to a generated "bodhi-<host-id>" server-side when omitted.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub tunnel_name: Option<String>,
}
```

`#[serde(rename_all = "snake_case")]` on the two struct DTOs above is a no-op today (their field names are already snake_case, same as `ApiAliasResponse`/`DownloadRequest`, neither of which carries the attribute) — included per this task's brief for explicitness/uniformity with the enums, not because it changes wire output. Register all four in `crates/routes_app/src/shared/openapi.rs`'s `components(schemas(...))` list per doc 23 §c's checklist step 4.

### Pointers added

- `23-codebase-persistence-routes-and-backend-tests.md`: one-line pointer added near its "Open questions" section — *"schema finalized in 21 §6, use that verbatim."*
- `24-codebase-frontend-settings-v2-and-e2e.md`: one-line pointer added near its `TunnelStatus`/MSW-handler blockers — *"schema finalized in 21 §6, use that verbatim (note: `TunnelStatusResponse`/`status`/`ENDPOINT_TUNNEL`, not `TunnelStatus`/`state`/`ENDPOINT_TUNNEL_STATUS`)."*

### Sources

- Codebase (file:line, verified by reading, `main` @ `d4268ddb`): `crates/services/src/models/model_objs.rs:373-377` (`DownloadStatus`), `:2044-2063,2096-2099` (`ApiAliasResponse`/`has_api_key`), `:1089-1099` (`Validate`-derived request struct convention); `crates/services/src/models/download_service.rs:34-49` (`DownloadRequest`); `crates/services/src/models/api_model_service.rs:218-224` (`.with_has_api_key` call site); `crates/bodhi/src/routes/users/-components/InviteLinkAction.tsx:19` (pre-auth `appInfo.url` consumer); `crates/routes_app/src/routes.rs:118` (`/info` in `optional_auth` group); `crates/routes_app/src/setup/routes_setup.rs:30-88` (`setup_show`).
- `docs/research/tunnel/23-codebase-persistence-routes-and-backend-tests.md:153-174,193-194,422-426` (route skeleton, admin-session scoping, open questions this section resolves).
- `docs/research/tunnel/24-codebase-frontend-settings-v2-and-e2e.md:82,95-99,104,176-177,185` (the conflicting `TunnelStatus`/`state`/`ENDPOINT_TUNNEL_STATUS` shape being corrected).
- Cloudflare, "DNS records" (Cloudflare Tunnel routing) — https://developers.cloudflare.com/cloudflare-one/networks/connectors/cloudflare-tunnel/routing-to-tunnel/dns/ — confirms a named tunnel's public hostname is an ordinary DNS record (CNAME to `<UUID>.cfargotunnel.com`, or a Full-setup proxied record) created by `cloudflared tunnel route dns`, i.e. publicly resolvable independent of anything BodhiApp's own API exposes.
- Cloudflare, "Cloudflare Tunnel FAQ" — https://developers.cloudflare.com/cloudflare-one/faq/cloudflare-tunnels-faq/ — general tunnel/DNS background (supporting context, not directly quoted above).

---

## Follow-up: headers at the origin — resolved (2026-09-15)

Resolves the UNVERIFIED flag in §4.1: *"whether `cloudflared`'s HTTP-to-origin proxying sets `X-Forwarded-Proto` by default."* Method: read `cloudflared` source at tag `2026.9.1` directly (`proxy/proxy.go`, `ingress/origin_proxy.go`, `ingress/origin_service.go`, `connection/http2.go`, `connection/quic_connection.go`, `connection/header.go` — all fetched successfully from `raw.githubusercontent.com/cloudflare/cloudflared/2026.9.1/...`), cross-checked against `13-cloudflare-edge-behavior-for-llm-api-traffic.md` §5 (not duplicated below — see that doc for the full Cloudflare-edge header table) and Cloudflare's own docs. This section adds the one thing §5 didn't verify: **which layer (edge vs. connector) actually sets each header**, established from `cloudflared`'s own code path.

### What the source shows

`cloudflared` reconstructs the outgoing HTTP request for both the QUIC and HTTP/2 connector protocols from the metadata the Cloudflare edge sends over the tunnel connection — `buildHTTPRequest()` (`connection/quic_connection.go:361-386`) sets `req.Host = metadata[HTTPHostKey]` and copies every `HTTPHeaderKey`-prefixed metadata entry straight into `req.Header` with no filtering; the HTTP/2 path (`connection/http2.go`) does the equivalent from real HTTP/2 request headers. `grep -in "x-forwarded\|cf-visitor\|cf-connecting-ip\|cf-ray" proxy/proxy.go ingress/origin_proxy.go ingress/origin_service.go connection/http2.go connection/quic_connection.go connection/header.go` at tag `2026.9.1` returns **zero matches** — none of these headers are set, read, or stripped anywhere in `cloudflared`'s own proxy/ingress/connection code. The only header mutations `cloudflared` itself performs on the outgoing origin request, all in `ingress/origin_proxy.go` (`httpService.RoundTrip`, `:35-58`) and `proxy/proxy.go` (`proxyHTTPRequest`, `:184-217`), are: `Connection: keep-alive`/`Upgrade` framing, an empty `User-Agent` if none was supplied, `Cf-Warp-Tag-<name>` (opt-in, only if the connector is run with `--tag`, `proxy/proxy.go:374-378`), and — conditionally — `X-Forwarded-Host` (see table). Everything else present on the request the edge handed to `cloudflared` (including any `X-Forwarded-Proto`, `CF-Visitor`, `CF-Connecting-IP`, `CF-Ray`, `X-Forwarded-For`) rides through unmodified to the origin. **Conclusion: any such header at the loopback origin was added by the Cloudflare edge, not by `cloudflared`** — `cloudflared` is a transparent relay for these specific headers.

### Table

| Header | Present at origin? | Set by | Evidence |
|---|---|---|---|
| `X-Forwarded-Proto` | Yes (`https` for tunnel traffic) | Cloudflare edge | Not referenced anywhere in `cloudflared` source (grep above, 0 matches) — passed through via `buildHTTPRequest`/`RoundTrip` unmodified. Cloudflare Fundamentals — HTTP request headers doc: "used to identify the protocol (HTTP or HTTPS) that a visitor used to connect to Cloudflare" — an edge-level header added to all proxied traffic (not gated on backend/connector type), and the tunnel's public leg is always TLS-terminated at the edge, so it is `https` for every tunnel request. |
| `CF-Visitor` | Yes, `{"scheme":"https"}` | Cloudflare edge | Same grep result (0 matches in `cloudflared`). Cloudflare Fundamentals doc: "a JSON object, containing only one key called `scheme`", value `http`/`https`. Redundant with `X-Forwarded-Proto` — doc 13 §5 already noted this; reconfirmed, no new BodhiApp read needed. |
| `Host` | Yes, = the public tunnel hostname (by default) | `cloudflared` (passthrough of edge-supplied value) | `connection/quic_connection.go:365,372` (`buildHTTPRequest`): `req.Host = metadata[HTTPHostKey]` — the edge sends the original public hostname as this metadata field. `ingress/origin_proxy.go:47-52` (`httpService.RoundTrip`): `req.Host` is only overwritten when `o.hostHeader != ""`, i.e. only if `originRequest.httpHostHeader` is explicitly configured (default `""`, per doc 13 §1's recommendation to leave it default). With the recommended default, this branch never runs, so the origin sees the public hostname unchanged — confirms doc 13 §6 / §5's `Host`-header claim, now traced to the exact conditional in source rather than inferred from docs. |
| `X-Forwarded-Host` | **No, by default** (new finding — narrower than doc 13/21 assumed) | `cloudflared`, conditionally | `ingress/origin_proxy.go:47-52` — `req.Header.Set("X-Forwarded-Host", req.Host)` runs **only inside the `if o.hostHeader != ""` branch**, i.e. only when an operator explicitly sets `originRequest.httpHostHeader` to override the `Host` header sent to the origin. BodhiApp's recommended tunnel config leaves `httpHostHeader` at its default (empty), per `13-cloudflare-edge-behavior-for-llm-api-traffic.md` §1's table — so this branch does not fire and `X-Forwarded-Host` is absent at the origin under BodhiApp's own recommended settings. Not mentioned in the Cloudflare Fundamentals HTTP-request-headers doc either (it documents edge-added headers; this one is connector-specific and off by default). |
| `X-Forwarded-For` | Yes | Cloudflare edge | 0 matches in `cloudflared` source. Cloudflare Fundamentals doc: "maintains proxy server and original visitor IP addresses." |
| `CF-Connecting-IP` | Yes | Cloudflare edge | 0 matches in `cloudflared` source. Cloudflare Fundamentals doc: "Provides the client IP address connecting to Cloudflare to the origin web server." Corroborates doc 13 §5's existing recommendation to prefer this over `X-Forwarded-For` for any future audit logging. |
| `CF-Ray` | Yes | Cloudflare edge | 0 matches for a *setter* in `cloudflared`'s proxy/ingress/connection layers — `connection/http2.go:147` only *reads* an existing `CFRay` header off the incoming request (`FindCfRayHeader(r)`) for internal tracing, it does not originate the value. Cloudflare Fundamentals doc confirms it as an edge-added per-request trace ID. |
| A tunnel-identifying header (e.g. distinguishing tunnel traffic from any other Cloudflare-proxied origin) | **None exists** | N/A | Grepped the same six source files for any header-setter that is unconditional and tunnel-specific — found none. `CF-Ray`/`CF-Connecting-IP`/`X-Forwarded-*`/`CF-Visitor` are added by the edge for *all* Cloudflare-proxied (orange-clouded) traffic uniformly, regardless of what backend/connector serves the zone (Tunnel, Argo Smart Routing, a plain reverse-proxied origin) — none of them is unique to "arrived via a named tunnel." The one per-request tag mechanism `cloudflared` does add, `Cf-Warp-Tag-<name>` (`proxy/proxy.go:29-31,374-378`, `TagHeaderNamePrefix`/`appendTagHeaders`), is opt-in via the connector's `--tag <name>=<value>` CLI/config flag, not present by default, and is a general connector-tagging feature (documented for WARP client traffic tagging), not a "this is tunnel traffic" flag BodhiApp could rely on out of the box. |

### Design implication

1. **`extract_scheme` can rely on `x-forwarded-proto` for tunnel requests, unchanged.** The UNVERIFIED flag in §4.1 is resolved: `X-Forwarded-Proto: https` reaches BodhiApp's loopback origin for every request that came in over the tunnel's public HTTPS leg, added by the Cloudflare edge and passed through by `cloudflared` untouched. No middleware code change is required — this confirms (does not revise) doc 13 §5's "no middleware code change is required" conclusion, now with a source-level rather than docs-only citation. `CF-Visitor` is redundant with it, as already noted in doc 13 §5; no need to add a second scheme source.
2. **The request-origin resolver should keep matching `Host` against the configured tunnel hostname (as already designed in §3's `OriginKind::Tunnel`), not try to detect "this is tunnel traffic" from a header.** No header — including `CF-Ray`, `CF-Connecting-IP`, or any `X-Forwarded-*` — is unique to tunnel-originated requests; they are present on any Cloudflare-edge-proxied request to the zone, tunnel or not. `Host`-string matching against the tunnel's own hostname (already the plan per §3/§4.1) remains the only deterministic signal available, and it is safe to use: `Host` reaches the origin unmodified from the edge's original value under BodhiApp's recommended (default) `httpHostHeader` config, per the table above.
3. **`X-Forwarded-Host` needs no handling.** It is absent under BodhiApp's recommended tunnel config (default `httpHostHeader`), so nothing in the request-origin resolver or `canonical_url_middleware` should be written to depend on it being present.

### Sources

- `cloudflared` source, tag `2026.9.1` (read directly via `raw.githubusercontent.com/cloudflare/cloudflared/2026.9.1/...`): `proxy/proxy.go`, `ingress/origin_proxy.go`, `ingress/origin_service.go`, `connection/http2.go`, `connection/quic_connection.go`, `connection/header.go`.
- Cloudflare Fundamentals — HTTP request headers: https://developers.cloudflare.com/fundamentals/reference/http-request-headers/ (fetched 2026-09-15; source for the edge-added-header quotes in the table above: `X-Forwarded-Proto`, `X-Forwarded-For`, `CF-Visitor`, `CF-Connecting-IP`, `CF-Ray`).
- Cloudflare Tunnel — origin parameters (`httpHostHeader`): https://developers.cloudflare.com/cloudflare-one/networks/connectors/cloudflare-tunnel/configure-tunnels/origin-parameters/ (fetched 2026-09-15; confirms default `""` and description "Sets the HTTP `Host` header on requests sent to the local service" — does not itself mention `X-Forwarded-Host`, consistent with that header being a `cloudflared`-source-level finding, not a documented one).
- `13-cloudflare-edge-behavior-for-llm-api-traffic.md` §5 (this doc's companion — read first per this section's brief; not duplicated here).
