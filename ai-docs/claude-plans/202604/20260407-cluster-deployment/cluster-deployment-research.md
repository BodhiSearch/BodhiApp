# BodhiApp Cluster Deployment — MCP Gateway Research & Architecture

**Date**: 2026-04-07
**Status**: Research / Pre-implementation

---

## 1. Problem Statement

BodhiApp currently deploys as a single instance (Tauri desktop or single Docker container). We want to support `bodhi_instance_mode=cluster` for horizontally-scaled Kubernetes deployments while keeping our MCP gateway fully compliant with the MCP specification (2025-03-26).

The MCP protocol presents specific challenges for distributed deployment because it is inherently **stateful** and **bidirectional** — properties that conflict with stateless container orchestration patterns.

---

## 2. MCP Spec — Stateful & Distributed Challenges

### 2.1 Session Lifecycle (Spec: Transports > Streamable HTTP)

The MCP session is created during an `initialize` handshake:

1. Client sends `initialize` POST (no `Mcp-Session-Id` header)
2. Server responds with `Mcp-Session-Id` header + negotiated capabilities
3. Client sends `notifications/initialized` POST (with `Mcp-Session-Id`)
4. All subsequent requests MUST include `Mcp-Session-Id`
5. Server returns `404` if session is unknown — client must fully re-initialize

**State tied to a session**: negotiated protocol version, negotiated capabilities (both sides), resource subscriptions, active SSE streams, event ID cursors, logging level.

**Cluster implication**: All requests for a given session must reach the same upstream MCP server instance, or the upstream returns 404.

### 2.2 Bidirectional Communication

| Direction                                     | Mechanism                                          |
| --------------------------------------------- | -------------------------------------------------- |
| Client -> Server                              | HTTP POST with JSON-RPC body                       |
| Server -> Client (response)                   | JSON response OR SSE stream on POST response       |
| Server -> Client (push)                       | Client opens GET -> receives long-lived SSE stream |
| Client -> Server (response to server request) | HTTP POST with JSON-RPC response                   |

The GET-initiated SSE stream is the hardest challenge — it's a **long-lived HTTP connection** for receiving unsolicited server push notifications (tool list changes, resource updates, progress, log messages, sampling requests).

### 2.3 Server Push Notifications

Servers can push: `notifications/resources/list_changed`, `notifications/resources/updated`, `notifications/tools/list_changed`, `notifications/prompts/list_changed`, `notifications/progress`, `notifications/message` (logging), `sampling/createMessage`, `roots/list`.

These arrive on the GET-initiated SSE stream or interleaved in POST-response SSE streams.

### 2.4 SSE Stream Resumability

- Server MAY assign `id` fields to SSE events
- Client reconnects with `Last-Event-ID` header
- Server MAY replay missed events (per-stream, not cross-stream)
- Without event IDs, messages during disconnect window are **lost**

### 2.5 Concurrency

- Multiple SSE streams MAY exist per session simultaneously
- Server MUST send each message on **only one** stream (no broadcasting)
- Multiple POSTs can be in-flight concurrently
- JSON-RPC batching is supported

---

## 3. Current BodhiApp MCP Proxy Architecture

### 3.1 What Exists Today

The MCP proxy (`crates/routes_app/src/mcps/mcp_proxy.rs`) is a **transparent HTTP reverse proxy**:

- **Endpoint**: `/bodhi/v1/apps/mcps/{id}/mcp` — accepts POST, GET, DELETE
- **Stateless**: `Mcp-Session-Id` is opaque — forwarded between client and upstream without interpretation
- **Streaming**: SSE responses streamed via `Body::from_stream()` without buffering
- **Auth injection**: Headers + query params resolved from DB (header auth or OAuth with token refresh)
- **Shared HTTP client**: Static `reqwest::Client` with connection pooling, no request timeout (for SSE)
- **Headers forwarded**: `content-type`, `accept`, `mcp-session-id`, `mcp-protocol-version`, `last-event-id` (request); `content-type`, `mcp-session-id`, `mcp-protocol-version`, `cache-control` (response)

### 3.2 In-Memory State

| State                   | Location                                                      | Cluster Impact                                       |
| ----------------------- | ------------------------------------------------------------- | ---------------------------------------------------- |
| `HTTP_CLIENT` (reqwest) | Static, per-pod                                               | Fine — each pod has its own pool                     |
| OAuth refresh locks     | `RwLock<HashMap<String, Arc<Mutex<()>>>>` in `mcp_service.rs` | **Problem** — local-only, race condition across pods |
| MCP session state       | None held                                                     | Good — proxy is already stateless                    |

### 3.3 Persisted State

All MCP configuration, auth parameters, and OAuth tokens are stored in the database (PostgreSQL in production). This is already cluster-safe via row-level security and tenant isolation.

---

