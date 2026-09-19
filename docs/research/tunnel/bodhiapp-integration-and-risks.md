# Integrate Cloudflare Remote Access with BodhiApp

This page maps the provider research to the current BodhiApp implementation. It records the active contracts, security boundaries, automated evidence, and remaining live validation.

- **Status:** Current snapshot
- **Last verified:** 2026-09-19
- **Implementation:** Cloudflare named tunnel backend and UI present; runtime-seam remediation in progress in the worktree

## Configuration contract

`BODHI_TUNNEL` follows the normal setting precedence and defaults to `SettingService::is_native()`. Do not add a separate multi-tenant hard exclusion.

Remote Access uses these durable settings:

- tunnel feature enablement
- explicit `cloudflared` path override
- explicit origin-certificate path override
- stable public hostname
- automatic reconnect preference

Local, LAN, and tunnel origins coexist. Never assign the tunnel hostname to `BODHI_PUBLIC_HOST`.

## Backend contract

`TunnelService` exposes status, setup, enable, preference update, authorization retry, reconnect, and disable operations. The admin API returns:

- capability and deployment availability
- binary discovery, source, version, and errors
- Cloudflare login and selected zone status
- connector state and public URL
- saved hostname and reconnect preference
- OAuth callback URL
- independent Keycloak synchronization state
- stable error codes for actionable failures

The current runtime work separates external effects behind three traits:

- `CloudflaredCli`: bounded one-shot CLI commands returning raw output
- `ConnectorProcess`: long-running connector spawn, output, stop, and wait
- `TunnelIo`: filesystem, binary discovery, Cloudflare HTTP, and loopback readiness I/O

Service code owns parsing and decisions. Tests can therefore drive lifecycle logic without shell scripts, real files, sockets, or sleeps.

## Request-origin and sign-in contract

Cloudflare terminates public TLS and forwards to BodhiApp over loopback HTTP. Cloudflare's [request-header reference](https://developers.cloudflare.com/fundamentals/reference/http-request-headers/#x-forwarded-proto) defines `X-Forwarded-Proto` as the visitor protocol. `cloudflared` [preserves the public `Host`](https://raw.githubusercontent.com/cloudflare/cloudflared/2026.9.1/ingress/origin_proxy.go) unless [`httpHostHeader`](https://developers.cloudflare.com/cloudflare-one/networks/connectors/cloudflare-tunnel/configure-tunnels/origin-parameters/#httphostheader) overrides it.

When the request arrives through the tunnel hostname, BodhiApp uses:

```text
https://public_hostname/ui/auth/callback
```

Local and LAN requests continue to use their own origin. Tunnel configuration must not rewrite global public-host settings.

## Keycloak synchronization

After the connector starts, BodhiApp updates the resource client's redirect URI through the Bodhi Keycloak extension. The gateway key is stable so a hostname change replaces the prior Cloudflare entry.

Synchronization has independent states:

- `syncing`: connector works; browser sign-in is not ready
- `synced`: browser sign-in can complete through the public hostname
- `unreachable`: network or server failure; retry can help
- `rejected`: Keycloak refused the update; permissions or deployment must change

A synchronization failure is degraded and non-fatal. API-key traffic and connector availability remain independent. Disable keeps the registration so re-enable is fast. `POST /tunnel/sync` retries a failed update.

## Frontend contract

The Remote Access page follows a progressive three-step flow:

1. detect and validate `cloudflared`
2. detect and validate Cloudflare sign-in
3. choose the subdomain, confirm exposure, and manage the connector

Completed steps fold. Active tunnel settings lock until Remote Access is off. The page displays create progress, DNS conflict confirmation, Keycloak degraded states, public URL actions, reconnect preference, and contextual troubleshooting.

The state selector in `design/Remote-Access.html` is a design-review harness. It is not a production component.

## Security boundaries

- The tunnel exposes the web UI and API surface to the internet
- Bodhi authentication remains required where routes require it
- Cloudflare Access is not configured by this feature
- DNS overwrite requires explicit confirmation
- Certificate contents, connector tokens, Keycloak secrets, and bearer tokens must not enter logs or API responses
- The connector token belongs in the child environment, not argv
- `cert.pem` remains an administrator-managed input
- Cloudflare and Keycloak error bodies require redaction before logging or display

