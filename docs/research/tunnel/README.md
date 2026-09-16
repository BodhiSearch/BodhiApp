# docs/research/tunnel/ — Remote access via Cloudflare named tunnel

Research inputs for exposing a locally installed BodhiApp (native/desktop) on a stable public URL. Research docs reflect what was true when written and are **not** kept continuously current — re-verify provider limits, CLI flags, and API scopes against official docs before implementation.

**Locked scope (2026-09-15):** Cloudflare **named tunnels only**, on the user's own zone. Quick tunnels (`trycloudflare.com`), Tailscale, and self-hosted frp are out of scope. Implementation plan: `docs/claude-plans/202609/tunnel/`.

## Current research (2026-09-15) — start here

| Doc | Covers |
|---|---|
| `10-cloudflared-cli-named-tunnel-lifecycle.md` | `cloudflared` CLI driven as a child process: `login`/`create`/`route dns`/`run`/`list`/`delete`/`token`, file paths/flags/env/log lines per OS, versioning, headless gotchas. Follow-up pins the exact `run` argv, `/ready` as the readiness signal, and the fake-`cloudflared` stub contract |
| `11-cloudflare-oauth-and-api-token-options.md` | Auth tiers for a desktop app: wrangler OAuth internals, Cloudflare self-managed OAuth clients (scope catalog verified: `argotunnel.write`, `dns.write`, `zone.read`), what `cert.pem` really is, API-token permission groups + verified prefilled create-token deep link |
| `12-remotely-managed-tunnel-api-and-rust-crate.md` | Remotely-managed tunnel via REST: exact `cfd_tunnel` call sequence + JSON, `--token` internals, `cloudflare` crate gap analysis (verdict: hand-rolled reqwest), local vs remote management |
| `13-cloudflare-edge-behavior-for-llm-api-traffic.md` | Edge behavior: SSE streams fine, ~100s first-byte 524 risk for non-streaming, body limits, Bot Fight Mode blocks SDK clients, headers at origin, plaintext loopback origin |
| `14-cloudflared-binary-detection-install-download.md` | Binary detection (PATH + well-known paths, `BODHI_EXEC_LOOKUP_PATH` precedent), install channels/asset names, version parsing, license (plain Apache-2.0), download-on-trigger design |
| `15-prior-art-desktop-apps-managing-cloudflared.md` | How 9Router, Unsloth Studio, HA add-on, proxypal, Pinokio supervise `cloudflared`; recommended Rust state machine/health probe/backoff/orphan handling |
| `20-codebase-child-process-and-app-lifecycle.md` | Codebase: `llama_server_proc` pattern to copy, start hook (`serve.rs` ready arm), stop hook (graceful-shutdown closure), service injection, status patterns, port stability |
| `21-codebase-settings-network-and-info.md` | Codebase: settings precedence/defaults for `BODHI_TUNNEL_*`, `NetworkService`, `/bodhi/v1/info` + finalized `origins`/tunnel DTO schema (§6), canonical-redirect/cookie-Secure/Host risks, headers-at-origin follow-up |
| `22-codebase-login-flow-and-keycloak-spi-redirect-uris.md` | Codebase + SPI: `auth_initiate` scheme/port composition bug for tunnel hosts, SPI `PUT .../resources/redirect-uris` design, `AuthService::update_redirect_uris`, 404-degrade compat, SPI release/deploy sequencing |
| `23-codebase-persistence-routes-and-backend-tests.md` | Codebase: `tunnels` table + migration, encryption helper reuse, route module skeleton, polling status pattern, services/routes_app/server_app test skeletons, fake-binary approach, Windows CI status |
| `24-codebase-frontend-settings-v2-and-e2e.md` | Codebase: Remote Access sub-page under Settings nav, hooks/MSW/component tests, one growing Playwright spec (stub-driven steps + opt-in real-Cloudflare step), GitHub runner egress evidence |
| `01-bodhi-app-codebase-map.md` | Original codebase map (still accurate for the desktop/container gate and route registration; superseded in detail by 20–24) |
| `sources.md` | Source links for the pre-lock research (docs 10–24 carry their own Sources sections) |

## Historical / superseded (pre-lock exploration, kept for context)

| Doc | Covers |
|---|---|
| `00-consolidated-research.md` | Cloudflare external research; §1 (quick-tunnel recommendation) superseded, §2–4 (Rust libs, policy, 9Router) still valid |
| `bodhiapp-cloudflare-tunnel-feasibility.md` | Early quick-then-named phased plan — superseded (banner inside) |
| `bodhiapp-self-hosted-tunnel-feasibility.md` | Early frp-based "premium path" — out of scope (banner inside) |
| `bodhiapp-tailscale-tunnel-feasibility.md` | Early Tailscale Funnel exploration — out of scope (banner inside) |
| `02-tailscale-research.md` | Tailscale external research — out of scope |
| `03-self-hosted-tunnel-providers.md` … `08-deployment-plan.md` | Self-hosted provider comparison, cost models, frp compatibility, deployment plan — out of scope |