## 4. Gap Analysis: Current Proxy vs Full MCP Spec in Cluster Mode

### 4.1 Critical Gaps

| Gap                              | Severity   | Description                                                                                                                                                                      |
| -------------------------------- | ---------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **OAuth refresh race condition** | **High**   | Process-local mutex. Two pods detecting expired token attempt concurrent refresh. Refresh tokens are single-use — second refresh invalidates the first.                          |
| **No SSE keepalive**             | **High**   | SSE streams pass through multiple proxy layers (Ingress, LB, CDN), each with idle timeouts (typically 60s). Without keepalive comments, streams are killed during quiet periods. |
| **No connection draining**       | **Medium** | Rolling deploys terminate pods with active SSE connections. Clients experience abrupt disconnection with no graceful signal.                                                     |

### 4.2 Specification Gaps (Not currently handled)

| Gap                                       | Severity   | Description                                                                                                                                                                 |
| ----------------------------------------- | ---------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **No Last-Event-ID buffering**            | **Medium** | Header is forwarded to upstream, but if the upstream connection breaks and is re-established on a different pod, buffered events are lost.                                  |
| **No server-to-client request support**   | **Low**    | Sampling (`sampling/createMessage`) and roots (`roots/list`) are server-initiated requests. Currently not proxied. No gateway in production fully supports this either.     |
| **No cross-replica notification routing** | **Low**    | Server push via GET SSE reaches only the pod holding that connection. If the client reconnects to a different pod, notifications between disconnect and reconnect are lost. |

### 4.3 Non-Gaps (Already handled)

| Feature                | Status  | Notes                                                  |
| ---------------------- | ------- | ------------------------------------------------------ |
| Session ID passthrough | Working | Opaque forwarding — upstream manages session state     |
| SSE streaming          | Working | `Body::from_stream()` without buffering                |
| Multi-method support   | Working | POST, GET, DELETE forwarded correctly                  |
| Accept header forcing  | Working | Correct `application/json, text/event-stream` for POST |
| Auth injection         | Working | Headers + query params from DB                         |
| Multi-tenant isolation | Working | RLS in PostgreSQL, application-level filtering         |

---

## 5. Architectural Patterns for Distributed MCP Gateways

Research across the MCP gateway ecosystem reveals three proven patterns. Each makes different trade-offs.

### Pattern A: Encrypted Session Tokens (Stateless Gateway)

**How it works**: The gateway encodes upstream session state (upstream session IDs, pinned backend addresses) into an encrypted token. This token IS the `Mcp-Session-Id` sent to the client. Any replica decrypts the token to recover routing information.

**Pros**: Truly stateless — no external session store needed. Any pod handles any request. Scales to thousands of pods.

**Cons**: Token size grows with upstream count. Every cross-pod request requires decrypting and re-establishing upstream connections (latency). Cannot support persistent GET SSE streams across pods (each reconnection is a new upstream session). Client sees an encrypted blob, not the server-issued ID.

**Requires**: Shared encryption key across all replicas (e.g., K8s Secret).

**Not applicable to BodhiApp**: Our proxy is already stateless via opaque passthrough. We don't need to encode/decode session state because we don't manage sessions — the upstream does. This pattern solves a problem we don't have.

### Pattern B: Distributed Session Store (External Cache)

**How it works**: The gateway stores `session_id -> upstream_pod_address` in a shared cache (Redis, Cosmos DB). Any replica looks up the target and routes directly to the correct upstream pod.

**Pros**: Any gateway replica can serve any request. Session state survives pod restarts. Clean separation between session routing and request handling.

**Cons**: External dependency (Redis/Cosmos). Cache coherence issues (stale entries if upstream pod dies). No session migration — if the upstream pod dies, the session is lost until cache expires.

**Requires**: Distributed cache + upstream pods with stable addresses (e.g., StatefulSet with headless service).

**Partially applicable to BodhiApp**: Useful IF we need to route to specific upstream MCP server instances. Currently, our upstream URLs are stable (configured per MCP instance in DB), so this is only relevant if upstream MCP servers themselves run in a cluster with session affinity requirements.

### Pattern C: Application-Level Session Forwarding (Pub/Sub)

**How it works**: Each worker/pod claims session ownership in Redis (SETNX with TTL). Non-owning pods forward requests to the owner via Redis Pub/Sub. Dead worker detection uses CAS (compare-and-swap) Lua scripts.

**Pros**: No sticky sessions needed at LB level. Full MCP spec support including server push forwarding. Stream resumability via event store.

**Cons**: Highest complexity. Pub/Sub forwarding adds latency. Transport objects (SSE connections) remain local — the owning pod must stay alive for push notifications.

**Requires**: Redis for ownership tracking, Pub/Sub, and event storage.

**Consider for Phase 3**: Only if BodhiApp needs full bidirectional MCP support (server-to-client requests, cross-replica push notifications).

