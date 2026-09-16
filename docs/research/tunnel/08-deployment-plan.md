# 08 — Deployment plan: DNS, regional frp, edge JWT, observability

Deployment and architecture study for the self-hosted tunnel service. Covers DNS on Cloudflare, regional/geo-routing, frp deployment + fork points, JWT edge auth, observability/metering, and licensing. Confirms the routing strategy (`<client-id>.getbodhi.io`) and resolves open deployment questions.

## 1. DNS & domain (`getbodhi.io` on Cloudflare)

### Wildcard record → frp

- Create a **wildcard record** `*.getbodhi.io` (A or CNAME) pointing at the frps gateway.
- Cloudflare supports wildcard records on **all plans**, and since May 2022 **wildcard proxying (orange cloud) is available on the free plan** too.
- Two modes:
  - **DNS-only (grey cloud):** `*.getbodhi.io → frps IP`. frps terminates TLS itself with a wildcard cert. Recommended for this feature — keeps Cloudflare out of the plaintext (LLM traffic) and avoids any CDN buffering on SSE.
  - **Proxied (orange cloud):** Cloudflare terminates TLS and forwards to frps origin over HTTP. Convenient (DDoS/WAF), but Cloudflare sees plaintext and may touch streaming. If used, enable WebSockets and verify SSE passthrough.

### Per-client records vs wildcard (door-open design)

frp has **no frps-to-frps federation** — each frpc binds to one frps. So a given `<client-id>` must resolve to the frps where that client's frpc is connected. This rules out "one wildcard → arbitrary nearest region" for frp.

Recommended, future-proof pattern:

1. **Wildcard fallback** `*.getbodhi.io → default-region frps` (one record).
2. **Specific records** `<client-id>.getbodhi.io → home-region frps` created/updated by the control plane via Cloudflare API when a client is homed to a non-default region.

DNS resolution always prefers an exact-name record over a wildcard, so this "wildcard default + specific override" gives:
- **Single region start:** only the wildcard exists; everything → one frps.
- **Home-region expansion later:** control plane adds specific records per client; no architecture change.
- **Global reach (multi-frpc) later:** control plane publishes a client to multiple regions and swaps in geo-DNS; door remains open.

### Regional stickiness / geo-DNS options

Clarification: **GoDNS** (`TimothyYe/godns`) is a *dynamic DNS (DDNS) client*, not geo-DNS — it updates a home IP in DNS, which is not what we need (frps has static IPs). The real geo-DNS options:

| Option | Type | Geo/latency | Cost | Notes |
|---|---|---|---|---|
| **Cloudflare DNS + Load Balancing** | Managed | Geo steering (country/region; DC-level is Enterprise) | Free DNS; LB add-on ~$5/mo + queries | Best default — already using Cloudflare; great API for per-client records |
| **AWS Route 53** | Managed | **Latency-based** + geolocation | ~$0.50/zone/mo + queries | Latency routing is arguably better than geo for "nearest"; strong API |
| **Constellix** | Managed | Geo + failover | Cheap (geo-focused) | Budget geo-DNS specialist |
| **NS1 (IBM)** | Managed | Geo/filters | Enterprise-priced | Overkill here |
| **gdnsd** | **Self-hosted** | Geo plugin (MaxMind GeoIP) + weighted/failover | Free (your infra) | "Own everything" authoritative DNS; more ops |
| **Bunny DNS** | Managed | Some geo | Cheap | Simpler alternative |

**Recommendation:** use **Cloudflare (DNS-only) + control-plane API** for per-client records now. Revisit **Route 53 latency routing** (or gdnsd if self-hosting DNS) only if/when you move to the "global reach (multi-frpc)" topology — at that point latency-based beats geo for "nearest," and it's a clean add-on, not a rewrite.

## 2. frp regional deployment & performance

### How frp serves (source-level)

- frp separates **control connections** (reliable/secure, frpc↔frps metadata) from **work connections** (per-request data), multiplexed with yamux. frps is lightweight and handles thousands of concurrent connections on 2 vCPU/4 GB.
- HTTP vhost routing lives in:
  - `pkg/util/vhost/http.go` — `HTTPReverseProxy.ServeHTTP`, routing, Basic Auth (`checkRouteAuthByRequest`).
  - `pkg/util/vhost/vhost.go` — `RouteConfig` (Domain/Location/RewriteHost/Username/Password/Headers/RouteByHTTPUser).
  - `pkg/util/vhost/https.go` — SNI-based TLS muxer for HTTPS vhost.
  - `server/proxy/http.go` / `server/proxy/https.go` — proxy registration and work-connection setup.
- Routing is by **Host (exact or wildcard) + path prefix (`locations`) + optional HTTP Basic user**. Unmatched paths → 404. This is the "enumerated endpoints" mechanism.

### Single-region start → expand

- **Start:** one frps (or a pair for HA) in one region; wildcard `*.getbodhi.io` → it.
- **Expand to home-region-per-instance:** add frps in more regions; control plane assigns each client's frpc to its nearest frps and writes a specific `<client-id>` DNS record → that region.
- **Expand to global reach (end-user nearest):** run one frpc per region per instance (multi-frpc) and switch DNS to geo/latency routing. This is additive; no re-architecture.

### Performance/HA notes

