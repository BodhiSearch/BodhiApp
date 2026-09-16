# Public reachability discovery on `/bodhi/v1/info`

> **Filing note:** plan-mode generated this path from the *previous* session's slug. On implementation,
> move this to `docs/claude-plans/202609/remote-access-info-discovery.md` and add it to
> `docs/claude-plans/202609/index.md` (dropping the stale `202607` entry), per the claude-plans rules.

## Context

BodhiApp's USP is consented, OAuth-scoped third-party access to a user's LLM APIs. Today that works
only through Chrome's Local Network Access: a third-party *website* can reach the user's
`localhost:11135`. But those apps often need to call the user's instance from **their own backend**,
which needs a publicly-routable URL. The just-shipped Remote Access feature (Cloudflare named tunnel,
`docs/claude-plans/202609/remote-access-plan.md`) creates that URL — but nothing advertises it.

So a third-party app needs one anonymous probe answering: *"can I reach this instance from my backend,
and at what address?"* `/bodhi/v1/info` is the only anonymous instance-identity endpoint, so it is the
natural and only place. Two facts are missing from it: whether the reported `url` is itself public
(`cloud.getbodhi.app` is, `http://0.0.0.0:11135` is not), and whether a tunnel exists and works.

### Is this safe to expose anonymously?

Mostly yes, with one caveat to accept consciously and one pre-existing issue worth a follow-up.

- **The tunnel URL is not a new secret.** Once live, anyone who can reach it already knows it.
- **Real new disclosure:** a site granted LNA to `localhost` learns a *stable, public,
  machine-identifying hostname* — a cross-site correlation vector. Inherent to the feature (the app
  needs the address), not to this endpoint, but it is a real change.
- **The genuine leak risk is the detail we drop.** `TunnelStatus` carries `error_message` (raw
  `cloudflared` stderr, can contain home-directory paths), `binary.path`, `login.cert_path`,
  `login.zone`, `subdomain`, `oauth_redirect_uri`. **None** may cross into `/info`. Verified: the new
  DTO structurally has no field able to carry them, so exclusion is enforced *by construction*, not
  by convention — a test pins it anyway.
- **Pre-existing, out of scope:** `/info` already publishes `version`, `commit_sha`, `client_id`
  anonymously. Low-risk on a LAN-only instance; with a live tunnel it becomes an exact build
  fingerprint on the open internet, helping someone target a known CVE in a specific build.

### Cost on an anonymous endpoint

`/bodhi/v1/info` is anonymous, hit on every bootstrap, and — because of this feature — reachable from
the open internet. So **it must never call `TunnelService::status()`**: `build_status()`
(`service.rs:954`) spawns `cloudflared --version` as a subprocess (30s TTL) and makes an outbound
Cloudflare REST call to resolve the zone (300s TTL). Fine behind the admin-only `GET /bodhi/v1/tunnel`;
not fine where unauthenticated traffic can drive it.

The new path adds **no subprocess, no outbound third-party call, and no lock contention** (verified:
`runtime` is a `std::sync::Mutex` whose guards are dropped before every `await`; long operations are
serialized by a separate tokio `operation` mutex this path never touches). It does add **two SQLite
point-lookups** (`tunnel_enabled()`, `get_setting(BODHI_TUNNEL_HOST)`) — marginal against the five the
handler already performs, but stated rather than implied away.

## Contract

Both fields are additive and optional, so no existing consumer breaks.

```jsonc
// local dev, tunnel live
{ "version": "0.0.61-dev", "commit_sha": "...", "status": "ready",
  "deployment": "standalone", "client_id": "bodhi-resource-...",
  "url": "http://0.0.0.0:11135",
  "url_public": false,
  "remote_access": [ { "provider": "cloudflared",
                       "url": "https://my-tunnel.bodhi.bot",
                       "enabled": true, "status": "ready", "auth_status": "ready" } ],
  "reference_api_url": "https://dev-api.getbodhi.app" }

// cloud.getbodhi.app — no tunnel, so the key is omitted entirely
{ ..., "url": "https://cloud.getbodhi.app", "url_public": true }

// tunnel configured but turned off — status/auth_status omitted, not null
{ ..., "url_public": false,
  "remote_access": [ { "provider": "cloudflared", "url": "https://my-tunnel.bodhi.bot",
                       "enabled": false } ] }
```