---

## 6. Recommended Architecture

### 6.1 Design Principles

1. **Keep the proxy stateless** — don't add session management that doesn't exist today
2. **Redis as the only new dependency** for cluster mode
3. **Graceful degradation** — cluster features enhance reliability but single-mode still works
4. **Phased rollout** — solve the critical gaps first, add spec completeness later
5. **Wait for spec evolution** — MCP is targeting stateless mode by default (June 2026)

### 6.2 Phase 1: Minimal Cluster Support

**Scope**: Make the existing proxy safe for multi-pod deployment.

#### 6.2.1 Distributed OAuth Refresh Lock

Replace the process-local `RwLock<HashMap<String, Arc<Mutex<()>>>>` with a Redis-based distributed lock.

**Algorithm**:
1. Attempt `SET oauth_refresh:{config_id} {pod_id} PX 30000 NX` in Redis
2. If acquired: perform token refresh, store new tokens in DB, release lock
3. If not acquired: poll token store (1s interval, 30s timeout) waiting for the winning pod to complete
4. Store `(access_token, refresh_token, expires_at)` atomically in a single DB write

**Fallback**: If Redis is unavailable, fall back to process-local mutex (accept the race condition risk). Log a warning.

#### 6.2.2 SSE Keepalive Injection

When the upstream response has `Content-Type: text/event-stream`, wrap the response stream with a keepalive-injecting adapter:

- Send `:keepalive\n\n` (SSE comment) every 30 seconds if no data flows
- This prevents idle timeout disconnections across the infrastructure chain (Ingress 60s, ALB 60s, Envoy 5min)
- SSE comments are invisible to MCP clients (they're not events)

**Implementation**: Wrap `Body::from_stream()` with a `tokio::select!` loop that races between the upstream stream and a 30-second timer.

#### 6.2.3 Connection Draining

On SIGTERM (Kubernetes pod termination):

1. Stop accepting new SSE GET connections (set a drain flag)
2. Continue serving in-flight POST requests
3. For active SSE streams: send a final `:closing\n\n` comment and close gracefully
4. Wait for `terminationGracePeriodSeconds` (recommend 120s for SSE workloads)

**Kubernetes config**: Set `terminationGracePeriodSeconds: 120` in the pod spec. Add a readiness probe that fails when draining.

#### 6.2.4 Config Model

```rust
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum BodhiInstanceMode {
    #[default]
    Single,   // SQLite, local OAuth mutex, no keepalive injection
    Cluster,  // PostgreSQL, Redis distributed lock, SSE keepalive, connection draining
}
```

Driven by `BODHI_INSTANCE_MODE` env var. When `Cluster`, require `REDIS_URL` configuration.

### 6.3 Phase 2: Enhanced Streaming Reliability

**Scope**: Make SSE streams recoverable across pod failures.

#### 6.3.1 Event Store for Last-Event-ID

When proxying SSE responses, buffer events in Redis with event IDs:

- Parse SSE events from the upstream stream
- Assign sequential event IDs (per-session, per-stream)
- Store in Redis as a ring buffer with TTL (e.g., last 100 events, 1h TTL)
- On reconnect with `Last-Event-ID`, replay buffered events before resuming upstream stream

**Data model**:
- Key: `mcp:events:{session_id}:{stream_id}` (sorted set)
- Event ID index: `mcp:event_idx:{event_id}` -> `{session_id}:{stream_id}:{position}` (with TTL)

#### 6.3.2 Graceful SSE Stream Migration

When a pod is draining, signal clients to reconnect:

1. Send a final SSE event with a custom `event: reconnect` type
2. Client-side code detects this and reconnects with `Last-Event-ID`
3. New pod picks up from the event store

### 6.4 Phase 3: Full Bidirectional Support (If Needed)

**Scope**: Support server-to-client requests (sampling, elicitation) across pod boundaries.

#### 6.4.1 Redis Pub/Sub Notification Bus

When an upstream MCP server sends a notification or request:

1. The pod receiving it publishes to `mcp:push:{session_id}` in Redis
2. The pod holding the client's SSE stream subscribes to that channel and forwards
3. Responses from the client are published back via `mcp:response:{request_id}`

**Dead connection detection**: If no subscriber is listening (client disconnected), buffer the notification with TTL for eventual delivery on reconnect.

#### 6.4.2 Session Ownership Tracking

- On SSE GET connection: `SET mcp:owner:{session_id} {pod_id} EX 300 NX`
- Heartbeat: refresh TTL every 60 seconds while SSE connection is alive
- On disconnect: delete ownership key
- Dead pod detection: ownership key expires naturally via TTL

### 6.5 Phase 4: MCP Spec Evolution (Monitor)

The MCP spec team is working on changes targeted for June 2026 that would simplify cluster deployment:

- **Stateless protocol mode**: Replace initialize handshake with per-request capability info. Each request is independent.
- **Server Cards** (`/.well-known/mcp.json`): Pre-connection capability discovery, eliminating the initialize round-trip.
- **Explicit subscription streams**: Replace general-purpose GET stream with item-specific subscriptions with TTL + ETags.
- **HTTP routing optimization**: Expose RPC method/tool name in HTTP path/headers for load balancer routing without JSON-RPC body parsing.

If these land, Phase 3 complexity becomes unnecessary. Phase 1-2 remains valuable regardless.

---

## 7. Implementation Priority & Effort

| Phase | Items                                                                            | Effort        | Dependencies       | Value                                |
| ----- | -------------------------------------------------------------------------------- | ------------- | ------------------ | ------------------------------------ |
| **1** | Distributed OAuth lock, SSE keepalive, connection draining, instance mode config | ~1-2 weeks    | Redis client crate | Enables safe multi-pod deployment    |
| **2** | Event store, Last-Event-ID buffering, graceful SSE migration                     | ~1-2 weeks    | Phase 1            | SSE streams survive pod failures     |
| **3** | Pub/Sub notification bus, session ownership, bidirectional support               | ~2-3 weeks    | Phase 2            | Full MCP spec in cluster (if needed) |
| **4** | Adopt stateless MCP mode                                                         | Wait for spec | June 2026 spec     | Eliminates Phase 3 complexity        |

---

## 8. Architectural Decisions

| Decision                    | Options                                                          | Recommendation                                                                                               | Rationale                                                                                                              |
| --------------------------- | ---------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------- |
| **Session management**      | A) Opaque passthrough B) Token encoding C) Distributed store     | **A — keep opaque passthrough**                                                                              | BodhiApp doesn't manage MCP sessions; upstream servers do. Adding session management creates complexity without value. |
| **External dependencies**   | A) Redis only B) Redis + message broker C) None                  | **A — Redis only**                                                                                           | Standard K8s dependency. Covers distributed locks, event store, and pub/sub in a single service.                       |
| **SSE stream model**        | A) Pure passthrough B) Intercept + buffer C) Terminate + re-emit | **B — Intercept + buffer**                                                                                   | Add keepalive injection and event ID tracking without fully terminating the stream.                                    |
| **Load balancing**          | A) Round-robin B) Header-based sticky C) App-level routing       | **A — Round-robin**                                                                                          | Proxy is stateless (opaque session ID passthrough). Any pod can handle any request.                                    |
| **Instance mode semantics** | What does `cluster` vs `single` change?                          | `single`: SQLite, local mutex, no keepalive. `cluster`: PostgreSQL, Redis, SSE keepalive, distributed locks. | Clear separation of concerns.                                                                                          |
| **Bidirectional support**   | A) Skip B) Phase 3 C) Full from start                            | **B — Phase 3 (if needed)**                                                                                  | No production MCP gateway fully supports bidirectional. Wait for concrete use case.                                    |

