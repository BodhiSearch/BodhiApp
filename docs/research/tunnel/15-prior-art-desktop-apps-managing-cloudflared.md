# 15 — Prior art: how other apps manage `cloudflared` (2026-09-15)

Process-management and UX patterns extracted from reading the actual source of apps that spawn/manage `cloudflared` as a child process. Extends `00-consolidated-research.md` §4 (9Router) with full source detail, and adds five more projects. Read `01-bodhi-app-codebase-map.md` first for how BodhiApp's own `llama_server_proc` manages child processes — the recommended design in §7 below aligns with it.

## Survey table

| Project | Language | Tunnel kind | Auth method | Supervision | Notable pattern |
|---|---|---|---|---|---|
| [9Router](https://github.com/decolua/9router) | Node.js (Next.js) | Quick only | none (no Cloudflare login) | App-level watchdog + network monitor, PID file | Quick-tunnel URL relayed through 9Router's own stable-hostname worker |
| [Unsloth Studio](https://github.com/unslothai/unsloth/pull/8715) (`studio/`) | Python (FastAPI) + TS frontend | Quick **and** named | `cloudflared tunnel login` (cert.pem) + token-based `run` | App-level state machine, OS process-lifetime binding | Zone-mismatch self-check, DNS negative-cache handling via DoH, SNI edge-probe before DNS resolves |
| [Home Assistant Cloudflared add-on](https://github.com/brenner-tobias/addon-cloudflared) | Bash + Docker | Named (+ token shortcut) | `cloudflared tunnel login` (cert.pem) or pasted `tunnel_token` | s6-overlay (container init) restart-on-crash | `--metrics` health port; `ingress.yml` built from add-on config; connectivity pre-flight (`nc` to Cloudflare edge ports) |
| [proxypal](https://github.com/heyhuynhgiabuu/proxypal) | Rust (Tauri) | Quick and named (token) | pasted tunnel token, or none | `tokio::process` + `kill_on_drop`, manual retry loop | stderr line-classifier state machine, Tauri event emission to UI |
| [n8n-desktop](https://github.com/tangtao646/n8n-desktop) (community fork) | Rust (Tauri) | N/A (binary mgmt only, in the code found) | — | — | Binary manager split into `cache/config/download/install/models/path_resolver/platform` submodules — good module layout reference |
| [Pinokio](https://github.com/pinokiocomputer/pinokiod) "Public Node" | Node.js | Quick only | none | Generic `kernel.shell` process wrapper (used for all scripts, not tunnel-specific) | Optional **passcode gate** implemented as a local reverse-proxy pipe server in front of the tunnel, not a cloudflared feature; QR code for pairing |
| Open WebUI | — | Named (manual) | Cloudflare dashboard, manual | None — no in-app automation | Docs-only integration: user runs `cloudflared` themselves against `open-webui:8080`; no process management code in the app |
| Jan.ai (menloresearch/jan) | — | — | — | — | **Checked, not present.** `gh search code "cloudflared" repo:menloresearch/jan` returns zero hits — no tunnel integration exists |
| LM Studio | closed-source | — | — | — | No public repository; **UNVERIFIED** — no evidence of a cloudflared integration found (LM Studio's own site/community docs use manual `cloudflared` reverse-proxy guides, same as Open WebUI) |
| n8n (official, `n8n-io/n8n`) | TypeScript | Quick (test-only) | none | testcontainers wrapper | `cloudflared` used only in `packages/testing/containers/services/cloudflared.ts` to give **E2E tests** a public URL for webhook callbacks — not a shipped product feature |

---

## 1. 9Router — deep dive (extends `00-consolidated-research.md` §4)

Source: `github.com/decolua/9router`, files under `src/lib/tunnel/`.

### Binary management (`src/lib/tunnel/cloudflare/cloudflared.js`)
- Downloads from `github.com/cloudflare/cloudflared/releases/latest/download/<platform-asset>`, per-OS/arch asset map (darwin `.tgz`, linux/win raw binary).
- **Binary validation before trusting a cached copy**: checks file size ≥ 1MB, then reads the first 4 bytes and matches the platform's magic number (ELF `7f454c46` on Linux, Mach-O `cffaedfe`/`cefaedfe` on macOS, PE `4d5a` on Windows). Re-downloads if invalid — guards against truncated/corrupt downloads surviving a crash mid-write.
- Cleans up `.tmp` partial-download files left from a previous crashed run before starting.
- In-flight download dedup via a shared `downloadPromise` so concurrent callers await the same download instead of racing.

### Process spawn (quick tunnel)
```js
spawn(binaryPath, ["tunnel", "--url", `http://127.0.0.1:${localPort}`,
  "--config", configPath, "--no-autoupdate", "--retries", "99"], { ... })
```
- Writes a placeholder `config.yml` into a fresh temp dir specifically to **avoid the default `~/.cloudflared/config.yml`**, which cloudflared refuses to run a quick tunnel alongside (matches `00-consolidated-research.md`'s note on this).
- `--retries 99` — relies on cloudflared's own internal reconnect loop before the app's watchdog ever needs to respawn the process.
- URL parsing: regex `/https:\/\/([a-z0-9-]+)\.trycloudflare\.com/gi` over combined stdout+stderr chunks, explicitly excluding the `api.trycloudflare.com` host cloudflared also logs. Takes the **last** match in a chunk (cloudflared can log the URL more than once).
- Distinguishes **first URL** (resolves the startup promise) from a **URL that changes mid-run** (calls an `onUrlUpdate` callback) — quick-tunnel URLs can rotate without the process exiting.
- `intentionalKill` flag suppresses the unexpected-exit handler during a deliberate stop/restart, so a kill triggered by the app itself doesn't get mistaken for a crash.

### Watchdog + network monitor (`src/shared/services/initializeApp.js`)
This is the most detailed supervision logic found in any of the surveyed projects:
- **Deferred startup** (`STARTUP_DEFER_MS = 3000`): heavy startup work (binary download, DNS probes) runs 3s after process boot so it doesn't block the first HTTP request.
- **Watchdog tick**: `setInterval(..., 60000)` (`WATCHDOG_INTERVAL_MS`), `.unref()`'d so it never keeps the Node process alive on its own.
- **Network monitor**: separate `setInterval(..., 5000)` that fingerprints active non-virtual IPv4 interfaces (`name:address` pairs, sorted+joined), detects both an interface change and a **sleep/wake gap** (elapsed time since last tick > 6× the poll interval), and does a real TCP reachability check to `1.1.1.1:443` before acting — pure interface presence is not treated as "online."
- **Restart-cooldown guard**: `RESTART_COOLDOWN_MS = 120000` prevents hammering on repeated watchdog ticks, but is **bypassed** for one-shot transition events (`startup`, `netchange`, `sleep`, `online`, `unexpected-exit`) via a `FORCE_RESTART_REASONS` regex — cooldown only throttles *repeating* ticks, not real state transitions.
- **Trust-the-child-if-alive rule**: if the process is still running, the watchdog does nothing — it trusts cloudflared's own `--retries 99` reconnect rather than killing and respawning (which would rotate the quick-tunnel URL).
- **Settle delay**: `NETWORK_SETTLE_MS = 2500` — waits for DHCP/DNS to settle after a detected network change before probing.
- Virtual-interface exclusion regex: `/^(utun|awdl|llw|anpi|bridge|gif|stf|ipsec|ap|tun|tap|vmnet|veth|docker)/i` — filters out VPN/AirDrop/container interfaces that flap and would cause false-positive "network changed" events.
- Signal handling: `SIGINT`/`SIGTERM`/`exit` handlers registered once, call `killCloudflared()` (and unrelated cleanup) before `process.exit()` — explicit orphan prevention on shutdown.

### Security gate (`src/dashboardGuard.js`)
The earlier consolidated doc characterized this as "refuses to enable remote access if using the default password." Reading the actual middleware, the real mechanism is narrower and worth restating precisely:
- Tunnel **enable/disable API routes** (`/api/tunnel/enable`, `/api/tunnel/disable`, …) are in a `LOCAL_ONLY_PATHS` allowlist — callable only from a verified-loopback request (checked via Host/Origin headers plus a trusted-proxy header scheme) or a machine-local CLI token. **A request arriving over the tunnel itself cannot toggle the tunnel.**
- The **dashboard** (not just the enable toggle) is separately gated by a `tunnelDashboardAccess` setting, **default-off**: if the incoming request's `Host` matches the tunnel's or Tailscale's own hostname and `tunnelDashboardAccess` is false, the dashboard redirects to `/login` regardless of auth state. So by default, exposing the API/data plane via tunnel does **not** also expose the admin dashboard over that same public hostname.
- The **LLM API surface** (`/v1`, `/v1beta`, `/api/v1`, `/codex`, `/responses`) is a separate public prefix that always requires either a valid API key or (for local callers) loopback — this is the one actually gated on "not the default/no auth," since a request without a valid API key is rejected outright, tunnel or not.

---

## 2. Unsloth Studio — closest architectural match (Python, PR not yet merged as of 2026-09-15)

Source: PR [`unslothai/unsloth#8715`](https://github.com/unslothai/unsloth/pull/8715), `studio/backend/cloudflare_tunnel.py` (2128 lines) + `studio/backend/utils/remote_access_settings.py`. This is a **named-tunnel, in-app-wizard** implementation — the closest existing prior art to BodhiApp's stated Phase-2 design (user's own Cloudflare account + domain, driven entirely from Settings).

### State machine
Module-level `_tunnel_state` string, transitions visible directly in the source:

```
off -> starting -> online
starting -> error
online -> stopping -> off
stopping -> error
error (recoverable) -> starting  (retry from _active_tunnel_exited)
```
`get_studio_tunnel_status()` (~line 870) exposes this plus the current URL to the frontend for polling.

### Setup wizard flow ("Custom" / named-tunnel mode)
1. User enters a hostname in Settings → Remote Access → Custom.
2. Backend shells `cloudflared tunnel --no-autoupdate login`, greps stderr for the `https://dash.cloudflare.com/argotunnel?...` authorization URL with a regex, and pushes it to the frontend via a callback (`on_login_url`) instead of relying on cloudflared's own browser auto-open — **necessary because the backend runs headless / the auto-open target may not match the user's browser context**.
3. `cloudflared tunnel create <name>` → parses the created tunnel UUID and credentials-file path from output.
4. `cloudflared tunnel route dns <name> <hostname>` → **self-validates the result** because cloudflared does not: it regex-matches the "add a DNS record" message cloudflared prints and compares the record name actually created against the hostname requested. This catches the case where the user authorizes a *different* zone than the hostname's zone — cloudflared happily creates `studio.other-domain.test.example.com` and exits 0 if `example.com` was the authorized zone, which is not the hostname the user asked for. The refusal is "positive evidence only": if a future cloudflared version stops reporting the created name, Studio accepts rather than false-rejects.
5. Writes a scoped `ingress.yml` (not the global `~/.cloudflared/config.yml`) referencing the tunnel by **UUID** (not name — the account-level `cert.pem` gets deleted after provisioning, so subsequent runs can't resolve by name) and pins:
   ```
   tunnel: <uuid>
   credentials-file: <path>
   ingress:
     - hostname: <hostname>
       service: http://localhost:<port>
     - service: http_status:404
   ```
6. Runs with `cloudflared tunnel --config <path> --no-autoupdate --metrics 127.0.0.1:0 run <uuid>` — `--metrics 127.0.0.1:0` binds the readiness/metrics HTTP server to a **kernel-assigned free port** (no fixed-port conflict risk across restarts), then reads the assigned port back from cloudflared's own log line to poll `/ready`.

### DNS propagation handling (the most transferable finding)
Two problems compound after `route dns` succeeds:
- **OS-level negative caching**: an early failed lookup can cache `NXDOMAIN` for up to **30 minutes**, so re-querying the OS resolver right after creating the record can wrongly report "still not there" long after Cloudflare's edge is actually ready.
- **DNS record propagation delay** on top of that.

Studio's fix: it **never trusts the OS resolver during the startup wait**. It queries Cloudflare's own DNS-over-HTTPS resolvers directly (`_DOH_URLS`, `application/dns-json` Accept header) with only `NOERROR`(0)/`NXDOMAIN`(3) treated as authoritative answers, and — separately — **verifies readiness at the Cloudflare edge before DNS resolves at all**, by resolving a name that's *always* live (`trycloudflare.com`) to get real edge IPs, then opening a raw TLS connection to that IP with `server_hostname=<the custom hostname>` (SNI-based routing means Cloudflare serves the tunnel by SNI regardless of whether the hostname's own DNS record has propagated yet) and GETting a marker health endpoint. Only after neither path confirms readiness within a bounded window does it fall back to waiting out the DoH negative-cache window. This lets Studio report the tunnel "online" almost immediately after the connector registers, instead of blocking on DNS propagation.

### Process lifetime binding (orphan prevention)
Three platform-specific strategies, chosen because none alone covers all three OSes:
- **Linux**: `PDEATHSIG` set on the child via `subprocess.Popen` kwargs (`utils/process_lifetime.child_popen_kwargs()`) — kernel kills the child automatically if the parent dies, even via `SIGKILL`.
- **Windows**: a Job Object the child is assigned to, which Windows tears down when the parent process exits (equivalent guarantee to PDEATHSIG).
- **macOS has no equivalent primitive.** Studio **records the spawned PID** (`_adopt_pid`) and only cleans it up at the **next launch**, explicitly documenting the gap: *"A force quit on macOS leaves the connector serving until the next launch reaps it."* This is an honestly-documented limitation, not a solved problem — worth carrying into BodhiApp's own design doc rather than assuming Rust solves it for free (Rust has no cross-platform PDEATHSIG either; `prctl(PR_SET_PDEATHSIG)` is Linux-only via the `libc`/`prctl` crates, and Windows Job Objects need the `windows` or `windows-sys` crate — see §7).
- Additionally, the child is spawned **on a dedicated "process-lifetime" thread**, specifically so that on Linux, `PDEATHSIG` means "die with the parent *process*," not "die when the worker thread that forked it returns" (a `fork()`/thread subtlety).

### Concurrency & credential handling
- Setup/teardown are mutually exclusive, serialized behind a **kernel file lock** — two backend instances can't provision at once.
- Tunnel credentials JSON and the origin certificate are written `0600`.
- The origin cert (`~/.cloudflared/cert.pem`) is **shared, global, per-user state** — Studio refuses to overwrite an existing one it didn't create, and only deletes it on teardown after proving (content digest) that its own login run wrote it.
- **Teardown cannot delete the DNS record** — `cloudflared tunnel route` has no delete subcommand (only `dns`/`lb`/`ip` to *add* routes). Teardown removes everything Studio owns locally (tunnel identity, credentials, cert if it owns it) and calls the Cloudflare API to delete the tunnel object itself, but leaves the DNS record and **tells the user which record to remove manually** rather than claiming a clean teardown it can't perform.
- **One tunnel per install** — provisioning refuses if an identity record already exists; changing the hostname is modeled as teardown-then-setup, not an in-place update.

---

## 3. Home Assistant "Cloudflared" add-on

Source: `github.com/brenner-tobias/addon-cloudflared` (Docker add-on, Bash + s6-overlay, not a general-purpose app but widely used and instructive for the **named-tunnel-via-CLI-login** flow and for delegating supervision to an OS-level init).

- **Two auth modes**, mutually exclusive by config: (a) `tunnel_token` set → skip everything else, just `cloudflared tunnel run --token <token>` (the Cloudflare-dashboard-issued remotely-managed tunnel token flow); (b) no token → interactive `cloudflared tunnel login` writes `cert.pem`, then `cloudflared tunnel create <name>` / `route dns -f <uuid> <hostname>` exactly as Unsloth Studio does, but driven by shell script instead of a supervised subprocess.
- **Connectivity pre-flight** (debug mode only): `nc -z` checks to `region1.v2.argotunnel.com:7844` and `region2.v2.argotunnel.com:7844` (both TCP and UDP) and `api.cloudflare.com:443` before attempting anything — surfaces firewall/router misconfiguration with a specific error instead of an opaque connect failure.
- **Idempotent re-run safety**: on every start it re-checks for an existing `tunnel.json` credentials file and an existing tunnel by UUID, and **hard-fails if the stored tunnel's name doesn't match the current config** (tunnel name is immutable once created; a mismatch means the user changed config without re-provisioning) rather than silently creating a second tunnel.
- **`--metrics="0.0.0.0:36500"`** exposed as the add-on's health port (see §2's `/ready` shape — confirmed from `cloudflared`'s own source in §6 below).
- **Process supervision is entirely delegated to s6-overlay** (the container init system): the add-on's `run.sh` ends in `exec cloudflared tunnel "${options[@]}"`, replacing the shell process. s6 restarts the service on unexpected exit per its own service-supervision policy; the add-on itself contains **no retry/backoff/watchdog code** — the only in-script resilience is `runWithRetry()` (exponential backoff, `2^attempt` seconds, capped at 4 attempts) applied to individual **setup API calls** (`create`, `route dns`, the tunnel-name lookup), not to the long-running connector.
- Exponential backoff helper, verbatim pattern:
  ```bash
  runWithRetry() {
    local max_retries="${1:-4}" retry_delay="${2:-2}" description="$3"; shift 3
    local attempt=1
    while [ $attempt -le "$max_retries" ]; do
      "$@" && return 0
      sleep "$retry_delay"
      attempt=$((attempt + 1)); retry_delay=$((retry_delay * 2))
    done
    return 1
  }
  ```

---

## 4. Rust/Tauri prior art

### proxypal (`heyhuynhgiabuu/proxypal`, `src-tauri/src/cloudflare_manager.rs`)
- Binary discovery checks an explicit list of well-known install paths (Homebrew arm64/x64, `/usr/bin`, `/usr/local/bin`, snap, Windows Program Files, `~/.local/bin`) **before** falling back to PATH lookup via `which`/`where`, with an explicit comment that **GUI apps on macOS do not inherit the shell's `PATH`**, so a bare PATH lookup silently fails in production even though it works when launched from a terminal in development. This is directly relevant to BodhiApp's Tauri desktop target.
- Uses `tokio::process::Command` with `.kill_on_drop(true)` — the async-runtime-native way to express "kill this child if I stop polling it," contrasted with BodhiApp's own deliberate choice of `std::process` + `Drop::kill()`/`wait()` after an earlier `tokio::process` version leaked orphans (`crates/llama_server_proc/src/server.rs:198-210`, see `feedback_llama_server_proc_std_process.md`). **Caution for BodhiApp**: `kill_on_drop` only sends the kill signal from `Drop`; it does not `wait()` synchronously in `Drop` (Tokio's async `Drop` can't block), so it does not by itself guarantee the child has exited before the parent's own process exits — the same class of problem BodhiApp already hit. Prefer the existing `std::process` pattern for the `cloudflared` supervisor too, not `tokio::process`.
- `stdout`+`stderr` piped, read on a background task, classified line-by-line into UI status events (`connecting → connected → reconnecting → error/disconnected`) via substring matching on cloudflared's own log text (`"registered"` + `"connection"`/`"connindex"`, `.trycloudflare.com` URL detection, `"err "`/`"failed"`/`"unable to"` for errors). No structured log parsing — cloudflared has no `--json`/structured-log CLI flag as of the versions inspected in this research, so **every project surveyed parses plain-text log lines with regex/substring matching**; this is the norm, not a shortcut any of them took.
- Explicit **retry-only-if-was-connected** rule: `MAX_RETRIES = 3` only applies when the process never reached a connected state (bad config / cloudflared missing); once connected at least once, retries are unlimited (treated as "reconnecting," not "failing").
- Fixed 5s delay between spawn retries (no backoff growth) — simpler than 9Router's cooldown or the HA add-on's exponential backoff.
- `tunnel.notify_stop` (`tokio::sync::Notify`) + `tokio::select!` between `child.wait()` and the stop signal is the idiomatic async-Rust "cancellable subprocess loop" shape.

### n8n-desktop (community Tauri fork, `tangtao646/n8n-desktop`, `src-tauri/src/api/cloudflared/mod.rs`)
- No process-supervision code was found in the files this research read (only binary management), but the **module layout is a clean reference for the download/install side** of a Rust cloudflared manager: `cache.rs`, `config.rs`, `download.rs`, `error.rs`, `install.rs`, `models.rs`, `path_resolver.rs`, `platform.rs` — separating "where is the binary" (`path_resolver`+`platform`), "is it cached and fresh" (`cache`), and "fetch/install it" (`download`+`install`) into distinct modules rather than one large file. Comments are in Chinese; behavior not independently verified beyond the module boundaries and the two functions read (`download_cloudflared`, `check_cloudflared_version`).

---

## 5. Pinokio "Public Node"

Source: `github.com/pinokiocomputer/pinokiod`, `kernel/api/cloudflare/index.js`.
- Quick tunnel only (`cloudflared tunnel --url <uri>`), spawned through Pinokio's **generic script-execution kernel** (`kernel.shell.start`/`kill`) — the same subsystem used to run any app's own launch script, not a tunnel-specific process manager. URL parsed via `/(https:.+?trycloudflare\.com)/` over the shell's streamed output.
- **Passcode gate is not a cloudflared feature** — when a passcode is configured, Pinokio starts a local **pipe/reverse-proxy server** in front of the real local URL and tunnels *that* instead, so the passcode is enforced by Pinokio's own HTTP layer before a request ever reaches the wrapped app. This is a transferable idea if BodhiApp ever wants an additional gate independent of Cloudflare Access: put a lightweight auth check in the app's own reverse-proxy path, not the tunnel.
- **QR code in the terminal** (`qrcode` npm package, `QRCode.toString(url, {type:"terminal"})`) for pairing a phone to the tunnel URL — a UX touch worth considering for BodhiApp's tunnel-status screen.

---

## 6. Confirmed from `cloudflared`'s own source: the `/ready` health endpoint

Every project above that health-checks a running tunnel does so against cloudflared's built-in `--metrics <addr:port>` HTTP server. Reading `cloudflared`'s own source (`github.com/cloudflare/cloudflared`, `metrics/` package) confirms the exact contract, which none of the surveyed apps' own docs state precisely:

- `/healthcheck` — always `200 OK`, body `"OK\n"`. Process-alive check only; does **not** mean the tunnel is connected to Cloudflare's edge.
- `/ready` — the actual connectivity health check:
  - **0 active edge connections**: `503 Service Unavailable`, JSON body `{"status": 503, "readyConnections": 0, "connectorId": "<tunnel-uuid>"}`
  - **≥1 active edge connection**: `200 OK`, JSON body `{"status": 200, "readyConnections": <n>, "connectorId": "<tunnel-uuid>"}`
- `/metrics` — Prometheus exposition format (`promhttp.Handler()`), for anyone wanting richer observability than a boolean probe.

This is the primary, verified building block for a Rust health-check task: bind `--metrics 127.0.0.1:0` (let the OS pick a free port, as Unsloth Studio does, rather than a fixed port that can collide across restarts or multiple instances), read the actual bound port back from `cloudflared`'s startup log line (it logs the metrics server address on boot), and poll `/ready` with a `reqwest` client — no log-scraping needed for steady-state health once the process is up, only for the initial URL/connection-established detection.

---

## 7. Recommended supervision design for BodhiApp (Rust)

Synthesized from the above, adapted to BodhiApp's existing conventions (`crates/llama_server_proc/src/server.rs` as the sibling pattern — `std::process` + OS reader threads, `Drop`-based cleanup, health poll loop) and its `services`/`routes_app` layering.

### 7.1 State machine

```
Disabled --enable()--> Installing --binary ready--> LoggedIn(named only)
LoggedIn --provision (create+route dns)--> Provisioned
Provisioned / Disabled(quick) --spawn--> Starting
Starting --edge/health confirms--> Connected
Connected --health probe fails N times--> Degraded
Degraded --health probe recovers--> Connected
Degraded --restart budget exhausted--> Error
Starting --spawn/auth failure--> Error
Connected/Degraded/Error --disable()--> Stopped
Stopped --enable()--> Starting  (skip Installing/LoggedIn/Provisioned if already done)
```

Notes drawn directly from the prior art:
- Keep **`Installing`/`LoggedIn`/`Provisioned`** as distinct, persisted states for the **named-tunnel** path only (Unsloth Studio's flow) — quick tunnels skip straight from `Disabled`/`Stopped` to `Starting`, matching 9Router.
- **`Degraded` is not `Error`.** Follow 9Router's "trust the child if it's still running" rule: a failed health probe while the OS process is still alive should first move to `Degraded` (visible in UI, no action taken) rather than immediately killing and respawning — respawning a quick tunnel rotates its URL, and respawning a named tunnel unnecessarily disrupts an established QUIC/HTTP2 session that may recover on its own (cloudflared has its own internal reconnect logic once running, as the HA add-on's reliance on `--retries`-style resilience shows).
- Persist the state machine's terminal fact (`tunnel_enabled: bool` + which mode) the same way 9Router does (`getSettings()`/`updateSettings()`), so an app restart re-enters `Starting` automatically for a tunnel the user last had enabled — mirrors BodhiApp's own `RunPod` auto-public-URL precedent noted in `01-bodhi-app-codebase-map.md` §2.

### 7.2 Health probe

Use `cloudflared`'s own readiness endpoint rather than log-scraping for steady-state health (§6):
1. Spawn with `--metrics 127.0.0.1:0` (OS-assigned free port — avoids fixed-port collision across restarts, matches Unsloth Studio).
2. Parse the metrics server's bound address from the first few lines of stdout/stderr (cloudflared logs it at startup) — this is the **only** log-parsing needed for the connected-URL/ready-port discovery; everything after that is structured HTTP polling.
3. Poll `GET http://127.0.0.1:<port>/ready` on an interval (start at ~2s, matching the HA add-on / 9Router health-check cadence); treat HTTP `200` with `readyConnections >= 1` as `Connected`, `503` as the process being up but not edge-connected (`Starting` if this is the first probe after spawn, `Degraded` if it was previously `Connected`).
4. For a **quick tunnel**, additionally still parse the `https://*.trycloudflare.com` URL from log lines with the same regex approach every surveyed project uses (`https:\/\/([a-z0-9-]+)\.trycloudflare\.com`, excluding the `api.trycloudflare.com` false match) — `/ready` confirms edge connectivity but doesn't hand back the assigned hostname.
5. For a **named tunnel**, skip DNS-based verification during startup (DNS negative-caching risk per Unsloth Studio §2) and instead treat `/ready` reaching `readyConnections >= 1` as sufficient to declare `Connected` — the configured hostname's DNS record was created once at provisioning time and doesn't need re-verification on every restart. Reserve the SNI-edge-probe trick (§2) only if BodhiApp later wants to confirm the *public* hostname specifically resolves before showing it to the user (e.g. right after first provisioning, not on every restart).

### 7.3 Process management — align with `llama_server_proc`

Reuse the exact pattern in `crates/llama_server_proc/src/server.rs:198-268`, not `tokio::process`:
- `std::process::Command::spawn()` with piped stdout/stderr, `std::thread::spawn` readers forwarding lines to `tracing` (as `monitor_output` does) **and** to a small in-memory ring buffer of recent lines for surfacing in error messages (every surveyed project does this — proxypal's `logTail`, 9Router's `logTail.slice(-4000)`, Unsloth Studio's `_error_summary(text)`).
- `Drop` impl: `kill()` + `wait()` synchronously, matching the documented reason `std::process` was chosen over `tokio::process` for `llama_server_proc` (`feedback_llama_server_proc_std_process.md`) — the async-`tokio::process` `kill_on_drop` used by proxypal does **not** block-wait in `Drop` and is exactly the class of gap that caused BodhiApp's earlier orphan-leak regression. Do not reintroduce it here for a second child-process type.
- Explicit `intentionalKill`-style flag (9Router) or a `notify_stop` cancellation signal (proxypal's `tokio::sync::Notify` pattern, adapted to a `std::sync` primitive such as `AtomicBool` + condvar or a `crossbeam_channel`) so the exit-handling code can distinguish "we killed it" from "it crashed" without spurious restart-storm errors.
- Register process-exit handling on the **`SIGINT`/`SIGTERM` shutdown path already present in `server_app`'s graceful shutdown** (not a new signal handler) — kill `cloudflared` before the axum server itself stops accepting connections, so a client mid-request through the tunnel gets a clean connection-reset rather than the tunnel silently pointing at a dead backend.

### 7.4 Orphan prevention beyond `Drop`

`Drop` alone only helps a **graceful** BodhiApp shutdown. Unsloth Studio's three-tier platform handling (§2) is the most rigorous prior art found and should be the target, not `Drop` alone:
- **Linux**: set `PR_SET_PDEATHSIG` on the child so the kernel kills it if BodhiApp's process dies via `SIGKILL`/crash, not just normal exit. In Rust, via `std::os::unix::process::CommandExt::pre_exec` calling `libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM)` in the child before `exec`, or the `unshare`/`prctl` crates. `pre_exec` is `unsafe` (async-signal-safety constraints) — keep it to the single `prctl` call, matching common Rust idiom for this pattern.
- **Windows**: assign the child to a Job Object with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`, so Windows tears it down when BodhiApp's process handle (and thus the job) is closed — via the `windows`/`windows-sys` crate's `CreateJobObjectW`/`AssignProcessToJobObject`/`SetInformationJobObject`.
- **macOS: no equivalent kernel primitive exists** (confirmed by Unsloth Studio's explicit workaround, and consistent with public knowledge — macOS has no PDEATHSIG analog). Accept the same documented gap Studio does: a force-quit (`kill -9` on BodhiApp itself, or a crash) can strand `cloudflared`. Mitigate by **writing a PID file on spawn** (matches 9Router's `pid.js` and Studio's `_adopt_pid`) and having the **next BodhiApp startup check for and kill a stale `cloudflared` PID** whose PID file exists but doesn't correspond to a process this BodhiApp instance just spawned — cheap, cross-platform, and closes the gap by the next launch even where PDEATHSIG/Job-Object can't.
- Additionally: kill by **port match** as a last-resort fallback (9Router's `killCloudflaredByPort`, using `pkill -f "cloudflared.*:<port>"` / a PowerShell `Get-CimInstance Win32_Process` filter) in case the PID file itself is stale/missing — not recommended as BodhiApp's primary mechanism (fragile command-line matching, boundary bugs are easy — 9Router's own regex has to guard against `:20128` matching `:201280`), but worth having as a defensive cleanup on startup alongside the PID-file check.

### 7.5 Backoff policy

Combine the cleanest parts of 9Router (cooldown + one-shot-event bypass) and the HA add-on (exponential backoff for *setup* calls, not the long-running connector):
- **Setup/provisioning API calls** (binary download, `tunnel create`, `route dns`, Keycloak redirect-URI sync): exponential backoff, e.g. base 2s, ×2 per attempt, capped at 3–4 attempts — directly reuse the HA add-on's `runWithRetry` shape.
- **Long-running connector restarts** (after `Degraded`→restart decision): fixed cooldown window (9Router uses 120s) between automatic restart attempts triggered by a repeating watchdog tick, but **bypass the cooldown** for one-shot state transitions: app startup (resume a previously-enabled tunnel), a detected network-interface change, sleep/wake, and the child process's own unexpected-exit event. This avoids both "restart storm on a flapping network" and "stuck for 2 minutes after the network visibly came back."
- **Trust the child while it's alive** — never kill-and-respawn purely because a single health probe failed; only restart when the OS process has actually exited (crash) or when a bounded number of consecutive `/ready` failures accumulate while the process is still running (a stuck-but-alive connector) — a threshold, not a single miss.

### 7.6 Shutdown ordering

1. On `disableTunnel()`/app shutdown: set the cancellation flag (§7.3) **before** killing the process, so the exit handler doesn't try to auto-restart what's being deliberately stopped.
2. Kill `cloudflared` (SIGTERM, wait with a bounded timeout, then SIGKILL — the `proc.terminate()` → `wait(timeout)` → `proc.kill()` two-step every project with a graceful path uses, e.g. Unsloth Studio's `_end_unrecorded_connector`).
3. Only after the process is confirmed exited (or the timeout is hit), if disabling permanently (not just an internal restart): sync the Keycloak redirect-URI removal (per the feature's stated "sync only on enable/disable and at startup" rule) and clear the persisted `tunnel_enabled` flag.
4. On the **whole-app** shutdown path (not just tunnel-disable), kill `cloudflared` **before** the main axum listener stops — mirrors point 7.3's note that a live tunnel pointing at an already-dead backend is a worse UX than a clean connection reset during the brief overlap window.

### 7.7 What NOT to copy

- **9Router's relay-worker for stable quick-tunnel URLs** — already rejected in `bodhiapp-cloudflare-tunnel-feasibility.md`; central infra + doesn't fix SSE buffering. Named tunnels are BodhiApp's only in-scope stable-URL path per the product decision, so this doesn't apply anyway.
- **`tokio::process` + `kill_on_drop`** (proxypal) — see §7.3; conflicts with BodhiApp's own established, hard-won `std::process` choice.
- **Pinokio's generic multi-purpose shell wrapper** — fine for a tool that runs arbitrary user scripts, but BodhiApp's `cloudflared` supervisor is a single well-known binary with a known CLI surface; a dedicated typed manager (as Unsloth Studio and 9Router both have) is more appropriate than a generic shell-command runner.

---

## Sources

- [9Router (decolua/9router)](https://github.com/decolua/9router) — `src/lib/tunnel/**`, `src/shared/services/initializeApp.js`, `src/dashboardGuard.js`, `src/app/api/tunnel/**` (read via GitHub Contents API, commit on `master` as of 2026-09-15)
- [Unsloth Studio remote-access PR — unslothai/unsloth#8715](https://github.com/unslothai/unsloth/pull/8715) — `studio/backend/cloudflare_tunnel.py` (2128 lines, read in full), PR description and reviewer-risk notes
- [Home Assistant Cloudflared add-on — brenner-tobias/addon-cloudflared](https://github.com/brenner-tobias/addon-cloudflared) — `cloudflared/rootfs/etc/s6-overlay/s6-rc.d/prepare/run.sh`, `cloudflared/rootfs/run.sh`, `cloudflared/config.yaml`
- [proxypal — heyhuynhgiabuu/proxypal](https://github.com/heyhuynhgiabuu/proxypal) — `src-tauri/src/cloudflare_manager.rs`
- [n8n-desktop (community fork) — tangtao646/n8n-desktop](https://github.com/tangtao646/n8n-desktop) — `src-tauri/src/api/cloudflared/mod.rs`
- [Pinokio — pinokiocomputer/pinokiod](https://github.com/pinokiocomputer/pinokiod) — `kernel/api/cloudflare/index.js`
- [Open WebUI Cloudflare Tunnel docs](https://docs.openwebui.com/reference/https/cloudflare-tunnel/) — confirms no in-app automation, manual dashboard/CLI setup only
- [cloudflared source — cloudflare/cloudflared](https://github.com/cloudflare/cloudflared), `metrics/` package (`metrics.go`, `readiness.go`) — `/healthcheck`, `/ready` (200/503 + `{"status","readyConnections","connectorId"}` JSON shape), `/metrics` (Prometheus) endpoint contracts, confirmed directly from source
- Jan.ai (`menloresearch/jan`) — checked via `gh search code "cloudflared" repo:menloresearch/jan`, zero results, confirming no integration exists
- `n8n-io/n8n` — `packages/testing/containers/services/cloudflared.ts` — cloudflared used only for E2E test infrastructure, not a shipped feature
- LM Studio — no public source repository found; UNVERIFIED whether any first-party Cloudflare Tunnel integration exists (community guides only, same manual pattern as Open WebUI)
- BodhiApp internal: `crates/llama_server_proc/src/server.rs:100-268`, `crates/llama_server_proc/CLAUDE.md` — the `std::process`-based child-management pattern this design aligns with