## Edge behavior and operator guidance

[Cloudflare's Tunnel troubleshooting contract](https://developers.cloudflare.com/cloudflare-one/networks/connectors/cloudflare-tunnel/troubleshoot-tunnels/common-errors/#cloudflare-tunnel-is-buffering-my-streaming-response-instead-of-streaming-it-live) says responses stream when the origin sends `Content-Type: text/event-stream`. BodhiApp already sets this header.

Operators still need to account for these Cloudflare behaviors:

- Cloudflare's [proxy read timeout](https://developers.cloudflare.com/fundamentals/reference/connection-limits/#between-cloudflare-and-origin-server) is 125s by default; exceeding it returns `524`, and only Enterprise zones can configure it
- [Maximum upload size](https://developers.cloudflare.com/network/maximum-upload-size/#upload-limits) is 100 MB on Free and Pro, 200 MB on Business, and up to 5 GB on Enterprise
- [Bot Fight Mode](https://developers.cloudflare.com/bots/get-started/bot-fight-mode/#considerations) may challenge API traffic and cannot be bypassed with WAF custom rules on that product tier
- Cloudflare documents [WARP and Zero Trust egress](https://developers.cloudflare.com/sandbox/api/tunnels/#limitations) as possible blockers for the connector handshake

Do not present these as BodhiApp configuration errors. Link to the relevant Cloudflare setting or limit.

## Process-lifecycle risks

Graceful shutdown stops the connector before BodhiApp's HTTP listener exits. Forced termination remains platform-specific:

- Linux can bind child death to parent death
- Windows can use a Job Object
- macOS has no equivalent parent-death primitive; reap a recorded stale child on the next launch

Do not restart a live child after one failed probe. `cloudflared` has its own reconnect behavior, and unnecessary respawn disrupts traffic.

## Recorded automated evidence

The implementation handoff records:

- focused settings, certificate parsing, hostname, error-code, runtime, and Keycloak sync tests
- `routes_app` and `server_app` test suites
- frontend state and component tests
- TypeScript checking and production build
- OpenAPI and generated client refresh
- formatting and diff checks

These checks were not rerun for this documentation rewrite. The recorded evidence validates local logic, not Cloudflare or deployed Keycloak behavior.

## Live validation still required

Before production sign-off, run:

1. create a tunnel on a clean Cloudflare account and reuse it after deleting local scratch state
2. verify incremental OpenAI, Anthropic, and Gemini streaming through the public hostname
3. complete browser sign-in through deployed Keycloak
4. force Keycloak unreachable and rejected states, then retry
5. access the instance from a second network
6. restart BodhiApp and confirm the same public hostname reconnects once
7. test sleep, wake, network change, graceful exit, crash, and stale-child cleanup on all desktop platforms
8. verify DNS replacement and old-host behavior
9. test representative plan limits and Bot Fight Mode behavior

Keep owner-run external validation separate from local automated results.

## Repository map

| Area | Current paths |
|---|---|
| Tunnel service and runtime | `crates/services/src/tunnels/` |
| Settings | `crates/services/src/settings/` |
| Keycloak client update | `crates/services/src/auth/auth_service.rs` |
| Admin routes | `crates/routes_app/src/tunnels/` |
| Tunnel-aware sign-in | `crates/routes_app/src/auth/routes_auth.rs` |
| Startup reconnect | `crates/server_app/src/serve.rs` |
| Frontend route and state model | `crates/bodhi/src/routes/tunnels/` |
| Frontend API hooks | `crates/bodhi/src/hooks/tunnels/` |
| Interactive design reference | `design/Remote-Access.html`, `design/tunnels/` |

## Related research

- [`named-tunnel-operating-model.md`](named-tunnel-operating-model.md)
- [`quick-tunnel-decision.md`](quick-tunnel-decision.md)
- [Cloudflare connection limits](https://developers.cloudflare.com/fundamentals/reference/connection-limits/)
- [Cloudflare Bot Fight Mode](https://developers.cloudflare.com/bots/get-started/bot-fight-mode/)
