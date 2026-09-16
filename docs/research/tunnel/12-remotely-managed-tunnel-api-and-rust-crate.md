# 12 — Remotely-managed named tunnel: REST API sequence + Rust crate survey (2026-09-15)

Scope: the Cloudflare REST API path to stand up a **named tunnel with `config_src=cloudflare`** (ingress config lives in Cloudflare's control plane, connector authenticates with a single token) — as opposed to the locally-managed flow (`cloudflared tunnel login` + `cert.pem` + `config.yml` on disk), which `00-consolidated-research.md` and `bodhiapp-cloudflare-tunnel-feasibility.md` already cover at a high level. This doc extends those with exact request/response shapes, verified against the live Cloudflare API docs, the `cloudflare-rs` and `cloudflared` source on GitHub, and crates.io, plus a 2026 survey of what real Rust projects actually ship.

**Corrects `00-consolidated-research.md`:** that doc says the `cloudflare` crate's `cfd_tunnel` module exposes `route_dns` "plus... DnsRouteResult types" as if that were the DNS-record mechanism. It is not — `route_dns` hits the legacy `PUT /zones/{zone}/tunnels/{tunnel_id}/routes` endpoint (Argo Tunnel-era), not `POST /zones/{zone_id}/dns_records`. Plain CNAME creation is a *different* module (`endpoints::dns`), covered in §3 below.

## 1. API call sequence for a remotely-managed named tunnel

All calls: `Authorization: Bearer <api_token>`, base `https://api.cloudflare.com/client/v4`. Every response is wrapped in the standard envelope `{success, errors[], messages[], result}`.

### 1.1 Create the tunnel (config lives in Cloudflare)

```
POST /accounts/{account_id}/cfd_tunnel
Content-Type: application/json

{
  "name": "bodhi-myinstance",
  "config_src": "cloudflare",
  "tunnel_secret": "<base64, ≥32 raw bytes>"
}
```
- `name` — required, must be unique within the account.
- `config_src` — `"local"` (default) or `"cloudflare"`. **This is the field that decides remotely- vs locally-managed** — everything else about the create call is identical.
- `tunnel_secret` — optional per the schema, but `cloudflared`'s own `subcommandContext.create()` always generates and sends one (32 random bytes, base64-std-encoded) [cloudflared `cmd/cloudflared/tunnel/subcommand_context.go`, lines ~139–150]. Generate and send it even for `config_src=cloudflare` — it becomes part of the token (see §2) and is what the connector uses to authenticate the QUIC/HTTP2 control stream; there is no way to fetch it back later, so persist it (or just persist the token, which embeds it).

Response `result` (verified against the live API reference page):
```json
{
  "id": "f70ff985-a4ef-4643-bbbc-4a0ed4fc8415",
  "account_tag": "023e105f4ecef8ad9ca31a8372d0c353",
  "name": "bodhi-myinstance",
  "config_src": "cloudflare",
  "status": "inactive",
  "created_at": "2026-09-15T05:20:00.12345Z",
  "connections": [],
  "conns_active_at": null,
  "conns_inactive_at": null,
  "tun_type": "cfd_tunnel"
}
```
`status` starts `"inactive"` (no connector has ever connected) and moves through `"degraded"` / `"healthy"` / `"down"` as connections register.

**Errors:** duplicate name fails create (community reports show the client-visible message `"failed to create tunnel: Create Tunnel API call failed: tunnel with name already exists"`, sourced from `cloudflared`'s own wrapped error text, not a documented numeric code) — UNVERIFIED: the Cloudflare API reference does not publish the numeric `code` for this specific case; treat any `success:false` on create as "probably a name collision" and surface the raw `errors[].message` to the user rather than pattern-matching a code.

### 1.2 Set the ingress configuration

```
PUT /accounts/{account_id}/cfd_tunnel/{tunnel_id}/configurations
Content-Type: application/json

{
  "config": {
    "ingress": [
      {
        "hostname": "bodhi.example.com",
        "service": "http://localhost:1135",
        "originRequest": {
          "noTLSVerify": true,
          "connectTimeout": 10,
          "http2Origin": false
        }
      },
      { "service": "http_status:404" }
    ],
    "originRequest": {
      "connectTimeout": 10
    },
    "warp-routing": { "enabled": false }
  }
}
```
- `ingress` is an **ordered** array; the **last rule must have no `hostname`** and typically a bare `service` (catch-all — `http_status:404` is the idiomatic no-match response). Cloudflare rejects a config with zero rules.
- `service` for BodhiApp is the loopback HTTP listener BodhiApp already binds (`http://localhost:<BODHI_PORT>` or `http://127.0.0.1:<port>`) — no separate cert needed since the origin is plain HTTP behind the tunnel.
- `originRequest.http2Origin` — leave `false`/omit; BodhiApp's axum server is HTTP/1.1, and SSE (`crates/server_core/src/fwd_sse.rs`) does not need HTTP/2 to the origin, only HTTP/2 or QUIC on the *edge* side, which `cloudflared` handles itself.
- Nested per-hostname `originRequest.access` (Cloudflare Access policy binding) — not needed for BodhiApp since auth is Keycloak/OIDC at the app layer, not Cloudflare Access.

Response `result` echoes the config plus a monotonically increasing `version` integer:
```json
{
  "account_id": "023e105f4ecef8ad9ca31a8372d0c353",
  "tunnel_id": "f70ff985-a4ef-4643-bbbc-4a0ed4fc8415",
  "version": 1,
  "config": { "...": "echoed" },
  "created_at": "2026-09-15T05:21:00Z",
  "source": "cloudflare"
}
```
This `version` is the same counter `cloudflared`'s `Orchestrator.UpdateConfig(version, config)` compares against (`orchestration/orchestrator.go` — the connector starts at an internal version of `-1` specifically so any config pulled from Cloudflare, which starts at `0`, always wins). **A second `PUT` to this endpoint while the tunnel is running is how you push config changes live** — the connector receives it over its already-open control stream and logs `"Updated to new configuration"` (`orchestration/orchestrator.go` — the `Orchestrator.UpdateConfig` log line), no restart required. This is the mechanism BodhiApp would use to update the ingress hostname if the user changes the tunnel's public hostname without restarting the connector.

### 1.3 Route DNS — create the CNAME

Two ways to get `bodhi.example.com` pointing at the tunnel; both are equally valid, pick one:

**(a) Raw DNS record create** (`POST /zones/{zone_id}/dns_records`) — what a from-scratch client (and `ytunnel`, see §3.3) actually does:
```json
{
  "type": "CNAME",
  "name": "bodhi.example.com",
  "content": "f70ff985-a4ef-4643-bbbc-4a0ed4fc8415.cfargotunnel.com",
  "proxied": true,
  "ttl": 1
}
```
`ttl: 1` means "automatic" (required when `proxied: true` — Cloudflare ignores/overrides TTL for proxied records anyway). Response `result` includes the new record `id` (needed later for cleanup) plus `created_on`/`modified_on`.

**(b) Legacy tunnel-route endpoint** (what the `cloudflare` crate's `route_dns` wraps): `PUT /zones/{zone_tag}/tunnels/{tunnel_id}/routes` with body `{"type": "dns", "user_hostname": "bodhi.example.com"}`. This is documented as the Argo-Tunnel-era route API; it creates the same CNAME under the hood and returns `{cname: "New"|"Updated"|"Unchanged", name, dns_tag}`. Functionally interchangeable with (a) for a single hostname; (a) is more transparent (you own the DNS-record ID for explicit cleanup) and is what current third-party tools use.

**Error — CNAME already exists:** documented error code **`81053`**, message *"An A, AAAA or CNAME record already exists with that host."* — this fires if the user has any existing A/AAAA/CNAME at that exact name (own prior tunnel attempt, or an unrelated record). Recovery: either let the user pick a different hostname, or (if BodhiApp created it) look it up with `GET /zones/{zone_id}/dns_records?type=CNAME&name=<host>` and reuse/update it (`PUT /zones/{zone_id}/dns_records/{id}`) instead of creating blind.

### 1.4 Get the connector token

```
GET /accounts/{account_id}/cfd_tunnel/{tunnel_id}/token
```
Response: `result` is a **plain string** (not an object) — the opaque `eyJ...` token, e.g. `{"success": true, "result": "eyJhIjoiMDIzZ...", "errors": [], "messages": []}`. Feed this string straight to `cloudflared tunnel run --token <value>` (§2). Retrievable any time after tunnel creation, as many times as needed — it is not one-shot; treat it as a credential, not a nonce (rotate by regenerating, not by re-fetching).

### 1.5 Status / connections

```
GET /accounts/{account_id}/cfd_tunnel/{tunnel_id}
```
Returns the same shape as create's response, current `status` and (deprecated but still populated) `connections[]` array with `id`, `client_id` (the connector instance UUID — stable per replica across reconnects, useful to dedupe if BodhiApp ever runs >1 connector for HA), `client_version`, `colo_name` (edge datacenter, e.g. `SJC`), `opened_at`, `origin_ip`.

A separate connections listing may exist as its own subresource in the current OpenAPI schema (`.../cfd_tunnel/{tunnel_id}/connections`) mirroring the one used for cleanup below; treat the `connections[]` field on the tunnel-get response as the authoritative source for a status UI — UNVERIFIED whether a standalone GET-connections endpoint returns materially different data.

### 1.6 Cleanup connections (before delete, or to force-disconnect a stale replica)

```
DELETE /accounts/{account_id}/cfd_tunnel/{tunnel_id}/connections[?client_id=<uuid>]
```
Omit `client_id` to drop *all* connectors; pass it to drop one specific replica (useful for BodhiApp's single-instance-per-app model: on tunnel *disable*, call this with no `client_id` to force-close before delete, rather than waiting for the connector process's own graceful shutdown to deregister — avoids a race where `DELETE .../cfd_tunnel/{id}` fails because a connection is still technically registered). Required token permission: `Cloudflare Tunnel: Edit` (equivalently, one of `Cloudflare One Connectors Write` / `Cloudflare One Connector: cloudflared Write`).

### 1.7 Delete the tunnel

```
DELETE /accounts/{account_id}/cfd_tunnel/{tunnel_id}?cascade=true
```
- Cascade semantics live in the **crate's own params struct**, not just prose: `cloudflare-rs`'s `DeleteTunnel::Params { pub cascade: bool }` (`cfd_tunnel/delete_tunnel.rs`) confirms `cascade` is a query param, default `false`. Per the docs: *"The tunnel must have no active connections."* Practically: run §1.6 cleanup first (or pass `cascade=true` to let the API do it), then delete. Response echoes the tunnel object with `deleted_at` populated.
- **Deleting the tunnel does not touch the DNS CNAME.** It becomes a dangling record pointing at a now-nonexistent `<uuid>.cfargotunnel.com` and must be separately removed (§1.8) — this is a common real-world foot-gun (see the `ytunnel` source in §3.3, which has a dedicated `delete_tunnel_dns_records` sweep specifically because Cloudflare doesn't cascade this).

### 1.8 Delete the DNS record

```
DELETE /zones/{zone_id}/dns_records/{record_id}
```
Needs the `record_id` captured at creation time (§1.3a) — if it wasn't persisted, recover it via `GET /zones/{zone_id}/dns_records?type=CNAME&content=<tunnel_id>.cfargotunnel.com` before delete.

### 1.9 Token-scope failure mode

A token missing the `Cloudflare Tunnel: Edit` or `DNS: Edit` scope on any of the above calls returns HTTP 403 with `errors: [{"code": 9109, "message": "Unauthorized to access requested resource"}]` (community-verified; also seen for tokens correctly-scoped but IP-restricted to a different address than the caller's). A malformed/expired token instead returns `{"code": 10000, "message": "Authentication error"}` at the "is this token valid at all" layer, before scope is even checked — useful to distinguish in error handling: 10000 → "token invalid, ask user to re-paste"; 9109 → "token valid but missing a permission, tell user which one." Neither numeric code is guaranteed stable by Cloudflare's docs (UNVERIFIED as a long-term contract) but both are widely observed in 2026 community reports and match the general Cloudflare API error-code space (`fundamentals/api/troubleshooting`).

Required token permission set (unchanged from `00-consolidated-research.md`, reconfirmed against `developers.cloudflare.com/fundamentals/api/reference/permissions/`): **Account → Cloudflare Tunnel → Edit**, **Zone → DNS → Edit**, **Zone → Zone → Read**. No Account Settings scope is needed for any call above.

## 2. Running the connector with a token

```bash
cloudflared tunnel run --token eyJhIjoiMDIzZ...
# or
TUNNEL_TOKEN=eyJhIjoiMDIzZ... cloudflared tunnel run
```
- Both forms are first-class; `TUNNEL_TOKEN` is what Cloudflare's own Docker examples use (`docker run cloudflare/cloudflared:latest tunnel --no-autoupdate run` with `-e TUNNEL_TOKEN=...`), `--token` is what interactive/scripted use favors. For a subprocess-managed connector (BodhiApp's pattern per `bodhiapp-cloudflare-tunnel-feasibility.md`), **prefer the env var** — it keeps the secret out of `ps`/process-list visibility, which `--token` does not.
- **Token contents, verified from `cloudflared` Go source** (`connection/connection.go`, `TunnelToken` struct):
  ```go
  type TunnelToken struct {
      AccountTag   string    `json:"a"`
      TunnelSecret []byte    `json:"s"`
      TunnelID     uuid.UUID `json:"t"`
      Endpoint     string    `json:"e,omitempty"`
  }
  ```
  So `base64 -d` on the token yields `{"a":"<account-id>","t":"<tunnel-uuid>","s":"<base64 tunnel_secret>"[,"e":"<optional edge endpoint override>"]}`. This is exactly the `AccountTag`/`TunnelID`/`TunnelSecret` triple used in a locally-managed `credentials.json`, plus an optional `e` (edge endpoint) field local credentials files don't carry — confirms the token is a self-contained superset of the local credentials file, not a separate mechanism.
- **Ingress is pulled from Cloudflare, not local disk, when `config_src=cloudflare`.** Verified in `orchestration/orchestrator.go`: the connector's `Orchestrator` starts at `currentVersion = -1` specifically ("Starting at -1 allows a configuration migration (local to remote) to override the current configuration") so that whatever version comes down from the control-plane RPC (`UpdateConfig(version, config)`, always ≥0) is applied unconditionally on first connect. There is no polling — configuration arrives pushed over the already-open QUIC/HTTP2 control stream, both at initial connect and on any subsequent dashboard/API edit (§1.2), applied live without a connector restart.
- **Log lines to match on for status detection** (subprocess stdout, for BodhiApp's process manager to parse): `cloudflared` logs a per-connection registration line and, on any live config push, `Updated to new configuration` at Info level with `version` and the full `config` JSON as structured fields (`orchestration/orchestrator.go`) — this is the reliable "config successfully applied" signal, more so than a generic "connected" line, since a connector can be network-connected but still running stale/invalid ingress. UNVERIFIED: exact wording of the initial connection-established log line (e.g. "Registered tunnel connection") — didn't locate its source file in this pass; treat any line containing `connIndex` and `connection=` fields as the per-edge-connection signal and confirm against a live run before wiring a parser to it.
- `cloudflared tunnel run` (no `--token`, using local `cert.pem` + `credentials.json` + `config.yml`) is the locally-managed counterpart — same binary, same subcommand, differs only in credential/config source.

## 3. Rust crates

### 3.1 Official `cloudflare` crate (cloudflare-rs) — verified against crates.io API + GitHub source

| | |
|---|---|
| crates.io latest published | **0.14.0**, published **2025-03-13** (per live `crates.io/api/v1/crates/cloudflare` response) |
| `master` branch (unreleased) | `Cargo.toml` on `master` already bumped to **0.14.1** — repo `pushed_at: 2026-04-23`, `archived: false`, 60 open issues. So there's a year-plus gap between the last crates.io release and ongoing (if slow) GitHub activity — pin carefully, don't assume `master` fixes are on crates.io. |
| License | BSD-3-Clause |
| Async runtime | `reqwest`-based; `reqwest = { version = "0.12.12", default-features = false, features = ["json", "multipart"] }` with a `default-tls` feature on by default and an opt-in `rustls-tls` feature (`cloudflare/Cargo.toml` on `master`) |
| MSRV | Not declared in `Cargo.toml`; edition `2021` (implies Rust ≥1.56, no explicit floor stated) — UNVERIFIED beyond that |

**`endpoints::cfd_tunnel` — exact file-by-file inventory** (`cloudflare/src/endpoints/cfd_tunnel/`, read directly off `master`):

| File | Endpoint | Method + path |
|---|---|---|
| `create_tunnel.rs` | `CreateTunnel` | `POST accounts/{account_id}/cfd_tunnel` — params `{name, tunnel_secret: Base64<Vec<u8>>, config_src: &ConfigurationSrc, metadata}` |
| `list_tunnels.rs` | `ListTunnels` | `GET accounts/{account_id}/cfd_tunnel` — filters: `name`, `uuid`, `is_deleted`, `existed_at`, `was_active_at`/`was_inactive_at`, prefix filters, pagination |
| `update_tunnel.rs` | `UpdateTunnel` | **`PATCH`** `accounts/{account_id}/cfd_tunnel/{tunnel_id}` (not PUT — doc comment is even copy-pasted from create and wrong, says "Create a Cfd Tunnel") — only re-sends `name`/`tunnel_secret`/`metadata`, **cannot touch `config_src`** |
| `delete_tunnel.rs` | `DeleteTunnel` | `DELETE accounts/{account_id}/cfd_tunnel/{tunnel_id}` with `Params { cascade: bool }` as a query param |
| `route_dns.rs` | `RouteTunnel` | `PUT zones/{zone_tag}/tunnels/{tunnel_id}/routes` — legacy Argo-Tunnel route API, **not** the `dns_records` CRUD API (own TODO comment: *"Exact same code as in argo_tunnel/route_dns.rs. Consider refactoring?"*) |
| `data_structures.rs` | `Tunnel`, `TunnelWithConnections`, `ActiveConnection`, `TunnelStatusType`, `ConfigurationSrc` (enum `Local`/`Cloudflare`), `RouteResult`/`DnsRouteResult` | types only |

**Confirmed MISSING** (verified by absence from the module + a `gh search code` over the whole repo returning zero hits for `configurations` or `token` anywhere near `cfd_tunnel`):
- **No `PUT .../cfd_tunnel/{id}/configurations`** — the crate cannot set ingress rules for a `config_src=cloudflare` tunnel at all. This is the single biggest gap: even if you use the crate for create/list/delete, you still need a raw HTTP call for §1.2.
- **No `GET .../cfd_tunnel/{id}/token`** — can't fetch the connector token through the crate; another raw call needed for §1.4.
- **No `DELETE .../cfd_tunnel/{id}/connections`** cleanup endpoint (§1.6).
- **No standalone tunnel-get** (`GET .../cfd_tunnel/{id}`) — only `ListTunnels` (filterable by `uuid`, which gets you the same result via a different path, but it's not the same typed endpoint).

**`endpoints::dns` DOES cover raw DNS-record CRUD** — `ListDnsRecords`, `CreateDnsRecord`, `UpdateDnsRecord`, `DeleteDnsRecord`, all under `zones/{zone}/dns_records[/{id}]`, matching §1.3a/§1.8 exactly. So for the "DNS half" of this feature the crate is complete; only the "tunnel half" (config + token + connections) is not.

**Verdict:** using the crate buys you 2 of 6 needed tunnel operations (create, delete) plus the full DNS-record CRUD, but you still hand-write the config PUT, token GET, and connections DELETE regardless — and you inherit its `reqwest 0.12.12` dependency (see below) plus its update cadence (>1 year between releases against ongoing but slow upstream churn) for a fraction of the surface. **Writing all ~6-8 calls as typed `reqwest` functions in-house is the better trade for BodhiApp** — no new dependency, no version-mismatch risk, and you were writing half of them by hand either way. This matches what the most actively-maintained 2026 third-party tool actually does (§3.3).

### 3.2 reqwest/rustls compatibility with BodhiApp

- BodhiApp workspace pins `reqwest = "0.13.2"` at the root (`Cargo.toml:113`), consumed via `workspace = true` in `crates/routes_app/Cargo.toml`, `crates/services/Cargo.toml:46,96`, `crates/server_core/Cargo.toml`, `crates/mcp_client/Cargo.toml`, `crates/llama_server_proc/Cargo.toml`, `crates/server_app/Cargo.toml`, `crates/lib_bodhiserver_napi/Cargo.toml` — no crate in the workspace sets `rustls-tls` or `native-tls` explicitly, so reqwest's own default (`default-tls`, i.e. platform native-tls/Schannel/Security-Framework/OpenSSL) applies workspace-wide.
- The `cloudflare` crate depends on **`reqwest 0.12.12`** (a different semver-major line than BodhiApp's `0.13.2`). Cargo resolves this by vendoring **two separate copies of reqwest** (and their respective hyper/h2/tokio-native-tls stacks) into the dependency graph — this compiles and runs fine (no hard conflict, Rust allows multiple major versions of a crate to coexist), but it duplicates the HTTP client stack in the binary: extra compile time, extra binary size, two TLS backends initialized independently. Both default to `default-tls` so there's no cross-linking of OpenSSL-vs-rustls symbols to worry about, just duplication.
- **Since the crate only covers 2 of 6 needed calls anyway (§3.1), skipping it avoids this duplication entirely** — write the Cloudflare REST calls with BodhiApp's existing `reqwest 0.13.2` client, no new TLS stack pulled in.

### 3.3 Other crates surveyed (crates.io, live as of 2026-09-15)

| Crate | Version / last publish | What it is | Relevant to BodhiApp? |
|---|---|---|---|
| `cloudflared` (crates.io) | 0.0.3, **2024-02-28** | Third-party toy wrapper, 167 LOC, ~4.5k downloads | No — abandoned, as `00-consolidated-research.md` already found; still true 19 months later |
| `cloudflare-rs` (crates.io — note: **different crate name** from `cloudflare`, easy to confuse) | 0.7.0, **2021-08-17** | An older/abandoned fork or predecessor under a name that collides with the GitHub repo's name | No — dead since 2021 |
| `cloudflare-quick-tunnel` | 0.3.1, 2026-05-14 | "Pure-Rust client for Cloudflare **quick tunnels**" | No — quick tunnels explicitly out of scope for this feature |
| `ytunnel` (github.com/yetidevworks/ytunnel) | 1.0.0, published to crates.io 2026-01-20, 262 downloads | **TUI CLI for managing named Cloudflare Tunnels with custom domains** — closest real-world analog to what BodhiApp needs | Not a library to depend on (it's a binary/CLI), but its `src/cloudflare.rs` is directly instructive — see below |
| `skyzen-cloudflare-admin` | 0.3.0, 2026-09-05 (days old) | "Shared Cloudflare API response envelope types and token-source configuration for building Cloudflare control-plane tooling" | Too new/unproven (76 total downloads, all "recent") to depend on; worth a second look in 6-12 months if it gains adoption, but not now |
| `lmrc-cloudflare` | 0.3.16, 2025-12-11 | Part of a broader "LMRC Stack" — DNS/zones/cache management, GitLab-hosted | Scoped to DNS/zones/cache, not tunnels — not applicable |
| `cftun` | 0.0.3, 2026-07-08 | "tiny Rust CLI that turns Cloudflare Tunnel into a free, persistent ngrok alternative" | Early/tiny (70 downloads); not surveyed in depth, but its existence confirms the pattern (small hand-rolled clients, not the official crate) is the norm for this niche |

**`ytunnel` source (read directly, `src/cloudflare.rs` on `master`) is the strongest real-world data point**: an actively-published, MIT-licensed 2026 tool built specifically to manage named Cloudflare Tunnels with custom domains **does not use the official `cloudflare` crate at all**. It hand-rolls a `CloudflareClient` over plain `reqwest 0.12` (with `rustls-tls`, not native-tls) implementing exactly the operations BodhiApp needs: `list_zones`, `list_tunnels`, `get_tunnel_by_name`, `create_tunnel` (POST `cfd_tunnel`, generates its own 32-byte `tunnel_secret`, writes a local `credentials.json`), `delete_tunnel`, `ensure_dns_record`/`get_dns_record`/`create_dns_record`/`update_dns_record`, `find_dns_records_for_tunnel`/`list_tunnel_dns_records`/`delete_dns_record_by_id` (a dedicated sweep specifically because tunnel delete doesn't cascade to DNS, confirming §1.7's caveat independently), plus a `tunnel_cname(id) -> "{id}.cfargotunnel.com"` helper matching §1.3 exactly. This is ~10 small `async fn`s over one `reqwest::Client`, not a large surface — corroborates the "write it in-house" recommendation in §3.1.

One divergence worth flagging: **`ytunnel` does not use `config_src=cloudflare` at all** — it creates the tunnel via API but then writes a *local* `credentials.json` and (per `src/daemon.rs`) always launches `cloudflared tunnel --config {path} run`, i.e. it stays on the locally-managed config path even though the tunnel object itself was created through the API. This is a hybrid nobody in this survey does the fully remote (`config_src=cloudflare` + `--token`) flow end-to-end in a shipped Rust tool as of this research — BodhiApp would be building an under-trodden path, which raises the value of testing §1.2/§2's config-push behavior directly against a real tunnel before committing to it, rather than trusting docs alone.

## 4. Locally-managed vs. remotely-managed for a desktop app

| | Locally-managed (`config_src=local`) | Remotely-managed (`config_src=cloudflare`) |
|---|---|---|
| What's stored where | `cert.pem` (origin cert from `cloudflared tunnel login`) + `<tunnel-id>.json` credentials + `config.yml` ingress — all on the user's disk, nothing in Cloudflare's control plane beyond the tunnel's existence | Only the tunnel object + its ingress config live in Cloudflare; the connector needs just the token (§2) |
| Auth tier required | **CLI login tier** — `cloudflared tunnel login` opens a browser OAuth flow against the Cloudflare dashboard, writes `cert.pem` scoped to one zone. No API token needed for tunnel lifecycle ops (`cloudflared tunnel create/route/run` all use `cert.pem`), though DNS record ops still eventually need either the CLI's own route command or dashboard access | **API token tier** — needs a scoped API token (§1.9) up front; no interactive login/browser flow, fully automatable, no `cloudflared tunnel login` step at all |
| Survives dashboard edits | **No** — if the user (or BodhiApp) edits ingress via the dashboard/API while `config_src=local`, `cloudflared` ignores it; the local `config.yml` is authoritative and dashboard edits to a locally-managed tunnel's ingress are rejected/no-effect. Any config change requires touching the local file and (for most changes) restarting the connector | **Yes, by design** — that's the whole point of `config_src=cloudflare`: edits via API/dashboard are pushed live to the running connector (§1.2/§2), no local file, no restart |
| Simplicity of cleanup | Messier — must also delete `cert.pem`/credentials/config.yml from disk on top of the API-side tunnel delete + DNS cleanup; state is split across two places (disk + Cloudflare) that can drift | Cleaner — only Cloudflare-side state to clean up (delete tunnel §1.7, delete DNS §1.8); no local files beyond an optionally-cached token string, which is safe to just discard |
| Fits BodhiApp's model | Requires either shelling out to `cloudflared tunnel login` and capturing a browser-based OAuth completion (awkward from a headless/managed backend flow), or asking the user to hand-run it once — a worse UX than "paste an API token" for the stated Tier-3 fallback in the feature brief | **Matches the stated Tier-3 plan exactly**: user pastes a scoped API token, BodhiApp drives everything via REST, runs `cloudflared tunnel run --token` (or the token-based Docker/env pattern) as a managed subprocess — no interactive browser step required from inside the app |

**Migrating a locally-managed tunnel to remote is a first-class, documented `cloudflared` operation — not a hack**: `cloudflared tunnel token <name-or-uuid>` (subcommand added in `cloudflared` **2022.3.0**, source at `cmd/cloudflared/tunnel/subcommands.go`, `buildTokenCommand`/`tokenCommand`) fetches the credentials token for *any* existing tunnel — including one originally created locally via `cloudflared tunnel login` + `tunnel create` — by calling the same `GET .../cfd_tunnel/{id}/token` endpoint as §1.4. Two things to keep straight about what this does and doesn't change:
- It always works regardless of the tunnel's `config_src`, because the token only carries auth credentials (`a`/`t`/`s` — §2), not ingress config.
- If the tunnel's `config_src` is still `"local"`, running with `--token` supplies auth but the connector still needs ingress rules from *somewhere* — practically, either keep a local `config.yml` around (defeats the "remote" point) or explicitly flip the tunnel to remote-managed by issuing a §1.2 `PUT .../configurations` call, which is the actual migration step; `cloudflared`'s own orchestrator code treats "a remote config arrives" as unconditionally authoritative (the `currentVersion = -1` starting point noted in §2) specifically to make a local→remote migration seamless whenever it happens.

**Recommendation for BodhiApp:** remotely-managed (`config_src=cloudflare` + token) is the right fit for the Tier-3 "paste an API token" path — it needs no interactive browser step, survives dashboard edits (relevant since Cloudflare's own dashboard will always be a valid alternate management surface users may touch), and cleans up to nothing but two API deletes. Tier-1 (drive the user's own `cloudflared` CLI, `tunnel login`) naturally produces a locally-managed tunnel instead — if Tier-1 is implemented, its output can still be upgraded to remote by adding one `PUT .../configurations` call after the fact, so the two tiers don't need fully separate lifecycle code paths, only a fork at "how did we get the tunnel + token" before converging on the same run-with-token step (§2).

## Sources

- Cloudflare API reference — Create Tunnel: https://developers.cloudflare.com/api/resources/zero_trust/subresources/tunnels/subresources/cloudflared/methods/create/
- Cloudflare API reference — Update Configuration: https://developers.cloudflare.com/api/resources/zero_trust/subresources/tunnels/subresources/cloudflared/subresources/configurations/methods/update/
- Cloudflare API reference — Get Token: https://developers.cloudflare.com/api/resources/zero_trust/subresources/tunnels/subresources/cloudflared/subresources/token/methods/get/
- Cloudflare API reference — Get Tunnel: https://developers.cloudflare.com/api/resources/zero_trust/subresources/tunnels/subresources/cloudflared/methods/get/
- Cloudflare API reference — Delete Tunnel: https://developers.cloudflare.com/api/resources/zero_trust/subresources/tunnels/subresources/cloudflared/methods/delete/
- Cloudflare API reference — Delete Connections: https://developers.cloudflare.com/api/resources/zero_trust/subresources/tunnels/subresources/cloudflared/subresources/connections/methods/delete/
- Cloudflare API reference — Create DNS Record: https://developers.cloudflare.com/api/resources/dns/subresources/records/methods/create/
- Cloudflare Tunnel common errors: https://developers.cloudflare.com/cloudflare-one/networks/connectors/cloudflare-tunnel/troubleshoot-tunnels/common-errors/
- Cloudflare Tunnel tokens reference: https://developers.cloudflare.com/tunnel/reference/tunnel-tokens/
- Cloudflare API token permissions: https://developers.cloudflare.com/fundamentals/api/reference/permissions/
- `cloudflare-rs` source (crate `cloudflare`), `cfd_tunnel` and `dns` endpoint modules, read at `master`: https://github.com/cloudflare/cloudflare-rs/tree/master/cloudflare/src/endpoints/cfd_tunnel and https://github.com/cloudflare/cloudflare-rs/tree/master/cloudflare/src/endpoints/dns
- `cloudflare-rs` `Cargo.toml` (master, unreleased 0.14.1, reqwest 0.12.12 dep): https://github.com/cloudflare/cloudflare-rs/blob/master/cloudflare/Cargo.toml
- crates.io API — `cloudflare` crate metadata (0.14.0, published 2025-03-13): https://crates.io/api/v1/crates/cloudflare
- crates.io API — `cloudflared` crate metadata (0.0.3, 2024-02-28): https://crates.io/api/v1/crates/cloudflared
- crates.io search for "cloudflare" (2026-09-15 snapshot used for §3.3 table): https://crates.io/api/v1/crates?q=cloudflare
- `cloudflared` Go source — `TunnelToken` struct: https://github.com/cloudflare/cloudflared/blob/master/connection/connection.go
- `cloudflared` Go source — tunnel-create error wrapping, `getTunnelTokenCredentials`: https://github.com/cloudflare/cloudflared/blob/master/cmd/cloudflared/tunnel/subcommand_context.go
- `cloudflared` Go source — `token` subcommand (local→remote token fetch, since 2022.3.0): https://github.com/cloudflare/cloudflared/blob/master/cmd/cloudflared/tunnel/subcommands.go
- `cloudflared` Go source — remote config push/orchestration: https://github.com/cloudflare/cloudflared/blob/master/orchestration/orchestrator.go
- `ytunnel` source (2026 third-party named-tunnel manager, hand-rolled reqwest client): https://github.com/yetidevworks/ytunnel/blob/master/src/cloudflare.rs and https://github.com/yetidevworks/ytunnel/blob/master/src/daemon.rs
- BodhiApp workspace `reqwest` pin: `Cargo.toml:113`; consumers: `crates/services/Cargo.toml:46,96`, `crates/routes_app/Cargo.toml:54,71`, `crates/server_core/Cargo.toml:25,48`, `crates/mcp_client/Cargo.toml:12`, `crates/llama_server_proc/Cargo.toml:17`, `crates/server_app/Cargo.toml:20`, `crates/lib_bodhiserver_napi/Cargo.toml:19`
- Cloudflare API troubleshooting (error code conventions): https://developers.cloudflare.com/fundamentals/api/troubleshooting/
- Community reports of error 81053 (CNAME conflict) and 9109 (unauthorized/scope): https://community.cloudflare.com/t/getting-this-error-while-adding-records-an-a-aaaa-or-cname-record-already-exists-with-that-host-code-81053/268212 and https://community.cloudflare.com/t/code-9109-unauthorized-to-access-requested-resource/428897
