# Remote Access (Cloudflare Tunnel) — Slice: status caching, supervision, error taxonomy, reconnect, Keycloak sync, security

Design only — no code changes made. Targets `crates/services/src/tunnels/service.rs` (current: 1220 lines), `crates/services/src/tunnels/tunnel_objs.rs`, `crates/routes_app/src/tunnels/routes_tunnels.rs`, `crates/server_app/src/serve.rs`. Builds on ground truth B1-B17 and locked decisions D1-D6 from the orchestrating prompt; does not re-derive them.

---

## 1. Status caching (B5)

### What is expensive today, and why cache only these two things

`build_status()` (service.rs:676) itself does no I/O beyond in-memory setting reads — cheap, don't touch. The expensive calls it triggers via `binary_status()` (233) and `login_status()` -> `zone_name()` (355) are:

- `binary_status()`: spawns `cloudflared --version` (`tokio::task::spawn_blocking` + `Command::output()`) — a process spawn on every call.
- `zone_name()`: an authenticated GET to `api.cloudflare.com/client/v4/zones/{id}` — a network round trip on every call.

`login_status()` (379) also does a local file stat + read + base64/JSON parse of the cert (`configured_cert`, `read_origin_cert`) — this is µs-cheap local I/O and must stay live every call so a corrupted/replaced cert file is detected immediately; only the network part (`zone_name`) is cached.

Everything else `status()` -> `build_status()` reads (`runtime.state`, `error_code`/`error_message`, `auth_sync`, `hostname`/`subdomain`/`public_url`/`oauth_redirect_uri`, `auto_reconnect`) is an in-memory setting or the live `runtime` mutex — no caching needed, must stay live.

### Cache shape

Add one new field to `DefaultTunnelService` (service.rs:133), a sibling to `runtime: Mutex<TunnelRuntime>`, using the same lock type for consistency (`std::sync::Mutex`, never held across an `.await`):

```rust
struct BinaryProbe { fingerprint: u64, at: Instant, status: TunnelBinaryStatus }
struct ZoneProbe { fingerprint: u64, at: Instant, outcome: std::result::Result<String, String> }

#[derive(Default)]
struct StatusCache { binary: Option<BinaryProbe>, zone: Option<ZoneProbe> }
```

`status_cache: Mutex<StatusCache>` — a **separate** mutex from `runtime`. Neither code path ever needs both locks at once (cache lookups happen inside `binary_status`/`login_status`, which are called *before* `build_status()` takes the `runtime` lock at service.rs:696), so there is no lock-ordering hazard and no reason to widen the `runtime` critical section with cache bookkeeping.

**Keying — fingerprint, not a manual invalidation flag.** `BinaryProbe.fingerprint` = a hash (`std::hash::Hasher`, e.g. `DefaultHasher`) of the resolved `(path, source)` candidate — computed the same cheap way `binary_status()` already resolves a candidate today (config -> `PATH` -> standard locations), just before the `--version` spawn. `ZoneProbe.fingerprint` = hash of `(cert.zone_id, cert.api_token)` from the already-parsed `OriginCertificate`. Content-based keys mean a changed `cloudflared_path` or a replaced cert file *is itself* a cache miss — no explicit "invalidate on setup PUT" code path is needed, which is also less code (D5).

**TTLs:**

| Cache | TTL | Rationale |
|---|---|---|
| `BinaryProbe` | 30s | Binary rarely changes; 30s bounds the worst-case staleness of a version/readiness display without spawning a process every 1s poll. |
| `ZoneProbe` | 300s (5 min) | Zone name for a fixed cert is effectively immutable; this is a paid network call to Cloudflare, so amortize hard. A wrong cached zone (only possible if Cloudflare renamed the zone under the same id, vanishingly rare) self-heals within 5 minutes with no user-visible harm — `subdomain` derivation and `enable()`'s hostname construction both tolerate a stale-but-still-correct zone string. |

### Where each cache is consulted

