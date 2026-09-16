# Remote access via Cloudflare named tunnel — implementation plan

**Date:** 2026-09-16 · **Trunk:** `main` @ `d4268ddb` (every `file:line` below re-read at that commit) · **Research:** [`docs/research/tunnel/README.md`](../../../research/tunnel/README.md) → docs `10`–`24` (the `0x`/feasibility docs are SUPERSEDED). This plan cites the research; it does not restate it.

> **Not the current starting point (2026-09-16).** The owner resequenced delivery after this plan was written: the riskiest part goes first, and capability is built outwards from there. Implementation begins at [`slice-1-prompt.md`](slice-1-prompt.md), **not** at Phase 0 or Phase 1 below. This document remains valuable as the architecture study, the risk register (§5) and the research index — its §0 in particular is load-bearing and still correct. Treat its phase sequence (§3) as superseded.

**Synthesis note.** Built from three drafts plus two judge passes: delivery arc from draft A (real tunnel by Phase 4, remote login by Phase 5), risk/state model and consent work from draft B, test spine from draft C. Both previously-open verifier items are now **resolved** — see §0.

---

## 0. Two facts that changed after the drafts were written

Read these first; several earlier contingencies are deleted, not softened.

**A. `route dns` works with `cert.pem` alone — RESOLVED, yes.**
[`11-cloudflare-oauth-and-api-token-options.md`](../../../research/tunnel/11-cloudflare-oauth-and-api-token-options.md) → *"Follow-up: cert.pem and DNS routing — resolved"*. Traced in `cloudflared` 2026.9.1 source: `credentials.User::Client()` builds one `cfapi.RESTClient` from the cert's four fields, and every call — `POST /accounts/{id}/cfd_tunnel`, `GET`/`DELETE` the same, and **`PUT /zones/{zone}/tunnels/{id}/routes`** — uses that single bearer. `login → create → route dns → run` needs no second credential on a single-owner account. The tunnel-permissions page's "additional DNS/Load Balancer permissions" language is about the **Cloudflare account role** of a restricted team member, not a second credential.
→ In this plan: `route dns` is the happy path, not a coin flip. Manual-CNAME guidance is demoted to a **defensive error path** for restricted team-member roles (Phase 3, `dns_routed=false`). Tier 3 is **not** "the repair path for tier 1"; that argument is deleted from the tier-order rationale (§1).

