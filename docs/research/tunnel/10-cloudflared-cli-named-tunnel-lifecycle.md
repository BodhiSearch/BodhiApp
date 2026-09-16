# 10 — `cloudflared` CLI lifecycle for a locally-managed named tunnel

**Date:** 2026-09-15
**Scope:** how an app (not a human) drives the `cloudflared` binary as a child process to run a **locally-managed named tunnel** (credentials-file / cert.pem based — not remotely-managed "token-only" tunnels, not quick tunnels). Extends `00-consolidated-research.md` and `bodhiapp-cloudflare-tunnel-feasibility.md`; does not repeat their quick-tunnel/policy findings.

**Primary source:** `github.com/cloudflare/cloudflared` tag **`2026.9.1`** (latest release as of 2026-09-15, published 2026-09-11). All `file:line`-less code citations below are `<path-in-repo>` at this tag; line numbers were read directly, not guessed — re-check against a newer tag before implementing, since the CLI is actively developed. Cloudflare docs cited are `developers.cloudflare.com` pages fetched 2026-09-15.

---

## 1. `cloudflared tunnel login`

```bash
cloudflared tunnel login
```

**What it prints** — stdout writes nothing; the URL is on **stderr** (comment in source: "AUTH-1423 for why we use stderr, the way git wraps ssh"), one of two messages depending on whether the OS-level browser-open succeeded (`token/transfer.go` `RunTransfer`):

- If browser launch failed: `Please open the following URL and log in with your Cloudflare account:\n\n<url>\n\nLeave cloudflared running to download the cert automatically.\n`
- If browser launch succeeded: `A browser window should have opened at the following URL:\n\n<url>\n\nIf the browser failed to open, please visit the URL above directly in your browser.\n`

Both cases print the **same URL**, so an app can always regex-capture it (`https://dash.cloudflare.com/argotunnel?...`) from stderr and open it itself — **it does not need to rely on cloudflared's own browser launch**, and can even suppress the "should have opened" noise by treating both message shapes the same way. Source: `token/transfer.go:32-52`.

**Browser-launch mechanism per OS** (`token/launch_browser_*.go`) — relevant because these external commands, not `cloudflared` itself, are what can silently fail in a sandboxed/GUI-launched app:
- macOS: `open <url>`
- Linux/BSD: `xdg-open <url>`
- Windows: `cmd /c start "" "<url>"`

**How long it waits** — `token/transfer.go` `transferRequest`/`poll`: an HTTP client with `clientTimeout = 60s`, looped for `pollAttempts = 10`. The Cloudflare LoginHelper endpoint itself long-polls within each 60s window and returns non-200 while waiting (client logs `Waiting for login...` at Info level each time and retries). **Effective max wait ≈ 10 × 60s = up to ~10 minutes**, after which `login` exits with error `Failed to fetch resource`. There is no separate `--timeout` flag for `login`.

**What it writes (cert.pem location, per OS)** — `login.go` `checkForExistingCert()` always targets `filepath.Join(config.DefaultConfigSearchDirectories()[0], "cert.pem")`, i.e. the **first entry** of the default search list, expanded via `go-homedir`. From `config/configuration.go:33` (`defaultUserConfigDirs = []string{"~/.cloudflared", "~/.cloudflare-warp", "~/cloudflare-warp"}`, plus `/etc/cloudflared`, `/usr/local/etc/cloudflared` appended on non-Windows):

| OS | First search dir (where `login` writes `cert.pem`) |
|---|---|
| macOS | `~/.cloudflared/cert.pem` |
| Linux | `~/.cloudflared/cert.pem` |
| Windows | `~/.cloudflared/cert.pem` → i.e. `%USERPROFILE%\.cloudflared\cert.pem` per `go-homedir` expansion of `~` |

**Discrepancy to flag:** Cloudflare's own docs page (`create-local-tunnel`, fetched 2026-09-15) states the Windows path is `C:\Users\<USERNAME>\AppData\Local\cloudflared\`. The source at `2026.9.1` shows `DefaultConfigSearchDirectories()` returning the same `~/.cloudflared`-style list on Windows as other OSes (no separate `defaultWindowsConfigDirs` branch exists in `config/configuration.go`). **UNVERIFIED which is authoritative** — the docs page may describe a different/older version, or describe the *service-mode* config dir (see §7) rather than the CLI default. Verify empirically on a Windows box before relying on either path; do not hardcode assumptions.

**Content shape** — `credentials/origin_cert.go` `OriginCert` struct, PEM-encoded with block type `ARGO TUNNEL TOKEN`, JSON payload:
```json
{"zoneID": "...", "accountID": "...", "apiToken": "...", "endpoint": ""}
```
So `cert.pem` **does** contain a scoped Cloudflare **API token** (`apiToken`) plus `accountID`/`zoneID` — not a zone-wide user credential, but a `login`-issued token with tunnel/DNS-edit scope for that account. `endpoint` is empty for the standard edge, or `"fed"` for FedRAMP (`--fedramp`/`-f` flag switches `baseLoginURL`/`callbackURL` to `dash.fed.cloudflare.com` / `login.fed.cloudflareaccess.org`). Source: `credentials/origin_cert.go:18-45`.

**Zone selection** — happens entirely inside the Cloudflare dashboard page opened by the URL (`https://dash.cloudflare.com/argotunnel`); the CLI has no zone-selection flag. The user picks the account/zone in the browser, and the resulting cert is scoped to that account (one `accountID`) — a `cert.pem` is **not** per-zone; DNS routing (`route dns`) later targets any zone the account+token can manage.

**Existing `cert.pem` behavior** — `login.go` `checkForExistingCert()`: if a non-empty file already exists at the target path, `login` **refuses to run** and prints: `You have an existing certificate at <path> which login would overwrite.\nIf this is intentional, please move or delete that file then run this command again.` It exits without error (returns `nil`) after logging at Error level — so from a process-exit-code perspective this looks like success; **detect this case by scanning stderr for "existing certificate"**, not by exit code alone. Source: `login.go:69-73`.

**Does `--origincert` / `TUNNEL_ORIGIN_CERT` redirect where `login` writes?** — **No.** `buildLoginSubcommand()` (`login.go:50-63`) only registers `loginURL`, `callbackStore`, `fedramp` flags — `origincert` is **not** among them, and `checkForExistingCert()` ignores it entirely, always resolving from `config.DefaultConfigSearchDirectories()[0]`. This means `login` **cannot** be pointed at a BODHI_HOME-owned path directly.

**Workaround for an app-owned cert location** — every *other* command (`create`, `run`, `route`, `token`, etc., via `configureCloudflaredFlags()` in `cmd.go:895-897`) **does** honor `--origincert` / env `TUNNEL_ORIGIN_CERT`, and `credentials.FindOriginCert()` (`credentials/origin_cert.go:120-142`) will happily read a cert from any path you point it at. So the practical pattern to never touch `~/.cloudflared` for anything beyond the one unavoidable `login` write:
1. Run `cloudflared tunnel login` (writes to `~/.cloudflared/cert.pem`, unavoidable).
2. Move/copy that file to `$BODHI_HOME/cloudflared/cert.pem` (optionally deleting the original — the file is not needed again in place).
3. Set `TUNNEL_ORIGIN_CERT=$BODHI_HOME/cloudflared/cert.pem` in the environment for every subsequent `cloudflared` invocation (`create`, `run`, `route dns`, `token`, `delete`, `list`, `info`, `cleanup`).
4. `create`'s credentials-JSON default location is *derived from the origin cert's directory* (`filepath.Dir(credential.CertPath())`, `subcommand_context.go:163`), so once `TUNNEL_ORIGIN_CERT` points into `$BODHI_HOME`, the tunnel's `<uuid>.json` also lands there by default — or pass `--credentials-file` explicitly to be fully deterministic (recommended for an app driving this programmatically; don't rely on the derived default).