- `binary_status()` (233): after resolving the candidate path/source (no cache needed for that resolution itself — it's already just `is_file()` checks), compute the fingerprint, lock `status_cache`, and return `probe.status.clone()` if `fingerprint` matches and `at.elapsed() < 30s`. On miss, drop the lock, run the existing `--version` spawn unchanged, then re-lock and store.
- `login_status()` (379): after `read_origin_cert` succeeds, compute the zone fingerprint and check `status_cache.zone` the same way. On a fresh hit, build `TunnelLoginStatus { state: Ready, ..., zone: Some(cached_zone) }` (or the cached `Err` branch) directly, skipping `zone_name()` entirely. On miss, call `zone_name(&cert).await` unchanged, store the `Result<String,String>` outcome (both Ok and Err are cached — a still-invalid cert should not be re-hit every 1s either), then build the status from it.
- `setup()` (~line 823 onward, `TunnelSetupRequest` branch that validates `origin_cert_path`): already calls `self.zone_name(&certificate).await` directly to validate the new cert before persisting the setting. Opportunistically also **write** that `Ok` result into `status_cache.zone` (with the new cert's fingerprint) before returning, so the `self.status().await` call at the end of `setup()` doesn't immediately re-hit Cloudflare in the same request. This is the only place `setup()` needs to touch the cache — no explicit "clear" call anywhere.
- `enable()` (953): its `self.login_status(true).await` call (~961) automatically benefits once caching lives inside `login_status()` — no separate change needed there.

### What must stay live (never cached)

`runtime.state`, `/ready` HTTP probe result (via `refresh_status()`, see §2 below), `error_code`/`error_message`, `auth_sync`, and every plain settings read in `build_status()`. These are all either already O(1) in-memory reads or the one thing (`/ready`) that genuinely needs freshness — caching it would directly lie to the UI about connector health.

### Tests

- `services` unit: `binary_status()` called twice within the TTL window spawns `cloudflared --version` exactly once (use a fake-`cloudflared` script fixture on `PATH`/`BODHI_TUNNEL_CLOUDFLARED_PATH` that appends to a counter file each invocation; assert the counter is 1 after 2 calls, 2 after a 3rd call made past a fake-clock TTL boundary — inject time via a small `Clock` trait or `tokio::time::pause()` + `advance()` if the cache uses `tokio::time::Instant`, otherwise structure the TTL check to take an injectable "now" for testability).
- `services` unit: `login_status()` called twice with an unchanged cert hits the mockito Cloudflare zone endpoint exactly once (extend `mock_server.assert()` pattern from `crates/services/src/auth/test_auth_service.rs`).
- `services` unit: swapping `BODHI_TUNNEL_CLOUDFLARED_PATH` between two calls busts the binary cache (2 distinct fingerprints -> 2 spawns) without any explicit invalidation call.
- `services` unit: swapping the cert file's `zoneID`/`apiToken` between two calls busts the zone cache the same way.
- `services` unit: a cached `Err` outcome (invalid cert) is also not re-fetched inside the TTL — assert the mock zone endpoint is hit once even when the cached result is an error.
- `routes_app`: `GET /bodhi/v1/tunnel` polled twice in quick succession returns the same `binary.version`/`login.zone` without a second `cloudflared`/Cloudflare call (integration-level confirmation of the unit behavior, using the router-level test harness with a fake binary + mockito).

---

## 2. Connector supervision (B6)

**Decision: keep poll-driven. Do not add a background watcher task.**

Reasons:

1. **The existing supervision primitive already does the hard part out-of-band.** The Unix `/bin/sh` liveness-pipe wrapper (service.rs:1013-1030, spawned inside `enable()`) is *itself* a lightweight supervisor: it holds a `UnixStream` write end (`RunningTunnel.liveness`, service.rs ~90-95) whose only job is to guarantee that closing it (in `stop_running`, 647) or the whole process dying reliably reaps `cloudflared` via the shell's `trap ... EXIT`. That already solves the leak/orphan problem (B7's real risk) independent of whether status-polling is push or pull.
2. **A background watcher adds a second async task with its own lifecycle to manage against shutdown** — it would need to be spawned once (where? `DefaultTunnelService::new`/`with_auth` are sync constructors, so it would need a `tokio::spawn` there or a lazy-start on first `enable()`), and reliably stopped on `disable()`/`Drop`/process shutdown (`ShutdownRuntimeCallback` in `crates/server_app/src/serve.rs:39-49`, which already calls `tunnel_service().disable()`). That is meaningfully more moving parts (a `JoinHandle` field, a cancellation channel, `Drop` ordering with the runtime mutex) for a UI that already polls this exact information every 1s.
3. **Worst-case staleness under poll-driven is bounded by the UI's own poll interval, which is already tighter than any watcher tick would sanely be.** `refresh_status()` (841) is invoked at the top of every `status()` call (905); the frontend polls at 1s while `connecting` and 10s otherwise (per B5). So:
 - **Connecting -> Connected/Failed transition**: detected within the current poll's own execution (≤1s from the actual event, bounded by network latency of the `/ready` GET plus `try_wait()`, both sub-100ms locally).
 - **Connected -> Failed (process exit) while idle** (no active poll, e.g. the admin closed the tab or the state was already `Connected` so the frontend backed off to 10s): worst case **10s** staleness before the next poll's `try_wait()` catches the exit code and flips `runtime.state` to `Failed` with `error_code = "cloudflared_exited"` (852).
 - This is acceptable: the tunnel being down for up to 10s before the UI shows it is a display-latency issue, not a correctness one — the connector really is dead in both cases, `disable()`/re-`enable()` remain correct at any time, and no other part of the app (API-key traffic, local UI) depends on this status being real-time (unlike the `/ready` check itself, which is fetched fresh every poll while a process is running, not cached — see §1).
4. **If polling is ever fully stopped** (e.g., the admin never opens the Remote Access page again after enabling), a dead connector goes undetected until the next `status()` call — but this is already true today and is not made worse by anything in this slice. If this is judged unacceptable in a future iteration, the fix is a low-frequency (~30s) `tokio::spawn`ed self-check loop *only while a connector is running*, started at the end of `enable()`'s success path and cancelled by dropping a `oneshot::Sender` held in `RunningTunnel` alongside `liveness` (checked in `stop_running`) — but that is out of scope for this slice per the analysis above.

### Tests

- `services` unit: after a fake-`cloudflared` process (a tiny shell/Rust fixture binary controllable to exit on demand) exits, the *next* `status()` call transitions `state` to `Failed` with `error_code == "cloudflared_exited"` and clears `running` — a call *before* the exit still reports `Connecting`/`Connected`. This proves the poll-driven boundary precisely.
- `services` unit: two consecutive `status()` calls after a live process's `/ready` starts responding 200 transition `Connecting` -> `Connected` on the call made after readiness flips, not before (mock the metrics HTTP endpoint with a local `TcpListener` + minimal handler, or reuse whatever fake-cloudflared harness is built for B16's gap).
- `server_app` integration: start the real server with `BODHI_TUNNEL_AUTO_RECONNECT` fixture + a fake-cloudflared binary, kill the child out-of-band (e.g., via a fixture-exposed control socket), assert a subsequent `GET /bodhi/v1/tunnel` (multi-turn, real HTTP) reflects `cloudflared_exited` — this is the multi-turn case the testing-philosophy doc requires beyond a single `routes_app` `oneshot()`.

