# 07 — frp ↔ BodhiApp compatibility deep dive

Deep dive on how the proposed tunnel software (**frp**) maps onto BodhiApp's requirements: domain naming, predictable `<client-id>` entry points, path enumeration, and JWT-at-edge. Documents frp's auth/extensibility and the final decision to **fork frp for edge JWT** (see also `08-deployment-plan.md`).

## 1. frp capability matrix vs. requirements

| Requirement | frp support | How | Gap? |
|---|---|---|---|
| TLS termination at edge | **Yes** | `vhostHTTPSPort` + TLS cert; or `https2http`/`https2https` plugins | None |
| Wildcard subdomains | **Yes** | `subdomain` under `subdomainHost`, or wildcard `customDomains` (e.g. `*.getbodhi.io`) | None |
| Predictable `<client-id>` hostname | **Yes** | `subdomain = "<client-id>"` → `<client-id>.getbodhi.io` | client-id must be DNS-safe (see §3) |
| Custom domain (premium) | **Yes** | `customDomains = ["user.domain"]` | None |
| Path enumeration / allow-list | **Yes** | `locations = ["/v1", "/bodhi/v1", "/ping", "/health"]`; unmatched paths → 404 | None (confirm exact 404 semantics at implementation) |
| Inbound HTTP auth (coarse) | **Partial** | `httpUser`/`httpPassword` (Basic Auth only) | **No JWT/OIDC for inbound requests → addressed by fork** |
| Control-plane auth (frpc↔frps) | **Yes** | `auth.method = token` or `oidc` (client credentials) | None — can reuse Keycloak |
| Per-proxy bandwidth limits | **Yes** | `transport.bandwidthLimit` | None |
| Runtime provisioning | **Yes** | Dynamic proxy management (Store API), dashboard, admin UI | None |

**Bottom line:** frp alone covers TLS, wildcard subdomains, path allow-listing, and control-plane OIDC. The one gap — per-request JWT validation — is closed by forking frp (below).

## 2. Why fork frp (instead of a separate edge gateway)

We need per-request JWT validation at the edge (verify signature + premium claim), which frp does **not** provide natively. The options were:

1. **Front frp with a separate auth gateway** (Caddy/Nginx/Traefik + oauth2-proxy) — works, but adds another component/hop and cost.
2. **Do JWT only in BodhiApp** — zero extra infra, but unauthenticated traffic still reaches the app (weaker "block at tunnel layer").
3. **Fork frp and add JWT middleware** (chosen) — JWT enforcement lives inside frp's HTTP vhost path; no extra component, no extra hop.

**Decision:** fork frp. BodhiApp's own JWT middleware (`validate_bearer_token` / `handle_external_client_token`) remains a second layer (defense in depth), but the fork drops invalid/expired/non-premium requests before they reach the app.

## 3. Is frp extensible? (and is forking the only way?)

frp has three extension surfaces, plus source modification:

1. **Client plugins** (compiled-in Go): `http_proxy`, `socks5`, `static_file`, `unix_domain_socket`, `https2http`, `https2https`, `http2https`. These transform the *local* (frpc) side of the connection — not per-request edge auth.
2. **Server plugins** (`doc/server_plugin.md`, `gofrp/plugin`): frps calls an **external HTTP webhook** on control-plane events (login, new proxy, ping). Useful for connection-time authorization/ops, **not** per-request HTTP auth.
3. **Dynamic management (Store API)** + **feature gates**: runtime proxy CRUD and experimental flags.
4. **Modify frp source** (Go): add a custom HTTP handler/middleware for per-request JWT/OIDC inside frp. This is the chosen path — frp is well-structured Go, so the change is localized (see §6 and `08` §4).

**Conclusion:** for per-request JWT at the edge, frp's built-in plugin hooks don't cover it — the chosen approach is to **fork frp and add the JWT check** in the HTTP vhost path. Basic Auth + path allow-list + TLS remain native.

## 4. Domain naming (confirmed: separate domain `getbodhi.io`)

Recommendation — isolate tunnels on a **separate domain** (registered on Cloudflare):

- **Prod:** `<client-id>.getbodhi.io`
- **Dev:** `<client-id>.dev.getbodhi.io`

Mechanics:

- **Wildcard DNS:** `*.getbodhi.io` → gateway IP(s); `*.dev.getbodhi.io` → dev gateway.
- **Wildcard TLS:** one wildcard cert per label (`*.getbodhi.io`, `*.dev.getbodhi.io`) via Let's Encrypt DNS-01 / Caddy.
- **No conflict with existing APIs:** `api/id/cloud.getbodhi.app` live on a different domain entirely, so there is zero DNS/TLS/cookie interaction. (A `*.tunnel.getbodhi.app` subdomain also would not collide with `api/id/cloud.getbodhi.app`, but a separate domain removes the shared-cookie and reputational blast radius.)

### client-id → subdomain rule

Keycloak client-ids are not guaranteed DNS-safe. Normalize deterministically:

- If the client-id is a UUID/hex, use it directly (already `[a-z0-9-]`).
- Otherwise slugify: lowercase, replace non-alphanumerics with `-`, strip leading/trailing `-`, clamp to 1–63 chars.
- Keep a stable mapping so `<client-id>` in the hostname always resolves to the same BodhiApp instance.
- Exposing the client-id in the hostname is fine — it's an identifier, not a secret (the *secret* stays in Keycloak).

## 5. BodhiApp compatibility

### Redirect URIs / Keycloak

When a tunnel is active, the public origin is `https://<client-id>.getbodhi.io`. BodhiApp must:

- Set `public_server_url()` to that origin so `login_callback_url()` = `https://<client-id>.getbodhi.io/ui/auth/callback`.
- Register the same as an allowed redirect URI in Keycloak (the planned `AuthService::update_redirect_uris` + external SPI endpoint).
- For stable tunnel URLs this is registered once (re-sync on re-provision).

### Canonical URL + secure cookies

- `public_scheme()` must be `https` so `is_secure_transport()` sets the session cookie `Secure` flag (see `crates/routes_app/src/routes.rs`).
- `get_public_host_explicit()` should reflect the tunnel host so the canonical-redirect middleware uses it (see `canonical_url_middleware.rs`).
- The tunnel feature should set `BODHI_PUBLIC_HOST`/`BODHI_PUBLIC_SCHEME` (or the equivalent in-memory setting) when a tunnel is provisioned.

### Enumerated endpoints (what frp should allow)

BodhiApp's public surface is exactly these prefixes — configure frp `locations` to allow only these and 404 everything else:

- `/ping`, `/health`
- `/bodhi/v1/*` — setup, auth, tenants, users, settings, tokens, MCP, apps
- `/v1/models`, `/v1/chat/completions`, `/v1/embeddings`, `/v1/responses` (OpenAI)
- `/v1/messages`, `/anthropic/v1/*` (Anthropic)
- `/v1beta/*` (Gemini)

### JWT (two layers)

- **Edge (forked frp):** verifies JWT signature against cached Keycloak JWKS, checks `iss`/`aud`/`exp` and the premium claim; skips JWT on the public-path allow-list (`/ping`, `/health`, public `/bodhi/v1/...`).
- **BodhiApp:** `auth_middleware` / `api_auth_middleware` re-validate bearer JWTs, enforce roles/scopes, and handle token exchange for external apps (`AuthContext::ExternalApp`). This is the fine-grained authz boundary and defense in depth.

## 6. Recommended architecture (fork frp for edge JWT)

```
Internet → forked frps (TLS + wildcard + path allow-list + JWT signature/premium claim)
        → frpc (on desktop BodhiApp)
        → BodhiApp (its JWT middleware = fine-grained authz, defense in depth)
```

- `frps` (forked) on the gateway: `vhostHTTPSPort` + wildcard cert, `subdomainHost = "getbodhi.io"`, control-plane `auth.method = oidc` against Keycloak (or per-instance token), plus the added JWT middleware.
- Per-instance `frpc` proxy: `type = http`, `subdomain = "<client-id>"`, `locations = ["/v1", "/bodhi/v1", "/ping", "/health"]`.
- JWT middleware injection points (see `08` §4): add `JWTConfig` to `RouteConfig` (`pkg/util/vhost/vhost.go`) and validate in `HTTPReverseProxy.ServeHTTP` (`pkg/util/vhost/http.go`) before routing.

## Open decisions

1. Whether to use frp's **OIDC control-plane auth against Keycloak** (clean) vs per-instance tokens (simpler).
2. The exact client-id → subdomain normalization function and where it lives (backend vs frp config generator).
3. Confirm frp's unmatched-`locations` behavior returns 404 (verify at implementation; if not, add a BodhiApp route guard or edge path rule).