Consumer: `const base = info.url_public ? info.url : info.remote_access?.find(r => r.status === 'ready')?.url`

**Field semantics** — these go verbatim into the OpenAPI descriptions; this is a third-party contract:

| Field | Meaning |
|---|---|
| `url_public` | The `url` above is reachable from the public internet. Deployment-declared; defaults `false`. |
| `remote_access` | Omitted when absent. Array is forward-compat: the backend models exactly one tunnel (D10), so it holds 0 or 1 entries today. Present only when the tunnel feature is available **and** a hostname is configured. |
| `provider` | `"cloudflared"`. Enum, not a bare string, so it can grow. |
| `url` | `https://{BODHI_TUNNEL_HOST}`. Always present on an entry. |
| `enabled` | **A connector process is running right now** — `runtime.running.is_some()` (`service.rs:986`). Not "the user switched it on". True while connecting. |
| `status` | `ready` \| `error` \| *omitted*. **`error_code.is_some()` ⇒ `error`, checked first**; else `Connected`→`ready`, `Failed`→`error`, `Connecting`/`Disabled`→omitted. |
| `auth_status` | `ready` \| `error` \| *omitted*. `Synced`→`ready`, `Unreachable`/`Rejected`→`error`, `NotAttempted`/`Syncing`→omitted. |

The `error_code` precedence exists because the same condition otherwise maps two ways: a DNS conflict
raised by `enable()` records the code **without moving state** (`service.rs:1322-1329`, leaving
`Disabled`), while the same conflict via `reconnect()` goes through `record_runtime_failure` and lands
on `Failed` (`service.rs:1490-1497`). Without the precedence rule, the common UI-driven case would be
byte-identical to "user turned it off". Only the code's **presence** is read, never its value — so
nothing leaks. This mirrors what the frontend's `deriveRemoteAccessState` already does.

Both collapses must be written as **exhaustive `match`es with no `_` wildcard**, so a future enum
variant forces a compile error rather than silently becoming "omitted".

## Phase 1 — `url_public`

Independently shippable; covers the cloud case alone.

**New setting `BODHI_PUBLIC_URL_REACHABLE`**, following the `BODHI_PUBLIC_*` family and the
`BODHI_CANONICAL_REDIRECT` boolean precedent. Env-var only, absent ⇒ `false`; nothing infers it.

- `crates/services/src/settings/constants.rs` — declare beside `BODHI_CANONICAL_REDIRECT` (~line 21),
  **and add to `SETTING_VARS` (lines 92-119)** or it is neither listed nor persistable.
- `crates/services/src/settings/default_service.rs` — `SettingMetadata::Boolean` in
  `setting_metadata_static` (lines 42-66); `Value::Bool(false)` in `build_all_defaults`.
- `crates/services/src/settings/setting_service.rs` — add `url_public() -> bool`, mirroring
  `tunnel_enabled()` (lines 187-206) for the `Value::Bool` / `"true"`-string / env-fallback dance,
  with a plain `false` terminal default.
- Do **not** add to `EDIT_SETTINGS_ALLOWED`. `BODHI_TUNNEL_AUTO_RECONNECT` (`constants.rs:119`) is the
  exact precedent: in `SETTING_VARS`, absent from the allowlist, therefore deployment-only and
  write-protected at `routes_settings.rs:111,184`. No UI work.
- `crates/routes_app/src/setup/setup_api_schemas.rs` — `url_public: bool` on `AppInfo`;
  `routes_setup.rs:30-88` — populate from `settings.url_public().await`.
- `devops/.env.example` — document beside the existing `BODHI_PUBLIC_HOST`/`PORT` entries.

## Phase 2 — `remote_access`

### 2a. Fix readiness self-healing (prerequisite — without it Phase 2b reports wrong data)