---

## 3. Error taxonomy (B10)

### Contract choice

Keep **`TunnelStatus.error_code` as the single channel** the frontend switches on (do not also thread new `TunnelError` variants through `errmeta_derive`/`BodhiErrorResponse.error.code` for these). Reasoning: `TunnelError` (service.rs:22-49) already models *request-time, single-call* failures returned synchronously from `enable`/`setup`/etc. (`Disabled`, `InvalidHostname`, `DnsConflict`, `MissingBinary`, `Command`, `Provisioning`, `RuntimePoisoned`) — these map naturally to HTTP error responses via `impl<T: AppError> From<T> for BodhiErrorResponse` (`crates/routes_app/src/shared/api_error.rs:96-108`), and `errmeta_derive` already gives each variant a stable `tunnel_error-<variant_snake_case>` code (see `crates/errmeta_derive/src/generate.rs:144-145`) which is exactly right for those.

But the codes this task must fix (`dns_conflict`, `cloudflared_exited`, `startup_reconnect_failed`) are not request-time failures — they are **background/runtime state** set on `TunnelRuntime` and surfaced through polling `GET /tunnel`, which always returns `200` with a `TunnelStatus` body (routes_tunnels.rs:10-22), never a `BodhiErrorResponse`. There is no HTTP error response to attach an `errmeta` code to for these. So: widen and freeze the *string* vocabulary already used for `TunnelStatus.error_code`, keep it a plain `Option<String>` (no new enum type on the wire — `TunnelStatus` is a 3rd-party-facing DTO per `feedback_api_versioning`, and turning `error_code` into a `ToSchema` enum now would be a breaking wire change with no benefit over a frozen string set the frontend already `match`es structurally instead of substring-matching).

