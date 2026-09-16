# BodhiApp — Cloudflare Tunnel feature: feasibility report

> **⚠️ SUPERSEDED / OUT OF SCOPE (as of 2026-09-15):** Named Cloudflare tunnels only is the locked scope as of 2026-09-15; quick tunnels, self-hosted frp, and Tailscale are out of scope — this doc's recommended-approach section (phased quick-tunnel-then-named-tunnel plan) is retained for historical context only. See `docs/research/tunnel/README.md` for the current locked scope and `10-cloudflared-cli-named-tunnel-lifecycle.md` onward for the live named-tunnel research.

**Status:** research complete, no code landed.
**Decision requested:** "both, phased" (quick tunnel as zero-config path; named tunnel as production path).

## Executive summary

**Feasible, with one critical caveat.** Cloudflare offers two tunnel flavors, and the "no login" flavor (quick tunnels) is **fundamentally unsuitable for the primary stated use case** — exposing the LLM AI APIs — because it does not support Server-Sent Events (SSE) streaming and gives unstable URLs. The production-grade flavor (named tunnels) requires the end user to supply their own Cloudflare account, API token, and domain.

The pragmatic and proven implementation pattern (used by 9Router) is to **bundle/auto-download the official `cloudflared` Go binary and manage it as a subprocess**, rather than depend on an immature Rust crate. BodhiApp's architecture is a clean fit: it already has the desktop-vs-container gate (`AppType` / `is_native()`), the settings/feature-flag plumbing, an axum route pattern, and the Keycloak client/service abstraction where the redirect-URI endpoint belongs.

**Recommended approach:** implement **Phase 1 = quick tunnels** (zero-config, for demos/testing and non-streaming access), **Phase 2 = named tunnels** (user brings a Cloudflare API token + domain, for stable URLs and SSE streaming). Do not market quick tunnels as a production public API.

## Direct answers

1. **Can Cloudflare create tunnels without login?** Yes — Quick Tunnels via `cloudflared tunnel --url http://localhost:1135`. No account/token/DNS needed. But: random `*.trycloudflare.com` URL that changes every restart, no SSE, ~200 in-flight request limit (429), no SLA, and Cloudflare labels it "testing and development only."
2. **Is there a Rust library?** Not a production one. The official `cloudflared` is a Go binary. The official `cloudflare` crate (cloudflare-rs, BSD-3-Clause) wraps the REST API for **named** tunnels only. The `cloudflared` crate on crates.io (0.0.3, 167 LOC, Feb 2024) is third-party and abandoned — avoid it. Use the binary + subprocess management.
3. **Policy for offering this in an app?** `cloudflared` is Apache-2.0 (+ runtime library exception) and redistributable with attribution. Quick tunnels are off-label for production. Named tunnels act on the user's own Cloudflare account/token — store the token securely and use least-privilege scopes (Tunnel Edit + DNS Edit + Zone Read). Plan for abuse, rate limits, and binary freshness.
4. **What does 9Router do?** It uses **quick tunnels** (not named) and maps the unstable URLs to stable `abc-tunnel.us` domains via its **own relay worker**. That relay is 9Router's central infra and does **not** fix the SSE buffering at the trycloudflare edge. So BodhiApp should either accept ephemeral URLs (quick) or use the user's domain (named) rather than build a relay.

## The critical finding: SSE streaming breaks on quick tunnels

BodhiApp's OpenAI/Anthropic/Gemini streaming responses are `text/event-stream` (SSE), forwarded via `crates/server_core/src/fwd_sse.rs`. Cloudflare's quick-tunnel edge **buffers `text/event-stream`**, so SSE events never reach the client. WebSockets work, but SSE does not.

Consequence: a quick tunnel can expose non-streaming endpoints (models list, non-streaming completions, embeddings) but **not** streaming chat — which is the heart of an LLM API. Named tunnels support SSE, so they are the correct production path.

## Recommended architecture (both, phased)

### Phase 1 — Quick tunnels (zero-config, no login)

- On enable: ensure/download `cloudflared`, spawn `cloudflared tunnel --url http://127.0.0.1:<port> --no-autoupdate`, parse the `https://*.trycloudflare.com` URL from stdout, store it, and register it as a Keycloak redirect URI.
- Expose tunnel status/URL in the UI with a clear "testing/demo, no streaming, URL changes on restart" warning.
- Re-register the Keycloak redirect URI on every tunnel start (URL changes each time).