`refresh_status()` (`service.rs:1183-1225`) is the **only** code that promotes `Connecting → Connected`,
and its only caller is `status()` — reached only from the admin route and admin-triggered mutations.
The existing supervisor (`supervise_tick`, `service.rs:625-652`, `SUPERVISOR_INTERVAL` 1s) only calls
`child.try_wait()`, so it catches process death but never readiness. Result today: on an unattended
instance, a fully-connected tunnel stays at `Connecting` indefinitely.

- Extract the loopback `/ready` probe out of `refresh_status()` into a shared helper, and call it from
  `supervise_tick` **only while a connector is live**, throttled to roughly every 5s (not every tick).
- Respect the existing `runtime.generation` guard so a tick belonging to a previous connector cannot
  clobber a newer one.
- This also improves the admin page (failure and readiness surface without a reload) and is directly
  relevant to outstanding owner-check **O4**.
- With this in place, `remote_access_info()` reads runtime state from memory only — the no-subprocess,
  no-outbound-call property becomes strictly true, and staleness is genuinely bounded at ~5s.

### 2b. The summary itself

**Types** in `crates/services/src/tunnels/tunnel_objs.rs`, beside `TunnelStatus` (services is upstream
of routes_app, and `TunnelStatus` sets this precedent). All three derive `ToSchema`:

```rust
pub struct RemoteAccessInfo { provider: RemoteAccessProvider, url: String, enabled: bool,
                              status: Option<RemoteAccessState>, auth_status: Option<RemoteAccessState> }
pub enum RemoteAccessProvider { Cloudflared }      // serde snake_case
pub enum RemoteAccessState { Ready, Error }        // serde snake_case
```
Both `Option` fields carry `#[serde(skip_serializing_if = "Option::is_none")]`, mirroring `client_id`
in `AppInfo`.

**Service method** — `remote_access_info(&self) -> Option<RemoteAccessInfo>`. It must be declared on
the **`TunnelService` trait** (not just the impl), since `setup_show` calls it through
`Arc<dyn TunnelService>`; that extends the `mockall::automock` surface, so existing
`MockTunnelService`-based tests need the new expectation or a default. Returns `Option`, never
`Result`, so `/info` can never 500 because of the tunnel; a poisoned runtime mutex yields `None`.
Reads **only** `settings.tunnel_enabled()`, `settings.get_setting(BODHI_TUNNEL_HOST)` and
`self.runtime()?` (`state`, `error_code.is_some()`, `auth_sync.state`, `running.is_some()`). Must not
call `binary_status()`, `login_status()`, `refresh_status()` or `build_status()`. `None` when the
feature is unavailable or no hostname is set.

**Wiring** — `AppInfo` gains `remote_access: Option<Vec<RemoteAccessInfo>>` with
`skip_serializing_if = "Option::is_none"`; `setup_show` calls
`auth_scope.tunnels().remote_access_info()` (`tunnels()` is unconditional on `AuthScopedAppService`,
`auth_scoped.rs:106`, verified reachable for anonymous callers with no panic path) and wraps `Some(x)`
as `Some(vec![x])`.

## Generated artifacts & frontend

- `make build.ts-client` (**not** the sub-scripts — they leave `dist/` stale and typecheck fails).
  `Option<Vec<T>>` + `skip_serializing_if` is an exercised shape (e.g. `mcps_api_schemas.rs:91,108`).
- `crates/bodhi/src/test-utils/msw-v2/handlers/info.ts` — `mockAppInfo()` needs a `url_public: false`
  default; `remote_access` can stay absent. Both fields optional, so the dozens of component tests
  that stub `/info` incidentally keep passing untouched.
- **No UI consumer.** Nothing in `crates/bodhi/src/` reads these yet; they exist for third parties.

## Tests

