# 03 — Self-hosted tunnel providers (open-source)

Research into open-source projects that let BodhiApp run its **own** tunnel service (relay/edge + client), instead of depending on Cloudflare or Tailscale. Goal context: a **premium** feature, **BodhiApp-only**, at **minimum cost**, at a comparable capability level to Cloudflare Tunnel / Tailscale Funnel.

## Evaluation criteria for BodhiApp

- **HTTP(S) reverse tunneling with virtual hosting** — each BodhiApp instance needs a unique public URL (subdomain) and ideally custom domains.
- **SSE/streaming pass-through** — the exposed API streams `text/event-stream`; the tunnel must not buffer it.
- **Auth gating** — only authorized BodhiApp clients may open tunnels; per-instance identity.
- **TLS termination** — Let's Encrypt or wildcard cert.
- **Self-hostable server (relay/edge)** — BodhiApp controls the infrastructure.
- **Client is embeddable/shippable** — runs on the user's desktop BodhiApp.
- **License** — must be permissively licensed for bundling (Apache-2.0/MIT/BSD-3 preferred).
- **Cost** — minimal relay bandwidth / lightweight binaries.

## Summary table

| Project | License | Lang | Stars | HTTP vhost / subdomains | Custom domains | Auth | SSE-safe | Cost fit |
|---|---|---|---|---|---|---|---|---|
| **frp** | Apache-2.0 | Go | ~109k | **Yes** (subdomain + customDomains) | Yes | **token + OIDC** | Yes (byte-for-byte HTTP) | Excellent |
| **sish** | MIT | Go | ~4.7k | **Yes** (subdomain + custom) | Yes | SSH keys / password | Yes (WS(S) supported) | Good |
| **zrok** | Apache-2.0 | Go (OpenZiti) | ~4.7k | **Yes** (reserved shares) | Yes (frontend) | Zero-trust identity + SDK/REST | Yes | Good (heavier stack) |
| **rathole** | Apache-2.0 | **Rust** | ~14.2k | No (L4 TCP/UDP relay) | No | Mandatory per-service token, Noise/TLS | Yes (relays TCP) | Great perf, needs L7 front-end |
| **chisel** | MIT | Go | ~16.5k | No (per-port TCP/UDP) | No | user:pass + ACL regex | Yes (relays TCP) | Good perf, needs L7 front-end |
| **inlets-pro** | EULA (commercial) | Go | ~574 | Yes | Yes | License key | Yes | Not OSS/min-cost |
| **headscale** | BSD-3 | Go | ~43.8k | No (private mesh) | No | Tailscale identity | N/A | Not for public ingress |

## 1. frp — top recommendation

`github.com/fatedier/frp` · **Apache-2.0** · Go · ~109k stars.

- Architecture: `frps` (server, on BodhiApp's public infra) + `frpc` (client, on the user's desktop BodhiApp).
- Protocols: **TCP, UDP, HTTP, HTTPS**, plus P2P (`xtcp`) and secret TCP (`stcp`).
- HTTP virtual hosting: **`subdomain`** (wildcard `*.your-domain`) and **`customDomains`** — exactly the "unique URL per instance" + "premium custom domain" model.
- Auth: **token** and **OIDC (client credentials grant)**. OIDC means BodhiApp can authenticate `frpc` against its existing **Keycloak** — a very clean fit.
- TLS enabled by default; also QUIC/KCP/tcp-mux/compression for efficiency; **per-proxy bandwidth limits** (useful for premium quotas).
- Runtime management: dashboard, admin UI, Prometheus, and a **dynamic proxy management API (Store)** to create/update/delete proxies programmatically from the BodhiApp backend.
- Health checks, load balancing, host-header rewrite, X-Forwarded-For, Proxy Protocol.
- Lightweight client: `tiny-frpc` (~3.5 MB) as an alternative to full `frpc`.
- **SSE-safe:** frp forwards HTTP as a byte-for-byte copy (no event-stream buffering).

Notes: v1 is stable/feature-complete; a v2 rewrite is planned long-term but v1 keeps receiving fixes/features. Some antivirus tools may flag `frpc` (networking tool).

## 2. sish — ngrok-style over SSH

`github.com/antoniomika/sish` · **MIT** · Go · ~4.7k stars.