### Phase 2 — Named tunnels (user's Cloudflare account + domain)

- User provides a scoped Cloudflare API token (+ optional account/zone IDs) and a DNS label.
- Use the official `cloudflare` crate (or raw REST calls) to create/list/delete the `cfd_tunnel` and route DNS, then run `cloudflared tunnel run --token <token>`.
- Stable URL + SSE streaming + production-legal. Encrypt the API token using the existing encryption-key pattern.

### Process manager (shared)

A small manager service should handle: binary download/version check, spawn, PID tracking, health check, watchdog/restart, and stop. This mirrors 9Router's `manager.js`/`pid.js`/watchdog, implemented natively in Rust.

## Keycloak redirect-URI flow

Goal: after a tunnel URL exists, add it to the client's allowed redirect URIs so users can authenticate against the public endpoint.

- The existing Rust backend only talks to the **custom Bodhi SPI** (`/realms/{realm}/bodhi/**`), not the standard Keycloak Admin REST API.
- Two implementation options (see codebase map §6):
  1. **Extend the Bodhi SPI** (recommended, consistent with the current design and the user's stated plan): add `AuthService::update_redirect_uris(client_id, uris)` → new SPI endpoint; implement it in the external Keycloak SPI.
  2. **Use the standard Keycloak Admin REST API** from Rust (`PUT /admin/realms/{realm}/clients/{id}`) — requires a realm-admin token, which introduces a new privilege surface.
- Because quick-tunnel URLs change on restart, the redirect URI must be re-synced on each tunnel start; named tunnels need it synced once (and cleaned up on tunnel deletion).

## Feature flag / desktop-vs-container gating

- Gate on `SettingService::is_native()` (runtime `AppType::Native` vs `Container`), defaulting **enabled for desktop, disabled for non-desktop**, overridable via a new setting (e.g. `BODHI_TUNNEL_ENABLED`).
- The `native` Cargo feature is compile-time and cannot be toggled at runtime, so it can gate code inclusion but not the default on/off behavior — use the runtime `AppType`/setting for that.
- Docker/container instances continue to use `BODHI_PUBLIC_HOST` / reverse proxy / infra gateway as they do today.

## Implementation surface (concrete)

- New `TunnelService` (trait + default impl) in `services`, wired into `AppServiceBuilder`.
- New route module (`crates/routes_app/src/tunnel/`) registered under `admin_session_apis`; `ENDPOINT_*` constants in `shared/openapi.rs`.
- New settings keys in `constants.rs` + accessors in `setting_service.rs`.
- `AuthService::update_redirect_uris` + external Keycloak SPI endpoint.
- `cloudflare` crate dependency only if/when Phase 2 lands.

## Risks & mitigations

| Risk | Mitigation |
|---|---|
| SSE streaming broken on quick tunnels | Position quick tunnels as demo/testing; make named tunnels the production path; label clearly in UI. |
| Unstable URL breaks redirect URIs | Auto re-sync redirect URI on each quick-tunnel start; named tunnels for stable URLs. |
| Off-label/ToS risk of quick tunnels | Clear consent + "not for production" messaging; default disabled outside desktop. |
| Abuse of a zero-config "expose me publicly" toggle | Explicit opt-in, warning, kill switch; rely on upstream WAF/reverse-proxy guidance. |
| `cloudflared` binary drift | Periodic re-download/version check; pin a supported version. |
| API token exposure (named) | Encrypt at rest using existing `BODHI_ENCRYPTION_KEY` pattern; least-privilege scopes. |
| WARP/Zero Trust egress blocking handshake | Detect timeout and surface a clear error (documented Cloudflare limitation). |

## Open decisions for implementation

1. Keycloak: extend the Bodhi SPI (option 1) vs. standard Admin REST API (option 2). Leaning option 1 per the user's plan.
2. Whether quick tunnels should expose the streaming endpoints at all (recommend: block streaming on quick-tunnel mode with a clear error, or fall back to non-streaming).
3. Whether to add a relay for stable URLs (not recommended; adds central infra and doesn't fix SSE).
4. Token storage UI for named tunnels (Phase 2) — where the user enters/rotates the Cloudflare API token.