**B. Headers at the loopback origin — RESOLVED.**
[`21-codebase-settings-network-and-info.md`](../../../research/tunnel/21-codebase-settings-network-and-info.md) → *"Follow-up: headers at the origin — resolved"*. `X-Forwarded-Proto: https` **does** reach the origin (added by the Cloudflare edge; `cloudflared` never touches it — zero grep hits across `proxy/`, `ingress/`, `connection/`). `Host` arrives **unmodified as the public hostname**, because `ingress/origin_proxy.go:47-52` rewrites it only when `originRequest.httpHostHeader` is explicitly set, which BodhiApp does not set. `X-Forwarded-Host` is therefore **absent** by default. No header is unique to tunnel traffic (`CF-Ray`/`CF-Connecting-IP`/`X-Forwarded-*` ride on all edge-proxied traffic).
→ In this plan: **`Host` == the configured tunnel hostname stays the deterministic tunnel signal** (that follow-up's own design implication #2), now on a resolved footing rather than as a hedge against unknown headers. `extract_scheme` needs no change. **Every `--http-host-header` contingency from the drafts is deleted** — setting it would also start injecting `X-Forwarded-Host`.

---

## 1. Summary & scope

**In.** Cloudflare **named** tunnel on the user's own zone; one tunnel per app instance; any admin enables it for the whole instance; the whole app is exposed (UI + `/v1`, `/anthropic/v1`, `/v1beta`). Feature gate `BODHI_TUNNEL_ENABLED`, default `SettingService::is_native()` (`setting_service.rs:189`), env-overridable, off in containers. Three Cloudflare auth tiers, all phased. `cloudflared`: detect → per-OS install guide → download-on-explicit-trigger. Keycloak redirect-URI sync via a new SPI endpoint, on enable/disable/startup only, 404-tolerant. `origins` on the unauthenticated `/bodhi/v1/info`. Two lifecycle levels (Turn off / Remove). Polling status. Admin-only "Remote Access" page under Settings. Tests at every layer; one growing Playwright spec, stub-driven on every run; real-Cloudflare E2E opt-in `@scheduled`.

**Out.** Quick tunnels (`trycloudflare`), Tailscale, frp; Cloudflare Access / service tokens in front of the app; per-path exposure; SSE status; `cloudflared service install` (Windows service / systemd / launchd); survive-reboot; multi-tenant tunnels; RP-initiated logout URIs; HA multi-connector.

**Locked decisions honored:** named tunnels only, own zone · gate default `is_native()` · instance-wide, any admin · all three tiers planned · detect → guide → download · SPI `PUT`+`GET /realms/{realm}/bodhi/resources/redirect-uris`, full replace, service-account bearer, 404 degrades to a warning · sync only on enable/disable/startup · `TunnelService` + `TunnelProvider` seam with `CloudflareTunnelProvider` as the only impl, no dead code for other providers · multi-origin model, **never** `BODHI_PUBLIC_HOST = tunnel host` · CLI control plane for tier 1 (`--output json`), data plane always `cloudflared tunnel run` per [doc 10 §A](../../../research/tunnel/10-cloudflared-cli-named-tunnel-lifecycle.md), readiness via `/ready` · in-house typed reqwest client for tiers 2/3, remotely-managed tunnel (`config_src=cloudflare`) + `TUNNEL_TOKEN` env · **`cert.pem` stays at `~/.cloudflared/cert.pem`** (detect existing, both Windows candidates); credentials JSON under `$BODHI_HOME` via `--credentials-file` · new leaf crate `crates/cloudflared_proc/`, `std::process` + reader threads + `Drop` kill/wait, composed into a `CompositeShutdownCallback` · `tunnels` singleton table per [doc 23 §(a)](../../../research/tunnel/23-codebase-persistence-routes-and-backend-tests.md) · Turn off vs Remove · polling `GET /bodhi/v1/tunnel` · Screen V2 page · one growing Playwright spec · SPI lands first · trunk-based, one commit per phase.

**Rejected alternatives on cert handling** (both appeared in drafts): relocating/moving `~/.cloudflared/cert.pem` into `$BODHI_HOME` — breaks the user's own `cloudflared` CLI; running `tunnel login` with a `HOME` override so the cert lands under `$BODHI_HOME` — the same deviation in softer form, and it rests on an unverified go-homedir claim. BodhiApp reads the cert where `cloudflared` put it and passes `TUNNEL_ORIGIN_CERT` on non-`run` commands.

### Recommended auth-tier order: **1 → 3 → 2** (owner may reject)

| Order | Tier | Why here |
|---|---|---|
| 1 | CLI (`cloudflared tunnel login/create/route dns/run`) | Owner's #1. No new credential stored by BodhiApp, no REST client needed, and per §0-A it covers create + route + run end to end on a single-owner account. MVP. |
| **2 (was 3)** | Scoped API token via two prefilled deep links | ~6-8 typed reqwest calls ([doc 12 §1](../../../research/tunnel/12-remotely-managed-tunnel-api-and-rust-crate.md)), and **that client is exactly what tier 2 needs** — so tier 2 becomes a "where does the bearer come from" delta rather than a second integration. No one-way door, works headless, permanent fallback. |
| **3 (was 2)** | Self-managed Cloudflare OAuth (PKCE **public** client) | The scope catalog is confirmed (`argotunnel.write`, `dns.write`, `zone.read` — doc 11 follow-up), but the **end-to-end grant is still unverified**, and shipping it requires BodhiApp to register a **public** OAuth client: logo + client/policy/tos URLs + DNS TXT domain verification, and **public visibility is permanent** ([doc 11 §1.2](../../../research/tunnel/11-cloudflare-oauth-and-api-token-options.md)). That is an owner decision and a one-way door, not an engineering step. |

The only two arguments for this order are the **one-way door** and **REST-client reuse**. (The earlier "tier 3 repairs tier 1's DNS gap" argument is void — §0-A.) **If rejected:** run Phase 8 ahead of Phases 7a/7b; the typed REST client lands in whichever comes first and nothing else moves.

---

## 2. Target architecture

### 2.1 Crates / modules touched

| Layer | New | Modified |
|---|---|---|
| `keycloak-bodhi-ext` (sibling repo, Phase 0) | `RedirectUrisRequest.java`, `RedirectUrisResponse.java`, `RedirectUrisEndpointTest.java` | `ResourceService.java` (beside `hasResourceAdmin` `:228`, reusing `checkForServiceAccount` `:303`); `BodhiResourceProvider.java` (beside `resources/has-resource-admin` `:134-145`); `httpyac-scripts/resource-management.http` |
| `crates/cloudflared_proc/` — **new leaf**, sibling of `llama_server_proc`, added to root `Cargo.toml` members | `src/{lib,error,detect,version,cert,cmd,process,pidfile,paths,download}.rs`, `CLAUDE.md`, `PACKAGE.md` | — |
| `crates/services` | `src/tunnels/{mod,tunnel_objs,error,tunnel_entity,tunnel_repository,tunnel_service,tunnel_provider,redirect_uris}.rs`; `src/tunnels/cloudflare/{mod,provider,cli_control_plane,api_client,token_links,oauth,download}.rs`; `src/db/sea_migrations/m20250101_000029_tunnels.rs` (P3), `…m20250101_000030_tunnels_oauth_tokens.rs` (P8) | `settings/constants.rs:77,82` (SETTING_VARS + 3 keys); `settings/setting_service.rs` (`tunnel_enabled()` beside `on_runpod_enabled` `:516`); `auth/auth_service.rs` (two SPI methods beside `register_client` `:66`/`:311`); `app_service/app_service.rs`; `app_service/auth_scoped.rs:90-110`; `db/sea_migrations/mod.rs`; `test_utils/app.rs:96-97,264-270,586-590` |
| `crates/routes_app` | `src/shared/request_origin.rs`; `src/middleware/cookies/secure_cookie_middleware.rs`; `src/tunnel/{mod,routes_tunnel}.rs` | `auth/routes_auth.rs:88-103`; `setup/routes_setup.rs:79-88,132-168`; `setup/setup_api_schemas.rs`; `middleware/redirects/canonical_url_middleware.rs:15-17,80-95`; `routes.rs:118,478-495,642,644`; `shared/{openapi,constants}.rs` |
| `crates/server_app` | `tests/test_live_tunnel.rs`; `tests/utils/tunnel_harness.rs` | `src/serve.rs:36-49,144-147,155-159` (`server.rs:58-62,74-90` keeps its signature) |
| `crates/lib_bodhiserver` | `tests-js/fixtures/bin/fake-cloudflared.mjs`; `tests-js/fixtures/tunnelFixtures.mjs`; `tests-js/pages/RemoteAccessPage.mjs`; `tests-js/specs/settings/remote-access-tunnel.spec.mjs` | `src/app_service_builder.rs:209-235`; `tests-js/test-helpers.mjs:54,60,82-88`; `tests-js/utils/auth-server-client.mjs` (+`getClientRedirectUris`) |
| `crates/bodhi/src` | `routes/settings/remote-access/{index.tsx,-components/*}`; `hooks/tunnel/*`; `test-utils/msw-v2/handlers/tunnel.ts` | `components/shell/shell-nav-config.tsx`; `lib/constants.ts`; `components/shell/ShellNav.test.tsx` |
| docs | — | `docs/architecture/security.md:9,183`; each touched crate's `CLAUDE.md`/`PACKAGE.md`; root `CLAUDE.md` + `crates/CLAUDE.md` chain |

The dependency chain gains `cloudflared_proc → services`, mirroring `llama_server_proc → services` exactly.

### 2.2 Trait sketch (signatures only)

```rust
// crates/cloudflared_proc — framework-free, std::process only (llama_server_proc precedent; tokio::process leaked orphans)
pub struct CloudflaredBinary { path: PathBuf }
impl CloudflaredBinary {
  pub fn detect(explicit: Option<&Path>, home: Option<&Path>) -> Detection;   // explicit > PATH > per-OS well-known (doc 14 §2)
  pub fn version(&self) -> Result<CalVer>;                                    // "cloudflared version YYYY.M.P (built …)"
  pub fn run_capture(&self, args: &[&str], env: &CmdEnv, timeout: Duration) -> Result<CmdOutput>;
  pub fn spawn_login(&self, env: &CmdEnv) -> Result<LoginChild>;              // stderr reader thread -> oneshot URL; Drop kills
  pub fn spawn_run(&self, args: &RunArgs) -> Result<CloudflaredRun>;          // pinned argv §2.7; secrets env-only
}
pub struct CloudflaredRun { child: Mutex<Option<Child>>, metrics_port: u16, log_tail: Arc<Mutex<VecDeque<String>>>, exit_rx: watch::Receiver<Option<ExitStatus>> }
impl CloudflaredRun {
  pub async fn wait_ready(&self, timeout: Duration) -> Result<Ready>;         // GET /ready on the metrics addr
  pub fn is_alive(&self) -> bool;
  pub fn recent_log_tail(&self) -> Vec<String>;
  pub fn stop(self, grace: Duration) -> Result<()>;                           // SIGTERM -> wait(grace) -> kill
}
impl Drop for CloudflaredRun { /* kill()+wait(), as llama_server_proc/src/server.rs:198-210 */ }
pub struct PidFile;
impl PidFile { pub fn write(dir: &Path, pid: u32, exe: &Path) -> Result<()>; pub fn reap_stale(dir: &Path) -> Result<ReapReport>; }

// crates/services/src/tunnels/tunnel_provider.rs — the reserved seam; CloudflareTunnelProvider is the only impl
#[async_trait] #[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait TunnelProvider: Send + Sync + std::fmt::Debug {
  fn kind(&self) -> TunnelProviderKind;
  async fn detect(&self) -> Result<CloudflaredDetection>;
  async fn begin_login(&self) -> Result<LoginSession>;                         // tier 1 only
  async fn provision(&self, spec: &ProvisionSpec, creds: &ProviderCredentials) -> Result<ProvisionedTunnel>;
  async fn route_dns(&self, t: &ProvisionedTunnel, hostname: &str, creds: &ProviderCredentials) -> Result<DnsRouteOutcome>;
  async fn start_connector(&self, t: &ProvisionedTunnel, local: &Url, creds: &ProviderCredentials) -> Result<Box<dyn Connector>>;
  async fn deprovision(&self, t: &ProvisionedTunnel, creds: &ProviderCredentials) -> Result<DeprovisionReport>;
}
pub trait Connector: Send + Sync + std::fmt::Debug {
  fn pid(&self) -> u32;
  fn state(&self) -> ConnectorState;
  fn exit_rx(&self) -> watch::Receiver<Option<i32>>;
  fn recent_log_tail(&self) -> Vec<String>;
  fn stop(self: Box<Self>, grace: Duration) -> Result<()>;
}

// crates/services/src/tunnels/tunnel_service.rs
#[async_trait] #[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait TunnelService: Send + Sync + std::fmt::Debug {
  async fn detect(&self) -> Result<CloudflaredDetection>;                      // 60s-cached (§2.8)
  async fn status(&self) -> Result<TunnelStatusResponse>;
  async fn login(&self) -> Result<TunnelLoginResponse>;                        // { login_url }
  async fn provision(&self, req: ProvisionTunnelRequest) -> Result<TunnelStatusResponse>;
  async fn retry_dns(&self) -> Result<TunnelStatusResponse>;
  async fn mark_dns_manual(&self) -> Result<TunnelStatusResponse>;
  async fn enable(&self) -> Result<TunnelStatusResponse>;                      // spawn -> ready -> KC sync -> persist
  async fn disable(&self) -> Result<TunnelStatusResponse>;                     // "Turn off": stop + KC sync(remove), keep provisioning
  async fn remove(&self) -> Result<TunnelStatusResponse>;                      // delete tunnel + DNS + local creds + row
  async fn set_api_token(&self, req: SetApiTokenRequest) -> Result<TunnelStatusResponse>;  // P7
  fn active_hostname(&self) -> Option<String>;   // SYNC, std::sync::RwLock snapshot — per-request consumers (§2.4)
  fn active_public_url(&self) -> Option<String>; // SYNC; Some iff Enabled AND Connected — feeds /info
  async fn on_startup(&self);                    // reap -> resume-if-enabled -> idempotent sync
  async fn on_shutdown(&self);                   // stop connector + any pending login child
}
```

`DefaultTunnelService { db, settings, auth, tenants, network, time, provider: Arc<dyn TunnelProvider>, live: std::sync::RwLock<LiveState> }` with `LiveState { connector, connector_state, status_in_memory, hostname, public_url, login, sync, detect_cache }`. Long operations run under `tokio::spawn` and write the row on completion (pattern `routes_files_pull.rs:196`); **the std lock is never held across `.await`** — which is why `active_hostname()`/`active_public_url()` are sync: the request-origin resolver and the canonical middleware call them on every request.

`AuthScope::tunnels()` beside `auth_scoped.rs:90-110` injects the tenant. Every mutating method first checks `settings.tunnel_enabled()` and `deployment_mode() != MultiTenant`.

### 2.3 State machine — admin intent vs connector liveness

Two levels, deliberately separate. This is the one **flagged problem** with [doc 21 §6](../../../research/tunnel/21-codebase-settings-network-and-info.md): its single `TunnelStatus` conflates admin intent with live connectivity, so the UI cannot express "enabled but reconnecting" and `/info` has no honest gate.

```
Persisted  tunnels.status column (three terminal values):         Disabled | Enabled | Error
In-memory  LiveStatus (what the DTO's `status` serializes):       Disabled | Provisioning | Enabled | Error
In-memory  ConnectorState (additive, never persisted):            Stopped | Starting | Connected | Degraded | Exited

[no row] --login()--> (cert on disk) --provision()--> Disabled(provisioned: tunnel_id AND hostname AND dns_routed)
provision step fails    --> Error(last_error); tunnel_id already persisted, so retry never re-creates
Disabled(provisioned)   --enable()--> Enabled ; connector Stopped->Starting->Connected ; KC sync(add)
Enabled                 --disable()--> Disabled(provisioned) ; connector Stopped ; KC sync(remove)
Enabled                 --child exits--> Enabled (intent kept) ; connector Exited  (auto-restart only from P10)
Enabled, boot, gate on  --> reap PID -> Starting -> Connected -> KC sync(add, idempotent)
Enabled, boot, gate off --> connector Stopped, row untouched, page shows "disabled by policy"
Error                   --enable()/retry--> Provisioning ; never auto-retried at boot
any                     --remove()--> disable() -> deprovision -> row deleted
```

**Invariants (each pinned by a named test, §5.2):**
1. A `Tunnel` entry appears in `/bodhi/v1/info.origins` **iff** `status == Enabled AND connector == Connected` (doc 21 follow-up §1: a provisioning/error tunnel is *absent*, never present-with-a-flag).
2. `active_hostname()` is `Some` **iff** `status == Enabled`, regardless of connector state — so a flapping tunnel never 301s visitors away via the canonical middleware.
3. The tunnel callback exists in Keycloak **iff** `status == Enabled`.

**Persistence write policy (settled; the drafts disagreed).** The persisted column carries **three** values — `disabled | enabled | error` — and `Provisioning` lives only in `LiveState`, beside `ConnectorState`, as an in-memory `LiveStatus`. (Doc 23 §a sketches a four-value enum, but `m20250101_000029` is still ours to shape; a column value with no writer anywhere is reserved dead surface, which this plan forbids. Immutability binds the migration *after* Phase 3 commits it, not before.) A crash mid-provision therefore cannot boot into a retry storm. `enabled` doubles as "desired on" for startup resume; `error` is sticky across restarts and the user re-enables explicitly.

**Row = doc 23 §a columns**, plus two added in the same migration: `dns_routed BOOLEAN NOT NULL DEFAULT false` (so "the user created the CNAME by hand" is representable) and `dns_record_id TEXT NULL` (tier-3 CNAME cleanup, doc 12 §1.7-1.8). **`m20250101_000029` (Phase 3) also ships doc 23 §a's tier-3 token triplet** (`encrypted_api_token`, `api_token_salt`, `api_token_nonce`), which Phase 7 then writes — Phase 7 adds no migration. `m20250101_000030_tunnels_oauth_tokens.rs` (Phase 8) adds only the OAuth refresh-token triplet plus `oauth_expires_at`. Next free number confirmed: latest committed is `m20250101_000028_expire_stale_app_access_requests.rs`.

**DTO deltas vs doc 21 §6 — all additive, all flagged as deviations:**

| Field | On | Why |
|---|---|---|
| `available: bool` | `TunnelStatusResponse` | gate state, so the page renders an unavailable state instead of an error |
| `connector: ConnectorState` | `TunnelStatusResponse` | the §2.3 split |
| `keycloak_sync: Option<KeycloakSyncStatus>` (`Synced`/`SpiOutdated`/`Failed` + message) | `TunnelStatusResponse` | the 404 degrade must be visible, not silent |
| `has_origin_cert: bool` | `TunnelStatusResponse` | tier-1 wizard step 1 — distinct from doc 21 §6's existing `has_credentials_file` |
| `dns_routed: bool` | `TunnelStatusResponse` | drives the defensive manual-CNAME panel |
| `created_at` / `updated_at` -> `Option<…>` | `TunnelStatusResponse` | the never-provisioned state has no row |
| `TunnelLoginResponse { login_url: String }` | new, response of `POST /tunnel/login` | doc 21 §6 has nowhere to carry it |
| `overwrite_dns: Option<bool>` | `ProvisionTunnelRequest` | sent only after an explicit user confirm on a DNS conflict |

All registered in `openapi.rs` with the rest; `make build.ts-client` after every phase that changes them.

### 2.4 Request-origin resolver — `crates/routes_app/src/shared/request_origin.rs`

One resolver replaces the ad-hoc composition at `routes_auth.rs:88-103` (which always appends `:{public_port()}` — the `redirect_uri` bug of [doc 22 §1.1](../../../research/tunnel/22-codebase-login-flow-and-keycloak-spi-redirect-uris.md)) and the inline loop at `routes_setup.rs:132-168`.

```rust
pub struct RequestOrigin { pub scheme: String, pub host: String, pub port: Option<u16>, pub is_tunnel: bool }
impl RequestOrigin { pub fn origin(&self) -> String; pub fn login_callback_url(&self) -> String; }

/// Pure and table-tested. `tunnel_host` comes from the SYNC TunnelService::active_hostname().
pub fn resolve_request_origin(
  headers: &HeaderMap,
  public_host_explicit: Option<&str>, public_server_url: &str,
  public_scheme: &str, public_port: u16,
  tunnel_host: Option<&str>,
) -> RequestOrigin;
```

Rules, in order:
1. `public_host_explicit.is_some()` → parse `public_server_url` verbatim (today's branch 1; RunPod/Docker unchanged).
2. `Host` (validated by `is_valid_hostname`, `shared/utils.rs:20`) matches `tunnel_host` case-insensitively with the port stripped → `https`, port `None`, `is_tunnel = true`. **This is the deterministic tunnel signal** — no header is tunnel-unique (§0-B).
3. Else scheme = `x-forwarded-proto` → `x-forwarded-scheme` → `public_scheme()` (same trust model and order as `canonical_url_middleware.rs:80-95`); host = `extract_request_host` (`shared/utils.rs:3`); port = **parsed from the raw `Host` header by the resolver itself**, else `public_port()` — `extract_request_host` strips the port (`host_str.split(':').next()`) before validating, so it yields the validated hostname only and can never supply the port.
4. No usable `Host` → parse `public_server_url`. "Unusable" means absent, or a hostname `is_valid_hostname` (`shared/utils.rs:20`) rejects — it allows only alphanumerics, `.` and `-`, so any host containing `_` falls here.

`port` is `None` when it equals the scheme default (80/443), and `origin()` elides it.

Home crate: **`routes_app`**, because only routes consume it. It is a pure function over `&HeaderMap` plus already-read setting values, so it unit-tests as a table with no axum harness and no `SettingService` mock; a thin `async fn request_origin(app: &dyn AppService, headers: &HeaderMap)` adapter does the setting reads once per call.

Consumers: `auth_initiate` (`routes_auth.rs:88-103`), `setup_create`'s request-host entry (`routes_setup.rs:150-157`), `canonical_url_middleware` (early return when `is_tunnel`), and the secure-cookie middleware (§2.5). **Not `/info.origins`** — `setup_show(auth_scope: AuthScope)` (`routes_setup.rs:30`) takes no `HeaderMap`, and origins is a settings-derived enumeration of every origin the instance serves, not a per-request resolution. Its signature is unchanged.

Shared header-free primitive in services: `tunnels::redirect_uris::desired_set(existing, base, tunnel_callback: Option<&str>) -> Vec<String>` = loopback hosts × scheme/port + LAN IP + explicit public callback (+ the tunnel callback when enabled), **unioned with entries already registered that BodhiApp cannot recompute** — so a full-replace `PUT` never drops the setup-time request-host entry or a manually added one — stably sorted. **For every non-loopback host it emits both the `http` and `https` variant.** Rule 3 lets `x-forwarded-proto` decide the *composed* scheme while `desired_set` only knows `public_scheme()`, so without this the two sets drift on the scheme axis and Keycloak answers `Invalid parameter: redirect_uri` — the exact failure Phase 5 exists to retire. The header is attacker-settable from the LAN (there is no `Host` allowlist or forwarded-header guard in `routes_app`, doc 21 §4.5), so registering both variants is the containment: an unregistered scheme can never be reached, and a spoofed `https` on a LAN host resolves to a callback that no listener serves.

**Deliberate behavior changes** (no back-compat outside the DB): 80/443 elision everywhere; a port-mapped container without `BODHI_PUBLIC_HOST` now composes the port the browser actually used instead of `public_port()`; **`auth_initiate`'s scheme now comes from `x-forwarded-proto` rather than always `public_scheme()`** (`routes_auth.rs:95`) — matched by `desired_set`'s dual-variant emission above, and pinned by a negative test. `setup_create` (`routes_setup.rs:137`) and `auth_initiate` share the code, so host and port cannot drift — pinned by `network-ip-setup-flow.spec.mjs` staying green.

### 2.5 Session-cookie `Secure` — decision (not left open)

**Chosen: per-request `Secure` rewrite outside the session layer, floored by today's global value** (the "custom `Set-Cookie` rewrite" candidate in doc 21 §4.4).

- `session_layer(secure)` is untouched (`session_service.rs:186-190`; cookie name `bodhiapp_session_id` at `:190`) and keeps being built with today's `is_secure_transport()` value (`setting_service.rs:368`), which becomes the **floor**.
- New `crates/routes_app/src/middleware/cookies/secure_cookie_middleware.rs`, layered **outside** the session layer (after `.layer(…session_layer(secure_cookie))` at `routes.rs:642`): resolve the origin (§2.4) before `next.run`, then on the response append `; Secure` to any `set-cookie` starting with `bodhiapp_session_id=` when the resolved scheme is `https`. It never strips a `Secure` the floor set. `SameSite=Strict` unchanged.
- **Rationale:** cookies are host-scoped, so `https://<tunnel-host>` gets a `Secure` cookie while `http://localhost:1135` and `http://<lan-ip>:1135` keep working. A global flip would break desktop loopback login and the flow `network-ip-setup-flow.spec.mjs` protects; two cookie names would double session plumbing for no gain. ~30 lines.
- **`docs/architecture/security.md`:** reword the remediated row at `:183` to "session cookie `Secure` is derived per request origin: set on https origins (tunnel / explicit public host / `x-forwarded-proto: https`); `BODHI_PUBLIC_SCHEME=https` still forces it globally", and add an Accepted Risk under `:9` — "non-`Secure` session cookie on loopback/LAN http origins, by design on desktop".
- Lands in Phase 2, with the resolver, before any session is minted through a tunnel.

### 2.6 Startup / shutdown wiring (exact anchors)

- **Shutdown.** `serve.rs:144-147` boxes `ShutdownLocalLlamaCallback` (`serve.rs:36-49`) into `Server::start_new` (`server.rs:61`); the graceful closure at `server.rs:74-90` invokes it at `:87-88` **before** axum stops accepting. Change: `CompositeShutdownCallback(Vec<Box<dyn ShutdownCallback>>)` in `serve.rs`, ordered `[ShutdownTunnelCallback, ShutdownLocalLlamaCallback]` so the connector drains first ([doc 15 §7.6](../../../research/tunnel/15-prior-art-desktop-apps-managing-cloudflared.md)). `Server::start_new`'s signature is unchanged. This covers serve/container (`shutdown.rs`) and the Tauri tray quit (`native_init.rs` → `ServerShutdownHandle::shutdown` → `serve.rs:93-100`) with no Tauri-specific code. `Drop` on `CloudflaredRun` is the panic net, the PID reap (Phase 6) is the SIGKILL net, PDEATHSIG / Job Objects are Phase 10.
- **Startup.** `serve.rs:155-159`, in the `Ok(())` ready arm right after `tracing::info!(server_url, public_url, "server started")` (`:158`): `tokio::spawn(service.tunnel_service().on_startup())`. Detached, so a slow `cloudflared` never delays the ready path the native shell awaits.
- **DI.** `app_service_builder.rs:209-235`: build `CloudflareTunnelProvider` + `DefaultTunnelService` after `download_service` (`:209`) and append the positional arg to `DefaultAppService::new` (`:213`; `derive_new`, order-sensitive — the new arg goes after `download_service` at `:234`). Trait accessor `AppService::tunnel_service()`; stub field + default + accessor in `test_utils/app.rs:96-97,264-270,586-590`, defaulting to a `MockTunnelService`.
- **Middleware state.** `canonical_url_middleware`'s state widens from `State<Arc<dyn SettingService>>` (`canonical_url_middleware.rs:17`) to `Arc<dyn AppService>` at `routes.rs:644` (the `from_fn_with_state` state argument; `:645` is the handler identifier), to reach `tunnel_service().active_hostname()`.

### 2.7 Pinned `cloudflared` invocations

Exactly [doc 10 §A](../../../research/tunnel/10-cloudflared-cli-named-tunnel-lifecycle.md), which is source-traced at tag `2026.9.1` (`buildRunCommand`'s own `cliFlags`; `--credentials-file` confirmed a `run`-level flag; `--url` registered on `run` at `cmd.go:716-724`). **All `run` flags go after `run`, and `--origincert` is not passed to `run` at all** (`run`'s own description: it does not need `cert.pem` when the tunnel is identified by UUID).

```
run (tier 1):   <bin> tunnel run \
                  --credentials-file $BODHI_HOME/cloudflared/<uuid>.json \
                  --url http://127.0.0.1:<bodhi-port> \
                  --no-autoupdate --metrics 127.0.0.1:<free-port> \
                  --loglevel info --protocol auto --output json <uuid>
run (tier 2/3): same, minus --credentials-file and minus the positional; TUNNEL_TOKEN in env (never --token on argv)
login:   <bin> tunnel login
create:  <bin> tunnel create --output json --credentials-file $BODHI_HOME/cloudflared/<name>.json <name>
         then rename to <uuid>.json and persist credentials_path (see below)
list:    <bin> tunnel list --output json -n <name>          # idempotency probe before create
route:   <bin> tunnel route dns [--overwrite-dns] <uuid> <hostname>
delete:  <bin> tunnel delete -f <uuid>
env, every non-run command: TUNNEL_ORIGIN_CERT=<resolved ~/.cloudflared/cert.pem>
```

- **The `credentials_path` column is the single source of truth for the credentials JSON.** The UUID only exists *after* `create` returns, so `create` writes the deterministic `<name>.json` and the service then renames it to `<uuid>.json` and persists `credentials_path` together with `tunnel_id`, before `route dns`. `run` and `remove` read the column; neither re-derives a name. `writeTunnelCredentials()` errors `<path> already exists` rather than overwriting (doc 10 §2), so `provision` removes a stale `<name>.json` it owns before invoking `create`, and a `create` failure is handled per the orphan probe below.
- `--url` is always `http://127.0.0.1:<port>`, never `0.0.0.0` (precedent: the native bind normalization in `native_init.rs`).
- The metrics port comes from `portpicker` (already a workspace dep via `llama_server_proc`), one per spawn, so two instances on one machine do not collide.
- Readiness: `GET http://127.0.0.1:<metrics>/ready` every 500 ms with a 60 s budget; **ready = HTTP 200 with `readyConnections > 0`** (doc 10 §B: the server returns 503/`0` until a connection registers, so the exact count is deliberately free and the stub's value is not part of the contract). No `Starting metrics server on` line within 10 s → fail fast (doc 10 §B/§C). Log lines are diagnostics only, never the state signal.
- **`create` failure does not imply rollback.** `cloudflared` deletes the just-created remote tunnel itself (`DeleteTunnel(id, cascade=true)`), but if that delete also fails it only tells the caller to run `cloudflared tunnel delete <uuid>` by hand (doc 10 §2). So a failed `provision` always re-probes with `list -n <name>`, adopts an orphan it finds, and otherwise surfaces the manual `tunnel delete` escape hatch in `last_error`.
- **Residual risk (kept, small):** nothing in this repo has spawned real `cloudflared` yet. The fake accepts the pinned flags in either position, and the **first real spawn in Phase 4 validates the pin**; if the real binary disagrees, the argv and the fake change in that phase and the pin is corrected here. The pin is not reordered on inference.

### 2.8 Endpoints, gate semantics, detect cache

All under `admin_session_apis` (`routes.rs:478-495`), tag `API_TAG_TUNNEL`, constant `ENDPOINT_TUNNEL` via `make_ui_endpoint!` (doc 23 §c; doc 24's `ENDPOINT_TUNNEL_STATUS`/`state`/`TunnelStatus`-as-response naming is superseded by doc 21 follow-up §2):

`GET /bodhi/v1/tunnel` · `GET /bodhi/v1/tunnel/detect` · `POST …/login` · `POST …/provision` · `POST …/dns/retry` · `POST …/dns/mark-manual` · `POST …/enable` · `POST …/disable` · `POST …/sync` · `DELETE /bodhi/v1/tunnel` · `POST …/api-token` + `GET …/zones` + `DELETE …/api-token` (P7a) · `POST …/oauth/initiate|callback` (P8) · `POST …/cloudflared/download` (P9).

**Gate-off semantics (settled).** The two read endpoints always answer `200` with `available: false` (and a `found: false`-shaped detect body) so the page renders a clean unavailable state. Every **mutating** endpoint returns `403` with the single code `tunnel_error-disabled_by_policy`, registered once in openapi. Multi-tenant deployments get `tunnel_error-unsupported_deployment` on the same routes. `on_startup` never resumes when the gate is off — that is the kill switch.

**Detect cache.** `GET /tunnel/detect` is polled by the page, and spawning `cloudflared --version` per poll is wasteful. The detection is cached 60 s in `LiveState`; an explicit "Re-check" button bypasses it.

---

## 3. Phases

**Sequencing rationale.** Phase 0 is a parallel track from day 1 (release + Railway lead time) and gates only Phase 5's live assertions. Phase 2 retires the correctness/security risks that exist with or without a tunnel (callback composition, canonical 301, cookie `Secure`) one phase before any tunnel code can trip on them. A real tunnel carries traffic at Phase 4; **remote login is the MVP capstone at Phase 5.**

**Conventions for every phase.** Work upstream→downstream (`cloudflared_proc` → `services` → `server_core` → `routes_app` → `server_app` → `lib_bodhiserver` → `bodhi`). `make build.ts-client` after any DTO change. Gate before commit: `make format`; `cargo test -p <touched>` cumulatively; `make test.backend 2>&1 | tee /tmp/tb-<phase>.log` (run once — never re-run a slow command just to re-read it); `cd crates/bodhi && npm test`; `make test.e2e` whenever `tests-js` changed. Live check in Chrome via `make app.run.live` — **always with `BODHI_TUNNEL_ENABLED=true`**, because `bodhiserver_dev` runs as `AppType::Container` and the gate defaults off there (same for every Playwright server config); **rebuild the dev-server binary first when the backend changed**, or new routes and query params are silently ignored. Then `graphify update .` (root `CLAUDE.md:147`) so the knowledge graph lands in the same commit as the code. Commit straight to `main` (trunk-based, no branches, no PRs). Assert `body.error.code`, never message text. Never `test.skip` on a missing env var — throw. Update each touched crate's `CLAUDE.md`/`PACKAGE.md` in the same commit — **and the plan-index entries**: every file added under `docs/claude-plans/202609/tunnel/` (retrospectives, next-phase kickoffs) needs an entry in `docs/claude-plans/202609/index.md`, per `docs/claude-plans/CLAUDE.md`'s MUST-follow rules. That `index.md` and the `202609/` pointer in `docs/claude-plans/index.md` already exist (created alongside this plan, with entries for this plan and the Phase-1 kickoff), so each phase only appends entries for the files it adds.

**Live checks are tagged agent-executable or owner-only.** A phase whose live check needs a real Cloudflare zone, a physical phone or a Windows box splits it: the agent runs the stub-driven half itself, then stops and hands the owner a numbered checklist to run and paste back **before** the commit. An owner-only check is never claimed as done by the agent.

### Phase 0 — SPI: `GET`+`PUT /realms/{realm}/bodhi/resources/redirect-uris` (keycloak-bodhi-ext)

- **Goal.** The endpoint BodhiApp calls in Phase 5, released and live on both hosted Keycloaks. `GET` is added beside the briefed `PUT` so sync is read-modify-write and tests can read back without realm-admin credentials.
- **Repo.** `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/keycloak-bodhi-ext`.
- **Work.** `RedirectUrisRequest.java` / `RedirectUrisResponse.java` in `ClientRequest.java` style (doc 22 §2.3 shapes). `ResourceService.setRedirectUris` / `getRedirectUris` beside `hasResourceAdmin` (`ResourceService.java:228`), both reusing `checkForServiceAccount` (`:303`) and acting only on `getIssuedFor()`'s client; `client.setRedirectUris(new HashSet<>(…))` + explicit commit; return the sorted set. `@PUT` / `@GET @Path("resources/redirect-uris")` in `BodhiResourceProvider.java` beside `resources/has-resource-admin` (`:134-145`), wrapped in `tracked("bodhi.resources.set-redirect-uris")` / `"…get-redirect-uris"`; add the `jakarta.ws.rs.PUT` import. `httpyac-scripts/resource-management.http`: `get_redirect_uris` / `set_redirect_uris` blocks with `{{resource_service_token}}`. `make openapi`.
- **Tests** (`RedirectUrisEndpointTest extends BaseTest`; helpers `getServiceAccountToken()` `BaseTest.java:151`, `registerClientAndReturnClientPair()` `:209`, `createUser()` `:289`, `getDashboardUserToken()` `:335`):
  `testSetRedirectUrisSuccess` · `testGetRedirectUrisReturnsSortedCurrentSet` · `testSetRedirectUrisWorksOnTenantClient` · `testSetRedirectUrisUnauthorizedUserToken` (401) · `testSetRedirectUrisUnauthorizedNoToken` (401) · `testSetRedirectUrisEmptyListRejected` (400) · `testSetRedirectUrisCannotMutateAnotherClient` (client B untouched after client A's PUT) · `testSetRedirectUrisKeepsWebOriginsPlus` (the `+` web-origin entry survives, doc 22 §4.1).
- **Gate (SPI repo).** `make test` (`Makefile:20` — clean, compile, all tests including integration) before the commit.
- **Release / verify** (doc 22 follow-up §D) — **both** checks on **both** hosts:
  1. **Owner checkpoint, not agent work.** Merge to `main` → `make release-server` (`Makefile:197`) → `release.yml` pushes `:vX.Y.Z` + `:latest`. Cutting a public release tag is a publishing action: the agent implements, runs `make test`, commits, then **stops and hands back** with the tag it proposes.
  2. **Owner checkpoint.** Confirm Railway Image Auto Updates (or click Deploy) for the `main-id` **and** the separately-managed `test-id` services. A dashboard action — the agent cannot do it and must not claim it.
  3. Agent-executable, after the owner confirms 1-2. Cheap route probe: **unauthenticated** `curl -X PUT https://{test-id,main-id}.getbodhi.app/realms/<realm>/bodhi/resources/redirect-uris` must return **401 (route exists)**, not 404.
  4. Agent-executable. Authenticated round-trip via httpyac with a **throwaway** resource client's service-account token: `PUT` then `GET` → 200 with matching sets.
- **Docs.** SPI `CLAUDE.md` / README endpoint table.
- **Commit (SPI repo).** `feat(resources): GET/PUT redirect-uris self-service endpoint for resource clients`.
- **Exit.** 401 unauthenticated and 200 authenticated on both hosts; the verified tag recorded in this phase's retrospective. BodhiApp untouched.

### Phase 1 — Feature gate, `cloudflared` detect, Remote Access page shell, test spine

- **Goal.** The whole seam exists end to end carrying the thinnest payload: "is `cloudflared` here, which version, is there an origin cert?" The fake binary and both harnesses land here, alongside their first consumer.
- **User-visible.** An admin opens Settings → Remote Access and sees a detect card: found (path + version + supported badge), missing (per-OS install guide), or gate-off (unavailable state).
- **services.** `settings/constants.rs`: `BODHI_TUNNEL_ENABLED`, `BODHI_TUNNEL_CLOUDFLARED_PATH`, `BODHI_TUNNEL_ORIGIN_CERT` appended to `SETTING_VARS` (`:82`), **not** to `EDIT_SETTINGS_ALLOWED` (`:77` — toggling has side effects, doc 21 §1). `setting_service.rs`: trait-default `tunnel_enabled()` (Default source → `is_native()` `:189`, the same read-time pattern as `public_scheme()` `:355`), plus `tunnel_cloudflared_path()` and `tunnel_origin_cert()`. New `src/tunnels/`: `tunnel_objs.rs` (`CloudflaredDetection { found, path, version, supported, source, has_origin_cert, os }`, `ConnectorState`, `TunnelStatus`, `TunnelAuthMode`), `error.rs` (`TunnelError` via `errmeta_derive`; `DisabledByPolicy` → `ErrorType::Forbidden`), `tunnel_provider.rs`, `cloudflare/provider.rs` (detect only), `tunnel_service.rs` (`DefaultTunnelService::detect` + gate checks + the 60 s cache). `AppService::tunnel_service()`, `AuthScope::tunnels()`, stub field (`test_utils/app.rs:96-97,264-270,586-590`), DI (`app_service_builder.rs:209-235`).
- **cloudflared_proc (new crate).** `Cargo.toml` mirroring `llama_server_proc` (workspace `reqwest`, `tokio` sync, `tracing`, `serde_json`, `portpicker`), registered in root `Cargo.toml` members. `detect.rs` (explicit setting → PATH scan → per-OS well-known list → `$BODHI_HOME/bin` (Phase 9), [doc 14 §2](../../../research/tunnel/14-cloudflared-binary-detection-install-download.md), both Windows `Program Files*` candidates; **deliberate choice:** a system install wins over the managed binary because Phase 9's download is a late, consent-gated fallback and users commonly already have Homebrew/apt `cloudflared` — doc 14 §4's "prefer OS package manager when present" (already signed/notarized, kept current by the OS) over doc 14 §2's own framing of the probe as merely "detect a system install to *avoid* redundant download", which would put `$BODHI_HOME/bin` first as `BODHI_EXEC_LOOKUP_PATH` does for llama-server; `home` injected from `EnvWrapper::home_dir()` `env_wrapper.rs:40` so tests use a temp home without touching process env). `version.rs` (`^cloudflared version (\S+) \(built`, CalVer, 365-day support window, doc 10 §6). `cert.rs` (`~/.cloudflared/cert.pem`; Windows `%USERPROFILE%\.cloudflared\cert.pem` **and** `%LOCALAPPDATA%\cloudflared\cert.pem` — the doc 10 §1 discrepancy). `error.rs`, `lib.rs`.
- **routes_app.** `API_TAG_TUNNEL`; `make_ui_endpoint!(ENDPOINT_TUNNEL, "tunnel")`; `tunnel/routes_tunnel.rs::tunnel_detect` registered in `admin_session_apis` (`routes.rs:478-495`); openapi schema/path/tag registration per the `crates/routes_app/CLAUDE.md` checklist.
- **UI.** `ROUTE_SETTINGS_TUNNEL = '/settings/remote-access/'`; nav sub-page `{ id:'remote-access', label:'Remote Access', icon:'radio-tower', adminOnly:true, hideInMultiTenant:true }` in `shell-nav-config.tsx`; `routes/settings/remote-access/index.tsx` (`AppInitializer authenticated allowedStatus="ready"`, `useShellChrome`, `DetailRail`); `-components/{DetectCard,InstallGuide,UnavailableState}.tsx` (guide copy per OS from doc 14 §1); `hooks/tunnel/{constants,useTunnelStatus,useTunnelActions,index}.ts` (`useGetTunnelStatus({ enablePolling })` copying `hooks/models/useDownloads.ts`); MSW `handlers/tunnel.ts`.
- **Test spine** (§4.2 — **Phase-1 slice only**, so nothing lands unused or coupled to Phase 0): `tests-js/fixtures/bin/fake-cloudflared.mjs`, `fixtures/tunnelFixtures.mjs`, `pages/RemoteAccessPage.mjs`, `specs/settings/remote-access-tunnel.spec.mjs`; `server_app/tests/utils/tunnel_harness.rs` with `fake_cloudflared()` and a minimal `live_tunnel_server(fake)` only. `poll_tunnel_status` / `raw_get_with_host` arrive in Phase 4 and the throwaway-resource-client fixture + `spi_redirect_uris` + `getClientRedirectUris` in Phase 5, each with its first consumer.
- **API/DTO.** `GET /bodhi/v1/tunnel/detect → CloudflaredDetection`; ts-client regen.
- **Tests.**
  - cloudflared_proc: `test_detect_prefers_explicit_path`, `test_detect_scans_path_then_well_known`, `test_detect_well_known_per_os` (cfg-gated), `test_parse_version_calver`, `test_is_supported_one_year_window`, `test_find_origin_cert_both_windows_candidates`.
  - **Stub self-tests** (`server_app`, no server, no Keycloak): `test_fake_cloudflared_version_line_parses`, `test_fake_cloudflared_login_writes_cert_and_prints_url_on_stderr`, `test_fake_cloudflared_create_writes_credentials_file_0400_and_json`, `test_fake_cloudflared_run_serves_ready_and_exits_on_sigterm`, `test_fake_cloudflared_scenario_ready_never_stays_503`, `test_fake_cloudflared_invocation_log_records_argv_and_env_presence`, `test_fake_cloudflared_unknown_subcommand_exits_64`.
  - services: `test_tunnel_service_detect_delegates_to_provider` (`MockTunnelProvider`), `test_tunnel_service_detect_cached_within_60s`, `test_tunnel_service_rejects_when_gate_off` (`tunnel_error-disabled_by_policy`), `test_tunnel_service_rejects_multi_tenant`, `test_setting_tunnel_enabled_defaults_to_is_native` (native true / container false / env override wins).
  - routes_app: `test_tunnel_endpoints_reject_unauthenticated` and `test_tunnel_endpoints_reject_non_admin` (`#[case]` per endpoint — the table grows every phase), `test_tunnel_detect_returns_detection`, `test_tunnel_detect_available_false_when_gate_off`.
  - server_app: `test_live_tunnel_detect_finds_fake_binary`, `test_live_tunnel_detect_missing_when_path_points_nowhere`.
  - UI vitest+MSW: `remote-access/index.test.tsx` — `renders detect card when found`, `renders install guide when missing`, `renders unavailable state when gate off`; `hooks/tunnel/useTunnelStatus.test.ts`; `ShellNav.test.tsx` — `hides remote-access for non-admin`, `hides remote-access in multi_tenant` (first `adminOnly` coverage in that file, gap noted in doc 24 §4).
  - Playwright: `test('Remote Access tunnel lifecycle (fake cloudflared)')` created with step `detect: shows version from stub`; separate `test('shows install guide when cloudflared is missing')` (a different server config means a separate `test`, never an `if`).
- **Live check.** `BODHI_TUNNEL_ENABLED=true make app.run.live` → the real Homebrew `cloudflared` + version; `BODHI_TUNNEL_CLOUDFLARED_PATH=/nonexistent` → install guide; no `BODHI_TUNNEL_ENABLED` → unavailable state. Launch the Tauri build from Finder once to confirm PATH-gap detection (doc 10 §7).
- **Docs.** New `crates/cloudflared_proc/{CLAUDE,PACKAGE}.md`; `crates/CLAUDE.md` index + chain; root `CLAUDE.md` chain; `crates/services/CLAUDE.md`; `crates/routes_app/CLAUDE.md`; `crates/bodhi/src/CLAUDE.md`; `crates/lib_bodhiserver/tests-js/CLAUDE.md` (fake binary + env knobs).
- **Commit.** `feat(tunnel): feature gate, cloudflared detection, Remote Access page shell and test spine`.
- **Exit.** All gates green; the page is live in Chrome in all three states; the stub is the single fixture consumed by `cloudflared_proc`, `server_app` and `tests-js`.

### Phase 2 — Request-origin resolver, per-origin cookie `Secure`, `/info.origins`

- **Goal.** Retire the three correctness/security risks that exist with or without a tunnel, and give the page a real Origins card.
- **User-visible.** The Remote Access page lists every origin the instance serves (loopback, LAN, explicit public). Login via a LAN IP composes a correct callback; behind an `x-forwarded-proto: https` proxy the session cookie is now `Secure`.
- **routes_app.** `shared/request_origin.rs` (§2.4, `tunnel_host` wired as `None` until Phase 4); `routes_auth.rs:88-103` and `routes_setup.rs:132-168` switch to it (behavior-preserving except the 80/443 elision); `setup_api_schemas.rs` gains `OriginKind` / `OriginInfo` / `AppInfo.origins` (doc 21 §6 verbatim); `routes_setup.rs:79-88` builds origins — loopback `localhost` + `127.0.0.1` (never `0.0.0.0`; a bind address is not an origin) × `scheme()/port()`, LAN via `auth_scope.network().get_server_ip()` (`auth_scoped.rs:102`), public when `get_public_host_explicit()` (`setting_service.rs:390`) is `Some` — `setup_show`'s signature is unchanged, this is settings-derived (§2.4); `middleware/cookies/secure_cookie_middleware.rs` registered at `routes.rs:642`; `canonical_url_middleware` state widened to `Arc<dyn AppService>` at `routes.rs:644` with the `is_tunnel` early return already wired (dormant until Phase 4).
- **services.** `tunnels/redirect_uris.rs::desired_set` (§2.4).
- **UI.** `-components/OriginsRail.tsx` — one row per `useGetAppInfo().origins` entry, `data-testid="origins"` with `data-test-state={kind}` per row, rendered in the Remote Access `DetailRail`; `test-utils/msw-v2/handlers/tunnel.ts` gains `origins` on the info handler. (Phase 4 only adds the `tunnel` row to the same component.)
- **API/DTO.** `AppInfo.origins: Vec<OriginInfo>` (additive); ts-client regen.
- **Tests.**
  - routes_app resolver table `test_resolve_request_origin` — `localhost:1135` with no proto → `http://localhost:1135`; `app.example.com` + `x-forwarded-proto: https` → `https://app.example.com` (no `:1135`); `app.example.com:8443` → port kept; `tunnel_host` match with no proto → https and no port; `Host: bad_host` (underscore → rejected by `is_valid_hostname`) → rule-4 fallback to `public_server_url`; absent `Host` → same fallback.
  - routes_app: `test_auth_initiate_callback_from_forwarded_https_host`, **`test_auth_initiate_forwarded_https_on_lan_host_is_in_desired_set`** (the scheme-axis negative test — a spoofed `x-forwarded-proto: https` on a LAN host composes a callback that `desired_set` also registers), `test_auth_initiate_callback_lan_ip_keeps_port`, `test_setup_create_registers_request_origin_not_public_port`, `test_desired_set_preserves_unknown_existing_uris`, `test_session_cookie_secure_only_on_https_origin`, `test_session_cookie_secure_floor_kept_when_public_scheme_https`, `test_info_origins_loopback_lan_public`.
  - services: `test_desired_set_adds_tunnel_when_enabled`, `test_desired_set_removes_tunnel_when_disabled`, `test_desired_set_emits_both_schemes_for_non_loopback_hosts`.
  - server_app: `test_live_info_origins_loopback_matches_bound_port` (`live_server` fixture, `live_server_utils.rs:284`).
  - UI vitest+MSW: `OriginsRail lists loopback and lan kinds`, `OriginsRail hides tunnel kind when absent`, `OriginsRail renders nothing when origins is empty`.
  - Playwright: step `origins: card lists loopback and lan origins`; **regression guard** — one added assertion in `network-ip-setup-flow.spec.mjs` that LAN login still lands.
- **Live check.** `curl -s localhost:1135/bodhi/v1/info | jq .origins`; LAN-IP login in Chrome round-trips; DevTools shows the loopback cookie without `Secure`.
- **Docs.** `docs/architecture/security.md:9,183` (§2.5); `crates/routes_app/CLAUDE.md` — "the resolver is the only place that composes externally visible URLs".
- **Commit.** `feat(origins): shared request-origin resolver, per-origin session cookie Secure, AppInfo.origins`.
- **Exit.** Existing auth/setup/canonical tests and the LAN + public-host E2E specs stay green; new tests green.

### Phase 3 — Tier 1: `cloudflared` login + provision (create + route dns), persisted

- **Goal.** A real tunnel object and DNS route on the developer's zone, driven from the UI. Nothing runs yet.
- **User-visible.** "Connect Cloudflare" opens the dashboard; after auth the card flips to connected; entering `bodhi-dev.<zone>` provisions and shows the tunnel id + hostname; status reads "Provisioned (off)".
- **services.** Migration `db/sea_migrations/m20250101_000029_tunnels.rs` — the doc 23 §a table plus `dns_routed` and `dns_record_id` (template `m20250101_000021_api_model_oauth_credentials.rs`; appended last in `sea_migrations/mod.rs`; immutable once committed, per Migration Governance). `tunnel_entity.rs`; `tunnel_repository.rs` (`get/upsert/delete`, singleton by `tenant_id`, `begin_tenant_txn`, `#[automock]`, `encrypt_api_key`/`decrypt_api_key` `db/encryption.rs:141,161` + `DbError::from_encryption` for the tier-3 token columns this migration already ships, written in Phase 7a). `DefaultTunnelService::{status, login, provision, retry_dns, mark_dns_manual}`. `cloudflare/cli_control_plane.rs`: `begin_login` (pre-check for an existing cert → short-circuit; spawn `tunnel login`; capture `https://dash.cloudflare.com/argotunnel…` from stderr; `existing certificate` on exit 0 means already logged in; 10-minute bound; a second `POST /tunnel/login` while one is pending → `tunnel_error-login_in_progress`); `provision` = `list --output json -n <name>` (idempotency probe) → `create` into `<name>.json` → rename to `<uuid>.json` → **persist `tunnel_id` + `credentials_path` before** `route dns` (remote-orphan guard) → `route dns` → `dns_routed=true`, or on failure keep `tunnel_id`, set `last_error` and surface the exact CNAME (`<host> CNAME <uuid>.cfargotunnel.com`, proxied). `--overwrite-dns` only after an explicit user confirm.
- **cloudflared_proc.** `cmd.rs` (`run_capture`; `spawn_login` with a stderr reader thread + oneshot URL; `Drop` kills); `paths.rs` (`ensure_dir_0700($BODHI_HOME/cloudflared)`).
- **routes_app.** `tunnel_show`, `tunnel_login`, `tunnel_provision` (`ValidatedJson<ProvisionTunnelRequest>`, FQDN lowercase ≤253), `tunnel_dns_retry`, `tunnel_dns_mark_manual`.
- **UI.** `-components/ConnectCard.tsx` (shape of `routes/mcps/new/-components/OAuthConnectPanel.tsx`; opens `login_url` in a new tab; polls detect until `has_origin_cert`), `-components/ProvisionForm.tsx`, `-components/DnsFailurePanel.tsx` (**defensive path only**, see §0-A: CNAME text + Retry + "I created it manually"), status pill `data-testid="tunnel-status" data-test-state={status}`.
- **API/DTO.** `GET /tunnel`, `POST /tunnel/login` → `TunnelLoginResponse`, `POST /tunnel/provision`, `POST /tunnel/dns/{retry,mark-manual}`; ts-client regen.
- **Tests.**
  - services: `test_tunnel_repository_upsert_and_get`, `test_tunnel_repository_isolated_by_tenant`, `test_tunnel_repository_get_missing_returns_none`, `test_login_returns_url_from_provider`, `test_login_short_circuits_when_cert_present`, `test_login_conflict_when_in_progress`, `test_tunnel_service_provision_checks_list_before_create`, **`test_tunnel_service_provision_persists_ids_before_route_dns`**, `test_provision_renames_credentials_to_uuid_and_persists_path`, `test_provision_create_failure_probes_for_remote_orphan`, `test_provision_dns_failure_keeps_tunnel_id_and_sets_error`, `test_provision_requires_cert`, `test_provision_rejects_second_tunnel`, `test_mark_dns_manual_sets_dns_routed`.
  - cloudflared_proc: `test_spawn_login_captures_dashboard_url`, `test_login_existing_cert_detected_on_exit_0`, `test_run_capture_create_parses_json`, `test_run_capture_timeout`.
  - routes_app: `test_tunnel_login_returns_login_url`, `test_tunnel_provision_validates_hostname` (422), `test_tunnel_show_masks_secrets` (no `credentials_path`, no token), `test_tunnel_provision_error_codes` (`#[case]` table).
  - server_app: `test_live_tunnel_login_then_provision_full_flow` (the stub writes the cert under the harness's temp `HOME`; asserts `status=disabled` + hostname + credentials file `0400`, and that `invocations.jsonl` recorded `create --output json --credentials-file <BODHI_HOME>/cloudflared/…` with `TUNNEL_ORIGIN_CERT` present), `test_live_tunnel_provision_route_dns_conflict_surfaces_error` (scenario `route_dns_conflict`).
  - UI: `ConnectCard opens login url and polls`, `ProvisionForm submits hostname`, `shows manual CNAME panel on dns error`, `useTunnelActions invalidates on success`.
  - Playwright: steps `login: connect card shows dashboard link`, `provision: hostname form persists tunnel`; separate `test('provision shows manual CNAME guidance on route conflict')`.
- **Live check — agent-executable.** `BODHI_TUNNEL_ENABLED=true make app.run.live` against the fake `cloudflared`: walk Connect → Provision → every card state in Chrome (connected, provisioned, DNS-conflict panel via `FAKE_CLOUDFLARED_SCENARIO=route_dns_conflict`), and confirm the row and `invocations.jsonl` match §2.7.
- **Live check — owner-only (real Cloudflare, owner zone, disposable subdomain).** The agent stops here and hands the owner this numbered checklist to run and paste back before the commit: (1) Connect → dashboard → the card flips; (2) provision `bodhi-dev.<zone>`; (3) Zero Trust → Tunnels lists it; (4) `dig bodhi-dev.<zone> CNAME` → `<uuid>.cfargotunnel.com`. Per §0-A this is expected to succeed with `cert.pem` alone; if it does not, the account is a restricted team member and the defensive panel must make it unblockable by hand.
- **Docs.** `crates/services/src/db/CLAUDE.md` (migration list); services / routes_app / cloudflared_proc / bodhi `CLAUDE.md`.
- **Commit.** `feat(tunnel): tunnels table, tier-1 cloudflared login and hostname provisioning`.
- **Exit.** A real tunnel + DNS record exist on the dev zone, created from the UI; the row persists; gates green.

### Phase 4 — Run and supervise the connector, Turn on/off, consent, tunnel origin

- **Goal.** Traffic flows.
- **User-visible.** Turn on (behind a consent dialog) → "Connecting…" → "Connected · https://bodhi-dev.<zone>"; a phone loads `/ping`, `/bodhi/v1/info` and `/ui/`; Turn off stops it; the Origins card gains the tunnel entry.
- **cloudflared_proc `process.rs`.** `std::process::Command` + piped stdout/stderr + `std::thread` readers → `tracing` + a 256-line ring buffer; `Mutex<Option<Child>>`; `Drop` = kill + wait (copy the shapes at `llama_server_proc/src/server.rs:100-107,139-163,198-210,216-231`). `wait_ready` = `/ready` polling per §2.7. `stop(grace)` = SIGTERM (`libc::kill` on unix; plain `kill()` on Windows until Phase 10) → poll `try_wait` ≤ grace → `kill()`. `pidfile.rs` writes `$BODHI_HOME/cloudflared/connector.pid` JSON `{pid, exe, started_at}` on spawn and clears it on stop.
- **services.** `CloudflareTunnelProvider::start_connector` (argv §2.7; metrics port via `portpicker`). `DefaultTunnelService::{enable, disable, on_shutdown, active_hostname, active_public_url}` — `enable` guards gate + provisioned + `dns_routed`, sets `ConnectorState::Starting` in `LiveState`, then `tokio::spawn`s `wait_ready` → on success persists `enabled` + `last_public_url` and sets `Connected`; on timeout or early exit persists `error` with the log tail. An exit watcher flips `Exited` while keeping intent `Enabled` (no auto-restart before Phase 10). `disable` = `stop(30s)` → persist `disabled`.
- **server_app.** `CompositeShutdownCallback` + `ShutdownTunnelCallback` (`serve.rs:36-49,144-147`).
- **routes_app.** `tunnel_enable` / `tunnel_disable`; `routes_setup.rs:79-88` adds the `Tunnel` origin when `active_public_url()` is `Some`; the canonical-middleware tunnel exemption goes live. **`AppInfo.url` keeps its current meaning** — doc 21 follow-up §1 proposes overriding it with the tunnel URL and names `routes/users/-components/InviteLinkAction.tsx:19` as the consumer, but that component returns `null` unless `deployment === 'multi_tenant'` (`:15-17`) and tunnels are forbidden in multi-tenant (§2.2, §5.1 row 17), so it can never observe a tunnel URL. It is the only `appInfo.url` reader in the UI. `origins` already carries the tunnel entry under invariant 1; changing a public unauthenticated DTO field for no consumer is not worth it.
- **UI.** `-components/TunnelStatusCard.tsx` (pill `data-test-state={connector}`, Turn on/off, public URL + copy, `last_error`; polling while `Starting`); `-components/ConsentDialog.tsx` — *"This exposes the whole app — the UI and the OpenAI/Anthropic/Gemini APIs — at `https://<host>` to anyone on the internet. Sign-in is still required, but the login page and the API endpoints become publicly reachable. On this Cloudflare zone: keep Bot Fight Mode **off** (it blocks SDK clients) and turn Always Use HTTPS **on**."* plus an "I understand" checkbox ([doc 13 §2,§4](../../../research/tunnel/13-cloudflare-edge-behavior-for-llm-api-traffic.md)); `OriginsRail.tsx` (from Phase 2) gains the `tunnel` row.
- **API/DTO.** enable/disable; `AppInfo.origins` gains the `tunnel` kind (`url` unchanged); ts-client regen.
- **Tests.**
  - cloudflared_proc: `test_process_reaches_ready_via_fake`, `test_process_ready_timeout_when_never_ready` (`ready_never`), `test_process_fails_fast_when_metrics_never_binds` (`metrics_bind_fail`), `test_process_stop_sends_sigterm_then_kills_after_grace` (`ignore_sigterm`), `test_run_drop_kills_child`, `test_process_argv_matches_pinned_contract` (from the invocation log; `--token` absent).
  - services: `test_enable_requires_provisioned_and_dns`, `test_enable_transitions_starting_to_connected`, `test_enable_persists_error_on_ready_timeout_and_kills_child`, `test_enable_conflict_when_already_enabled`, `test_disable_stops_connector_and_keeps_provisioning`, `test_connector_exit_marks_exited_keeps_enabled`, `test_active_public_url_only_when_enabled_and_connected`, `test_on_shutdown_stops_connector`.
  - routes_app: `test_tunnel_enable_returns_starting`, `test_tunnel_enable_gate_off_403`, `test_setup_show_origins_includes_tunnel_when_active`, `test_setup_show_url_unchanged_when_tunnel_active`, `test_canonical_redirect_skips_active_tunnel_host`, `test_canonical_still_redirects_non_tunnel_host_when_public_host_explicit`.
  - server_app: `test_live_tunnel_enable_connects_and_info_lists_origin` (bounded poll, no sleeps), `test_live_tunnel_info_origins_lists_tunnel_only_when_connected`, `test_live_tunnel_disable_kills_connector_process`, `test_live_tunnel_request_with_tunnel_host_not_redirected` (`raw_get_with_host`), `test_live_tunnel_crash_after_ready_sets_error` (`crash_after_ready`), `test_tunnel_child_dies_with_graceful_shutdown`, `test_tunnel_run_argv_and_env_contract`.
  - UI: `ConsentDialog.test.tsx`, `TunnelStatusCard` per state, `OriginsRail shows the tunnel row only when a tunnel origin is present`.
  - Playwright: steps `enable requires explicit consent`, `enable: status goes starting then connected with public url`, `origins: rail lists tunnel origin`, `disable: back to provisioned and rail drops tunnel`; the fixture's `afterEach` asserts the stub pid exited.
- **Live check — agent-executable.** Against the fake: Turn on behind the consent dialog → Connecting → Connected with the public URL; `raw_get_with_host` proves the tunnel host is not 301'd; Turn off → Stopped; quit the app → `pgrep cloudflared` empty; every card state walked in Chrome.
- **Live check — owner-only (real Cloudflare + a phone on another network).** Numbered checklist handed back before the commit: (1) Turn on → Connected; (2) the phone opens `https://<host>/ping` and `/bodhi/v1/info`; (3) `/ui/` renders — login still fails at Keycloak with `Invalid redirect_uri`, exactly what Phase 5 fixes; (4) a streaming `POST /v1/chat/completions` with an existing `sk-bodhiapp_…` token arrives incrementally (doc 13 §1); (5) the backend log shows `Host: <host>` and `x-forwarded-proto: https`, confirming §0-B in this deployment; (6) Turn off → edge 530. **This is also where the §2.7 argv pin meets a real binary** — if it rejects the flags, fix the argv and the fake here.
- **Docs.** `crates/server_app/CLAUDE.md` (composite shutdown), `crates/cloudflared_proc/CLAUDE.md` (argv + ready + PID contract, Windows caveat), `docs/architecture/security.md` (public-exposure consent).
- **Commit.** `feat(tunnel): run and supervise cloudflared connector, turn on/off with consent, tunnel origin in /info`.
- **Exit.** A phone reaches the app through the real tunnel; orphan/shutdown tests green on macOS and Linux.

### Phase 5 — Keycloak redirect-URI sync and login through the tunnel — **MVP capstone**

- **Prereq.** Phase 0 verified live on **both** `test-id` and `main-id`. CI is live (§4.5), so the live tests below run on GitHub Actions the moment this phase is pushed to trunk — an undeployed SPI fails the build, which is the intended signal.
- **Goal.** `https://<host>/ui/login` completes OAuth and the app is fully usable remotely.
- **User-visible.** Turn on registers `https://<host>/ui/auth/callback` in Keycloak; login works from a phone; Turn off removes it; an older auth server yields a visible warning, not a failure.
- **services.** `AuthService::update_redirect_uris(client_id, secret, uris) -> Result<Option<Vec<String>>>` and `get_redirect_uris(...) -> Result<Option<Vec<String>>>` (doc 22 §3.1; `Ok(None)` on 404; pattern beside `register_client` `auth_service.rs:311`; `MockAuthService` regenerates). `DefaultTunnelService::sync_redirect_uris(add: bool)` — credentials from `TenantService::get_standalone_app` (`tenant_service.rs:13`) → `GET` current → `desired_set` → **`PUT` only when the set actually changed** → `Ok(None)` gives `keycloak_sync = SpiOutdated` with copy *"auth server predates redirect-URI sync; update it to sign in through the tunnel URL"* (the tunnel stays Enabled) → any other error gives `Failed` + message (status still Enabled) → persist `last_synced_at`. Called after `Connected` in `enable`, before persisting in `disable`, and idempotently in `on_startup`. `POST /tunnel/sync` is the manual retry.
- **routes_app.** The resolver now receives `tunnel_service().active_hostname()`; `tunnel_show` exposes `keycloak_sync`.
- **UI.** Rail row "Sign-in through tunnel": Synced / SPI-outdated warning / Failed + Retry sync; `last_synced_at`.
- **tests-js.** `auth-server-client.mjs::getClientRedirectUris(clientId, clientSecret)` calls the SPI `GET` **only**, with `getResourceServiceAccountToken` (`:259`), and **throws** on 404/401 with a message naming the required SPI release. No realm-admin fallback: an `if`-guarded substitute is the forbidden E2E branching, and it would make the `enable`/`disable` steps pass green against a Keycloak where Phase 0 was never deployed — the one failure these steps exist to catch.
- **API/DTO.** `keycloak_sync`; ts-client regen.
- **Tests.**
  - services (mockito): `test_update_redirect_uris_success`, `test_update_redirect_uris_404_is_ok_none`, `test_update_redirect_uris_500_maps_api_error`, `test_get_redirect_uris_404_is_ok_none`.
  - services (tunnels): `test_enable_syncs_add_tunnel_callback`, `test_disable_syncs_remove_tunnel_callback`, `test_sync_skips_put_when_unchanged`, `test_sync_404_sets_spi_outdated_not_error`, `test_sync_error_keeps_enabled_sets_failed`, `test_on_startup_resyncs_when_enabled`.
  - routes_app: `test_auth_initiate_tunnel_host_composes_https_no_port`, `test_session_cookie_secure_on_tunnel_host`, `test_tunnel_show_exposes_keycloak_sync`.
  - server_app (**throwaway Keycloak resource client per test**, §4.2): `test_live_tunnel_enable_registers_tunnel_redirect_uri` (read back via `spi_redirect_uris`; asserts the tunnel callback present **and** the loopback `:51135` entries intact), `test_live_tunnel_disable_removes_tunnel_redirect_uri`, `test_live_tunnel_auth_initiate_via_tunnel_host` (`raw_get_with_host`, parse `login_url`'s `redirect_uri` = `https://<host>/ui/auth/callback`; `Set-Cookie` carries `Secure`), `test_live_login_initiate_through_tunnel_host_keycloak_accepts_redirect_uri` (fetch the composed authorize URL → login page, not `Invalid parameter: redirect_uri`).
  - UI: `shows sync warning banner`, `shows synced indicator`.
  - Playwright: steps `enable: keycloak lists tunnel callback` and `disable: keycloak callback removed`, both against the spec's **own throwaway** resource client, never the shared one; separate `test('tunnel host is exempt from canonical redirect')` — a server with an explicit `publicHost` plus fake tunnel host `tunnel.localhost`, where `page.goto('http://tunnel.localhost:<port>/ui/')` is not 301'd (Chrome resolves `*.localhost` without DNS).
- **Live check (Chrome + phone).** Turn on → the Keycloak admin console shows the callback; phone `/ui/login` → Keycloak → back to `https://<host>/ui/chat`; DevTools shows the cookie with `Secure`; `http://localhost:1135` login still works in another profile; Turn off → the callback is gone; point `BODHI_AUTH_URL` at a local `make dev.up` Keycloak **without** the SPI change → warning banner, tunnel stays Enabled.
- **Docs.** `crates/services/CLAUDE.md` (SPI method table, sync semantics); `crates/routes_app/CLAUDE.md`; `docs/architecture/security.md` (tunnel origin note).
- **Commit.** `feat(tunnel): Keycloak redirect-URI sync on enable/disable, login through the tunnel host`.
- **Exit.** **MVP** — remote login and chat through the real tunnel; the SPI-outdated path demonstrated; gates green.

### Phase 6 — Startup resume, Remove, orphan reap, kill switch

- **Goal.** Survives restarts, can be fully undone, and honors the env kill switch.
- **User-visible.** Relaunch with an enabled tunnel → reconnects and re-syncs with no clicks; Remove deletes the tunnel + local credentials (tier 1 names the CNAME to delete by hand); `BODHI_TUNNEL_ENABLED=false` prevents any resume.
- **services.** `on_startup` — `PidFile::reap_stale` (kill only when the pid is alive **and** the process exe basename contains `cloudflared`) → gate on and row `enabled` → the enable path including sync. `remove` — stop if running → `tunnel delete -f <uuid>` (with `TUNNEL_ORIGIN_CERT`) → delete the file named by the persisted `credentials_path` (**keep `cert.pem`** — it is the user's Cloudflare sign-in; an explicit "also forget Cloudflare sign-in" opt-in removes it) → sync removal → delete the row; `DeprovisionReport.dns_manual_cleanup: Option<String>` surfaced once (tier 1 has no CLI route-delete; the REST tiers delete it in Phase 7). `TunnelError::RemoveIncomplete` keeps the row for retry.
- **server_app.** Startup hook at `serve.rs:155-159`.
- **routes_app / UI.** `DELETE /bodhi/v1/tunnel`; `-components/RemoveDialog.tsx` + the post-remove notice; a "Resumes automatically on launch" hint.
- **Tests.**
  - services: `test_resume_starts_when_row_enabled`, `test_resume_noop_when_disabled_or_error`, **`test_startup_does_not_resume_when_gate_off`**, `test_resume_reaps_stale_pid`, `test_remove_deletes_row_and_credentials`, `test_remove_keeps_origin_cert_by_default`, `test_remove_reports_manual_dns_cleanup`, `test_remove_failure_keeps_row`.
  - cloudflared_proc: `test_pidfile_written_and_cleared`, `test_pidfile_reap_kills_stale_cloudflared_only`, `test_pidfile_reap_ignores_reused_pid_of_other_exe`.
  - routes_app: `test_tunnel_remove_returns_never_provisioned`, admin-only case for `DELETE`.
  - UI vitest+MSW (handler for `DELETE /bodhi/v1/tunnel`): `RemoveDialog requires explicit confirm`, `RemoveDialog copy names the hostname`, `shows manual DNS cleanup notice when dns_manual_cleanup is present`, `hides the notice when absent`, `useTunnelActions.remove invalidates status`.
  - server_app: `test_live_tunnel_startup_resumes_enabled_tunnel` (two `live_server` starts over one `BODHI_HOME`), `test_live_tunnel_startup_does_not_resume_when_gate_off`, `test_live_tunnel_startup_reaps_orphan_connector`, `test_live_tunnel_shutdown_stops_connector_before_listener_closes`.
  - Playwright: steps `turn off keeps hostname and stops connector`, `remove: clears provisioning and shows manual dns notice`; separate `test('resumes enabled tunnel after server restart')`.
- **Live check.** Quit while Enabled → relaunch → Enabled again; `kill -9` the app → relaunch → the old `cloudflared` is reaped; Remove → the dashboard tunnel is gone; `BODHI_TUNNEL_ENABLED=false` → no resume.
- **Docs.** `crates/server_app/CLAUDE.md`, `crates/services/CLAUDE.md` (lifecycle table), `crates/bodhi/src/CLAUDE.md`.
- **Commit.** `feat(tunnel): startup resume, remove semantics, orphan connector reap and kill switch`.
- **Exit.** Tier 1 is feature-complete behind the gate; three live scenarios verified.

### Phase 7a — Typed Cloudflare REST client + token intake

- **Goal.** The REST client tier 2 will reuse, and a verified, encrypted API token — nothing provisions remotely yet. (Tier 1 needed two phases for strictly less surface; one commit for the whole tier-3 stack is not a thin slice and has no intermediate live-verifiable state.)
- **User-visible.** "Use an API token instead": two prefilled deep links (account-scoped `argotunnel`; zone-scoped `dns` + `zone` — one combined link silently drops the account key, doc 11 follow-up §3), paste, verify, and the card lists the zones the token can reach.
- **services.** `cloudflare/api_client.rs` (workspace `reqwest` through the existing safe wrapper): `verify_token`, `list_accounts`, `list_zones`; typed error mapping `10000 -> TokenInvalid`, `9109 -> TokenMissingPermission{which}`. `token_links.rs`. `set_api_token` verifies, probes the two permission surfaces to name a missing one, then encrypts the triplet into the Phase-3 columns (no migration). Dev-only base-URL override `BODHI_TUNNEL_CLOUDFLARE_API_URL` via `get_dev_env` (inert in release).
- **routes_app / UI.** `POST /tunnel/api-token` (the response never echoes the token), `GET /tunnel/zones`, `DELETE /tunnel/api-token`; `-components/ApiTokenCard.tsx` (two link buttons, `type=password` field, permission-diagnosis errors), zone picker, `has_api_token` badge.
- **Tests.** services (mockito) one per call plus `test_set_api_token_rejects_invalid_10000`, `test_set_api_token_names_missing_permission_9109`, `test_set_api_token_stores_v2_ciphertext_only`, `test_status_never_serializes_token`, `test_delete_api_token_clears_columns`; routes_app `test_api_token_response_masks`, `test_api_token_admin_only`, `test_zones_admin_only`; server_app `test_live_tunnel_set_api_token_lists_zones` against `tests-js/fixtures/fake-cloudflare-api.mjs`, registered as a `webServer` entry in the shape of `test-mcp-auth-server` (`crates/lib_bodhiserver/playwright.config.mjs:133-145`); UI card tests including the exact URL-encoding of both links; Playwright step `api-token: paste token then zones are listed`.
- **Live check — owner-only.** Paste a real token minted from the two links; the zone list matches the dashboard; `ps` shows no token; the DB ciphertext is `v2:`-prefixed.
- **Docs.** `crates/services/CLAUDE.md` (client + error codes); `docs/architecture/security.md` (token at rest, never logged or echoed, revocation guidance).
- **Commit.** `feat(tunnel): typed Cloudflare REST client and API-token intake`.
- **Exit.** A pasted token verifies, its zones render, and the DB holds only `v2:` ciphertext; gates green.

### Phase 7b — Tier-3 remote provisioning and `TUNNEL_TOKEN` data plane

- **Goal.** Provision, enable and remove with no `cloudflared login` at all.
- **User-visible.** Pick a zone, provision remotely (`config_src=cloudflare`), turn on; Remove also deletes the CNAME.
- **services.** `api_client.rs` gains `create_tunnel(config_src=cloudflare, tunnel_secret)`, `put_configuration` (ingress hostname → `http://127.0.0.1:<port>`, catch-all `http_status:404`), `get_token`, `create/find/delete_dns_record`, `delete_connections`, `delete_tunnel?cascade=true` (doc 12 §1); `81053 -> DnsRecordExists`. The provider branches on `auth_mode`: REST control plane; the data plane passes `TUNNEL_TOKEN` in env with no positional and no `--credentials-file`; `deprovision` deletes the DNS record and then the tunnel.
- **UI.** Provision/Remove flow reuses Phase 3/6 components with tier-3 copy; the manual-CNAME panel is not reachable in this tier.
- **Tests.** services `test_provision_tier3_creates_remote_tunnel_dns_and_config`, `test_provision_tier3_dns_record_exists_81053`, `test_remove_tier3_deletes_dns_then_tunnel`; cloudflared_proc `test_start_connector_tier3_uses_env_not_argv`; server_app `test_live_tunnel_api_token_flow_with_mock_cloudflare_api` (provision → enable → remove end to end); Playwright `test('API-token tier provisions with stub Cloudflare API')` (separate `test` — different server configuration).
- **Live check — owner-only.** Provision + enable + login on a real zone with no `cloudflared login`; `ps` shows no token; Remove leaves no DNS record.
- **Docs.** `crates/services/CLAUDE.md` (tier matrix); `crates/cloudflared_proc/CLAUDE.md` (the token data-plane argv).
- **Commit.** `feat(tunnel): tier-3 remotely-managed tunnel provisioning and TUNNEL_TOKEN data plane`.
- **Exit.** Tier 3 provisions, enables and removes end to end against the fake Cloudflare API; no token in argv, logs or status; gates green.

### Phase 8 — Tier-2 Cloudflare OAuth (PKCE public client) — owner gate first

- **Owner pre-step (one-way door; blocks the phase, does not skip it).** Register BodhiApp's self-managed OAuth client (`dash.cloudflare.com/?to=/:account/oauth-clients`): logo, client URL, policy/tos URLs, DNS TXT domain verification, scopes `argotunnel.write dns.write zone.read offline_access`, loopback redirect `http://127.0.0.1:<port>/ui/settings/remote-access/oauth/callback`; **public visibility is permanent** (doc 11 §1.2). Then a ~1-hour spike on a throwaway account: complete one PKCE grant and confirm the access token actually authorizes `POST /accounts/{id}/cfd_tunnel` and a DNS write (doc 11 follow-up, "remains unverified" items 1-3). Record the decision and the client id here before any code.
- **Work.** Migration `m20250101_000030_tunnels_oauth_tokens.rs` (encrypted refresh-token triplet + `oauth_expires_at`); `cloudflare/oauth.rs` (authorize URL, PKCE S256, `state` in session mirroring `routes_auth.rs`'s callback handling, exchange/refresh feeding the Phase-7a client's bearer source); `POST /tunnel/oauth/initiate|callback`; `-components/OAuthCard.tsx` reusing the MCP `OAuthConnectPanel` shape. If `port() != 1135` the tier is hidden with an explanation (the registered redirect is fixed).
- **Tests.** services `test_oauth_authorize_url_pkce_s256`, `test_oauth_exchange_stores_encrypted_refresh`, `test_oauth_state_mismatch_rejected`, `test_oauth_refresh_on_401_then_retry_once`; routes_app initiate/callback tests; server_app against a mockito IdP; UI card tests; Playwright against `fake-cloudflare-api.mjs` extended with `/oauth2/auth|token`.
- **Live check.** The real consent screen once → tunnel provisioned and enabled.
- **Commit.** `feat(tunnel): Cloudflare OAuth tier (PKCE public client)`. **Exit.** Grant verified live, or the phase is parked with findings recorded in §5.1 row 10.

### Phase 9 — Download `cloudflared` on explicit trigger

- **Work.** `cloudflare/download.rs`: GitHub `releases/latest` metadata → asset per OS/arch (doc 14 §1 table) → stream to `$BODHI_HOME/bin/cloudflared[.exe].tmp` → sha256 against GitHub `assets[].digest` (TOFU-grade — say so in the consent copy) → rename → `chmod +x` → `--version` smoke check. macOS: run `codesign -dv` and explain the outcome rather than silently stripping quarantine; Linux `noexec`: attempt-and-explain; Windows SmartScreen copy. The consent copy names Apache-2.0 (doc 14 §4). `POST /tunnel/cloudflared/download { accept: true }`; in-memory progress on status; detection prefers a system install over `$BODHI_HOME/bin`; the button lives inside `InstallGuide`.
- **Tests.** services (mockito GitHub) `test_download_picks_asset_for_target`, `test_download_rejects_digest_mismatch`, `test_download_sets_executable_bit`, `test_download_requires_consent_flag`, `test_detect_prefers_system_install`; routes_app admin-only + codes; server_app `test_live_download_then_detect_finds_managed_binary` (mock GitHub serves the stub); UI vitest `download button is disabled until consent is checked`, `shows progress while downloading`, `shows checksum-mismatch error`, `hides the button when a binary is already detected`; Playwright step `download: consent then detect finds managed binary`.
- **Live check — agent-executable, and load-bearing here.** Rename the system `cloudflared` away on macOS → the page shows the install guide with the download button → trigger it in Chrome → confirm the checksum path runs, the `codesign -dv` outcome is explained rather than silently worked around, the `--version` smoke check passes, and detect then reports the managed binary. Restore the system install → detect flips back to preferring it.
- **Docs.** `crates/services/CLAUDE.md` (download + digest policy); `crates/cloudflared_proc/CLAUDE.md` (managed-binary path and detect precedence); `docs/architecture/security.md` (TOFU-grade digest, Apache-2.0 provenance, no silent quarantine strip).
- **Commit.** `feat(tunnel): download cloudflared on explicit user consent with checksum verification`.
- **Exit.** A managed binary is downloaded, checksum-verified and detected; detect still prefers a re-appearing system install; gates green.

### Phase 10 — Hardening

- **Work.** Linux `PR_SET_PDEATHSIG(SIGTERM)` via `pre_exec` (doc 15 §7.4). Windows Job Object `KILL_ON_JOB_CLOSE` + `CREATE_NEW_PROCESS_GROUP` + `GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT)` for graceful stop (doc 10 §4.6). `Degraded` after N consecutive `/ready` 503s while the child is alive (never kills). Restart on `Exited` while intent is `Enabled`, 120 s cooldown, bypassed for startup / network-change / sleep-wake one-shots (doc 15 §7.5). Steady-state `/ready` probe every 30 s. Supported-version nudge (365 days). Rail copy: prefer `stream: true` for work with >~100 s to first byte (524, doc 13 §2), keep Bot Fight Mode off, turn Always Use HTTPS on, upload limits.
- **Tests.** cloudflared_proc `test_pdeathsig_kills_child_when_parent_sigkilled` (`#[cfg(target_os = "linux")]`, helper parent), `test_windows_ctrl_break_graceful` (`cfg(windows)`, manual-QA record); services `test_degraded_after_n_ready_failures`, `test_restart_after_exit_respects_cooldown`, `test_no_restart_storm_on_flapping`; server_app `test_live_connector_restart_after_crash_respects_cooldown` (`TestTimeService`); Playwright step `resilience: status shows degraded then recovers` (`flap_ready`).
- **Manual Windows QA checklist** (no CI signal, §4.5): tray quit kills the child; force-kill BodhiApp and the child dies (Job Object); relaunch reaps; phone login through the tunnel.
- **Live check — agent-executable.** On Linux: `kill -9` the parent → the child dies (PDEATHSIG). With `flap_ready`, the card shows Degraded then recovers in Chrome; `crash_after_ready` shows a restart honoring the 120 s cooldown, and no restart storm. Rail copy renders in both themes.
- **Live check — owner-only.** The Windows checklist above, signed off on a real Windows box.
- **Docs.** `crates/cloudflared_proc/CLAUDE.md` (PDEATHSIG / Job Object, graceful-stop matrix per OS); `crates/services/CLAUDE.md` (degraded/restart policy); `crates/bodhi/src/CLAUDE.md` (rail copy); `docs/architecture/security.md` if the exposure copy changes.
- **Commit.** `feat(tunnel): process-lifetime binding, degraded detection with cooldown restarts, edge guidance`.
- **Exit.** The child dies with the parent on Linux (automated) and on Windows (manual QA checklist signed off); the restart cooldown is proven by `TestTimeService`; gates green.

### Phase 11 — Real-Cloudflare `@scheduled` E2E

- **Work.** Run the doc 24 follow-up spike workflow once (measure registration latency and protocol per OS, then delete the workflow). Repo secrets `INTEG_TEST_CLOUDFLARE_API_TOKEN` / `_ACCOUNT_ID`, vars `_ZONE_ID` / `_DOMAIN` on a **disposable zone** (never `getbodhi.app`); `.env.test.example` lines. `test('named tunnel on a real zone @scheduled @cloudflare-live')`: real `cloudflared` from PATH, tier-3 path, unique `e2e-<runid>.<domain>`, asserting `https://…/ping`, a browser login and a streamed chat completion; `afterAll` sweeps DNS records and tunnels by the `e2e-` prefix via REST. `crates/lib_bodhiserver/playwright.config.mjs:16,27-28` already excludes `@scheduled` from normal runs — **load-bearing**, since CI fires on every push (§4.5). Throw on missing env — never skip.
- **Live check — owner-only.** One manual `--grep @scheduled` run on the disposable zone; confirm the sweep leaves no tunnel and no DNS record behind, and that a normal `make test.e2e` still skips the spec.
- **Docs.** `crates/lib_bodhiserver/tests-js/CLAUDE.md` (the `@scheduled` tag, required secrets/vars, sweep contract); `.env.test.example`.
- **Commit.** `test(tunnel): scheduled real-Cloudflare named tunnel E2E with teardown sweep`.
- **Exit.** One scheduled run creates and then fully sweeps a real tunnel + DNS record on the disposable zone; the normal suite is unaffected.

---

## 4. Test infrastructure

### 4.1 Fake `cloudflared` — one file, two consumers

**Path:** `crates/lib_bodhiserver/tests-js/fixtures/bin/fake-cloudflared.mjs` (`#!/usr/bin/env node`, executable bit committed and re-applied by the Rust fixture with `std::fs::set_permissions`). Playwright reaches it through `FAKE_CLOUDFLARED_BIN` exported from `test-helpers.mjs`; `cloudflared_proc` and `server_app` reach it through `CARGO_MANIFEST_DIR/../lib_bodhiserver/tests-js/fixtures/bin/fake-cloudflared.mjs` — the same cross-crate resolution `live_server_utils.rs:64-69` already uses for `llama_server_proc/bin`. One stub, doc 10 §D's two named consumers, zero drift. *(Rejected: a second 2-line POSIX wrapper under `server_app/tests/resources/` — a second file to keep in sync. If the exec bit ever fails to survive a checkout, the harness spawns `node <path>` instead.)* Node ≥ 22 is already a repo requirement — **throw** if it is missing, never skip.

**Contract** (doc 10 §D; output strings verbatim from doc 10 §1/§2/§3/§5 **except the `create`-collision error, which doc 10 §2 marks UNVERIFIED** — the real Cloudflare API text for "name already in use" is unknown, so the stub emits a plausible-but-clearly-fake API-style error and no production code string-matches it):

| Invocation | Behavior |
|---|---|
| `--version` | `cloudflared version 2026.9.1 (built 2026-09-11-0000 UTC)`; scenario `old_version` gives `2024.1.0` |
| `tunnel login` | doc 10 §1 stderr text containing `https://dash.cloudflare.com/argotunnel?callback=fake`; after `FAKE_CLOUDFLARED_LOGIN_DELAY_MS` (300) writes `cert.pem` (PEM `ARGO TUNNEL TOKEN` + `{"zoneID","accountID","apiToken","endpoint"}`) at `$TUNNEL_ORIGIN_CERT` or `$HOME/.cloudflared/cert.pem`; exit 0. Scenario `login_existing_cert` prints `You have an existing certificate at …`, exit 0 |
| `tunnel create [--output json] --credentials-file <p> <name>` | writes `<p>` mode `0400` with `{AccountTag,TunnelSecret,TunnelID,Endpoint}`; stdout `{id,name,created_at}`; on collision exit 1 with a fake API-style error (`Failed to create: FAKE-API: tunnel name unavailable`) — never a real-looking message, because provision drives retry off **exit code plus the `list -n <name>` probe**, never message text (the same discipline §2.7 applies to `route_dns_conflict`). Refuses to overwrite an existing target file, matching `writeTunnelCredentials()` |
| `tunnel list --output json [-n <name>]` | JSON array from `$FAKE_CLOUDFLARED_STATE/tunnels.json` |
| `tunnel route dns [--overwrite-dns] <t> <host>` | `Added CNAME <host> which will route to this tunnel`; scenario `route_dns_conflict` exits 1 with an API-style error unless `--overwrite-dns` |
| `tunnel delete -f <t>` | removes the credentials file and the state entry; exit 0 |
| `tunnel token <t>` | base64 `{"a","s","t","e"}` |
| `tunnel run … --metrics 127.0.0.1:<port> …` | binds `<port>`; `GET /ready` returns `503 {"status":503,"readyConnections":0,…}` for `FAKE_CLOUDFLARED_READY_DELAY_MS` (300) then `200 {"status":200,"readyConnections":1,"connectorId":"00000000-0000-0000-0000-000000000000"}` (doc 10 §D's pinned value; the supervisor's rule is `200 && readyConnections > 0`, §2.7, so the count is deliberately free); stdout emits the zerolog envelope doc 10 §C pins — `{"level":"info","time":"<rfc3339>","message":"Starting metrics server on 127.0.0.1:<port>/metrics"}` (the fail-fast parser is written against this object shape) — then `{"level":"info","time":"<rfc3339>","message":"Registered tunnel connection","connIndex":<0-3>,…}` × 4; stays foreground; SIGTERM/SIGINT prints `Unregistered tunnel connection` and exits 0. Accepts `TUNNEL_TOKEN` env with no positional (tier 3) and **fails if `--token` appears on argv**. Accepts the pinned flags in either position (§2.7). |

**Scenarios** via `FAKE_CLOUDFLARED_SCENARIO`: `ready_never`, `metrics_bind_fail` (exit 1 before binding), `crash_after_ready` (exit 2 after 1 s), `ignore_sigterm` (needs a second signal), `flap_ready` (200/503 every 2 s), `route_dns_conflict`, `login_existing_cert`, `old_version`. An unknown subcommand exits 64 with usage on stderr.

**Invocation log.** Every invocation appends one JSONL line to `$FAKE_CLOUDFLARED_LOG`: `{ts, argv, env:{TUNNEL_TOKEN:<bool>, TUNNEL_ORIGIN_CERT:<string|null>}, cwd, pid}` — **secret presence only, never values**. Both suites assert the wire shape from it.

**Injection knobs:** `BODHI_TUNNEL_ENABLED=true` (required for every dev/test binary — `bodhiserver_dev` is `AppType::Container`), `BODHI_TUNNEL_CLOUDFLARED_PATH`, `BODHI_TUNNEL_ORIGIN_CERT`, `FAKE_CLOUDFLARED_SCENARIO`, `FAKE_CLOUDFLARED_LOG`, `FAKE_CLOUDFLARED_STATE`, plus the dev-only `BODHI_TUNNEL_CLOUDFLARE_API_URL` (Phase 7).

### 4.2 `server_app` live harness — `tests/utils/tunnel_harness.rs`

**Each piece lands in the phase of its first consumer** — the harness is never built ahead of use, and nothing in Phase 1 touches the Phase-0 SPI.

*Phase 1:*
- `fake_cloudflared() -> FakeCloudflared { bin, log, state_dir, home: TempDir }` — chmods the stub, truncates the log, and points `HOME` at a temp dir for the test (safe under `#[serial_test::serial(live)]`) so a developer's real `~/.cloudflared/cert.pem` is never picked up; `invocations() -> Vec<Invocation>`; `with_scenario(&str)`.
- `live_tunnel_server(fake) -> TestServerHandle` — the existing minimal-app-service setup (`live_server_utils.rs:59-90`) plus `BODHI_TUNNEL_ENABLED=true` and `BODHI_TUNNEL_CLOUDFLARED_PATH`. Nothing else.

*Phase 4:*
- `poll_tunnel_status(client, base, cookie, pred, timeout)` — a bounded loop, never a fixed sleep.
- `raw_get_with_host(addr, host_header, path) -> (StatusCode, HeaderMap, String)` — a raw tokio `TcpStream` HTTP/1.1 writer, because neither reqwest nor a browser reliably spoofs `Host`; used for every tunnel-host canonical / cookie / `auth_initiate` assertion.

*Phase 5 (first phase that reads Keycloak state):*
- A **throwaway Keycloak resource client per test**, layered onto `live_tunnel_server`: `auth_service.register_client(...)` (`auth_service.rs:311`) → `forward_request("POST", "test/clients/{id}/dag", …)` (`BodhiResourceProvider.java:331`) to enable Direct Access Grants so the password grant works → a tenant row created with it → cleanup in a `Drop` guard. **Why:** the shared `INTEG_TEST_RESOURCE_CLIENT_ID` (`live_server_utils.rs:166-167`) must never receive a full-replace redirect-URI `PUT` — it would clobber URIs every other live test depends on, on every CI push (§4.5).
- `spi_redirect_uris(app_service, client_id, client_secret) -> Vec<String>` — service-account token plus the Phase-0 `GET`, **unconditionally**: it never falls back to a realm-admin read, and it panics with a message naming the required SPI release on 404/401, so a missing Phase-0 deploy fails loudly instead of passing green.
- Env unchanged: `INTEG_TEST_AUTH_URL/_REALM/_USERNAME/_PASSWORD`, hard-failing when unset.

### 4.3 Playwright — fixtures, page object, one growing spec

- `fixtures/tunnelFixtures.mjs::getTunnelServerConfig(authServerConfig, port, { scenario, cloudflaredPath, tunnelEnabled, clientId, clientSecret })` → a server-manager config whose `envVars` carry `BODHI_TUNNEL_ENABLED`, `BODHI_TUNNEL_CLOUDFLARED_PATH` and the `FAKE_CLOUDFLARED_*` knobs. Unknown keys pass through `buildEnvFromConfig` (`test-helpers.mjs:54`, merge at `:82-88`), which already points `HOME` at the temp `bodhiHome`, so the stub's `cert.pem` lands there.
- `pages/RemoteAccessPage.mjs extends BasePage` — `open()` via `navViaShell('settings','remote-access')` (`BasePage.mjs:27`), `expectDetect`/`expectStatus` on `[data-testid][data-test-state]`, `clickConnect`, `provision`, `enable`, `turnOff`, `remove`, `publicUrl`, `origins`, `expectSyncState`. No `page.evaluate`, no context fetch, no `waitForTimeout`, no if/else.
- `specs/settings/remote-access-tunnel.spec.mjs` — `beforeAll` builds the auth config and a **throwaway** resource client via `authClient.createResourceClient(...)` (`auth-server-client.mjs:211`), cleaned up in `afterAll`; throws on missing env. **One** growing `test('Remote Access tunnel lifecycle (fake cloudflared)')` with a `test.step` per behavior; a separate `test()` only when the server configuration differs (missing binary, gate off, restart, API-token tier, OAuth tier). `reducedMotion:'reduce'` for the V2 rail screens.
- Runs inside `make test.e2e` in the `standalone` project; the `@scheduled` live spec runs only via the scheduled invocation. The config lives at **`crates/lib_bodhiserver/playwright.config.mjs`** (crate root, not under `tests-js/`).
- **Runtime budget.** That config is `fullyParallel: false`, `workers: 1`, `retries: 0` (`:29,34,35`) with a 120 s per-test timeout (`:12`), so the growing lifecycle test is serialized and unretried while accumulating ~15 `test.step` blocks, each with a real stubbed spawn, `/ready` polling and live Keycloak round-trips. Policy: put an explicit `test.setTimeout(<n>)` at the top of the lifecycle test and **re-evaluate it every phase** — each phase's retrospective records the spec's wall-clock time (kickoff §6 step 6). `retries: 0` stays (flakiness is absorbed at POM level by design); if the wall clock exceeds ~4 minutes, split the lifecycle test at a **server-configuration** boundary (provisioning vs. run/sync) rather than letting it grow unbounded — never by adding a conditional.

### 4.4 Unit layers

`cloudflared_proc` spawns the real stub (fast, no network). `services` uses `MockTunnelProvider` / `MockTunnelRepository` / `MockAuthService` / `TestDbService` plus mockito for REST/OAuth/GitHub, and **never spawns a process** (`crates/CLAUDE.md` layer rule). `routes_app` uses `build_test_router()` for the 401/403 tiers and `AppServiceStubBuilder` + `MockTunnelService` for handler logic, asserting `body["error"]["code"]`. UI uses vitest + MSW `handlers/tunnel.ts`, typed from the generated `TunnelStatusResponse` after `make build.ts-client`.

### 4.5 CI implications

**CI is live on every commit this plan lands.** `.github/workflows/build.yml` and `playwright.yml` both trigger on `push: branches: [main, working], paths: ['crates/**','xtask/**']` (plus `pull_request` to `main`), with no disabling gate: `build.yml` runs `build-and-test` (`:21`, Postgres services, coverage) **and** a second `playwright-tests` job (`:119`, `npm run test:playwright:ci`), and `playwright.yml` builds `bodhiserver_dev` and runs the same suite. Every trunk push in this plan fires both on `ubuntu-latest`.

Consequences, all first-class from Phase 1:
- The fake stub must run **on CI**, not just locally: Node ≥ 22 is present via `setup-environment`, but the committed exec bit and `tunnel_harness.rs`'s temp-`HOME` isolation have to hold on a fresh ubuntu checkout (the harness re-chmods, and falls back to `node <path>`).
- `#[serial_test::serial(live)]` tunnel tests and the throwaway-Keycloak-client harness execute against the shared `INTEG_TEST` auth server on **every push** — which is exactly why no live test may ever `PUT` the shared resource client (§4.2), and why the throwaway client's `Drop` cleanup must be leak-free.
- The `@scheduled` `grepInvert` (`crates/lib_bodhiserver/playwright.config.mjs:16,27-28`) is **load-bearing**, not tidiness: without it Phase 11's real-Cloudflare spec would run on every push.
- **No Windows CI has ever existed** (doc 23 follow-up). The Windows stop / Job-Object paths (Phase 10) have no CI signal and are covered by a manual QA checklist plus `cfg`-gated unit tests that at least compile there; the stub stays POSIX-only (no `.cmd` shim) until a Windows job exists.

Phase-0 deployment must precede the Phase-5 push, because that push runs the live SPI assertions on CI immediately.

---

## 5. Risks & open decisions

### 5.1 Open items (each with a default the owner can accept silently)

| # | Item | Default in this plan |
|---|---|---|
| 1 | Auth-tier order | **1 → 3 → 2** (§1). Rejecting it moves Phase 8 ahead of Phases 7a/7b and nothing else. |
| 2 | `route dns` on a **restricted team-member** Cloudflare account (role lacks DNS/Load Balancer) | Resolved for single-owner accounts (§0-A). For the restricted case: `dns_routed=false`, `last_error` names the exact CNAME, `mark_dns_manual` unblocks, and Phase 7's REST path automates it. |
| 3 | The §2.7 argv pin has never met a real binary in this repo | Keep doc 10 §A verbatim; the fake accepts either flag position; the first live spawn in Phase 4 validates and, if needed, corrects both. |
| 4 | Tier-3 token shape | One token with two policies (zone-scoped link prefilled, account policy added via "+ Add policy"). Alternative: two tokens, two encrypted triplets, one extra migration. |
| 5 | Reusing an existing `~/.cloudflared/cert.pem` | Offer it explicitly, never silently; never move or rewrite it. |
| 6 | Nav visibility when the gate is off | Nav shown to admins, page renders an unavailable state. The alternative (`features.tunnel` on `/info`) publicly reveals native-ness. |
| 7 | `test-id.getbodhi.app` is separately managed with an undocumented deploy path | Phase 0 exits only on an explicit 401 probe plus an authenticated 200 from **both** hosts. |
| 8 | Windows: two cert-path candidates, no CI, no SIGTERM before Phase 10 | Probe both candidates; hard `kill()` until Phase 10; manual Windows QA before release. |
| 9 | Port-80/443 elision changes today's non-explicit callback output | Accept (no back-compat outside the DB); `setup_create` and `auth_initiate` share the resolver so they cannot drift. |
| 10 | Tier-2 one-way door; end-to-end grant unverified; loopback redirect acceptance unknown | Phase 8 is blocked on an explicit owner go plus a throwaway-account spike. |
| 11 | Anonymous `/info` reveals the tunnel hostname and `url` | Accept (doc 21 follow-up §1: already public via DNS + CT; only `url` + `kind` exposed, never auth state). |
| 12 | `tunnel login` can hold a child for ~10 minutes | Held in `LiveState`; `on_shutdown` kills it; a second `POST /tunnel/login` returns `tunnel_error-login_in_progress`. |
| 13 | Two app instances on one machine | `portpicker` per spawn; the PID file is per `BODHI_HOME`. |
| 14 | 524 after ~100 s non-streaming; Bot Fight Mode blocks SDK clients | Consent + rail copy only (Phases 4, 10). No request rewriting. |
| 15 | Downloaded macOS binary signing / quarantine | Phase 9 runs `codesign -dv` and explains rather than silently stripping quarantine. |
| 16 | Vite HMR websocket bypasses the tunnel (doc 21 §5) | Accept; `app.run.live` only. |
| 17 | Multi-tenant rows | Never created (gate off, routes 403); no RLS suite while MT is out of scope. |

### 5.2 Risk → phase → pinning test

| Risk | Retired in | Pinned by |
|---|---|---|
| `redirect_uri` composed as `http://host:1135` through the tunnel (doc 22 §1.1) | P2 / P5 | `test_auth_initiate_callback_from_forwarded_https_host`; `test_live_login_initiate_through_tunnel_host_keycloak_accepts_redirect_uri` |
| Canonical 301 bounces tunnel visitors (doc 21 §4.1) | P2 hook, P4 live | `test_canonical_redirect_skips_active_tunnel_host`; `test_live_tunnel_request_with_tunnel_host_not_redirected` |
| Global cookie `Secure` flag (doc 21 §4.4) | P2 | `test_session_cookie_secure_only_on_https_origin`; `test_session_cookie_secure_floor_kept_when_public_scheme_https` |
| `/info` advertises a tunnel that is not actually up (invariant 1) | P4 | `test_active_public_url_only_when_enabled_and_connected`; `test_live_tunnel_info_origins_lists_tunnel_only_when_connected` |
| Orphaned `cloudflared` on three OSes | P4 (Drop + composite shutdown), P6 (PID reap), P10 (PDEATHSIG / Job Object) | `test_run_drop_kills_child`; `test_tunnel_child_dies_with_graceful_shutdown`; `test_live_tunnel_startup_reaps_orphan_connector`; `test_pdeathsig_kills_child_when_parent_sigkilled` |
| SPI version skew (doc 22 §5) | P0 live, P5 degrade | `RedirectUrisEndpointTest`; `test_update_redirect_uris_404_is_ok_none`; `test_sync_404_sets_spi_outdated_not_error` |
| Full-replace sync drops URIs BodhiApp cannot recompute | P0 (`GET`), P2 (`desired_set`) | `test_desired_set_preserves_unknown_existing_uris`; `testSetRedirectUrisKeepsWebOriginsPlus` |
| Full-replace `PUT` clobbers the **shared** Keycloak test client | P5 (harness) | `test_live_tunnel_enable_registers_tunnel_redirect_uri` runs against a throwaway client and asserts the loopback entries survive |
| `tunnel create` is not idempotent → remote orphan | P3 | `test_tunnel_service_provision_checks_list_before_create`; `test_tunnel_service_provision_persists_ids_before_route_dns` |
| Tier-3 token on argv / in logs / at rest | P7a / P7b | `test_start_connector_tier3_uses_env_not_argv`; `test_status_never_serializes_token`; `test_set_api_token_stores_v2_ciphertext_only` |
| Public exposure without consent; no kill switch | P4 consent, P6 gate | E2E `enable requires explicit consent`; `test_startup_does_not_resume_when_gate_off` |
| Detect spawns `cloudflared --version` on every poll | P1 | `test_tunnel_service_detect_cached_within_60s` |

---

## 6. Non-goals / deferred

Quick tunnels (`trycloudflare`), Tailscale, frp — locked out. Cloudflare Access / service tokens fronting the app (rail copy may point at it as optional hardening). Exposing a subset of paths. SSE status streaming. `cloudflared service install` (Windows service / systemd / launchd) — the connector lives and dies with the app. Survive-reboot. HA / multiple connectors. Multi-tenant tunnels and a `tunnels` RLS suite. RP-initiated logout (`post.logout.redirect.uris`, doc 22 §4.2). `AppInfo.features` for hiding the nav entry on containers. Automatic `cloudflared` self-update (`--no-autoupdate` always). Automating "Always Use HTTPS" via the zone API. `CF-Connecting-IP` audit logging. Enterprise `proxy_read_timeout` tuning. Enforcing streaming for tunnel-host requests (copy only). QR-code / passcode gate (doc 15 §5). Migrating a tier-1 locally-managed tunnel to remotely-managed config (doc 12 §4) — possible later with no new lifecycle code. Reusing `cert.pem`'s embedded API token for REST calls (doc 11 §2) — keeps tier 1 pure-CLI today. Vite HMR through the tunnel (dev-only). A Windows `.cmd` stub shim, until a Windows CI job exists.