- ngrok/serveo-style, but uses **standard SSH** (`ssh -R 80:localhost:8080 example.com`) — no custom client required.
- Supports HTTP(S), WS(S), TCP, SNI routing; subdomains, custom domains, wildcards.
- Auth via **SSH keys and/or passwords**, dynamic key reload, restrictive multi-tenant binding policies (ban/bind subdomains).
- Single binary + Docker; production-grade self-hosting; load balancing modes; service console.
- **SSE-safe** (WebSockets explicitly supported; HTTP reverse proxy).
- Trade-off: auth is SSH-key based (no OIDC), and a Rust client would use `russh` or shell out to `ssh`. Strong choice if you want SSH-based per-user identity and zero custom client.

## 3. zrok — zero-trust ngrok alternative with SDK

`github.com/openziti/zrok` · **Apache-2.0** · Go (on OpenZiti) · ~4.7k stars.

- Self-hostable ngrok-style: `zrok share public localhost:8080`.
- HTTP/HTTPS, TCP, UDP, file shares; public and private shares; **reserved shares** (stable subdomains) and custom domains via a frontend.
- Zero-trust identity (OpenZiti); built-in controller + web console; **Go SDK** and a REST API.
- **SSE-safe** (HTTP reverse proxy).
- Trade-off: more moving parts (OpenZiti overlay/controller), SDK is Go (Rust integration via REST API or CLI). Great if you want a turnkey "tunnel product" with accounts/shares, but heavier than frp for a minimal relay.

## 4. rathole — lightweight Rust relay

`github.com/rathole-org/rathole` · **Apache-2.0** · **Rust** · ~14.2k stars.

- High throughput, low memory, tiny binary (~500 KiB); TCP and UDP relay for NAT traversal.
- Mandatory **per-service tokens**; Noise or TLS transport; WebSocket transport option; hot reload.
- **SSE-safe** at the L4 layer (it relays raw TCP; your app's HTTP SSE passes through).
- **Trade-off:** it is **L4 only** — no HTTP virtual hosting, subdomains, or TLS termination. You would build your own HTTP/TLS/domain front-end (e.g. Caddy/Nginx) in front of `rathole`. Most cost-efficient relay if you accept that extra layer; Rust aligns with BodhiApp's stack.

## 5. chisel — TCP/UDP over HTTP

`github.com/jpillora/chisel` · **MIT** · Go · ~16.5k stars.

- Fast TCP/UDP tunnel over HTTP (WebSocket), secured by SSH; single binary (client+server).
- Authfile (user:pass + regex address ACLs), reverse port forwarding, auto-reconnect, Let's Encrypt TLS, SOCKS5, optional backend proxy.
- Firewall-friendly (WebSocket transport, works behind CDNs).
- **SSE-safe** (raw TCP relay), but like rathole it has **no HTTP vhost/subdomain routing** — you'd add your own L7 front-end. Better for TCP tunneling than HTTP virtual hosting.

## 6. inlets-pro — not open-source / not min-cost

`github.com/inlets/inlets-pro` · **EULA + subscription** (commercial). The OSS toolchain (`inletsctl`, `inlets-operator`, docs) is MIT, but the actual tunnel server/client binary (`inlets-pro`) requires a paid license. Exclude for an open-source, minimum-cost build; useful only as a commercial reference.

## 7. headscale — not applicable to public ingress

`github.com/juanfont/headscale` · **BSD-3** · Go · ~43.8k stars.

- Self-hosted implementation of the **Tailscale control server** → a private WireGuard mesh (tailnet). It does **not** implement Tailscale Funnel (public exposure) — open issue #1040 — nor Serve (#1921). It solves private connectivity, not "public API endpoint," so it is out of scope for this feature.

## Reference list for further options

- `github.com/anderspitman/awesome-tunneling` — comprehensive comparison of tunneling solutions (ngrok, Cloudflare, Tailscale, frp, sish, rathole, zrok, chisel, boringproxy, localtunnel, bore, and more).

## Bottom line

For a BodhiApp-owned, BodhiApp-only, minimum-cost tunnel service with Cloudflare/Tailscale-level HTTP ingress:

- **Primary: frp** — feature-complete HTTP(S) vhost ingress (subdomains + custom domains), TLS, token/OIDC auth (reuse Keycloak), bandwidth limits, and a dynamic management API. Apache-2.0.
- **Alternative: zrok** — more turnkey (accounts/shares/UI), but heavier (OpenZiti) and Go SDK.
- **sish** — strong if you prefer SSH-key identity and a single Go binary.
- **rathole** — most efficient relay, but you must add the HTTP/TLS/domain layer yourself (and it's the only Rust option).
