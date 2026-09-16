# 02 — Consolidated research: Tailscale tunnels (Funnel / Serve)

Consolidated external research for integrating Tailscale as a tunnel provider. Answers the same questions posed for Cloudflare, plus free/paid tier limitations.

## 1. Does Tailscale allow creating tunnels without logging in?

**No.** Tailscale has no anonymous / zero-login tunnel equivalent to Cloudflare's Quick Tunnels. Every tunnel requires a **Tailscale account and an authenticated device** in a tailnet:

- Interactive: `tailscale login` opens a browser OAuth flow.
- Non-interactive: `tailscale up --authkey=<pre-auth-key>` (pre-auth keys are created via the v2 API or admin console).

Tailscale Funnel additionally requires one-time enablement:

- Tailscale v1.38.3+.
- MagicDNS enabled.
- Valid HTTPS certificates for the tailnet.
- A `funnel` node attribute in the tailnet policy file.
- The first `tailscale funnel` invocation triggers a **web approval** flow (or the tailnet admin pre-configures the `funnel` node attribute + HTTPS certs).

So the onboarding is fundamentally heavier than Cloudflare quick tunnels, but in exchange you get a **stable** URL and **SSE streaming**.

## 2. The two tunnel surfaces: Funnel (public) vs Serve (private)

| | Tailscale Funnel | Tailscale Serve |
|---|---|---|
| Audience | Public internet (anyone) | Only devices in your tailnet |
| URL | `https://<node>.<tailnet>.ts.net` | `https://<node>.<tailnet>.ts.net` |
| Account/login | Required | Required |
| Use case | This feature (public API) | Internal/private access only |

For BodhiApp's stated goal (public API endpoint reachable by apps on the internet), **Funnel** is the relevant surface.

## 3. Free vs paid (important nuance)

**Funnel is available on every plan, including the free Personal plan.** Unlike Cloudflare — where the no-login option (quick tunnels) is free/demo-only and the production option (named tunnels) needs a domain — Tailscale's public tunnel has **no paid gate**. The free/paid split is about tailnet seats and enterprise features, not the tunnel itself.

Plans (as of research date):

| Plan | Price | Relevant limits |
|---|---|---|
| **Personal (free)** | $0 | Up to 6 users, unlimited user devices, 3 ACL groups, 50 tagged resources, 1,000 ephemeral mins/month. **Non-commercial use only.** Funnel included. |
| Standard | $8/user/mo | Unlimited users, 10 ACL groups, SCIM/MDM, advanced roles. Funnel included. |
| Premium | $18/user/mo | 300 ACL groups, 10,000 ephemeral mins, network flow logs, log streaming, priority support. Funnel included. |
| Enterprise | Custom | Custom limits, MSAs/SLAs. |

## 4. Funnel limitations (all plans, unless noted)

These are the "be aware before implementing" items:

- **No custom domains** — Funnel can only use `*.ts.net` DNS names (your tailnet domain). There is no bring-your-own-domain option (open GitHub issue #11563).
- **Ports restricted to 443, 8443, 10000** for public exposure.
- **TLS-only** — Funnel is an HTTPS (or TLS-terminated TCP) reverse proxy. It is not a raw arbitrary-TCP tunnel (a TCP forwarder exists but still only on those three ports).
- **Non-configurable bandwidth limits** — public traffic transits Tailscale's Funnel relay servers; you cannot tune or guarantee throughput.
- **Beta status** — Funnel is documented as beta.
- **macOS** requires the **open-source variant** of the Tailscale app (App Store / Standalone system extension can expose ports but not files/directories).
- **Serve/Funnel port conflict** — the same port cannot be Serve (private) and Funnel (public) simultaneously.
- **One-time approval** — enabling Funnel requires a web approval / policy change (`funnel` node attribute), which needs Owner/Admin/Network-admin.
- **Let's Encrypt rate limits** — frequent cert re-requests can trigger a ~34-hour cooldown.
- **DNS propagation** — public DNS for a tailnet domain can take up to ~10 minutes.

### Streaming / SSE (the critical LLM question)

Funnel is a generic **TLS-terminated reverse proxy** to `http://127.0.0.1:<port>` (plus a TCP forwarder). There is **no documented SSE buffering limitation** — unlike Cloudflare quick tunnels, which explicitly buffer `text/event-stream`. Therefore **SSE streaming and WebSockets should pass through Funnel normally.** This is the key advantage Tailscale Funnel has over Cloudflare quick tunnels for serving streaming LLM APIs.

## 5. Rust libraries

There is no single "official" Rust SDK, but there are useful crates:

| Crate | Scope | Notes |
|---|---|---|
| `tailscale-localapi` (jtdowney) | **LocalAPI** (local `tailscaled` socket) | Status, node/tailnet info, cert/key, whois. Unix socket or TCP+password (macOS/Windows sandboxed). |
| `tsclient` (caius) | **LocalAPI** | LocalAPI client; OS-aware socket URL. |
| `tailscale-client` / `tailscale.rs` (agentsea) | **v2 REST API** (control plane) | Auth, user/tailnet info, create auth keys. |

Key architectural fact: the `tailscale` CLI is itself just a thin client over the **LocalAPI** (`/var/run/tailscale/tailscaled.sock` on Linux; localhost TCP with password on macOS/Windows when sandboxed). `tailscaled` is the privileged daemon that does the networking.

Practical approach (mirrors 9Router): **download/install `tailscaled` + `tailscale`, drive them via the CLI** — `tailscale funnel --bg --yes <port>`, then read the URL/status from `tailscale funnel status --json` or `tailscale status --json`. Use `tailscale-localapi`/`tsclient` if you want to talk to the daemon directly instead of shelling out.

## 6. License & policy

- `tailscale` and `tailscaled` are **BSD-3-Clause** (open source, with a `PATENTS` grant in the repo). They can be bundled/redistributed with the license + copyright notice. The **GUI wrappers** (macOS App Store, Windows GUI, mobile apps) are **not** open source — but the CLI + daemon are.
- The **free Personal plan is "non-commercial use only."** A commercial product's end users exposing services through a free Personal tailnet may be off-label; the user's own Tailscale account terms govern.
- Traffic over Funnel transits **Tailscale's relay servers** (encrypted end-to-end, but Tailscale sees no plaintext). Bandwidth is subject to Tailscale's non-configurable limits.

## 7. What 9Router does for Tailscale

9Router's Tailscale path (from its source/DeepWiki):

- Uses a **custom userspace socket** (`TAILSCALE_SOCKET`) in its data dir to avoid root where possible; falls back to the system socket.
- `startDaemonWithPassword` runs the daemon, using a stored sudo password via `execWithPassword`.
- `startFunnel(localPort)` runs `tailscale funnel`; `provisionCert` ensures HTTPS.
- `installTailscale` handles installation (may require sudo on Linux/macOS).
- `tailscale-check` probes install/daemon/login state before enabling.
- Security gate: refuses to enable remote access if the dashboard uses the default password or login is disabled.

Confirms the **binary + CLI + process/socket management** approach is the working pattern, same as its Cloudflare path.

## 8. Cloudflare vs Tailscale for this feature

| | Cloudflare quick | Cloudflare named | Tailscale Funnel |
|---|---|---|---|
| Login required | **No** | Yes (account + API token + zone) | Yes (account + device auth + one-time Funnel approval) |
| Stable URL | No (changes every restart) | Yes (`<name>.<zone>`) | Yes (`<node>.<tailnet>.ts.net`) |
| Custom domain | No | Yes | **No** (`*.ts.net` only) |
| SSE streaming | **Broken** | Works | Works |
| Free tier | Yes (dev-only) | No (needs zone) | Yes (Personal, all plans) |
| Ports | Any | Any | 443 / 8443 / 10000 |
| Bandwidth | ~200 in-flight (429) | Plan-based | Non-configurable (relay) |
| Production-grade | No (ToS: testing/dev) | Yes | Beta; works but no custom domain |

Net: for a streaming LLM public API, **Tailscale Funnel gives stable URLs + SSE + free on Personal**, at the cost of requiring a Tailscale login and one-time Funnel approval, and no custom domain. Cloudflare quick tunnels give zero login but break streaming and have unstable URLs; Cloudflare named tunnels give custom domains + streaming but require a domain and API token.
