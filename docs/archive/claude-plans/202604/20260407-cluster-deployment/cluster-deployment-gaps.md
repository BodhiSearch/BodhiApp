# BodhiApp Cluster Deployment — Known Gaps (Deploy As-Is)

**Date**: 2026-04-07
**Status**: Accepted gaps for cluster deployment without code changes
**Transport**: Streamable HTTP only (no deprecated SSE transport, no stdio)

---

## Context

This document lists the gaps if BodhiApp's MCP proxy is deployed in a multi-pod Kubernetes cluster **without any code changes**. Each gap is assessed for likelihood, user impact, and whether it's a spec violation or an operational concern.

---

## Gap 1: OAuth Token Refresh Race Condition

**Type**: Correctness bug
**Severity**: High
**Likelihood**: High (any time two pods serve the same MCP instance with an expired OAuth token)

**What happens**: The OAuth refresh lock (`RwLock<HashMap<String, Arc<Mutex<()>>>>`) is process-local. When two pods detect the same expired OAuth token simultaneously:
1. Both pods attempt to refresh the token
2. Refresh tokens are single-use — the second refresh invalidates the token issued by the first
3. One or both pods end up with an invalid access token
4. Subsequent requests to the upstream MCP server fail with 401

**User impact**: Intermittent auth failures on OAuth-authenticated MCP instances. Users see errors when calling tools. Self-heals only when the next refresh cycle succeeds without a race.

**Workaround**: Single-replica deployment for pods serving OAuth-authenticated MCP instances. Or accept occasional auth errors (client retries may succeed if they hit the pod that won the race).

---

## Gap 2: SSE Streams Killed by Infrastructure Timeouts

**Type**: Operational reliability
**Severity**: High
**Likelihood**: High (any SSE stream with >60s idle period)

**What happens**: Streamable HTTP responses can be `text/event-stream` (SSE). These long-lived streams pass through Kubernetes Ingress (default 60s idle timeout), cloud load balancers (AWS ALB 60s, GCP 30s), and reverse proxies (Nginx 60s `proxy_read_timeout`). If the upstream MCP server sends no data for longer than the shortest timeout in the chain, the connection is killed.

**Affected operations**:
- GET requests for server push notifications (long-lived by design)
- POST requests that return SSE streams for long-running tool calls with progress updates

**User impact**: SSE streams drop silently. Server push notifications stop arriving. Long-running tool calls may appear to hang then fail.

**Workaround**: Configure infrastructure timeouts higher (e.g., ALB idle timeout to 3600s, Nginx `proxy_read_timeout 3600s`). This shifts the problem but doesn't eliminate it — some infrastructure layers don't allow configurable timeouts.

---

## Gap 3: No Connection Draining for SSE Streams

**Type**: Operational reliability
**Severity**: Medium
**Likelihood**: High (every rolling deployment)

**What happens**: During rolling deployments, Kubernetes sends SIGTERM to pods and removes them from the Service endpoints. Active SSE streams are abruptly terminated. The default `terminationGracePeriodSeconds` is 30s, which may not be enough for SSE clients to detect the disconnection and reconnect.

**User impact**: Every deployment causes all active MCP SSE connections to drop. Clients see connection errors. If many clients reconnect simultaneously, a "thundering herd" effect may overload the remaining pods.

**Workaround**: Increase `terminationGracePeriodSeconds` to 120s in the pod spec. This gives more time for graceful shutdown but doesn't signal clients to reconnect proactively.

---

## Gap 4: No SSE Keepalive Comments

**Type**: MCP spec gap (SHOULD-level)
**Severity**: Medium
**Likelihood**: Medium (depends on upstream MCP server behavior)

**What happens**: The MCP spec recommends that Streamable HTTP servers send periodic SSE comments to keep connections alive. BodhiApp's proxy passes through whatever the upstream sends but does not inject its own keepalives. If the upstream server also doesn't send keepalives, the proxy-to-client leg of the connection is vulnerable to timeout even if the upstream-to-proxy leg stays alive.

**Spec reference**: "Servers SHOULD send periodic SSE comments (lines beginning with :) to keep the connection alive" (Transports > Streamable HTTP > Server-Sent Events)

**User impact**: Same as Gap 2 — streams die during idle periods. Compounded when the proxy adds another hop in the timeout chain.

**Workaround**: Rely on upstream MCP servers implementing their own keepalive. Most well-implemented MCP servers do this.

---

## Gap 5: No Last-Event-ID Buffering at Gateway Level

