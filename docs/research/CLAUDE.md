# docs/research/ — CLAUDE.md

Deep-research and design inputs for in-flight features. Research docs are planning inputs — they reflect what was true when written and are **not** kept continuously current.

## model-router/ — Composite Model Routing (fallback strategy)
Active design; **proposal written, no code landed yet** — verify against the codebase before assuming any of it exists.

| Doc | Covers |
|---|---|
| `bodhiapp-model-router-implementation-proposal.md` | The proposed BodhiApp model-router composite-alias design (start here) |
| `00-consolidated-research.md` | Consolidated cross-gateway research synthesis |
| `fallback-routes.md` | Survey of OSS gateways with fallback/failover/health-check reference value |
| `findings-*.md` | Per-gateway findings (LiteLLM, Bifrost, Portkey, Ogem, free gateways) feeding the synthesis |
| `phasewise-impl/` | Phased implementation breakdown (foundation/passthrough → in-request fallback → health & recovery); see its `README.md` |

## tunnel/ — Remote access via Cloudflare named tunnel
Current implementation research, last consolidated 2026-09-19. The directory contains the supported Cloudflare named-tunnel model, the Quick Tunnel decision record, a future Tailscale Funnel assessment, and the BodhiApp integration and risk snapshot. Start at `tunnel/README.md`. Historical implementation plans live in `docs/claude-plans/202609/tunnel/`.
