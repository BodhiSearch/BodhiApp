# BodhiApp — self-hosted tunnel service: feasibility & recommendation

> **⚠️ SUPERSEDED / OUT OF SCOPE (as of 2026-09-15):** Named Cloudflare tunnels only is the locked scope as of 2026-09-15; quick tunnels, self-hosted frp, and Tailscale are out of scope — this doc's recommended-approach section ("Premium path (recommended)", the frp-based architecture) is retained for historical context only. See `docs/research/tunnel/README.md` for the current locked scope.

**Status:** research complete, no code landed.
**Context:** premium, BodhiApp-only tunnel offering on BodhiApp's own infrastructure (no Cloudflare/Tailscale SaaS dependency).

## Verdict

**Feasible and the most sensible path for a premium feature.** The cleanest building block is **frp** (Apache-2.0): it already provides the full "Cloudflare Tunnel-like" HTTP ingress stack — wildcard subdomains, custom domains, TLS, token/OIDC auth, bandwidth limits, and a dynamic management API — as a self-hosted server (`frps`) + client (`frpc`). Its **OIDC auth** can authenticate clients against BodhiApp's existing **Keycloak**, which fits the current architecture unusually well.

> Later decisions (see `07-frp-bodhi-compatibility-deep-dive.md` and `08-deployment-plan.md`): the tunnel domain is `getbodhi.io`, the routing is `<client-id>.getbodhi.io` subdomains, and edge auth is implemented by **forking frp to add per-request JWT validation** (signature + premium claim via cached Keycloak JWKS).

`zrok` is the main "turnkey product" alternative; `sish` is a good SSH-identity alternative; `rathole`/`chisel` are efficient L4 relays that would require BodhiApp to build its own HTTP/TLS/domain front-end.

## Recommended architecture

```
[ public internet ]
       │  https://<instance>.<bodhi-tunnel-domain>
       ▼
[ BodhiApp edge: frps + wildcard TLS ]
       │  (outbound connection initiated by client)
       ▼
[ frpc on user's desktop BodhiApp ] → http://127.0.0.1:1135
```

- **Edge (BodhiApp infra):** one or more `frps` instances on cheap VPS/cloud with a wildcard DNS record (`*.getbodhi.io` → edge IP) and a wildcard TLS cert (or `frps` + Caddy/Nginx for ACME).
- **Connector (user side):** `frpc` (or `tiny-frpc`) bundled/auto-downloaded by the desktop BodhiApp, configured by the BodhiApp backend to expose `127.0.0.1:<port>`.
- **Control plane (BodhiApp backend):** issues per-instance tokens and assigns subdomains; uses frp's **dynamic proxy management API (Store)** to create/update/delete tunnels; enforces premium entitlement.

## How this maps to "premium, BodhiApp-only"

- **Only BodhiApp clients:** frp token or OIDC auth + a server-side management layer that only provisions proxies for valid BodhiApp accounts/instances. The client cannot bind arbitrary hostnames — the backend generates the `frpc` config with an assigned subdomain/token.
- **Per-instance URL:** `subdomain = "<instance-id>"` → `https://<instance-id>.getbodhi.io`.
- **Premium custom domain:** `customDomains = ["<user-domain>"]` with the user pointing a CNAME at the edge; gate this behind the premium tier.
- **Quotas/cost control:** frp per-proxy `transport.bandwidthLimit`, plus your own metering at the edge/control plane. Relay bandwidth is the dominant cost; frp's QUIC/tcp-mux/compression keep it efficient.
- **SSE streaming:** frp forwards HTTP byte-for-byte, so streaming LLM responses work (unlike Cloudflare quick tunnels).

## Why frp over the alternatives

| Need | frp | zrok | sish | rathole |
|---|---|---|---|---|
| Unique subdomain URL | Yes | Yes (reserved share) | Yes | No (DIY) |
| Custom domain (premium) | Yes | Yes (frontend) | Yes | No (DIY) |
| TLS termination | Yes | Yes | Yes | No (DIY) |
| OIDC/Keycloak auth | **Yes (OIDC)** | Zero-trust (custom) | SSH keys | Per-service token |
| Dynamic management API | **Yes (Store)** | Yes (REST/SDK) | CLI/config | WIP |
| Bandwidth quotas | **Yes** | Partial | No | No |
| Stack/weight | Go, mature | Go + OpenZiti (heavy) | Go, single binary | Rust, ~500 KiB |

frp wins on breadth of built-in features that map 1:1 to the premium model, with the least custom glue. zrok is the fallback if you'd rather run a ready-made accounts/shares product than build the provisioning layer.

## Implementation surface (reusing BodhiApp's existing patterns)

- **New `TunnelService`** in `services`, with provider backends. For self-hosted frp: a `frpc` subprocess manager + a client that calls the BodhiApp control plane / frp Store API to provision proxies.
- **Control plane endpoints** (new, in `routes_app` under `admin_session_apis`/`manager_session_apis`): provision/revoke tunnel, get status/URL, set custom domain, quota status.
- **Settings keys** in `constants.rs`: `BODHI_TUNNEL_ENABLED`, `BODHI_TUNNEL_PROVIDER=frp|zrok|...`, `BODHI_TUNNEL_SERVER`, `BODHI_TUNNEL_TOKEN`, `BODHI_TUNNEL_DOMAIN`, etc.
- **Keycloak redirect-URI sync** (unchanged from prior plans): register `https://<instance>.getbodhi.io/ui/auth/callback` once (stable URL), or on tunnel (re)provision.
- **Feature flag:** gate on `SettingService::is_native()` (desktop on, non-desktop off by default), overridable via settings. Premium entitlement enforced by the backend, not the client.

## Cost model (minimum cost)

- Run `frps` (single Go binary) on one or two inexpensive VPS instances with bandwidth; a wildcard TLS cert via Let's Encrypt or Caddy.
- Relay bandwidth is the real cost driver — metering + `bandwidthLimit` per instance caps exposure. frp P2P (`xtcp`) can offload large bulk transfers off the relay, though normal API traffic still transits the edge.
- No per-domain SaaS fees; you own the subdomain wildcard (`*.getbodhi.io`) and any customer custom domains are just CNAMEs.

## Risks & mitigations

| Risk | Mitigation |
|---|---|
| frp v1 → v2 future rewrite | Pin a supported version; v1 keeps receiving fixes; abstract behind `TunnelService`. |
| Antivirus flags `frpc` | Document/sign binaries; prefer `tiny-frpc` or build from source with a BodhiApp-signed binary. |
| Abuse of public endpoints | OIDC/token auth + backend-issued provisioning only; per-instance bandwidth caps; kill switch; WAF in front of edge. |
| SSE/streaming must not be buffered | frp HTTP is byte-for-byte; add an e2e streaming test as part of the build. |
| Relay bandwidth cost | Meter + cap per instance; use QUIC/mux/compression; consider P2P for large transfers. |
| Wildcard TLS/DNS ops | Use Caddy/Nginx for ACME + wildcard DNS; automate cert renewal. |

## Open decisions

1. **frp vs zrok** for the first self-hosted provider (recommend frp for minimal glue; zrok if a turnkey accounts/shares product is preferred).
2. Where the frp **Store/dynamic API** calls live — in the Rust `TunnelService` (via HTTP to `frps` admin API) vs. in a separate BodhiApp control-plane service.
3. Whether to issue **per-instance tokens** or use **OIDC (Keycloak)** for `frpc` auth (OIDC is the more elegant fit given existing Keycloak).
4. Premium custom-domain DNS validation flow (CNAME check before enabling a custom domain).