Introduce one Rust-side enum purely for internal type safety when *setting* the field (eliminates typo risk across the ~6 call sites that currently write raw string literals), converted to the wire string via `Display`/`as_str()`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display)]
#[strum(serialize_all = "snake_case")]
enum TunnelErrorCode {
  DnsConflict,
  CloudflaredExited,
  BinaryMissing,
  BinaryUnsupported,
  BinaryInvalid,
  CertificateMissing,
  CertificateInvalid,
  ZoneLookupFailed,
  ProvisioningFailed,
  CredentialsRecoveryFailed,
  ReconnectZoneUnavailable,
  ReconnectHostnameStale,
}
```

(Mirrors the existing `strum::Display`/snake_case pattern already used for `EnvType`/`AppType` in `crates/services/src/settings/setting_objs.rs:6-13`.) `record_runtime_failure` (499) takes `TunnelErrorCode` instead of `&str` and does `code.to_string()` once, at the single point that writes `runtime.error_code`.

### Full code table

| Code | Set where / when | User-facing message | UI recovery action |
|---|---|---|---|
| `dns_conflict` | `enable()` (~980), when `dns_conflicts()` (725) finds a foreign CNAME and `replace_dns` was not set | "A DNS record already exists for `<hostname>`." | Open the existing "Replace DNS record?" confirmation, then retry `enable()` with `replace_dns: true`. |
| `cloudflared_exited` | `refresh_status()` (841, ~852), connector `try_wait()` returns an exit status | "The Cloudflare Tunnel connector exited unexpectedly." | Offer "Reconnect" (re-`enable()` with the saved subdomain, `replace_dns: false` — DNS is unchanged, see §4) and a link to view connector logs. |
| `binary_missing` | `enable()`/`resolved_binary()` (811) when `binary_status().state == Missing` | "cloudflared was not found. Install it or set a custom path." | Link to the Setup step / `PUT /tunnel/setup` form for `cloudflared_path`. |
| `binary_unsupported` | same call site, `state == Unsupported` | "cloudflared `<version>` is older than the minimum `<minimum_version>`." | Link to upgrade instructions; Setup form still accepts an override path. |
| `binary_invalid` | same call site, `state == Invalid` | "cloudflared did not respond to `--version`: `<error>`." | Same Setup-form link; surface the raw `binary.error` text. |
| `certificate_missing` | `enable()` -> `origin_cert()` (~805) / `login_status` `Missing` | "No Cloudflare sign-in certificate found. Run `cloudflared tunnel login`." | Link to Setup step for `origin_cert_path`, plus the exact CLI command to copy. |
| `certificate_invalid` | `login_status()` (`Invalid` from `read_origin_cert` failure) | "The configured certificate could not be read: `<error>`." | Same Setup-form link. |
| `zone_lookup_failed` | `login_status()` (`Invalid` from a `zone_name()` `Err`, i.e. Cloudflare rejected the token or the network call failed) | "Cloudflare rejected the certificate: `<error>`." | Retry button (bypasses the cache from §1 — see below); Setup-form link to re-run `cloudflared tunnel login`. |
| `provisioning_failed` | `enable()`, any `TunnelError::Provisioning` from `tunnel_id()` (543) or the `route dns` command (~995) | "Tunnel provisioning failed: `<cloudflared stderr, truncated>`." | Retry `enable()` as-is (idempotent per B3); no state changed on failure. |
| `credentials_recovery_failed` | **new**, the B2 fix path: `cloudflared tunnel token --cred-file <path> <name>` used to (re)materialize credentials for an existing tunnel fails | "Could not recover tunnel credentials: `<cloudflared stderr, truncated>`." | Retry `enable()`; if it persists, Setup-form link plus a copy-pasteable manual `cloudflared tunnel token` command as a fallback (matches D2's "copy-pasteable commands" precedent for the orphan cleanup). |
| `reconnect_zone_unavailable` | `reconnect()` (1103), when `login_status(true).zone` is `None` at startup | "Cloudflare sign-in needs attention before the tunnel can reconnect." | "Enable" button re-opens the full setup+enable flow manually; connector stays `Disabled` (not `Failed` with a stuck spinner — see below). |
| `reconnect_hostname_stale` | `reconnect()` (1103), when the saved `BODHI_TUNNEL_HOST` no longer has the current zone as a suffix (zone/account changed under the saved cert) | "The saved address `<hostname>` no longer matches the signed-in Cloudflare account." | "Reconfigure" button opens the subdomain step pre-filled with the old subdomain against the *new* zone. |

Note `reconnect_zone_unavailable` / `reconnect_hostname_stale` replace the single `startup_reconnect_failed` (B9) — see §4 for why these two specific splits and not others (e.g. a generic provisioning failure during reconnect still reuses `provisioning_failed`/`dns_conflict`/etc. from the table above, since `reconnect()` ultimately calls `enable()` and should surface the *same* code an interactive `enable()` failure would, not a generic catch-all — only the two conditions that are unique to *unattended startup* get their own codes).

**Retry semantics for `zone_lookup_failed`:** since §1 caches the zone-lookup outcome (including `Err`) for 300s, a user-initiated "Retry" after fixing their token must not wait out the TTL. Route it through `POST /tunnel/sync`? No — that endpoint is for Keycloak sync (§5), unrelated. Instead, `PUT /tunnel/setup` with the *same* `origin_cert_path` value already re-validates via a direct (uncached) `zone_name()` call in `setup()` itself (service.rs ~830-834) and — per §1 — writes a fresh cache entry as a side effect; that is already the correct "Retry" affordance and needs no new endpoint or force-bypass parameter.

### Tests

- `services` unit, one per table row: construct the triggering condition (fake binary exiting non-zero for `binary_invalid`, a mock 403 from Cloudflare for `zone_lookup_failed`, a `route dns` fixture returning nonzero for `provisioning_failed`, etc.) and assert `status().error_code == Some("<code>")` with the exact string, not a prefix/substring.
- `services` unit: `TunnelErrorCode::DnsConflict.to_string() == "dns_conflict"` etc. for the full enum (locks the wire vocabulary against accidental renames).
- `routes_app`: `PUT /bodhi/v1/tunnel` with a DNS-conflicting hostname and `replace_dns: false` returns `200` (not an HTTP error) with `TunnelStatus.error_code == "dns_conflict"` — confirms this is a status field, not a `BodhiErrorResponse`, at the wire level (guards the §3 design choice itself).
- Frontend (Vitest/MSW): a component test per code asserting the *specific* recovery affordance renders (Setup link vs Reconnect button vs Replace-DNS dialog vs Reconfigure), replacing whatever substring-matching test exists today for `"dns record already exists"`.

---

## 4. Reconnect (B9)

### Policy given D2 (reuse + retarget, never delete)

**Yes — if the DNS record for the saved hostname already points at our own tunnel, `reconnect()` must proceed without treating it as a conflict**, and this is already almost true today by construction, not by an explicit check: `reconnect()` (1103) hardcodes `replace_dns: false` and calls `enable()`, whose `dns_conflicts()` (725) check (§B4) only flags a conflict when `record.content` differs from `"<our tunnel_id>.cfargotunnel.com"` (line ~749, `!record.content.eq_ignore_ascii_case(&expected)`). Since `tunnel_id()` (543) resolves to the *same* stable tunnel (D1's fixed name means the same Cloudflare tunnel every time), an unchanged saved hostname's DNS record — if it was ever successfully routed by a prior `enable()` — already points at our own tunnel and produces `dns_conflicts() == false`. So **no new check needs to be added for the steady-state "same hostname, our own tunnel" case; the existing `dns_conflicts()` correctly returns `false` and `replace_dns: false` is fine.**

What *does* need to change is what happens **before** that DNS check even runs, because `enable()`'s early zone/subdomain derivation (~961-972) can itself fail first:

1. `login_status(true).zone` is `None` (cert missing/invalid, or a genuine Cloudflare outage) — today this is folded into the same `startup_reconnect_failed` string as case 2. Split it out as `reconnect_zone_unavailable` (§3): the *tunnel itself* was never wrong, sign-in just isn't currently usable, so the recovery action ("Enable" to walk through setup again) is different from a DNS problem.
2. `hostname.strip_suffix(&format!(".{zone}"))` (1119) returns `None` — the saved hostname's suffix no longer matches the *current* zone. This means the Cloudflare account/zone under the cert changed since the hostname was saved (e.g. a different `cert.pem` was dropped in, or the account's zone was renamed) — a real configuration drift, not a transient failure. Keep this as its own `reconnect_hostname_stale` (§3), distinct from case 1, because the fix is different: case 1 needs "finish signing in", case 2 needs "pick a subdomain again against the new zone".

Both remain **one-shot, non-blocking, visible-failure** as today: `record_runtime_failure` (499) only ever sets `runtime.state = Failed` plus the code/message — it never retries, never blocks the caller, and the call site is a detached `tokio::spawn` (see below), so a reconnect failure can never delay server startup or hold up the readiness signal.

### Where it is triggered, confirmed

`crates/server_app/src/serve.rs:161-166`:
```rust
tokio::spawn(async move {
  if let Err(err) = reconnect_service.tunnel_service().reconnect().await {
    tracing::warn!(err = ?err, "Cloudflare Tunnel startup reconnect failed");
  }
});
```
This runs *after* `ready_rx.await` succeeds (i.e., after the HTTP server is already accepting connections and the readiness signal has fired) and is a fire-and-forget `tokio::spawn` — it cannot block `get_server_handle()`'s return, matches the "non-blocking" requirement exactly as-is. No change needed here beyond the `reconnect()` body itself producing the two new codes instead of one.

### Tests

- `services` unit: `reconnect()` with a saved hostname whose DNS already CNAMEs to the (fixed-name) tunnel's own id succeeds and does **not** set any `error_code` — proves the "reuse without conflict" path needs no `replace_dns: true`.
- `services` unit: `reconnect()` with `login_status` returning no zone (cert file deleted) sets `error_code == "reconnect_zone_unavailable"` and leaves `runtime.running` `None`.
- `services` unit: `reconnect()` with a saved hostname under a *different* zone than the current cert resolves sets `error_code == "reconnect_hostname_stale"`.
- `server_app` integration: start the server with a pre-seeded `BODHI_TUNNEL_HOST` setting and a fake-cloudflared fixture that already "owns" that DNS record; assert the server becomes ready (readiness not blocked) and a subsequent `GET /bodhi/v1/tunnel` shows `state: connected` (or `connecting`) with no error — the multi-turn proof that reconnect-on-startup doesn't regress into a conflict for the common case.

---

## 5. Keycloak sync (B11)

### Run condition: only when the redirect URI actually changed

**Decision: `sync_redirect()` should run only when the computed `redirect_uri` differs from the last one successfully synced (or never synced), not on every `enable()`.** The Keycloak PATCH itself is idempotent per `gateway` (it's a set, not an append — confirmed by the request shape `{"gateway": "cloudflared", "redirect_uri": ...}` in `AuthService::update_tunnel_redirect_uri`, `crates/services/src/auth/auth_service.rs:628-651`, which PATCHes a single `redirect_uris` resource keyed by gateway), so correctness never required a change-check — this is purely a cost/noise concern the task calls out explicitly. A reconnect to an *unchanged* hostname (the common path: server restarts, `reconnect()` re-`enable()`s the same saved subdomain) should not make a client-credentials token request plus a Keycloak PATCH on every single process start.

**Implementation:** track the last-synced URI on `TunnelRuntime` (97) alongside `auth_sync`:
```rust
struct TunnelRuntime {
  ...
  auth_sync: TunnelAuthSyncStatus,
  synced_redirect_uri: Option<String>,
}
```
In `enable()`, right before calling `self.sync_redirect(&redirect_uri).await` (~1051), compare `redirect_uri` against `runtime.synced_redirect_uri`; skip the call (leave `auth_sync` exactly as it was — do not reset it to `NotAttempted`, since it's still accurate) when equal. `sync_redirect()` itself sets `runtime.synced_redirect_uri = Some(redirect_uri.to_string())` only in the `Ok(())` branch (769-800) — a failed sync must *not* mark the URI as synced, so the next `enable()`/`sync()` retries it, matching the existing `POST /tunnel/sync` retry affordance (`sync_authorization()`, which should also update `synced_redirect_uri` on success for the same reason).

This also directly fixes the "not accumulating URIs" concern from the task: since the PATCH already replaces (not appends) the URI for the `cloudflared` gateway, there was never a literal accumulation risk server-side, but skipping redundant calls removes the *client-side* churn (a client-credentials token fetch is itself a real Keycloak round trip, not free) on every reconnect.

### `tenant_service.get_standalone_app()` returns `None` (multi-tenant deployment)

Already correctly handled as a distinct, non-retryable case in `sync_redirect()` (789): `Ok(None) => Err("No standalone authorization client is configured.".to_string())`, which flows into `auth_sync.state = Failed` with that message. This is the right *mechanism*; two refinements:

1. **State should arguably not be `Failed` for this specific case** — `Failed` implies "we tried and it didn't work, retry might help"; a multi-tenant deployment with no standalone app will *never* have this succeed, so retrying via `POST /tunnel/sync` is pointless work. However, changing `TunnelAuthSyncState` to add a third terminal variant (e.g. `Unsupported`) is a wire-schema change to a `ToSchema` enum (`tunnel_objs.rs`) with UI fallout (new state to render) for a purely cosmetic distinction — **recommend keeping `Failed`** and instead making the *message* explicit and permanent-sounding: `"Browser sign-in through this tunnel is not available in multi-tenant deployments; API-key access still works."` This keeps the DTO surface stable while telling the user not to bother retrying, without inventing a new enum variant for one deployment mode.
2. **User-facing message must not imply a bug.** Replace the current generic "No standalone authorization client is configured." (which reads like a misconfiguration) with the message above, worded as an expected limitation, in `sync_redirect()`'s `Ok(None)` branch (service.rs ~797).

### No secret reaches logs or the API response

Audited call chain: `sync_redirect()` (769) calls `auth_service.update_tunnel_redirect_uri(&tenant.client_id, &tenant.client_secret, ...)` (`auth_service.rs:628`), which internally calls `get_client_access_token` (not shown here but same pattern as other `AuthService` methods) and then does `self.client.patch(&endpoint).bearer_auth(access_token.secret())...` (642-643) — the token is only ever placed in the `Authorization` header via `bearer_auth`, never interpolated into a logged string. The two log calls in that function, `log::log_http_request("PATCH", &endpoint, "auth_service", None)` (639) and `log::log_http_error("PATCH", &endpoint, "auth_service", &message)` (664), pass `None` for the request body and a Keycloak-supplied `error`/`error_description` string for the error case respectively — neither logs the token or `client_secret`. On the `DefaultTunnelService` side, `sync_redirect()` only ever stores `error.to_string()` (the `AuthServiceError` display, or the hardcoded strings above) into `runtime.auth_sync.error`, truncated to 500 chars (800) — this is the exact same field returned in `TunnelStatus.auth_sync.error` over the API, so the only content reaching the client is Keycloak's own `error`/`error_description`, never a token or secret. **No change needed here; call this out as a confirmed-clean finding, not a gap.**

### Tests

- `services` unit (mockito, pattern from `test_auth_service.rs`): calling `enable()` twice with the same resulting hostname hits the Keycloak PATCH endpoint exactly once (`mock_server.assert()` after the second call).
- `services` unit: calling `enable()` with two *different* subdomains (hostname changes) hits the PATCH endpoint twice, once per distinct `redirect_uri`.
- `services` unit: a failed PATCH (mockito 500) leaves `synced_redirect_uri` unset, so a subsequent `enable()` with the *same* hostname retries the sync (mock hit count == 2, not 1).
- `services` unit: `tenant_service.get_standalone_app()` returning `Ok(None)` (a test-double `TenantService`) sets `auth_sync.state == Failed` with the exact non-alarming message text above, and asserts the token/secret fields never appear anywhere in `TunnelAuthSyncStatus.error` (trivially true since the branch never constructs a token) — mainly a message-content regression test.
- `routes_app`: `GET /bodhi/v1/tunnel` response body, serialized to JSON, contains no substring matching a plausible client secret/token fixture value used in the test setup — a cheap response-leak guard.

---

## 6. Security review of the whole surface

Read `docs/architecture/security.md` first (done); it has **no existing entry for the tunnel feature**, so nothing below is a re-report of an accepted risk. New findings only, with `file:line`:

1. **Credentials file has no explicit permission hardening.** `credentials_path()` (service.rs:520-524) only does `std::fs::create_dir_all` on the parent directory; the JSON credentials file itself (containing the tunnel's private key material) is written either by `cloudflared tunnel create --credentials-file <path>` (tunnel_id, ~578-591) or, once the B2 fix lands, by `cloudflared tunnel token --cred-file <path>` — in both cases file *creation* and its mode are entirely up to `cloudflared`'s own umask-relative default, not something Bodhi sets. **Finding:** add an explicit `std::fs::set_permissions(&credentials, Permissions::from_mode(0o400))` (Unix-only; Windows has no equivalent POSIX mode, ACLs would be a separate follow-up) immediately after either code path successfully writes the file, rather than trusting cloudflared's default umask behavior across platforms/installs. Low severity (same trust boundary as the rest of `BODHI_HOME`, which is already user-owned), but cheap and directly named in the task's B17 context (cloudflared >= 2022.3.0 confirmed to support `tunnel token --cred-file`, so this new write path is real, not hypothetical).
2. **`TUNNEL_ORIGIN_CERT` passed via `Command::env` — confirmed correct, no finding.** `command_output()` (526-541) sets `.env("TUNNEL_ORIGIN_CERT", cert)` where `cert` is a `PathBuf` (a file path, not the cert *contents*) — this only tells `cloudflared` where to read its own credential from; the path itself is not secret. No argv or env value here carries the `apiToken`/token material — that only ever exists in-process as the parsed `OriginCertificate` struct (56-63) used for direct `reqwest` calls with `.bearer_auth()`, never passed to a child process. Confirmed clean.
3. **argv audit — confirmed clean, no finding.** `run_args()` (622-645), the `route dns` args (~991-994), and `tunnel_id()`'s `list`/`create` args (543-608) are all built from: fixed literal flags, the locally-computed `tunnel_name(hostname)`, the validated `hostname`/`subdomain`, the credentials file path, and the local port/metrics address. None of these carry the Cloudflare API token or any Keycloak secret. `ps`/`/proc/*/cmdline` visibility of these argv values is limited to hostnames and local paths, which are not secrets. Confirmed clean.
4. **`monitor_output()` logs every cloudflared stdout/stderr line at `info` unfiltered — new finding.** service.rs:609-620 (`info!(stream, message = %line, "cloudflared")`) logs cloudflared's own operational output verbatim. cloudflared's connector logs at `--loglevel info` (set by `run_args`, service.rs ~627) do not normally print the origin cert's token or credentials-file *contents* (only the file path, via its own arg echo at startup, which is not secret) — but this has not been verified against every cloudflared version/flag combination, and a future `--loglevel debug` (if ever exposed as a setting) could change that. **Finding:** this is not exploitable today (nothing secret is currently known to appear in cloudflared's own log lines at `info`), but flag it as a forward-looking risk: if `--loglevel` ever becomes user-configurable, `monitor_output` should redact known-sensitive patterns (e.g. anything matching the cert's own `apiToken` value, held in-process) before the `info!` call, the same defense-in-depth spirit as the existing SafeReqwest scheme-only validation elsewhere in the codebase. No code change proposed for this slice; documented as a constraint for whoever adds a log-level setting later.
5. **Cloudflare API token from the origin cert is held in process memory unencrypted (`OriginCertificate.api_token: String`, service.rs:56-63) for the lifetime of every call that parses it.** This mirrors the existing accepted pattern for other in-memory secrets in the codebase (e.g. decrypted API keys held as plain `String` during a request) and is not a new class of risk — **not filed as a new finding**, consistent with "only new findings" instruction; noted here only to explain why it is excluded.
6. **`dns_conflicts()` and `zone_name()` (service.rs:355, 725) use `reqwest::Client::new()` (service.rs:143, the shared `client` field) with no explicit TLS/redirect hardening beyond reqwest's defaults.** reqwest defaults to following redirects (up to 10) and validating TLS certs — for a fixed, hardcoded `https://api.cloudflare.com` host this is low risk (no user-controlled URL, unlike the SSRF-accepted-risk AI/MCP paths in security.md), but **finding:** consider `reqwest::Client::builder().redirect(Policy::none())` for these two specific Cloudflare API calls, since a legitimate Cloudflare API response should never redirect and disabling it removes a class of exploit entirely for near-zero cost. Low severity, cheap to add, worth doing in the same change since this crate already builds one `reqwest::Client` for exactly this purpose.

### Tests

- `services` unit: after `tunnel_id()`'s create-branch or the future token-recovery path writes the credentials file, assert `fs::metadata(path).permissions().mode() & 0o777 == 0o400` (Unix-only `#[cfg(unix)]` test).
- `services` unit: snapshot-assert the full argv vector built by `run_args()` and the `route dns` command contains no substring equal to a test fixture's `apiToken`/`client_secret` value (a lightweight regression guard for finding #3, cheap to keep permanently).
- `services` unit: construct a mockito response for the zone/DNS endpoints that issues a 302 redirect and assert the call fails closed (does not silently follow it) once the `Policy::none()` change from finding #6 lands.

---

## Cross-cutting notes

- All new/changed code in `service.rs` must add **no new comments** and should **remove** any existing comment in a touched region that merely restates the code (per D5) — e.g. the `// This consumes cloudflared's operational stdout/stderr only...` comment at monitor_output (609-611) documents a genuine non-obvious safety property (it is *not* proxying request bodies) and should be **kept**; a comment that just says what the next line does should be deleted if that line is touched.
- Settings additions: none required. `TunnelRuntime.synced_redirect_uri` (§5) and the `status_cache` field (§1) are pure in-memory runtime state, not persisted settings — consistent with D6 (no DB migration needed; this feature already lives entirely in the existing settings table plus process-lifetime runtime state, and neither new field changes that).
- Regenerate `openapi.json`/ts-client only if `TunnelStatus`'s shape changes — per §3's decision, **it does not** (still `Option<String>` for `error_code`), so this slice needs **no** `cargo run --package xtask openapi` / `npm run generate` step by itself, only whatever the sibling slices (status/DTO shape, if any) already require.