**No-TTY / non-interactive concern:** `login` is inherently a "print a URL, poll" flow — it does not require a TTY and does not prompt for input, so it is safe to run under a managed subprocess. The only interaction it wants is *someone, somewhere* completing the browser OAuth dance within ~10 minutes.

---

## 2. `cloudflared tunnel create <name>`

```bash
cloudflared tunnel [--origincert <path>] create [--credentials-file <path>] [--secret <base64>] <NAME>
```

Flags on `create` itself: `--output` (`json`/`yaml`), `--credentials-file`/`--cred-file` (env `TUNNEL_CRED_FILE`), `--secret`/`-s` (env `TUNNEL_CREATE_SECRET`, base64, ≥32 bytes decoded; random 32-byte secret generated if omitted). Source: `subcommands.go:251-291`.

**Output (human-readable, default)** — `subcommand_context.go:188-197`:
```
Tunnel credentials written to <path>. cloudflared chose this file based on where your origin certificate was found. Keep this file secret. To revoke these credentials, delete the tunnel.

Created tunnel <name> with id <uuid>
```
(The "cloudflared chose this file..." sentence only appears when `--credentials-file` was **not** passed, i.e. the default-derivation path was used.)

**Output with `--output json`** — the raw `cfapi.Tunnel` struct is JSON/YAML-encoded to stdout instead (`renderOutput()`, `subcommands.go:703-714`) and the human message is suppressed.

**Credentials JSON file — exact shape.** Written by `writeTunnelCredentials()` (`subcommands.go:301-314`) from the `connection.Credentials` struct, which has **no custom JSON tags**, so `encoding/json` uses the Go field names verbatim (`connection/connection.go:64-70`):
```json
{
  "AccountTag": "<32-hex-char account id>",
  "TunnelSecret": "<base64 of 32 random bytes>",
  "TunnelID": "<uuid>",
  "Endpoint": ""
}
```
- File name: `<TunnelID>.json` (`tunnelFilePath()`, `subcommands.go:293-296`).
- File mode: **`0400`** (owner-read-only) — `writeTunnelCredentials()` line 313.
- Default directory (no `--credentials-file`): `filepath.Dir(<resolved origincert path>)` — i.e. same directory as `cert.pem` (or your `TUNNEL_ORIGIN_CERT` directory, see §1 workaround).
- `Endpoint` is `""` for the default edge, `"fed"` if the cert came from a `--fedramp` login.

**`--credentials-file` idempotency / overwrite** — `writeTunnelCredentials()` errors with `<path> already exists` if the target file is already present (`subcommands.go:305-307`); it never silently overwrites. If the write fails for *any* reason, `create()` **automatically deletes the just-created remote tunnel** (`client.DeleteTunnel(tunnel.ID, true)`, cascade=true) and returns a composite error describing both problems, or — if the delete itself also failed — instructs the caller to `cloudflared tunnel delete <uuid>` manually (`subcommand_context.go:178-186`). **Implication for a driving app:** a `create` failure can still have created a remote tunnel object; always treat `create` failure as "check `cloudflared tunnel list` for an orphan" rather than assuming full rollback succeeded.