| Layer | File | What |
|---|---|---|
| services — settings | `settings/test_setting_service.rs` | `url_public()`: default `false`; env `"true"`/`"false"`; `Value::Bool`; garbage string ⇒ `false`. |
| services — supervisor | `tunnels/test_tunnel_service.rs` | **Blocker-1 regression:** with `FakeCloudflared` connected, assert state reaches `Connected` via supervisor ticks alone, with **no** `status()`/admin call. Assert a stale-generation tick cannot clobber a newer connector. |
| services — tunnel | `tunnels/test_tunnel_service.rs` | `remote_access_info()` ⇒ `None` when feature unavailable and when no hostname. Table-driven over every `TunnelConnectionState` × `TunnelAuthSyncState` pair. **Blocker-2 regression:** `dns_conflict` ⇒ `status: error` on *both* the `enable()` and `reconnect()` paths. **Guard: `FakeCloudflared::calls()` is empty afterwards** — this pins the whole no-subprocess premise. |
| routes_app | `setup/test_setup.rs` | Extend the 7 existing `test_app_info_*` cases for `url_public`. New cases with `MockTunnelService` for present/absent `remote_access`. **Leak-guard: serialize the response, assert the JSON contains none of `error_message`, `error_code`, `cert_path`, `binary`, `zone`, `subdomain`, `oauth_redirect_uri`.** |
| routes_app — OpenAPI | `shared/test_openapi.rs` | `test_app_info_endpoint` still passes; new schemas land in the spec. |
| server_app integration | `tests/test_live_multi_tenant.rs` | Existing raw-JSON `/info` assertions (lines 88-96, 204-214) gain `url_public == false` and `remote_access` key absent on multi-tenant. |
| E2E | — | **None, deliberately.** No UI surface consumes these fields; asserting them needs a raw `page.request.get`, which violates the black-box E2E rule. The pre-existing `/info` fetch in `multi-tenant-lifecycle.spec.mjs:74-78` is left alone. |

Gates per the layered methodology: `cargo test -p services` → `-p routes_app` → `-p server_app` →
`make test.backend`, then `make build.ts-client`, `cd crates/bodhi && npm test`, `make format`.

Known-unrelated failures, do not chase: `test_list_model_aliases` (missing Python `gguf` locally) and
the three MCP frontend test files (26 tests) — both verified failing independent of this work.

## Deployment action required

`cloud.getbodhi.app` reports `url_public: false` until `BODHI_PUBLIC_URL_REACHABLE=true` is set on it.
**No in-repo file configures that host** — no fly.toml, k8s manifest or workflow sets `BODHI_PUBLIC_*`;
its runtime env lives in the hosting platform or an infra repo not present here. `devops/.env.example`
is the only in-repo place to document the convention. Owner action, outside this change.

## Verification

1. `make app.run.live`, then `curl -s localhost:11135/bodhi/v1/info | jq` → `url_public: false`, no
   `remote_access` key (tunnel off).
2. `BODHI_PUBLIC_URL_REACHABLE=true make app.run.live` → `url_public: true`.
3. Enable the tunnel from `http://localhost:11135/ui/tunnels/`, re-curl → one entry, `status: "ready"`,
   `auth_status: "ready"`.
4. **Blocker-1 check:** restart the app with auto-reconnect on, **never open the tunnel page**, wait
   ~10s, curl → `status: "ready"`. (Before 2a this would report the key with no `status`.)
5. **The point of the whole design:** with the tunnel live, hit `/info` ~50× in a loop and confirm zero
   `cloudflared` subprocesses spawn (`ps`) and zero Cloudflare API calls. Contrast `GET /bodhi/v1/tunnel`,
   which legitimately does both.
6. Kill the connector out-of-band → `enabled: false`, `status: "error"` (supervisor catches it).
7. Trigger a DNS conflict from the UI → `status: "error"`, **not** an off-looking entry.
8. Turn the tunnel off → entry persists, `enabled: false`, no `status`/`auth_status` keys.
9. Confirm the full payload still contains no path, zone, subdomain or error text.

## Follow-ups (not in scope)

- **"Unavailable" vs "available but unconfigured"** both omit the key, so a third-party onboarding flow
  cannot distinguish "this build can't do Remote Access" from "it can — tell the user to turn it on".
  The primary question ("can I reach it now") is unaffected.
- **RunPod inconsistency.** `on_runpod_enabled()` already forces `https` + a public
  `*.proxy.runpod.net` host, so RunPod instances are genuinely public yet report `url_public: false`.
  Deriving it would contradict the env-var-only decision, so it is left alone — but it is a real gap.
- **Anonymous build fingerprint.** `version` + `commit_sha` on a now-internet-reachable `/info`.
- All three belong in `docs/claude-plans/techdebt.md` under `# Remote Access`.
