# Operate a named Cloudflare Tunnel for BodhiApp

This page defines the supported Cloudflare operating model for BodhiApp Remote Access. It covers prerequisites, credentials, commands, process supervision, and provider limits.

- **Status:** Current
- **Last verified:** 2026-09-19
- **Reference version:** `cloudflared 2026.9.1`

## Prerequisites

The administrator needs:

- a Cloudflare account
- a domain configured as a Cloudflare zone
- permission to manage Cloudflare Tunnels and route a public hostname
- a supported `cloudflared` binary installed on the BodhiApp machine
- a successful `cloudflared tunnel login`

[Cloudflare makes the Tunnel connector available at no cost](https://blog.cloudflare.com/tunnel-for-everyone/). Paid plans affect contractual service levels and some general Cloudflare limits.

## Credential model

`cloudflared tunnel login` opens Cloudflare's browser flow and writes `cert.pem`. Cloudflare documents the certificate as the account-wide credential for creating, routing, listing, and deleting tunnels. Its [source representation](https://raw.githubusercontent.com/cloudflare/cloudflared/2026.9.1/credentials/origin_cert.go) contains `accountID`, `zoneID`, `apiToken`, and an optional API endpoint.

For a zone owner with the required role, the same certificate supports the selected CLI flow:

- list or create the tunnel
- route a hostname through the tunnel-specific route endpoint
- fetch a connector token

BodhiApp stores only the configured certificate path. It does not copy the certificate contents into settings.

The connector token must not appear in argv, logs, or an API response. Pass it through `TUNNEL_TOKEN` in the child environment.

[Cloudflare's tunnel permissions reference](https://developers.cloudflare.com/cloudflare-one/networks/connectors/cloudflare-tunnel/do-more-with-tunnels/local-management/tunnel-permissions/) describes the certificate and account-role requirements.

## Binary policy

BodhiApp detects `cloudflared` in this order:

1. the explicit `BODHI_TUNNEL_CLOUDFLARED_PATH` setting
2. the process `PATH`
3. common platform install locations

The current BodhiApp minimum is `2025.2.0`; this is an application policy in the [tunnel service](../../../crates/services/src/tunnels/service.rs), not a Cloudflare plan limit. Recheck Cloudflare's supported-version policy before changing it.

BodhiApp guides installation but does not silently download or update `cloudflared`. Package-manager installation remains preferable because the operating system handles signatures, permissions, and updates.

The binary uses the [Apache License 2.0](https://raw.githubusercontent.com/cloudflare/cloudflared/2026.9.1/LICENSE).

Use [Cloudflare's download page](https://developers.cloudflare.com/cloudflare-one/networks/connectors/cloudflare-tunnel/downloads/) for current installation commands.

## Provisioning lifecycle

[Cloudflare documents the generic local-tunnel commands](https://developers.cloudflare.com/cloudflare-one/networks/connectors/cloudflare-tunnel/do-more-with-tunnels/local-management/tunnel-useful-commands/). The table below records BodhiApp's selected flags and idempotent lifecycle:

| Stage | Command or action | Durable result |
|---|---|---|
| Sign in | `cloudflared tunnel login` | `cert.pem` under the administrator's profile |
| Resolve | `cloudflared tunnel list --output json -n tunnel_name` | Existing tunnel ID, if present |
| Create | `cloudflared tunnel create --output json --credentials-file scratch_path tunnel_name` | New Cloudflare tunnel ID |
| Discard scratch | Delete `scratch_path` after create | No runtime dependency on a credentials file |
| Route DNS | `cloudflared tunnel route dns [--overwrite-dns] tunnel_id hostname` | Stable proxied hostname |
| Fetch token | `cloudflared tunnel token tunnel_name` | Connector token on stdout |
| Run | Start the connector with `TUNNEL_TOKEN` | Active outbound connection |

The tunnel name derives from the Bodhi resource-client identifier. This makes it stable per instance and avoids collisions between BodhiApp installations that share a Cloudflare account.

Changing the subdomain reuses the same tunnel. BodhiApp must not create a new tunnel per hostname. Replacing an existing DNS record requires explicit administrator confirmation.

The create command needs a temporary credentials path because `cloudflared` writes one during creation. BodhiApp removes that file and fetches a fresh connector token for each run.

## Connector invocation

The selected connector shape is:

```text
cloudflared tunnel \
  --no-autoupdate \
  --metrics 127.0.0.1:0 \
  --loglevel info \
  --protocol auto \
  run \
  --url http://127.0.0.1:bodhi_port
```

Set `TUNNEL_TOKEN` in the child environment. Do not pass `--origincert` or a token on the command line.

`--metrics 127.0.0.1:0` lets the operating system choose an unused loopback port. Parse that address once from startup output, then use the HTTP readiness endpoint.

## Readiness and supervision

Poll `http://metrics_address/ready` as the connector-state source. The [`cloudflared` readiness handler](https://raw.githubusercontent.com/cloudflare/cloudflared/2026.9.1/metrics/readiness.go) reports whether an edge connection is ready:

- `200`: at least one active edge connection
- `503`: the process is alive but has no ready edge connection

Do not use connection log text as the steady-state signal. Keep logs for diagnostics and redact secrets before storage or display.

The supervisor must:

- serialize enable and disable operations
- distinguish intentional stop from unexpected exit
- drain stdout and stderr so the child cannot block
- treat transient readiness loss as degraded before restarting a live child
- stop and wait for the connector during graceful shutdown
- make one asynchronous reconnect attempt after BodhiApp becomes ready
- expose reconnect failure without blocking application startup

`cloudflared` handles `SIGINT` and `SIGTERM` as graceful shutdown requests. The local 2026.9.1 CLI reports a default 30s grace period, and [cloudflared issue #198](https://github.com/cloudflare/cloudflared/issues/198) documents that a second signal forces shutdown.

## Stable URL and DNS behavior

A named tunnel uses a stable `<label>.<zone>` hostname. Cloudflare's [Quick-versus-Named reference](https://developers.cloudflare.com/sandbox/api/tunnels/#how-they-differ-from-quick-tunnels) records hostname stability, account requirements, and single-label Universal SSL coverage.

Turning Remote Access off stops the connector but does not delete:

- the Cloudflare tunnel
- the DNS route
- the saved hostname
- the Keycloak gateway registration

This preserves fast restart and reconnect behavior. Removing Cloudflare-side resources is a separate administrative action.

## Provider limits

Cloudflare documents these account-level Tunnel quotas:

- 1,000 tunnels per account
- 1,000 routes per account, shared with Cloudflare Mesh
- 25 active `cloudflared` replicas per tunnel

Enterprise accounts can request some quota increases. These resource quotas are separate from plan-dependent HTTP limits.

See [Cloudflare One account limits](https://developers.cloudflare.com/cloudflare-one/account-limits/) for current values.

## Primary references

- [Cloudflare Tunnel documentation](https://developers.cloudflare.com/cloudflare-one/networks/connectors/cloudflare-tunnel/)
- [Locally managed tunnel permissions](https://developers.cloudflare.com/cloudflare-one/networks/connectors/cloudflare-tunnel/do-more-with-tunnels/local-management/tunnel-permissions/)
- [`cloudflared` downloads](https://developers.cloudflare.com/cloudflare-one/networks/connectors/cloudflare-tunnel/downloads/)
- [`cloudflared` repository](https://github.com/cloudflare/cloudflared)
- [Cloudflare One account limits](https://developers.cloudflare.com/cloudflare-one/account-limits/)
