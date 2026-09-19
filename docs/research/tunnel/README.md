# Remote access research

This directory records the current research behind BodhiApp Remote Access. It covers the supported Cloudflare design, the rejected Quick Tunnel option, the current BodhiApp integration risks, and Tailscale Funnel as a future provider candidate.

- **Status:** Current
- **Last verified:** 2026-09-19
- **Supported scope:** Cloudflare account-managed named tunnels on a domain the administrator controls

## Start here

| Document | Use it to |
|---|---|
| [`named-tunnel-operating-model.md`](named-tunnel-operating-model.md) | Review prerequisites, credentials, CLI lifecycle, supervision, and provider limits |
| [`quick-tunnel-decision.md`](quick-tunnel-decision.md) | Review the official SSE restriction, live GET/POST test, and no-go decision |
| [`tailscale-funnel-future-option.md`](tailscale-funnel-future-option.md) | Evaluate Tailscale Funnel as a future stable public provider without a user-owned domain |
| [`bodhiapp-integration-and-risks.md`](bodhiapp-integration-and-risks.md) | Review current code contracts, security boundaries, evidence, and validation gaps |

## Current decisions

- Use a locally managed named Cloudflare Tunnel
- Require the administrator to install `cloudflared`, sign in, and select a Cloudflare zone
- Reuse `cert.pem` through `cloudflared`; do not collect a separate Cloudflare API token
- Keep the public hostname stable across restarts
- Preserve local and LAN origins alongside the tunnel origin
- Sync the tunnel callback with Keycloak after the connector starts
- Treat Keycloak sync failure as degraded sign-in, not tunnel failure
- Keep Quick Tunnels out of the product while Cloudflare marks Server-Sent Events (SSE) unsupported
- Keep Tailscale Funnel as a future option pending live SSE, sign-in, lifecycle, and platform validation

## Scope boundaries

The current snapshot excludes:

- account-less `trycloudflare.com` Quick Tunnels
- a Bodhi-hosted or self-hosted `frp` gateway fleet
- remotely managed tunnel provisioning through the Cloudflare REST API
- a Cloudflare OAuth client owned by BodhiApp
- historical implementation plans that the shipped named-tunnel design superseded

Re-open an excluded option only when its product requirement changes. Record the new evidence in a focused document and update the decision snapshot.

## How to read the evidence

Each document separates these evidence types:

- **Provider contract:** Current Cloudflare or Tailscale documentation and source
- **Local validation:** A command or network test run against the recorded version and date
- **BodhiApp evidence:** Current repository source or tests
- **Inference:** A conclusion drawn from the evidence; revalidate it before implementation when the provider or code changes
