# Exclude Cloudflare Quick Tunnels from Remote Access

This page records why BodhiApp does not offer account-less `trycloudflare.com` tunnels. It distinguishes Cloudflare's support contract from behavior observed in one live test.

- **Status:** Current decision
- **Last verified:** 2026-09-19
- **Decision:** Do not implement Quick Tunnels as a Remote Access mode

## Decision basis

[Cloudflare's Quick Tunnel documentation](https://developers.cloudflare.com/cloudflare-one/networks/connectors/cloudflare-tunnel/do-more-with-tunnels/trycloudflare/) states that Quick Tunnels:

- are for testing and development
- have no uptime guarantee
- use a random hostname that changes when the process restarts
- allow 200 concurrent in-flight requests before returning `429`
- do not support Server-Sent Events (SSE)
- can fail to start when a default `.cloudflared/config.yaml` exists

BodhiApp's [`fwd_sse`](../../../crates/server_core/src/fwd_sse.rs) and [`DirectSse`](../../../crates/server_core/src/direct_sse.rs) responses use SSE for streamed model output. The [MCP proxy](../../../crates/routes_app/src/mcps/mcp_proxy.rs) also accepts `text/event-stream`. A user-facing Remote Access feature cannot depend on behavior the provider marks unsupported.

## Why the agent marketing does not change the contract

[TryCloudflare](https://try.cloudflare.com/) markets reachable URLs for browser previews, screenshots, webhooks, eval harnesses, and human review. Those workflows can use ordinary HTTP or WebSockets.

The page does not promise SSE support. A `cloudflared` contributor described the restriction as a demo-product guardrail in [`cloudflare/cloudflared#1449`](https://github.com/cloudflare/cloudflared/issues/1449#issuecomment-2820773049). Cloudflare's [Quick-versus-Named comparison](https://developers.cloudflare.com/sandbox/api/tunnels/#how-they-differ-from-quick-tunnels) says the `trycloudflare.com` edge buffers `text/event-stream` while named tunnels support it.

## Live SSE verification

A synthetic test used `cloudflared 2026.9.1` on 2026-09-19. The loopback origin returned six `text/event-stream` events at 500 ms intervals with anti-buffering headers.

| Request shape | Result through Quick Tunnel |
|---|---|
| `GET /sse` | Headers arrived, then all six events arrived together after the origin closed |
| `POST /sse` | Each event arrived separately at approximately 500 ms intervals |

The result matches issue #1449. It corrects an over-broad earlier claim: Quick Tunnels do not buffer every POST event stream in current practice.

The POST result does not establish support. Cloudflare can change it without violating the published contract. Standard browser `EventSource` and GET-based MCP SSE remain broken.

## Effect on BodhiApp

| Surface | Method | Assessment |
|---|---|---|
| OpenAI chat completions and Responses | POST | Streamed in the synthetic method match, but unsupported by Cloudflare |
| Anthropic Messages | POST | Same method and response type; unsupported by Cloudflare |
| Gemini streaming generation | POST | Plausible, not separately tested, and unsupported |
| Browser `EventSource` | GET | Buffered until close |
| GET-based MCP SSE | GET | Buffered until close |
| Non-streaming UI and API requests | Request/response | No SSE blocker, but all other Quick limits remain |

The main chat path may work today. The complete product experience and provider guarantee do not meet the requirement.

## Comparison with named tunnels

| Dimension | Quick Tunnel | Named tunnel |
|---|---|---|
| Cloudflare account | Not required | Required |
| Domain | Not required | Required for the public hostname |
| URL | Random and restart-scoped | Stable |
| SSE contract | Unsupported | Supported |
| WebSockets | Supported | Supported |
| Quick-specific concurrency cap | 200 in-flight requests | No corresponding Quick cap |
| Availability promise | None | Depends on the zone plan and Cloudflare terms |
| Intended use | Testing and development | Production-capable path |

[Cloudflare Tunnel is available at no cost](https://blog.cloudflare.com/tunnel-for-everyone/). Named tunnels require an account and a domain on Cloudflare DNS, but not an inherently paid Tunnel plan.

## Reconsideration criteria

Reconsider Quick Tunnels only when one of these conditions changes:

1. Cloudflare documents SSE support for the POST streaming shape BodhiApp uses
2. BodhiApp selects another account-less provider with documented streaming support
3. The product creates a separate experimental preview mode with explicit reliability and protocol limits

Observed POST behavior alone is not enough to reopen the decision.

## Primary references

- [Cloudflare Quick Tunnels](https://developers.cloudflare.com/cloudflare-one/networks/connectors/cloudflare-tunnel/do-more-with-tunnels/trycloudflare/)
- [Cloudflare Sandbox tunnel comparison](https://developers.cloudflare.com/sandbox/api/tunnels/)
- [`cloudflare/cloudflared#1449`](https://github.com/cloudflare/cloudflared/issues/1449)
- [TryCloudflare](https://try.cloudflare.com/)
