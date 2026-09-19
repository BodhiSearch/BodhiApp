# Evaluate Tailscale Funnel as a future Remote Access provider

This report evaluates Tailscale Funnel as a future public-tunnel provider for BodhiApp. It covers product fit, lifecycle, transport behavior, limits, security, and the implementation work needed beside Cloudflare.

- **Status:** Future option, not implemented
- **Last verified:** 2026-09-19
- **Research depth:** Quick revalidation of seven primary sources
- **Local validation:** Not run; the Tailscale CLI is not installed on this machine
- **Recommendation:** Keep Funnel as a candidate, subject to a live SSE and browser-sign-in spike

## Executive assessment

Tailscale Funnel can expose BodhiApp at a public `*.ts.net` HTTPS URL without requiring a user-owned domain. Unlike Cloudflare Quick Tunnels, Funnel uses a persistent daemon configuration and resumes after reboot when configured with `--bg`.

Funnel is not an anonymous tunnel. The administrator needs a Tailscale account, an authenticated device in a tailnet, MagicDNS, HTTPS certificates, and permission to enable the `funnel` node attribute. The first CLI enablement opens a web approval flow. [Tailscale documents these prerequisites and limits](https://tailscale.com/docs/features/tailscale-funnel#get-started-with-funnel).

The main unresolved requirement is Server-Sent Events (SSE). Funnel uses an encrypted TCP proxy and Tailscale documents no SSE-specific buffering restriction. However, the current docs do not explicitly guarantee SSE behavior. BodhiApp should not ship the provider until a live test proves incremental OpenAI, Anthropic, Gemini, and Model Context Protocol (MCP) streaming.

## Funnel and Serve have different product semantics

Tailscale exposes two local-sharing modes:

| Mode | Audience | BodhiApp fit |
|---|---|---|
| [Funnel](https://tailscale.com/docs/features/tailscale-funnel) | Public internet | Matches the existing public Remote Access feature |
| [Serve](https://tailscale.com/docs/features/tailscale-serve) | Authenticated devices in the same tailnet | A separate private-access feature, not a transparent Cloudflare replacement |

The first Tailscale provider should target Funnel. Serve could follow later if BodhiApp introduces a private tailnet-only mode with different user expectations.

## Provider prerequisites

Funnel currently requires:

- Tailscale `1.38.3` or later
- an authenticated node in a tailnet
- MagicDNS enabled
- HTTPS enabled for the tailnet
- the `funnel` node attribute in the tailnet policy
- Owner, Admin, or Network admin permission to change the policy when approval is needed
- a platform that can run the Tailscale CLI

The first `tailscale funnel` command can open a browser approval flow. After approval, Tailscale creates HTTPS certificates and updates the policy. These requirements come from the [Funnel setup and troubleshooting reference](https://tailscale.com/docs/features/tailscale-funnel#get-started-with-funnel).

Funnel is [available on all Tailscale plans](https://tailscale.com/docs/features/tailscale-funnel). The [free Personal plan is limited to non-commercial use](https://tailscale.com/pricing#free-trial), so commercial Bodhi deployments need a plan permitted by the customer's Tailscale terms.

## Proposed lifecycle

BodhiApp should configure an existing Tailscale installation instead of supervising `tailscaled` as its own child process.

1. Detect the `tailscale` CLI and query daemon/login state
2. Guide the administrator through Tailscale installation or login when unavailable
3. Run the first Funnel command and let Tailscale open its approval flow
4. Configure a persistent reverse proxy to BodhiApp's loopback port
5. Read the active URL and state from the JSON status command
6. Sync the stable callback URL with Keycloak under a distinct `tailscale` gateway
7. Disable only the Bodhi-managed Funnel mapping when the administrator turns Remote Access off

The selected CLI surface is:

```text
tailscale funnel --bg --yes http://127.0.0.1:bodhi_port
tailscale funnel status --json
tailscale funnel --https=443 off
```

The exact disable command must retain the flags used to enable the mapping. `tailscale funnel reset` clears all Funnel configuration on the node and is too broad for a normal BodhiApp disable action. See the [Funnel CLI reference](https://tailscale.com/docs/reference/tailscale-cli/funnel).

With `--bg`, Tailscale persists the configuration and automatically resumes it after a device reboot or `tailscale down` followed by `tailscale up`. Without `--bg`, BodhiApp would need to restart Funnel itself.

## URL and callback behavior

Funnel uses the node's tailnet DNS name, such as `https://node_name.tailnet_name.ts.net`. Treat the URL as stable while the node name and tailnet name remain unchanged.

Funnel currently accepts only names in the tailnet's `*.ts.net` domain. The [custom-domain feature request](https://github.com/tailscale/tailscale/issues/11563) remains open, and a CNAME alone does not work because Funnel routes by the TLS Server Name Indication value.

The stable URL reduces callback churn compared with Cloudflare Quick Tunnels. BodhiApp still needs to resync Keycloak when the node name, tailnet name, or provider changes.

## Transport and streaming assessment

Tailscale describes Funnel as an encrypted TCP proxy through Funnel relay servers. The relay cannot decrypt the proxied content; the Tailscale daemon on the device terminates TLS and forwards the request to the local service. See [How Funnel works](https://tailscale.com/docs/features/tailscale-funnel#how-funnel-works).

This architecture should preserve HTTP response streaming, but that conclusion is an inference, not a published SSE contract. The pre-implementation spike must verify:

- POST SSE events arrive incrementally for OpenAI, Anthropic, and Gemini routes
- GET SSE remains incremental for MCP clients that use it
- long-lived streams survive idle periods and normal model latency
- client disconnects cancel origin work
- response headers and chunk boundaries remain intact

WebSockets need a separate check. An open [`tailscale/tailscale#18651`](https://github.com/tailscale/tailscale/issues/18651) reports stripped query parameters on WebSocket upgrade requests through Serve/Funnel. BodhiApp should avoid treating general WebSocket compatibility as proven.

## Provider limits

[Tailscale's Funnel reference](https://tailscale.com/docs/features/tailscale-funnel#get-started-with-funnel) documents these limits:

- public listener ports are restricted to `443`, `8443`, and `10000`
- connections must use TLS
- public traffic is subject to non-configurable bandwidth limits
- the same public port cannot be Serve and Funnel at the same time
- Funnel remains a beta feature
- frequent certificate requests can trigger Let's Encrypt rate limits

The public listener restriction does not require BodhiApp to change its local port. Funnel can listen on `443` and reverse proxy to BodhiApp on `127.0.0.1:bodhi_port`.

The current Funnel page contains ambiguous macOS variant guidance: its limitations mention open-source variants, while its sharing section says App Store and Standalone system-extension variants can share ports. Test each supported BodhiApp distribution before publishing installation guidance.

## Security and privacy

Funnel makes the selected Bodhi service reachable by anyone on the internet. Tailnet membership does not protect the public Funnel URL. Bodhi authentication, explicit exposure confirmation, and provider-specific disable controls remain required.

Tailscale says Funnel relay servers cannot decrypt the content carried through the encrypted proxy. The local Tailscale daemon terminates TLS before forwarding to BodhiApp. This differs operationally from Cloudflare's edge-termination model and should be documented in the product privacy explanation.

The `tailscale` and `tailscaled` source is available under the [BSD 3-Clause License](https://raw.githubusercontent.com/tailscale/tailscale/main/LICENSE). BodhiApp should initially detect and guide installation rather than redistribute platform packages, whose packaging and service setup vary by operating system.

## BodhiApp integration changes

Supporting Funnel requires provider-aware state rather than Cloudflare-specific assumptions:

- add `Tailscale` to the remote-access provider model
- isolate Cloudflare certificate, zone, DNS, and tunnel-ID state from shared connection state
- add Tailscale CLI, daemon, login, policy, URL, and status diagnostics
- use `tailscale` as a separate Keycloak redirect gateway key
- prevent Cloudflare and Tailscale from exposing the instance simultaneously unless multi-provider behavior is designed explicitly
- adapt the UI so provider selection precedes provider-specific setup
- preserve the shared exposure confirmation, URL actions, reconnect state, and authorization-sync states

The Tailscale implementation should use `tailscale funnel status --json` rather than parse presentation text. Tailscale documents the JSON flag but not a stable response schema, so the spike must capture and version-pin the fields BodhiApp needs.

## Risks and contrarian findings

- **SSE remains unproven:** TCP-proxy architecture is encouraging, but only a live incremental test can satisfy BodhiApp's requirement
- **Funnel is beta:** Tailscale can change behavior and limits before general availability
- **No custom domains:** `*.ts.net` may be unacceptable for branded or enterprise deployments
- **Account onboarding remains:** Funnel removes the domain requirement, not the Tailscale account and device-login requirement
- **Personal plan restrictions:** the free plan is not intended for commercial use
- **Daemon ownership differs:** BodhiApp configures system Tailscale state that can outlive the app and may be shared with other local services
- **Bandwidth is unspecified:** Tailscale documents a non-configurable limit without publishing a throughput figure
- **macOS guidance conflicts:** packaging support needs a platform spike

## Recommendation and validation gate

Keep Tailscale Funnel as the leading future provider for users who want a stable public URL without owning a domain. Do not schedule full implementation until a focused spike passes these gates:

1. Incremental POST and GET SSE through Funnel
2. Browser sign-in and Keycloak callback sync through the `*.ts.net` hostname
3. Stable URL recovery after reboot and Tailscale restart
4. Disable without removing unrelated Funnel or Serve configuration
5. Supported behavior across BodhiApp's macOS, Windows, and Linux distributions
6. Acceptable latency and throughput for local model streaming

Cloudflare named tunnels remain the supported provider until those gates pass.

## Sources

- [Tailscale Funnel](https://tailscale.com/docs/features/tailscale-funnel): prerequisites, architecture, limits, approval, and plan availability
- [`tailscale funnel` CLI](https://tailscale.com/docs/reference/tailscale-cli/funnel): background mode, status JSON, reset, disable, and restart behavior
- [Tailscale Serve](https://tailscale.com/docs/features/tailscale-serve): private tailnet-only comparison
- [Tailscale pricing](https://tailscale.com/pricing): Personal-plan and commercial-use constraints
- [Custom-domain request](https://github.com/tailscale/tailscale/issues/11563): current custom-domain gap
- [WebSocket query issue](https://github.com/tailscale/tailscale/issues/18651): unresolved protocol-risk example
- [Tailscale license](https://raw.githubusercontent.com/tailscale/tailscale/main/LICENSE): BSD 3-Clause terms

## Rerun inputs

```text
workflow: firecrawl-deep-research
topic: Tailscale Funnel as a future BodhiApp Remote Access provider
depth: quick
output: markdown report
```
