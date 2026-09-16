# 00 — Consolidated research: Cloudflare Tunnels

Consolidated external research for the Cloudflare Tunnel feature. Answers the four questions posed, with sources.

## 1. Does Cloudflare allow creating tunnels without logging in?

> **⚠️ SUPERSEDED / OUT OF SCOPE (as of 2026-09-15):** Named Cloudflare tunnels only is the locked scope as of 2026-09-15; quick tunnels, self-hosted frp, and Tailscale are out of scope — this section's quick-tunnel findings are retained for historical context only (they still stand as external facts about Cloudflare, but are no longer a recommended approach for BodhiApp). See `docs/research/tunnel/README.md` for the current locked scope.

**Yes — via "Quick Tunnels" (TryCloudflare).** No Cloudflare account, API token, DNS record, or domain is required.

```bash
cloudflared tunnel --url http://localhost:1135
# prints a random URL: https://<random-words>.trycloudflare.com
```

`cloudflared` makes an outbound-only connection to Cloudflare's edge and prints the assigned public URL. It requires no authentication.

### Hard limitations of Quick Tunnels (matter a lot for this feature)

- **Ephemeral, unstable URL** — a new random `*.trycloudflare.com` hostname on **every process restart**. There is no way to pin or reserve a quick-tunnel hostname.
- **No Server-Sent Events (SSE).** The `trycloudflare.com` edge buffers `text/event-stream` responses, so SSE events never reach the client. **WebSockets work normally.** This directly breaks streaming LLM responses.
- **~200 in-flight request hard limit.** Beyond that the edge returns `429`. Documented as "currently 200"; subject to change.
- **No SLA / no uptime guarantee.** Cloudflare positions it as a debug aid.
- **"Testing and development only"** per Cloudflare's own docs — not intended for production traffic.
- Quick tunnels are not supported if a `config.yaml` is present in `~/.cloudflared` (must rename it).

### When you DO need to log in: "Named Tunnels"

Named tunnels give a **stable** `https://<name>.<your-zone>` hostname and **support SSE**, but require:

1. A Cloudflare **account** and a **zone** (domain on Cloudflare DNS).
2. A Cloudflare **API token** with scopes:
   - Account · Cloudflare Tunnel · Edit
   - Zone · DNS · Edit
   - Zone · Zone · Read
   - (optional) Account · Account Settings · Read
3. The **account ID** and **zone ID** (can be inferred if the token is scoped to exactly one of each).

Named tunnels create two Cloudflare-side resources: a Tunnel object (`cfd_tunnel`) and a proxied `CNAME` (`<name>.<zone>` → `<tunnel-id>.cfargotunnel.com`). They survive restarts and are the recommended option for production, webhooks, and OAuth callbacks.

### Comparison

| | Quick tunnel | Named tunnel |
|---|---|---|
| Cloudflare login/token | Not required | Required |
| Hostname | Random `*.trycloudflare.com` | Stable `<name>.<zone>` |
| Stable across restart | No | Yes |
| SSE streaming | **Not supported** | Supported |
| WebSockets | Supported | Supported |
| Uptime/SLA | None (debug aid) | Backed by zone's standard SLA |
| Production use | Off-label ("testing/development only") | Intended |

## 2. Rust libraries for Cloudflare tunnels

There is **no official Rust SDK for the `cloudflared` daemon protocol.** The official `cloudflared` client is a **Go** binary (`github.com/cloudflare/cloudflared`).

### The realistic options

| Option | Maturity | Use |
|---|---|---|
| **Shell out to the `cloudflared` binary** | Production-grade (this is what 9Router and most tools do) | Download the official binary, spawn `cloudflared tunnel --url ...`, parse the `trycloudflare.com` URL from stdout/logs, manage process lifecycle. Language-agnostic; no Rust crate needed. |
| **`cloudflare` crate (cloudflare-rs)** | Official, but "Work in Progress" | Wraps the Cloudflare v4 REST API. For **named tunnels only** (needs an account + API token). |
| **`cloudflared` crate (crates.io)** | Abandoned / toy | Third-party, not production-ready. Avoid. |

### `cloudflare` crate (official, cloudflare-rs)

- Repo: `github.com/cloudflare/cloudflare-rs`; license **BSD-3-Clause**.
- Latest `0.14.0` (Mar 2025), ~670k downloads, async + blocking clients.
- Has `endpoints::cfd_tunnel` and `endpoints::argo_tunnel` modules. `cfd_tunnel` exposes: `create_tunnel`, `list_tunnels`, `delete_tunnel`, `update_tunnel`, `route_dns` (plus `Tunnel`, `TunnelWithConnections`, `ActiveConnection`, `DnsRouteResult` types).
- This is sufficient to create/list/delete named tunnels and route DNS via the API — but you still need the `cloudflared` binary to actually **run** the tunnel (or implement the QUIC/HTTP2 edge protocol yourself, which is not recommended).