**Type**: MCP spec gap (MAY-level)
**Severity**: Low
**Likelihood**: Medium (any SSE disconnection + reconnection)

**What happens**: The proxy forwards `Last-Event-ID` from the client to the upstream, and this works correctly when the reconnection hits the same upstream server. However, if the upstream connection was re-established (e.g., due to proxy pod restart), the upstream may not have the event history to replay. The gateway does not buffer events to fill this gap.

**Spec reference**: "If an MCP server has previously assigned event IDs to its SSE events, the server SHOULD attempt to replay any events that the client may have missed" (Transports > Streamable HTTP > Resumability)

**User impact**: After a connection disruption, clients may miss notifications (tool list changes, resource updates). This is a data loss gap, not a crash. Clients can manually refresh to recover.

**Workaround**: None at the gateway level. Upstream MCP servers that implement event ID assignment and replay will handle this correctly as long as the upstream connection itself is not disrupted.

---

## Gap 6: Server-to-Client Requests Not Supported

**Type**: MCP spec gap (capability-dependent)
**Severity**: Low
**Likelihood**: Low (most MCP servers don't use these features through gateways)

**What happens**: The MCP spec allows servers to send requests to clients:
- `sampling/createMessage` — server asks client's LLM to generate text
- `roots/list` — server asks client for filesystem roots
- `elicitation` — server asks client for structured user input

BodhiApp's proxy does not intercept or route these. They flow through as part of the SSE stream to the client, but:
- The proxy does not advertise these capabilities during initialization (it doesn't participate in the handshake)
- If the client sends a response via POST, it is forwarded normally to the upstream

**User impact**: These features work in single-mode because the proxy is transparent. In cluster mode, they still work as long as the client's POST response reaches the same upstream server (which it does via `Mcp-Session-Id` passthrough). No additional cluster gap here beyond the general SSE reliability gaps above.

**Workaround**: None needed — transparent proxy behavior is sufficient for this.

---

## Gap 7: No Cross-Replica Notification Routing

**Type**: Architectural limitation
**Severity**: Low
**Likelihood**: Low (only affects GET-based server push when client reconnects to a different pod)

**What happens**: If a client has a GET SSE stream on Pod A for server push notifications, and Pod A dies, the client reconnects to Pod B. Pod B opens a new GET stream to the upstream. Notifications sent by the upstream between the disconnect and reconnect are lost.

**User impact**: Missed notifications during pod failover. The client doesn't know tools/resources changed until it manually refreshes. Self-heals when the new SSE stream is established.

**Workaround**: Accept the gap. Clients should implement periodic refresh as a fallback for missed notifications. The MCP spec does not require gateways to buffer notifications across replicas.

---

## Summary Table

| Gap                         | Type          | Severity | Spec Level      | Accept?                                                   |
| --------------------------- | ------------- | -------- | --------------- | --------------------------------------------------------- |
| OAuth refresh race          | Bug           | High     | N/A (app logic) | Accept with single-replica workaround for OAuth instances |
| SSE timeout kills           | Operational   | High     | SHOULD          | Accept with infrastructure config tuning                  |
| No connection draining      | Operational   | Medium   | N/A             | Accept with higher `terminationGracePeriodSeconds`        |
| No SSE keepalive            | Spec gap      | Medium   | SHOULD          | Accept — rely on upstream servers                         |
| No Last-Event-ID buffering  | Spec gap      | Low      | MAY             | Accept — upstream handles replay                          |
| Server-to-client requests   | Spec gap      | Low      | Capability      | Accept — transparent proxy is sufficient                  |
| Cross-replica notifications | Architectural | Low      | N/A             | Accept — self-heals on reconnection                       |

---

## Deployment Recommendations (No Code Changes)

If deploying as-is in cluster mode, apply these infrastructure-level mitigations:

1. **Single replica for OAuth-heavy workloads** — or accept intermittent auth failures
2. **Increase timeouts across the chain**:
   - `terminationGracePeriodSeconds: 120` in pod spec
   - Ingress: `proxy-read-timeout: 3600`, `proxy-send-timeout: 3600`
   - ALB idle timeout: 3600s
   - Nginx: `proxy_buffering off;` for SSE responses
3. **Client-side retry logic** — ensure the frontend MCP client reconnects on SSE disconnection with exponential backoff
4. **Health checks** — readiness probe on the HTTP server (not SSE endpoint)
5. **Pod disruption budget** — `minAvailable: 1` to prevent all pods from being terminated simultaneously during deploys
