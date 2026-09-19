# services/src/tunnels — PACKAGE.md

See [CLAUDE.md](CLAUDE.md) for the seam's design rules.

## File index

| File | Contents |
|------|----------|
| `runtime.rs` | `CloudflaredCli`, `ConnectorProcess`, `ConnectorHandle`, `TunnelIo`, `RawOutput`, `HttpRequest`/`HttpResponse`, `TunnelRuntimeError`. All traits carry `#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]`. |
| `runtime_impl.rs` | `SystemCloudflaredCli`, `SystemConnectorProcess`, `SystemConnectorHandle`, `SystemTunnelIo` — the only platform-specific code in the feature. |
| `service.rs` | `TunnelService`, `DefaultTunnelService`, `TunnelError`, and all parsing and lifecycle logic. |
| `tunnel_objs.rs` | Setup diagnostics, enable/preferences requests, the admin status snapshot, and `RemoteAccessInfo`. |

## Tunnel identity

Named `bodhi-app-tunnel-<uuid>`, derived from the instance's OAuth client id with
`RESOURCE_CLIENT_PREFIX` stripped, resolved once and memoised. Stable across restarts so the tunnel
is reused, and unique per instance so two machines sharing a Cloudflare account cannot adopt each
other's.

## Credentials

Nothing durable is persisted. The origin certificate travels to `cloudflared` as
`TUNNEL_ORIGIN_CERT` and the connector's run token as `TUNNEL_TOKEN`, both in the child's
environment and never on its argv. The one file written is the scratch JSON that
`cloudflared tunnel create --credentials-file` requires: created under `$BODHI_HOME/tmp/tunnels`,
never read back, and discarded whether `create` succeeded or failed.

## Status, and the anonymous counterpart

`status()` is admin-only and may probe the binary and the Cloudflare API. Binary and zone probes sit
behind TTL caches keyed by a content hash of their inputs; the certificate parse and the `/ready`
probe stay live.

`remote_access_info()` is served on the anonymous `/bodhi/v1/info` and is built from settings plus
in-memory runtime alone. It must never spawn `cloudflared` or call the Cloudflare API: that endpoint
is unauthenticated, runs on every bootstrap, and becomes internet-reachable through the tunnel
itself, so anonymous traffic must not be able to drive a subprocess or a third-party API call. A test
pins this. `RemoteAccessInfo` also has no field able to carry a path, the zone, or raw stderr, so the
admin/anonymous split holds by construction rather than by remembering to strip fields.

## Supervision

A supervisor task checks for connector exit every `supervisor_interval` and re-runs the readiness
probe every `ready_probe_interval`. Without the latter, only an admin loading `GET /bodhi/v1/tunnel`
ever promoted a connector from `connecting` to `connected`. State changes are published on a
broadcast channel so a caller can await them instead of polling.

## Authorization-server sync

Sync state distinguishes `unreachable` (transport failure or 5xx — retrying is the remedy) from
`rejected` (4xx or refused credentials — permissions must change first). Changing the subdomain
deletes the previous CNAME best-effort, and only when that record points at this tunnel. Turning the
tunnel off deliberately leaves both the DNS record and the redirect registration in place.

## Settings

`BODHI_TUNNEL` follows normal setting precedence and defaults to `SettingService::is_native()`. The
saved binary path, certificate path, hostname, and auto-reconnect preference are all separate from
`BODHI_PUBLIC_HOST` — never set that for a tunnel, since loopback, LAN, and tunnel origins coexist.