- frp **HA** = multiple frpc backends behind one frps (`loadBalancer.group` + health checks) — it is backend HA, not frps clustering. frps HA is achieved via DNS failover/anycast + running frpc against multiple frps if needed.
- Enable frp's Prometheus endpoint (`enablePrometheus`) and dashboard for per-proxy traffic/connection metrics.

## 3. Routing strategy (confirmed: subdomain)

- **`<client-id>.getbodhi.io` (subdomain) — correct.** Routing is by Host header, independent of auth. Public (no-JWT) endpoints still route correctly.
- `tunnel.getbodhi.io` + JWT-dispatch — **incorrect**: public endpoints without a JWT can't be routed, and it couples routing to auth.
- `tunnel.getbodhi.io/{client-id}/` path prefix — **incorrect**: BodhiApp's endpoints are already path-based (`/v1/*`, `/bodhi/v1/*`), so a client-id path prefix collides and would require path rewriting (breaks streaming/SSE and absolute URLs).

## 4. Edge JWT (fork frp)

Decision: **fork frp and add per-request JWT validation** in the HTTP vhost path.

### Where to change

- Add a `JWTConfig` to `RouteConfig` (`pkg/util/vhost/vhost.go`): issuer, JWKS URL, audience, required claim, cache TTL, optional public-path allow-list.
- Inject validation in `HTTPReverseProxy.ServeHTTP` (`pkg/util/vhost/http.go`) **before** routing/forwarding, and in the HTTPS listener handler after TLS termination.
- Use Go's `github.com/golang-jwt/jwt` + a cached JWKS (fetch on `kid` miss + hourly refresh).

### JWT check logic

- Extract `Authorization: Bearer <token>`.
- Verify signature against cached **Keycloak JWKS** (`{auth_url}/realms/{realm}/protocol/openid-connect/certs`).
- Validate `iss` (issuer), `aud`, and `exp`.
- Check the **premium subscription claim** (e.g. `bodhi_tunnel_subscription == "active"` or `subscription_tier == "premium"`), injected by Keycloak via a client-scope/user-attribute mapper.
- **Skip JWT on public paths** (allow-list): `/ping`, `/health`, and any explicitly public BodhiApp endpoints (`/bodhi/v1/setup`, auth initiate/callback, etc.). Routing for those still uses Host + path.
- Keycloak is hosted in North America (Railway); JWKS caching means no per-request cross-region dependency. Hourly refresh covers rotation; a `kid` miss triggers immediate refetch.

> Note: frp had a past auth-bypass advisory around `routeByHTTPUser` (GHSA-pq96-pwvg-vrr9). Don't use `routeByHTTPUser` for security; our JWT check is independent and sits before routing.

## 5. Observability & metering (abuse detection)

- **frp metrics:** enable `enablePrometheus` on frps; scrape per-proxy traffic bytes, connection counts, and request rates.
- **frp dashboard** (optional, cached 7-day monitor data) + **Store API** for programmatic proxy status.
- **Per-proxy bandwidth limits** (`transport.bandwidthLimit`) as hard enforcement.
- **Control-plane metering:** aggregate Prometheus metrics per client-id; alert on burst/bandwidth/anomalous paths. The forked JWT gives per-request identity for audit.
- **Premium entitlement:** enforced at edge via the JWT claim; re-checked by BodhiApp's own middleware (defense in depth).

## 6. License

- **frp is Apache-2.0** (not MIT), a permissive license.
- Commercial use, modification, and private redistribution are **allowed with no payment**. Forking to add JWT is permitted.
- If you **distribute** the modified frp publicly, preserve copyright/license/attribution notices. For a private premium service (frps hosted by you, frpc distributed to BodhiApp users), you are **not** required to open-source your fork; just keep the Apache-2.0 notices with the binaries you ship.

## 7. What to prepare (deployment checklist)

1. Register/point `getbodhi.io` nameservers to Cloudflare.
2. Create wildcard `*.getbodhi.io` (DNS-only) → frps IP; provision wildcard TLS cert (Let's Encrypt DNS-01 / Caddy).
3. Deploy frps (forked) in one region (2 nodes for HA); enable Prometheus + dashboard.
4. Build the BodhiApp control-plane piece: assign client-id → subdomain + token; generate frpc config; call Cloudflare API for specific DNS records (later); call frp Store API for provisioning.
5. Add Keycloak client-scope mapper to inject the premium subscription claim into access tokens.
6. Fork frp: add `JWTConfig` + validation + public-path skip; CI build + sign the frpc/frps binaries.
7. Configure frpc `locations` allow-list (`/v1`, `/bodhi/v1`, `/ping`, `/health`) + token/OIDC control-plane auth.
8. Wire BodhiApp: set `public_server_url()` to `https://<client-id>.getbodhi.io`; register the redirect URI in Keycloak.
9. Metering: scrape frps Prometheus; alert on abuse; per-proxy bandwidth caps.

## 8. Resolved vs. remaining

Resolved: domain (separate `getbodhi.io`), routing (subdomain), edge JWT (fork frp + cached JWKS + premium claim), regional start (single region, wildcard default + per-client override later), observability (Prometheus + limits), license (Apache-2.0, free commercial use).

Remaining to confirm at implementation:
- Exact TLS termination point for the JWT fork (frps `vhostHTTPSPort` vs Cloudflare proxy) — recommend frps DNS-only TLS termination.
- The precise Keycloak claim mapper (user attribute vs client scope) and its claim name.
- Whether to also run frp's OIDC control-plane auth against Keycloak (recommended) or per-instance tokens.
