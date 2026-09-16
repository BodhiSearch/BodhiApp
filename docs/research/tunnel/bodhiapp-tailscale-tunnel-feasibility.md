# BodhiApp — Tailscale Funnel feature: feasibility report

> **⚠️ SUPERSEDED / OUT OF SCOPE (as of 2026-09-15):** Named Cloudflare tunnels only is the locked scope as of 2026-09-15; quick tunnels, self-hosted frp, and Tailscale are out of scope — this doc's recommended-approach section (Tailscale Funnel as a production path alongside/instead of Cloudflare) is retained for historical context only. See `docs/research/tunnel/README.md` for the current locked scope.

**Status:** research complete, no code landed.

## Executive summary

**Feasible, and in one important way better suited to the LLM API use case than Cloudflare quick tunnels:** Tailscale Funnel streams SSE fine and gives a stable `*.ts.net` URL. But it is **not zero-config** — every tunnel requires a Tailscale account/login and a one-time Funnel approval, and Funnel has no custom-domain support (always `*.ts.net`).

For a locally-installed desktop BodhiApp exposing a public LLM API, the trade-off is:

- **Cloudflare quick tunnels** = no login, but **no SSE streaming** and an unstable URL (bad fit).
- **Cloudflare named tunnels** = stable URL + custom domain + SSE, but the user must bring a Cloudflare account + API token + domain.
- **Tailscale Funnel** = stable URL + SSE + free on the Personal plan, but requires a Tailscale login + one-time approval and no custom domain.

Recommendation: treat **Tailscale Funnel as a strong candidate for the "stable URL + streaming" path** (competing with Cloudflare named tunnels), not as a drop-in for the zero-login quick-tunnel demo path. If the goal is truly zero-config, Cloudflare quick tunnels are the only no-login option — and they break streaming.

## Direct answers

1. **No-login tunnels?** No. Tailscale requires an account + authenticated device (interactive OAuth or pre-auth key) plus a one-time Funnel approval. There is no anonymous tunnel.
2. **Free vs paid?** Funnel is **free on every plan** (including the free Personal plan, which is non-commercial-use-only). The paid tiers add seats/ACL groups/logging, not tunnel access. The real limits are architectural: `*.ts.net` only, ports 443/8443/10000, TLS-only, non-configurable bandwidth, beta.
3. **Rust library?** No official SDK; `tailscale-localapi` and `tsclient` wrap the local daemon socket, and `tailscale-client`/`tailscale.rs` wraps the control-plane v2 API. The practical path is to drive the `tailscale`/`tailscaled` binaries via CLI (or LocalAPI).
4. **Policy?** `tailscale`/`tailscaled` are BSD-3-Clause and redistributable. The free Personal plan is non-commercial; the user's own Tailscale account terms govern Funnel usage and bandwidth.

## Where it fits the recommended phased plan

The prior Cloudflare plan was "quick tunnel (demo) → named tunnel (production)." Tailscale slots in as an **alternative production path**:

- **Phase A — Cloudflare quick tunnels:** zero-config demo/non-streaming only (unchanged).
- **Phase B — production options (choose/offer one or both):**
  - **Tailscale Funnel:** stable `*.ts.net` URL, SSE streaming, free Personal; requires Tailscale login + one-time approval.
  - **Cloudflare named tunnels:** stable custom-domain URL, SSE streaming; requires Cloudflare account + API token + zone.

A unified `TunnelService` abstraction can back all three (quick cloudflare, named cloudflare, tailscale funnel) with the same process-manager/settings/route plumbing, differing only in the provider-specific "provision + get URL" logic.

## Tailscale Funnel lifecycle (implementation surface)

1. **Detect/install** `tailscaled` + `tailscale` (open-source variant on macOS), or use an existing system install.
2. **Authenticate** the device: interactive `tailscale login` (browser OAuth) or `tailscale up --authkey=<pre-auth-key>`.
3. **Enable Funnel once:** `tailscale funnel` (triggers web approval) or pre-configure the `funnel` node attribute + HTTPS certs in the tailnet policy.
4. **Expose the port:** `tailscale funnel --bg --yes <port>` (proxies to `http://127.0.0.1:<port>`).
5. **Read the URL/status:** `tailscale funnel status --json` (or `tailscale status --json`) → `https://<node>.<tailnet>.ts.net`.
6. **Register redirect URI** in Keycloak (stable URL → do it once; re-sync only if the node/tailnet name changes).
7. **Teardown:** `tailscale funnel ... off` and/or `tailscale funnel reset`.

## Keycloak redirect-URI flow

Same as Cloudflare, but simpler in one respect: the Tailscale Funnel URL is **stable**, so the Keycloak redirect URI only needs to be registered once (not on every restart, unlike quick tunnels). The `AuthService::update_redirect_uris` + external Bodhi SPI (or standard Keycloak Admin REST) approach from the Cloudflare plan carries over unchanged.

## Feature flag / desktop-vs-container gating

Identical to the Cloudflare plan: gate on `SettingService::is_native()` (runtime `AppType::Native` vs `Container`), default **enabled for desktop, disabled for non-desktop**, overridable via a new setting (e.g. `BODHI_TUNNEL_PROVIDER=tailsace`/`cloudflare`/`off`). The `native` Cargo feature is compile-time; use the runtime gate for default behavior.

## Risks & mitigations

| Risk | Mitigation |
|---|---|
| Requires Tailscale login (no zero-config) | Offer Cloudflare quick tunnels for the zero-config demo path; make the login/authkey flow explicit in the UI. |
| One-time Funnel approval + policy change | Pre-flight check (`tailscale funnel status` / policy) with a clear "approve in browser" step; surface admin requirements. |
| No custom domain (`*.ts.net` only) | Set user expectations; offer Cloudflare named tunnels when a custom domain is required. |
| Ports limited to 443/8443/10000 | Bind the tunnel to an allowed port; document the constraint. |
| Non-configurable bandwidth / beta | Don't promise throughput/SLA; treat as convenience exposure, not guaranteed infra. |
| Free Personal plan is non-commercial | Surface the plan constraint; rely on the user's own Tailscale account terms. |
| macOS open-source variant requirement | Detect platform and guide the user to the correct Tailscale install. |
| sudo needed for daemon install on Linux/macOS | Reuse 9Router's pattern (sudo helper / stored password) or require the user to install Tailscale first. |
| `tailscaled`/`tailscale` binary drift | Re-check version; Tailscale releases frequently. |

## Open decisions

1. Which production provider first: Tailscale Funnel or Cloudflare named tunnels? (Tailscale = no domain needed but `*.ts.net`; Cloudflare = custom domain but API token + zone.)
2. Whether to support both interactive login and pre-auth-key flows for Tailscale (recommend interactive for v1, auth-key for advanced).
3. Whether the zero-config Cloudflare quick-tunnel demo path is still in scope, given it can't stream.
4. Keycloak SPI vs Admin REST (unchanged from the Cloudflare plan — leaning SPI).