**Name collisions / idempotency:**
- **UUID as name is rejected client-side** before any network call: `if _, err := uuid.Parse(name); err == nil { return errors.New("you cannot use UUIDs as tunnel names") }` (`cfapi/tunnel.go:96-98`).
- Duplicate **active** tunnel name: rejected by the Cloudflare API (server-side `POST /accounts/{account}/cfd_tunnel`), surfaced through `statusCodeToError()` (`cfapi/base_client.go:221-243`) as `Failed to create: <API error message>` when the response is JSON with an `errors[]` array, else a generic `API call to create failed with status <code>: <text>`. **UNVERIFIED**: the exact Cloudflare API error code/text for "name already in use by an active tunnel" (server-side, not in this repo) — verify empirically or via `developers.cloudflare.com/api` before parsing it programmatically; do not string-match a hardcoded message without testing against a live account.
- `create` is **not** idempotent — running it twice with the same name (once the first succeeded) creates a **second** tunnel object with a different UUID (Cloudflare allows duplicate names among tunnels as long as they're not simultaneously "active" in a way the API rejects — behavior not fully re-derivable from client source alone). An app should check `cloudflared tunnel list --output json -n <name>` (or track the UUID it created) before calling `create` again, rather than relying on the API to reject a repeat call.

---

## 3. `cloudflared tunnel route dns [--overwrite-dns] <tunnel> <hostname>`

```bash
cloudflared tunnel route dns [--overwrite-dns] <TUNNEL name-or-uuid> <HOSTNAME>
```

`--overwrite-dns` (alias `-f`, env `TUNNEL_FORCE_PROVISIONING_DNS`) — `subcommands.go:177-182`.

Implementation: `cfapi.NewDNSRoute(hostname, overwriteExisting)` → `PUT /zones/{zone}/tunnels/{tunnel_id}/routes` (`cfapi/hostname.go:150-162`, path built as `<zone-level base>/{tunnelID}/routes`), body `{"type":"dns","user_hostname":"<hostname>","overwrite_existing":<bool>}`.

**Success output** — `SuccessSummary()` (`cfapi/hostname.go:83-92`), logged as `sc.log.Info()` with field `tunnelID=<uuid>`:
| Server-reported `cname` change | Printed message |
|---|---|
| `new` | `Added CNAME <hostname> which will route to this tunnel` |
| `unchanged` (CNAME already exists **and already points at this tunnel**) | `<hostname> is already configured to route to your tunnel` |
| `updated` | `<hostname> updated to route to your tunnel` (comment in source: *"this is not currently returned by tunnelstore"* — treat as effectively unused) |

**CNAME exists pointing elsewhere, `--overwrite-dns` not set:** the API call returns a non-200 status; `routeCommand()` returns that error directly to the caller (no special-cased retry). Top-level `cmd.go:69-72` defines the *generic* guidance string shown by the app-level (non-`route`) `--hostname` classic-tunnel path: `failed to provision routing, please create it manually via Cloudflare dashboard or UI; most likely you already have a conflicting record there. You can also rerun this command with --overwrite-dns to overwrite any existing DNS records for this hostname.` — **UNVERIFIED whether `route dns` itself reprints this exact string** (it's referenced as `routeFailMsg` but that variable's use sites weren't confirmed in the fetched excerpt); treat the underlying signal as **"non-zero exit + non-200 API response"**, and drive the retry with `--overwrite-dns` yourself rather than string-matching.

**With `--overwrite-dns`, CNAME exists pointing elsewhere:** the API updates the record to point at this tunnel; expect `cname: "updated"` per the table above (server generally reports `updated` when a real change happened) — **UNVERIFIED exact server semantics for "existing non-tunnel record + overwrite"**; verify against a live zone.

See doc 11 follow-up "cert.pem and DNS routing — resolved" for which credential authorizes this call.

---

## 4. Running the tunnel

### 4.1 Minimal invocation, ad-hoc ingress (single upstream, no ingress rules needed)

```bash
cloudflared tunnel \
  --origincert "$BODHI_HOME/cloudflared/cert.pem" \
  run \
  --credentials-file "$BODHI_HOME/cloudflared/<uuid>.json" \
  --url http://127.0.0.1:<port> \
  --no-autoupdate \
  --metrics 127.0.0.1:<metrics-port> \
  --loglevel info \
  --protocol auto \
  <name-or-uuid>
```

`--url` is registered by `configureProxyFlags()` (default `http://localhost:8080`, env `TUNNEL_URL`) and is available on `run` (`cmd.go:716-724` appends `configureProxyFlags(false)...` to `run`'s flags) — this is the ad-hoc, single-service ingress path; no `config.yml` required. Source: `cmd.go:941-948`.

### 4.2 Ingress config file (multiple hostnames/paths)

```bash
cloudflared tunnel --config /path/to/config.yml run <name-or-uuid>
```
Minimal `config.yml` (per `developers.cloudflare.com/.../configuration-file/`, fetched 2026-09-15):
```yaml
tunnel: 6ff42ae2-765d-4adf-8112-31c55c1551ef
credentials-file: /root/.cloudflared/6ff42ae2-765d-4adf-8112-31c55c1551ef.json

ingress:
  - hostname: example.com
    service: http://localhost:8000
  - service: http_status:404   # required catch-all, must be last
```
Cloudflare's own hard requirement: *"Configuration files that contain ingress rules must always include a catch-all rule that concludes the file."* Rules are matched top-to-bottom, first match wins; `hostname`/`path` may be omitted to match everything (the catch-all). For BodhiApp's single-hostname case, `--url` (4.1) is simpler than authoring a `config.yml` and avoids an extra file to keep in sync with `BODHI_HOME`.

### 4.3 Other `run` flags relevant here

| Flag | Env var | Default | Source |
|---|---|---|---|
| `--credentials-file` / `--cred-file` | `TUNNEL_CRED_FILE` | — | `subcommands.go:113-120` |
| `--credentials-contents` | `TUNNEL_CRED_CONTENTS` | — | pass raw JSON inline instead of a file; **takes precedence over `--credentials-file`** |
| `--token` | `TUNNEL_TOKEN` | — | base64 JSON `TunnelToken` (see §5); takes precedence over credentials |
| `--token-file` | `TUNNEL_TOKEN_FILE` | — | file containing the token string |
| `--origincert` | `TUNNEL_ORIGIN_CERT` | `credentials.FindDefaultOriginCertPath()` | needed only to *locate* the credentials file when you don't pass `--credentials-file` explicitly — **not needed at all if you pass `--credentials-file`/`--token`/`--token-file` directly**, per the `run` command's own description: *"it does not need access to cert.pem from `cloudflared login` if you identify the tunnel by UUID"* (`subcommands.go:735-737`) |
| `--no-autoupdate` | `NO_AUTOUPDATE` | `false` | disables cloudflared's **self-replacing binary update** (checks for a new release and restarts itself) — **always pass this** for a supervised child process; let the app control binary upgrades instead |
| `--autoupdate-freq` | — | per `updater.DefaultCheckUpdateFreq` (not fetched; historically 24h) | irrelevant once `--no-autoupdate` is set |
| `--no-prechecks` | `TUNNEL_NO_PRECHECKS` | `false` | skips connectivity pre-checks at startup |
| `--metrics` | `TUNNEL_METRICS` | `localhost:0` → auto-tries `localhost:20241..20245`, else random port | see §4.4 |
| `--protocol` / `-p` | `TUNNEL_TRANSPORT_PROTOCOL` | `auto` | `auto` (QUIC, falls back to HTTP/2), `quic`, `http2` — full text: *"'auto' - starts with QUIC and falls back to HTTP/2 (the default and recommended option); 'quic' - based on QUIC, relying on UDP egress to Cloudflare edge; 'http2' - using Go's HTTP2 library, relying on TCP egress"* (`connection/protocol.go:10`) |
| `--loglevel` | `TUNNEL_LOGLEVEL` | `info` | values `{debug, info, warn, error, fatal}` (`cliutil/logger.go:30-31`) — **debug level logs full request URL/method/headers**, a documented sensitive-data warning in the flag's own usage text |
| `--logfile <path>` | `TUNNEL_LOGFILE` | — | single file, append mode |
| `--log-directory <dir>` | `TUNNEL_LOGDIRECTORY` | — | rolling log (`lumberjack`, 1MB per file, 5 backups, keep-forever); **mutually exclusive with `--logfile`** — if both set, `--logfile` wins and a warning is logged (`logger/create.go:150-152`) |
| `--output` | `TUNNEL_MANAGEMENT_OUTPUT` / `TUNNEL_LOG_OUTPUT` | `default` | `json` for structured (one-JSON-object-per-line) logs to the console writer — this is the flag the task calls "`--output json`/structured logs"; it affects **console** log formatting only, not `create`/`list --output` (that's a *different* `--output` flag scoped to those subcommands — same name, different purpose, don't conflate) |
| `--grace-period` | `TUNNEL_GRACE_PERIOD` | `30s` | see §4.5 |
| `--pidfile` | `TUNNEL_PIDFILE` | — | PID written **after first successful connection** (not at process start) — do not use this file's existence as an immediate "process started" signal |
| `--connector-label` | — | hostname-derived | human label for this connector, shown in `tunnel info` |

Source for the table: `cmd.go:882-935`, `cliutil/logger.go:16-59`, `subcommands.go:113-130`.

### 4.4 `/ready` and `/metrics` on the `--metrics` listener

Default binding: `--metrics` unset → tries `localhost:20241`, `20242`, `20243`, `20244`, `20245` in order, first free one wins; if none are free, binds a random port (`metrics/metrics.go:30-52`, `CreateMetricsListener` at `:112-140`). **This means a fixed, predictable port is not guaranteed unless you pass `--metrics 127.0.0.1:<your-port>` explicitly** — always pass it explicitly for a supervised child process so you know where to poll.

Startup log line: `Starting metrics server on <addr>/metrics` (Info level, `metrics/metrics.go:164`).

**`/ready`** — exact JSON body, `metrics/readiness.go:29-56`:
```json
{"status": 200, "readyConnections": 4, "connectorId": "3fa85f64-5717-4562-b3fc-2c963f66afa6"}
```
`status` is `200` when `readyConnections > 0`, else the HTTP status **and** the JSON `status` field are both `503` with `readyConnections: 0`. `connectorId` is this connector's own UUID (stable per process run — not the tunnel ID). There is also `cloudflared tunnel ready` (needs `--metrics` set), which polls `http://<metrics-addr>/ready` once and exits 0 if `200`, non-zero with body dump otherwise (`subcommands.go:481-503`) — useful as a health-check subcommand the app can shell out to instead of parsing JSON itself.

**`/metrics`** — standard Prometheus text exposition format via `promhttp.Handler()` (`metrics/metrics.go:79`); not JSON. Scrape it like any Prometheus target if you want connection counts, retries, registration failures, etc. as time series (specific metric names weren't enumerated in this pass — grep `connection/metrics.go` / `supervisor/metrics.go` if per-metric names are needed later).

**Other endpoints on the same listener** (`metrics/metrics.go:72-101`): `/healthcheck` → plain `OK\n`; `/quicktunnel` → `{"hostname":"<quick-tunnel-hostname>"}` (empty for named tunnels); `/config` → current versioned ingress config JSON (only if orchestrator present); `/debug/pprof/*` (Go profiler, **`/debug/pprof/cmdline` is explicitly blocked** to avoid leaking secrets from `os.Args`).

### 4.5 Connected / disconnected log lines (structured fields, not just prose)

From `connection/observer.go:47-65` and `connection/control.go:140-146`:

| Event | Level | Message | Key fields |
|---|---|---|---|
| Attempting registration | Debug | `Registering tunnel connection` | `connIndex`, `ip`, `protocol` |
| Registered successfully | **Info** | `Registered tunnel connection` | `connection` (registration UUID), `connIndex`, `location` (edge colo code, e.g. `SJC`), `ip`, `protocol` |
| Graceful unregister complete | **Info** | `Unregistered tunnel connection` | `connIndex`, `ip` |
| HTTP/2 edge connection lost (non-graceful) | Info | `Lost connection with the edge` | `connIndex` |
| Duplicate connection rejected by edge | Error (via `ConnAwareLogger`) | `Unable to establish connection.` | wrapped `ErrDuplicateConnection` |

With `--output json`, these arrive as one JSON object per line with the above fields plus `time`, `level`, `message`. **Practical parse targets:** `"Registered tunnel connection"` = tunnel is live and serving; `"Unregistered tunnel connection"` = this connection index cleanly drained; a tunnel with **4 connection indices** (0-3, HA default) is only fully up once you've seen 4 `Registered` lines, or — simpler — poll `/ready`'s `readyConnections` count instead of counting log lines.

### 4.6 Graceful shutdown (SIGTERM/SIGINT) and exit codes

`cmd/cloudflared/tunnel/signal.go:11-21`: `waitForSignal()` listens for **both `SIGTERM` and `SIGINT`** (same handling — no difference in behavior). On receipt: logs `Initiating graceful shutdown due to signal <sig> ...` (Info) and closes the shared `graceShutdownC` channel, which every control-stream goroutine watches (`connection/control.go:120-146`). Each connection then sends `GracefulShutdown` to the edge and waits for it to drain, **bounded by `--grace-period` (default 30s, env `TUNNEL_GRACE_PERIOD`)** — flag usage text: *"Waiting for in-progress requests will timeout after this grace period, **or when a second SIGTERM/SIGINT is received**."* So: send one signal → wait up to `grace-period`; send a second signal → immediate forced shutdown regardless of remaining grace period. This is the mechanism to use for a fast, deterministic "stop the tunnel" from a supervising app: send `SIGTERM`, wait `grace-period + small buffer`, and if still running send a second `SIGTERM` to force it.

**Windows:** there is no POSIX `SIGTERM` delivery from an arbitrary parent process in the general case; Go's `os/signal` on Windows only reliably delivers `os.Interrupt` (≈ Ctrl+C / `CTRL_C_EVENT`) to a console-attached child. If BodhiApp spawns `cloudflared` as a **foreground child process** on Windows, use `GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, ...)` (requires the child be started with `CREATE_NEW_PROCESS_GROUP`) rather than assuming `taskkill`/`Process.Kill()` triggers graceful shutdown — a plain kill is a hard terminate, skipping the unregister/drain path entirely. If instead installed as a **Windows service** (`cloudflared service install`), the Windows Service Control Manager's stop command is what closes `graceShutdownC` (see the comment at `main.go:59`: *"Windows service manager closes this channel when it receives stop command"*) — this is the more reliable Windows path if the app is willing to manage the service lifecycle via SCM APIs instead of raw process signals. **UNVERIFIED**: exact exit code cloudflared returns on graceful vs forced vs error shutdown — `urfave/cli`'s `app.Run()` result isn't explicitly checked against `os.Exit` codes in the fetched `main.go`/`generic_service.go` (`app.Run(os.Args)` return value is discarded on the generic-service path), so **do not depend on a specific non-zero code to distinguish clean-stop from crash**; rely on the log lines (§4.5) and/or `/ready` transitioning to `503` instead. The `update` subcommand documents its own special case: *"To determine if an update happened in a script, check for error code 11"* — irrelevant once `--no-autoupdate` is set, but confirms cloudflared does use specific exit codes for *some* conditions.

---

## 5. `list` / `info` / `cleanup` / `delete` / `token`

```bash
cloudflared tunnel [--origincert <path>] list [--output json|yaml] [-d] [-n <name>] [--id <uuid>]
cloudflared tunnel [--origincert <path>] info [--output json|yaml] <name-or-uuid>
cloudflared tunnel [--origincert <path>] cleanup [-c <connector-uuid>] <name-or-uuid> [<name-or-uuid> ...]
cloudflared tunnel [--origincert <path>] delete [-f] <name-or-uuid> [<name-or-uuid> ...]
cloudflared tunnel [--origincert <path>] token [--cred-file <path>] <name-or-uuid>
```

**`list`** — no filters ⇒ excludes deleted tunnels by default (`-d`/`--show-deleted` includes them); default human output is a tab-aligned table `ID  NAME  CREATED  CONNECTIONS` with connections summarized as `<n>x<colo>` per edge location, prefixed by a one-line hint `You can obtain more detailed information for each tunnel with 'cloudflared tunnel info <name/uuid>'` (`subcommands.go:422-441`). Empty result: `No tunnels were found for the given filter flags. You can use 'cloudflared tunnel create' to create a tunnel.` `--output json`/`yaml` returns the raw `[]*cfapi.Tunnel` array instead (skip the text UI entirely — **use `--output json` for anything programmatic**).

**`info`** — human output is `NAME/ID/CREATED` header plus a `CONNECTOR ID CREATED ARCHITECTURE VERSION ORIGIN IP EDGE` table, or `This tunnel has no active connectors.` if none. `--output json` gives `{ID, Name, CreatedAt, Connectors:[...]}` (`subcommands.go:525-600`, `Info` struct fields inferred from usage).

**`cleanup`** — deletes stale **connection** records (not the tunnel object itself) for the given tunnel ID(s); optional `-c/--connector-id` scopes it to one connector. Maps to `DELETE .../{tunnelID}/connections[?client_id=...]` (`cfapi/tunnel.go:238-249`). Useful, per `run`'s own doc string, when reconnection is failing due to stale registered-connection state.

**`delete`** — `-f`/`--force` (env `TUNNEL_RUN_FORCE_OVERWRITE`) is passed straight through as the **`cascade`** parameter on `DELETE /accounts/{account}/cfd_tunnel/{id}?cascade=true` (`cfapi/tunnel.go:180-195`, wiring at `subcommand_context.go:211-233`). Without `-f`, the API rejects deletion of a tunnel that has active connections or non-deleted dependencies (routes); with `-f`, cascade=true removes those dependencies too. Already-deleted tunnel: local pre-check via `tunnel.DeletedAt` returns `Tunnel <id> has already been deleted` **without an API call**. On success, `delete` also best-effort removes the local `<uuid>.json` credentials file (logs a warning, doesn't fail the command, if that removal itself errors).

**`token <name>`** — *"This command only works for Tunnels created since cloudflared version 2022.3.0"* (command's own `Description`, `subcommands.go:855`). Confirms: **yes, `token` works for locally-managed (credentials-file-based) tunnels**, not just remotely-managed ones — it fetches a fresh `TunnelToken` for an existing tunnel by name/UUID from the API (doesn't require the local credentials file to already exist). Two output modes:
- No `--cred-file`: prints the **base64-encoded JSON token** to stdout (single line, no trailing prose) — this is exactly what you'd feed to `run --token <value>` or the Docker/one-liner quick-deploy pattern.
- `--cred-file <path>`: decodes the token and writes it out in the **same `Credentials` JSON shape as `create`** (§2) at that path instead of printing it — a clean way to (re)materialize a lost credentials file for a tunnel the app already knows the UUID of, without needing the original `cert.pem` context beyond auth.

`TunnelToken` wire shape (short JSON keys, base64-then-JSON, `connection/connection.go:79-84`): `{"a":"<accountTag>","s":"<base64 secret>","t":"<uuid>","e":"<endpoint>"}` — note this is a **different encoding** from the `create`-written credentials file (long field names, not base64-wrapped); `ParseToken()` (`subcommands.go:799-810`) base64-decodes then JSON-unmarshals to get it.

All of `create`/`list`/`route`/`delete`/`token`/`info` fire a **non-blocking, best-effort, fire-and-forget** background HTTPS call to Cloudflare's update-check API on every invocation (`updater.StartWarningCheck()`/`LogWarningIfAny()`, `cmd/cloudflared/updater/check.go`) — it uses a non-blocking channel read (`select { case ...: default: return "" }`) so it essentially never has time to respond before the command finishes and is logged as a `Warn` only if it *happens* to arrive in time. Harmless, but means every CLI invocation attempts one extra outbound HTTPS request regardless of `--no-autoupdate` (that flag only governs the **binary self-replace** behavior inside `run`, not this warning check).

---

## 6. Versioning

```bash
cloudflared --version       # "cloudflared version <ver> (built <time> with <buildtype>)"
cloudflared --version -s    # or --short: prints just "<ver>" (first space-delimited token)
```
`app.Version = fmt.Sprintf("%s (built %s%s)", Version, BuildTime, buildTypeMsg)` (`main.go:79`, `cliutil/build_info.go:47-53`). `Version` is Cloudflare's **CalVer** scheme (`YYYY.M.PATCH`, e.g. `2026.9.1`), which makes the support-window policy directly computable from the string itself — no separate metadata call needed.

**Cloudflare's support policy** (developers.cloudflare.com "Downloads" page, fetched 2026-09-15, quoted verbatim): *"Cloudflare supports versions of `cloudflared` that are within one year of the most recent release. As of January 2023 Cloudflare will support `cloudflared` version 2023.1.1 to cloudflared 2022.1.1. Breaking changes unrelated to feature availability may be introduced that will impact versions released more than one year ago."* The page does **not** document a programmatic "am I outdated" check.

**Practical detection for BodhiApp:** parse `--version`'s `YYYY.M` prefix, compare to the current date; if `today - version_date > ~365 days`, treat the bundled/detected binary as unsupported and prompt for an update, mirroring Cloudflare's own stated window. There's no dedicated `cloudflared version --check` subcommand; `cloudflared update` (not `--no-autoupdate`-gated) is the CLI's own self-update path (exits with code `11` per `main.go`'s `update` command docstring when an update was actually applied) — an app-managed binary should **not** invoke `cloudflared update` itself if it wants to control the binary version deterministically; instead compare `--version` output against the latest GitHub release (`api.github.com/repos/cloudflare/cloudflared/releases/latest`, used to derive `2026.9.1` for this research) and re-download if stale.

---

## 7. Headless / automation gotchas

- **No TTY requirement for any of the commands covered here.** `login`, `create`, `route dns`, `run`, `list`, `delete`, `token`, `cleanup` are all non-interactive by design (they either take all arguments as flags or, for `login`, poll a URL). Safe to run fully detached under a process-manager (no PTY needed), confirmed by the absence of any `term.IsTerminal`/stdin-read calls in the flows above except the **console log writer's own color-detection** (`logger/create.go` `createConsoleLogger`: `NoColor: config.noColor || !term.IsTerminal(...)` — this only affects ANSI color codes in the log stream, not blocking behavior; pass `--output json` to sidestep it entirely for a supervised child whose stdout you parse).
- **`TUNNEL_*` env vars are first-class**, not just CLI-flag sugar — nearly every flag documented above has a matching env var (`TUNNEL_ORIGIN_CERT`, `TUNNEL_CRED_FILE`, `TUNNEL_CRED_CONTENTS`, `TUNNEL_TOKEN`, `TUNNEL_TOKEN_FILE`, `TUNNEL_URL`, `TUNNEL_METRICS`, `TUNNEL_LOGLEVEL`, `TUNNEL_LOGFILE`, `TUNNEL_LOGDIRECTORY`, `TUNNEL_GRACE_PERIOD`, `TUNNEL_TRANSPORT_PROTOCOL`, `NO_AUTOUPDATE`, `TUNNEL_NO_PRECHECKS`, `TUNNEL_FORCE_PROVISIONING_DNS`, `TUNNEL_CREATE_SECRET`, `TUNNEL_PIDFILE`, `TUNNEL_MANAGEMENT_OUTPUT`/`TUNNEL_LOG_OUTPUT`). A Rust process-manager can set these in the child's environment map instead of building an argv, which sidesteps shell-quoting/argv-length concerns entirely and keeps secrets (token, cred-contents) out of `ps`/`/proc/*/cmdline` — **notably, `cmd.go`'s own `nonSecretFlagsList` (line ~76) is an explicit allow-list of flag names considered safe to log verbatim, implying the project itself treats most other flags/args, and by extension raw argv, as potentially sensitive** (e.g. a `--token` passed as an argv flag would appear in `/proc/<pid>/cmdline` on Linux — this is exactly why `/debug/pprof/cmdline` is deliberately blocked on the metrics server, per the comment in `metrics/metrics.go:8`: *"the sensitive /debug/pprof/cmdline endpoint is explicitly blocked... to prevent leaking secret command-line arguments (e.g. tunnel tokens)"*). **Prefer env vars over argv for `--token`/`--credentials-contents`.**
- **macOS GUI-launched app PATH problem** — this is a general Tauri/macOS gotcha, not cloudflared-specific: an app launched from Finder/LaunchServices (as opposed to a Terminal) inherits a minimal PATH (typically just `/usr/bin:/bin:/usr/sbin:/sbin`), **not** the user's shell `PATH`, so a `cloudflared` installed via Homebrew (`/opt/homebrew/bin/cloudflared` on Apple Silicon, `/usr/local/bin/cloudflared` on Intel) will not be found by a naive `Command::new("cloudflared")`. Resolve by: (a) bundling/downloading `cloudflared` into an app-owned path (e.g. `$BODHI_HOME/bin/cloudflared`) and always invoking by absolute path — the approach already recommended in `00-consolidated-research.md` §2 — or (b) probing well-known install locations (`/opt/homebrew/bin`, `/usr/local/bin`, `/usr/bin`) in addition to `PATH` before giving up. Option (a) is strictly better since it also solves version pinning (§6).
- **Windows service vs foreground** — `cloudflared service install` registers a Windows service named **`Cloudflared`** (`windows_service.go:27-28`) that reads its config from **`%PROGRAMDATA%\cloudflared`** (`programDataEnvVar = "PROGRAMDATA"`, `configDirName = "cloudflared"`, `windows_service.go:33-34`) — a **different** directory than the per-user `~/.cloudflared` CLI default (§1), and one that typically requires elevated/admin rights to write to. For BodhiApp (desktop app, per-user, no admin requirement assumed), **run `cloudflared run` as a plain foreground child process of the Tauri app** (matching how BodhiApp already manages the llama.cpp subprocess, per `crates/llama_server_proc`) rather than installing it as a Windows service — avoids the elevation requirement and keeps lifecycle fully under the app's own process-supervision code, at the cost of the tunnel not surviving a user logout/reboot without the app itself being relaunched (acceptable for a desktop app that already needs to be running to serve the LLM APIs anyway).
- **Linux service** — `cloudflared service install` on Linux installs a systemd unit; not detailed here since it's out of scope for a foreground-child-process design; flag only that `cloudflared service install/uninstall` subcommands exist per-OS with OS-specific implementations (`linux_service.go`, `macos_service.go`, `windows_service.go`, `generic_service.go` as the no-op fallback for unsupported OSes) if BodhiApp ever wants a "survive reboot" mode later.
- **`--metrics` port collision under multiple app instances** — if BodhiApp itself might run more than one instance (unlikely for a desktop app, but worth flagging), do not rely on the default port-scan behavior (§4.4); assign each instance's `cloudflared` a distinct explicit `--metrics 127.0.0.1:<port>` derived from the app's own already-allocated port range, to avoid two `cloudflared` processes racing for `20241-20245`.
- **`--no-prechecks` tradeoff** — skips cloudflared's own startup connectivity self-test; leaving pre-checks **on** (default) gives better first-run error messages (e.g. "WARP client is blocking egress" style failures) at the cost of a small startup delay — recommend leaving default (`false`) for the first `run` per session so failures are diagnosable, since BodhiApp's own retry/backoff wrapper can then decide whether to disable it on subsequent attempts.

---

## CLI path vs API path — verdict for a desktop app

Confirms and sharpens `00-consolidated-research.md` §2's conclusion, now with lifecycle detail:

**For it:**
- Every management operation this doc covers (`login`, `create`, `route dns`, `run`, `list`, `delete`, `token`, `cleanup`) is a **single, well-defined, scriptable CLI invocation** with predictable stdout/stderr/exit-code(ish) shapes and matching `TUNNEL_*` env vars — there is no operation here that *requires* the raw Cloudflare REST API instead.
- `run` is **only available via the CLI** — the QUIC/HTTP2 edge protocol that actually carries traffic is not something the `cloudflare` Rust crate (REST-only) can do; you need the `cloudflared` binary running as a process regardless of how you manage the tunnel *object*.
- The CLI's own credentials-file and cert.pem handling already solves secret storage/rotation (`token` command re-issues credentials without re-running `login`), so there's less custom secret-management code to write than doing raw REST calls with a stored API token.

**Against it (reasons you might still want the `cloudflare` Rust crate for *management*, keeping `cloudflared` only for `run`):**
- `create`/`route dns`/`delete`/`list` outputs are **stable enough with `--output json`** to parse reliably, but every non-`run` operation is still a **process spawn + stdout capture** round-trip (tens-to-hundreds of ms of fork/exec overhead per call) versus a direct HTTPS REST call from within the Rust process — matters if BodhiApp wants snappy UI feedback while, e.g., listing existing tunnels in a settings panel.
- The **exact wording of several error paths is server-side and outside this repo's Go client code** (§2 name-collision, §3 CNAME-conflict) — a REST client that gets the raw Cloudflare API JSON error (with a stable numeric `code`) is more robust to parse than scraping cloudflared's `Failed to <op>: <text>` wrapper, which itself re-wraps that same JSON when present (`cfapi/base_client.go:222-228`) — so going through the API crate for `create`/`route`/`delete`/`list` gets you the *same* underlying error, one wrapping layer removed.
- **Recommendation**: use `cloudflared run` (CLI, subprocess) for the data-plane exactly as this doc describes, since there's no alternative; for the control-plane (`create`/`route dns`/`delete`/`list`/`token`), either approach works — shelling out to the CLI with `--output json` is *simpler to build first* (no new crate, reuses the exact commands documented here, matches 9Router's proven pattern per `00-consolidated-research.md` §4), while the `cloudflare` Rust crate is a reasonable **follow-up refactor** if the process-spawn latency or error-parsing fragility becomes a real problem once the feature ships. Either way, `cert.pem`'s `apiToken` field (§1) is itself a usable Cloudflare API token, so nothing about choosing the CLI for control-plane calls forecloses adding the REST crate later — the same credential works for both.

---

## Sources

- `github.com/cloudflare/cloudflared` @ tag `2026.9.1` (fetched via `raw.githubusercontent.com`, 2026-09-15):
  - `cmd/cloudflared/main.go`, `cmd/cloudflared/generic_service.go`, `cmd/cloudflared/windows_service.go`
  - `cmd/cloudflared/tunnel/cmd.go`, `subcommands.go`, `login.go`, `subcommand_context.go`, `credential_finder.go`, `signal.go`
  - `cmd/cloudflared/flags/flags.go`
  - `cmd/cloudflared/cliutil/logger.go`, `build_info.go`
  - `cmd/cloudflared/updater/check.go`, `update.go`
  - `credentials/origin_cert.go`
  - `connection/connection.go`, `connection/control.go`, `connection/observer.go`, `connection/protocol.go`, `connection/http2.go`
  - `cfapi/hostname.go`, `cfapi/tunnel.go`, `cfapi/base_client.go`
  - `config/configuration.go`
  - `logger/configuration.go`, `logger/create.go`
  - `metrics/metrics.go`, `metrics/readiness.go`, `metrics/config.go`
  - `token/transfer.go`, `token/launch_browser_darwin.go`, `token/launch_browser_unix.go`, `token/launch_browser_windows.go`
  - Release metadata: `https://api.github.com/repos/cloudflare/cloudflared/releases/latest` (tag `2026.9.1`, published 2026-09-11)
- `https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/get-started/create-local-tunnel/` (fetched 2026-09-15) — CLI walkthrough, `config.yml` example, claimed Windows cert path (flagged as possibly stale vs. source, §1)
- `https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/downloads/` (fetched 2026-09-15) — version support policy (quoted verbatim, §6)
- `https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/configure-tunnels/local-management/configuration-file/` (fetched 2026-09-15) — ingress config reference, catch-all rule requirement

---

## Follow-up: Pin the exact cloudflared invocation + stdout/log-line contract for a named tunnel run

**Date:** 2026-09-15. **Why:** `23-codebase-persistence-routes-and-backend-tests.md` §(e) and `24-codebase-frontend-settings-v2-and-e2e.md` §6(d)/§6 both wrote a fake-`cloudflared` stub against a guessed CLI surface, flagging explicitly that the real Rust-side supervisor "doesn't exist yet." This section is that pin, re-verified against `cloudflared` source at tag `2026.9.1` (same tag as the rest of this doc; re-checked `metrics/readiness.go`, `metrics/metrics.go`, `connection/observer.go`, `connection/control.go`, `cmd/cloudflared/tunnel/subcommands.go` directly from `raw.githubusercontent.com` on 2026-09-15, plus `github.com/rs/zerolog@master` for JSON field-key defaults) rather than re-derived from this doc's own prose.

### A. The pinned argv

```bash
"$BODHI_HOME/cloudflared/bin/cloudflared" tunnel run \
  --credentials-file "$BODHI_HOME/cloudflared/<tunnel-uuid>.json" \
  --url http://127.0.0.1:<bodhi-local-port> \
  --no-autoupdate \
  --metrics 127.0.0.1:<metrics-port> \
  --loglevel info \
  --protocol auto \
  --output json \
  <tunnel-uuid>
```

This **is** the §4.1 "minimal invocation" already in this doc, made definitive by (a) dropping `--origincert` — confirmed unnecessary here: `run`'s own `Description` string says it "does not need access to cert.pem ... if you identify the tunnel by UUID", and `--credentials-file` is registered directly as a `run`-subcommand flag (`cmd.go` `buildRunCommand()`: `credentialsFileFlag` is in `run`'s own `cliFlags` list, not inherited from the parent `tunnel` command) — and (b) adding `--output json` so the fallback log-line contract (§C) is a stable one-JSON-object-per-line stream instead of the ANSI/console format.

**Why `--credentials-file`, not `--token`, is the pinned default** — `cmd/cloudflared/tunnel/subcommands.go` `runCommand()`: if `--token`/`--token-file` is set, the trailing `<TUNNEL>` positional argument is **not required at all** (`tokenStr != ""` branch returns via `sc.runWithCredentials(token.Credentials())` without ever reading `c.Args().First()`); otherwise the positional tunnel name/UUID is mandatory (`c.Args().First()`, erroring `"cloudflared tunnel run" requires the ID or name of the tunnel...` if blank and no `tunnel:` key in a config file). Both work, but BodhiApp's own control-plane flow (§2 of this doc) already produces a `<uuid>.json` credentials file as the direct output of `tunnel create` — using it means **no extra `cloudflared tunnel token <name>` round-trip** is needed just to get a `run`-time secret. `--credentials-file`'s value is a **file path**, not the secret itself, so — unlike `--token` — it is safe to place directly in argv without violating this doc's own §7 ("Headless / automation gotchas") guidance to keep secrets out of `/proc/*/cmdline`; the actual secret (`TunnelSecret`) stays inside the `0400`-mode JSON file (§2). If a future refactor prefers the token path, pass it via the `TUNNEL_TOKEN` **env var**, never `--token` on argv, per that same guidance.

**Positional `<tunnel-uuid>`**: pass the UUID form (not the human name) — avoids an extra name→UUID resolution step and matches what `create --output json` already returns.

**Flags intentionally omitted from the pin**: `--logfile`/`--log-directory` (BodhiApp captures stdout/stderr from the child process itself, per the existing `llama_server_proc` pattern — no need for `cloudflared` to also write its own log file); `--pidfile` (§4.3 of this doc already notes it's written only after first successful connection, so it's not a useful "process started" signal, and the Rust supervisor already has the OS PID from spawning the child directly); `--connector-label` (cosmetic only, shown in `tunnel info`, not needed for the supervisor's own state machine — could be added later purely for the user-facing "Connectors" table if BodhiApp ever surfaces `tunnel info` in the UI).

### B. Provisioning → active signal: `/ready` polling wins, not log-line scraping

**Verified field names** (re-fetched `metrics/readiness.go` at tag `2026.9.1` directly, not re-derived from this doc's earlier prose):

```go
type body struct {
    Status           int       `json:"status"`
    ReadyConnections uint      `json:"readyConnections"`
    ConnectorID      uuid.UUID `json:"connectorId"`
}
```
i.e. exactly `{"status": 200, "readyConnections": 4, "connectorId": "<uuid>"}` as §4.4 already stated — confirmed byte-for-byte against source, not a paraphrase. `makeResponse()` (`metrics/readiness.go`): `readyConnections = tracker.CountActiveConns(); if readyConnections > 0 { return 200, readyConnections } else { return 503, readyConnections }`.

**Decision: BodhiApp's supervisor uses `GET http://127.0.0.1:<metrics-port>/ready` polling as the sole authoritative signal for the provisioning→active state transition.** Rationale:
- **Stable, versioned JSON contract** vs. log lines, which depend on `--loglevel`/`--output` being set exactly as expected and on zerolog's message text never being reworded upstream (it has already changed once historically — this repo's own §4.5 table is sourced from the current tag, with no guarantee across `cloudflared` releases).
- **One boolean-shaped check** (`status == 200`) instead of counting to 4 `"Registered tunnel connection"` lines (one per HA connection index, §4.5) — simpler supervisor state machine, no need to track a running count across log lines.
- **Decoupled from `--output`/`--loglevel` flags** — `/ready` works identically regardless of what those are set to, whereas log-line scraping requires the pinned `--loglevel info --output json` combination (§A) to hold forever.
- **Already the pattern `cloudflared` itself recommends** for exactly this use case: `cloudflared tunnel ready` (needs `--metrics` set) polls `/ready` once and exits 0/non-zero (§4.4) — i.e. the upstream project's own answer to "is the tunnel up" is `/ready`, not log parsing.

**Polling parameters (BodhiApp-side decision, not from cloudflared source):** poll `GET /ready` on the `--metrics` bind address at a short fixed interval (e.g. every 500ms–1s) starting immediately after spawn, treating `200` with `readyConnections > 0` as **active**; treat a connection-refused error on the poll (port not bound yet) as **still provisioning**, not a failure, for a bounded startup window (recommend 30–60s, matching `--grace-period`'s own default magnitude, §4.6) before surfacing a "failed to start" error state. Do **not** poll `/ready` as a liveness check after reaching active — a closed connection or process exit is better detected via the child process's own exit/wait, which the supervisor already has for free.

### C. stdout/stderr contract: log lines kept only as first-run diagnostics, not the state-machine signal

With `--output json --loglevel info` pinned (§A), every log line is one JSON object per line with keys `level`, `time`, `message` plus event-specific fields — confirmed against `github.com/rs/zerolog` (`master` as of 2026-09-15) `globals.go`: `TimestampFieldName = "time"`, `LevelFieldName = "level"`, `MessageFieldName = "message"` (these are zerolog's compile-time defaults; `cloudflared`'s `logger/create.go` does not override them for the JSON writer path). Example line for the key event:
```json
{"level":"info","time":"2026-09-15T12:00:00Z","message":"Registered tunnel connection","connection":"<uuid>","connIndex":0,"location":"SJC","ip":"...","protocol":"quic"}
```
(field keys `connection`, `connIndex`, `location`, `ip`, `protocol` confirmed against `connection/observer.go` `logConnected()` — re-fetched at tag `2026.9.1`, matches §4.5's table exactly, byte-for-byte on the message string `"Registered tunnel connection"`.)

**What the supervisor should still watch stdout/stderr for (fallback / diagnostic role only, never the primary state transition):**

| Substring (in the JSON `message` field, or raw stderr for `login`) | Meaning | Used for |
|---|---|---|
| `"Registered tunnel connection"` | one HA connection index is up | first-run progress indicator in the UI ("connecting... 1/4"); **not** the enable/active trigger — `/ready` is |
| `"Unregistered tunnel connection"` | one connection index cleanly drained (confirmed `connection/control.go`, still present at `2026.9.1`, same message text) | graceful-shutdown confirmation logging only |
| `"Lost connection with the edge"` | non-graceful drop of one connection index | retry/backoff telemetry, surfaced to logs, not user-facing state by itself (still counted via `/ready`'s `readyConnections`, which will drop) |
| `"Starting metrics server on"` | confirms the `--metrics` listener actually bound — **the one log line worth watching even with `/ready` polling**, since a bind failure (port already in use) means `/ready` will never be pollable and the supervisor should fail fast on this line's *absence* within a short startup window rather than only timing out the poll loop | metrics-listener-bind-failure fast path |
| `"existing certificate"` (stderr, from `login` only, not `run`) | `login` refused to run because `~/.cloudflared/cert.pem` already exists (§1) — exit code is `0` in this case, so this is the **only** way to detect it | one-time `login` flow, not the `run` supervisor |

**Why keep any log-line watching at all, given §B's verdict:** two gaps `/ready` cannot cover — (1) diagnosing *why* a tunnel never reaches ready (`--no-prechecks` left at its default `false` per this doc's own §7 ("Headless / automation gotchas") recommendation, so precheck failures like "WARP client is blocking egress" are the first useful diagnostic text, and they only appear on stdout, never through `/ready`); (2) confirming the metrics listener itself bound (see table above) — if binding failed, polling `/ready` is polling nothing, and the process may otherwise look "running" (PID alive) with no way to distinguish "still starting" from "will never come up" without a bounded timeout. Recommendation: **log-line scraping is a bounded, best-effort diagnostic sidecar to the poll loop, active only during the provisioning window, never re-engaged for steady-state monitoring** — steady-state health is `/ready` polling (or, once active, simply watching for the OS process to exit).

### D. Fake-`cloudflared` test stub — exact contract to implement

For `crates/lib_bodhiserver/tests-js/fixtures/bin/fake-cloudflared.mjs` (per `24-codebase-frontend-settings-v2-and-e2e.md` §6) and `crates/server_app/tests/resources/fake-cloudflared` (per `23-codebase-persistence-routes-and-backend-tests.md` §(e)), both stubs must, for `tunnel run <args...>`:
1. Parse (or ignore, but not choke on) the full pinned argv from §A, including `--credentials-file`, `--url`, `--no-autoupdate`, `--metrics 127.0.0.1:<port>`, `--loglevel info`, `--protocol auto`, `--output json`, and the trailing `<tunnel-uuid>`.
2. **Actually bind an HTTP listener on the `--metrics` address** and serve `GET /ready` returning `{"status":200,"readyConnections":1,"connectorId":"00000000-0000-0000-0000-000000000000"}` (any fixed fake UUID) after a short simulated-connect delay (e.g. 100–500ms) — this is now the **primary** thing the stub must get right, per §B's verdict; a stub that only prints log lines and never binds `/ready` would pass against an implementation built to this doc's §B decision but silently diverge from real `cloudflared` behavior.
3. Emit `{"level":"info","time":"<rfc3339>","message":"Starting metrics server on 127.0.0.1:<port>/metrics"}` on stdout immediately after binding, and `{"level":"info","time":"<rfc3339>","message":"Registered tunnel connection","connIndex":0,...}` shortly after, matching §C's table, so the diagnostic-sidecar code path is also exercised by the same test.
4. Stay in the foreground and only exit on `SIGTERM` (once) or `SIGINT`, per §4.6 of this doc — do not daemonize, do not exit immediately after printing (a stub that exits immediately cannot be used to test the supervisor's shutdown/grace-period path at all).
5. For `tunnel login`/`tunnel create`/`tunnel route dns`/`tunnel token`, match the exact human-output strings already quoted verbatim in §1/§2/§3/§5 of this doc rather than inventing new wording, since those are the strings a real supervisor's non-`run` parsing (if any) would match against.

This supersedes the illustrative stub sketch in `23-codebase-persistence-routes-and-backend-tests.md` §(e) (that sketch's `tunnel run` case only echoes two log lines and never binds `/ready` — update it to match this §D before implementing, since §B makes `/ready` the primary signal, not a log line).

### Sources (this follow-up section only; re-fetched 2026-09-15, in addition to the doc-wide Sources list above)

- `https://raw.githubusercontent.com/cloudflare/cloudflared/2026.9.1/metrics/readiness.go` — `/ready` JSON shape, verified verbatim
- `https://raw.githubusercontent.com/cloudflare/cloudflared/2026.9.1/metrics/metrics.go` — `/metrics`, `/healthcheck`, `/quicktunnel`, `/config`, `/debug/pprof/cmdline` routing; `"Starting metrics server on %s/metrics"` log line
- `https://raw.githubusercontent.com/cloudflare/cloudflared/2026.9.1/connection/observer.go` — `"Registering tunnel connection"` / `"Registered tunnel connection"` messages and field keys, verified verbatim
- `https://raw.githubusercontent.com/cloudflare/cloudflared/2026.9.1/connection/control.go` — `"Unregistered tunnel connection"` message, `GracefulShutdown` call site
- `https://raw.githubusercontent.com/cloudflare/cloudflared/2026.9.1/cmd/cloudflared/tunnel/subcommands.go` — `buildRunCommand()`'s `cliFlags` (confirms `--credentials-file` is a `run`-level flag), `runCommand()`'s token-vs-positional-arg branching logic
- `https://raw.githubusercontent.com/cloudflare/cloudflared/2026.9.1/cmd/cloudflared/tunnel/flags... ` (`flags.Metrics = "metrics"`, `TunnelTokenFlag = "token"`) — confirmed via `cmd/cloudflared/tunnel/subcommands.go` and `cmd/cloudflared/flags/flags.go`
- `https://raw.githubusercontent.com/cloudflare/cloudflared/2026.9.1/logger/create.go` — confirms no override of zerolog's default field-key names for the JSON writer path
- `https://raw.githubusercontent.com/rs/zerolog/master/globals.go` — `TimestampFieldName = "time"`, `LevelFieldName = "level"`, `MessageFieldName = "message"` (zerolog defaults `cloudflared` inherits)