---

## 9. Risk Assessment

| Risk                                     | Likelihood | Impact | Mitigation                                                |
| ---------------------------------------- | ---------- | ------ | --------------------------------------------------------- |
| SSE streams break during rolling deploys | High       | Medium | Connection draining (Phase 1) + client reconnection       |
| OAuth token refresh race across pods     | High       | High   | Redis distributed lock (Phase 1)                          |
| Upstream MCP server rejects reconnection | Medium     | Medium | Client-visible error; user retries; event store (Phase 2) |
| Redis unavailability in cluster mode     | Low        | High   | Health check; degrade to local-only; log warning          |
| Notification loss during pod switch      | Medium     | Low    | Acceptable for Phase 1; event store in Phase 2            |
| MCP spec changes making Phase 3 obsolete | Medium     | Medium | Phased approach avoids premature investment               |
| Infrastructure timeout chain kills SSE   | High       | Medium | SSE keepalive injection (Phase 1)                         |

---

## 10. Open Questions

1. **Upstream MCP server clustering**: If the upstream MCP servers BodhiApp proxies to are themselves clustered, we may need Pattern B (distributed session store) to pin to specific upstream pods. Is this a use case we need to support?

2. **Bidirectional use cases**: Do any of our MCP integrations require server-to-client requests (sampling, elicitation)? If not, Phase 3 can be deferred indefinitely.

3. **Redis availability**: Is Redis already part of our planned cluster infrastructure, or does it need to be added? What about managed Redis services (ElastiCache, Memorystore)?

4. **Client reconnection behavior**: How do our MCP clients (the frontend UI) handle SSE disconnection and reconnection? Do we need to add retry logic with exponential backoff?

5. **MCP spec timeline**: Should we wait for the June 2026 spec update before investing in Phase 3, or do we have concrete use cases that require it sooner?