### `cloudflared` crate on crates.io (third-party — do NOT use)

- Repo: `github.com/KABBOUCHI/cloudflared`; author Georges KABBOUCHI.
- Version `0.0.3` (Feb 2024), 167 lines of Rust, ~4.5k total downloads, ~37 recent.
- License MIT OR Apache-2.0. Only two public types (`Tunnel`, `TunnelBuilder`).
- Effectively abandoned and far too immature for a production feature.

### Recommendation

Bundle/auto-download the official `cloudflared` binary and drive it as a managed subprocess (start, health-check, parse URL, restart, stop). Use the official `cloudflare` crate only if/when adding **named-tunnel** management against the user's Cloudflare API token.

## 3. Policy / what to be aware of when offering this in an app

### Redistribution of `cloudflared`

- The `cloudflared` software is licensed **Apache 2.0** with a **Runtime Library Exception** (see Cloudflare downloads/license page; GitHub lists Apache-2.0).
- You may bundle and redistribute the binary in a product. You must preserve attribution (the Apache `NOTICE`/copyright notices). The Runtime Library Exception relaxes attribution for portions statically linked into a binary.

### Quick tunnels and Cloudflare's terms

- Quick tunnels are explicitly **"intended for testing and development only."** Offering them as the default production path for a public API would be off-label and risky (no SLA, rate limits, possible termination).
- Users effectively accept Cloudflare's Terms/Privacy Policy by running `cloudflared`.

### Named tunnels and the user's own account

- The **end user** brings their own Cloudflare account, API token, and domain. The app acts on the user's behalf with a scoped token.
- The user remains bound by their own Cloudflare Self-Serve Subscription Agreement / account terms.
- The app is responsible for **securely storing the API token** (BodhiApp already has an encryption-key pattern for client secrets — see codebase map).
- Token scopes should be least-privilege: Tunnel Edit + DNS Edit + Zone Read only.

### Practical risks to plan for

- **Abuse / ToS**: a zero-config "expose my API publicly" toggle invites abuse; add clear user consent, a rate-limit/WAF note, and a kill switch.
- **SSE/streaming**: quick tunnels silently break streaming LLM responses. This is the single biggest product risk.
- **Unstable URLs**: every restart invalidates the Keycloak redirect URI and any bookmarked/shared endpoint.
- **No SLA**: don't market quick tunnels as a reliable public endpoint.
- **cloudflared binary freshness**: Cloudflare supports versions within ~1 year; the app should re-check/download the binary.
- **WARP/Zero Trust egress**: local WARP can block the tunnel handshake — need a clear error path.

## 4. What does 9Router actually do?

9Router (a Next.js OpenAI-compatible gateway) implements both **Cloudflare Tunnel** and **Tailscale Funnel** to expose its API gateway (port 20128).

### Cloudflare: uses QUICK tunnels (not named tunnels)

- Downloads and manages the `cloudflared` binary per-host-OS (validates magic numbers, caches in its data dir).
- Runs quick tunnels (`trycloudflare.com` URLs).
- **Because quick-tunnel URLs change on restart, 9Router maps them to stable `abc-tunnel.us` domains via a 9Router-owned relay worker.** This is 9Router's own central infrastructure, not a Cloudflare feature.
- Manages the process with a PID file, a watchdog (60s interval, 120s restart cooldown, 2.5s network settle), network-interface change detection, and auto-resume on startup / crash.
- Has a security gate: refuses to enable remote access if the dashboard uses the default password or login is disabled.

### Tailscale Funnel

- Uses the user's tailnet + `tailscale funnel` for a stable HTTPS URL; may require `sudo` for daemon install.

### Key takeaways for BodhiApp

1. 9Router confirms the **binary-subprocess** approach is the pragmatic, working pattern.
2. 9Router's **stable URL comes from their own relay**, not from Cloudflare — meaning a self-contained BodhiApp either accepts ephemeral URLs (quick tunnels) or uses the user's domain (named tunnels). The relay adds central infra + a third-party dependency + does **not** fix the SSE buffering at the trycloudflare edge.
3. 9Router does **not** implement named tunnels.

See `01-bodhi-app-codebase-map.md` for how this maps onto BodhiApp's own architecture.
