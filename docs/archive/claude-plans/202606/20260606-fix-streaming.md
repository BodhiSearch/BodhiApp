# Plan: Fix Streaming Disconnect — Remove reqwest Total Timeout for AI API Client

## Context

The error `ERR_INCOMPLETE_CHUNKED_ENCODING 200 (OK)` at exactly the ~30s mark is caused by the **reqwest total request timeout of 30 seconds** in `DefaultAiApiClientFactory`. When the timeout fires mid-stream, reqwest aborts the upstream TCP connection, which causes `Body::from_stream` to terminate the chunked response to the browser mid-transfer. Chrome reports this as `ERR_INCOMPLETE_CHUNKED_ENCODING`.

This is confirmed by two facts:
1. The error occurs at exactly ~30s — matching `DEFAULT_TIMEOUT_SECS = 30`.
2. The upstream response headers (provided by the user) show `transfer-encoding: chunked` and `content-type: text/event-stream` — this is a long-lived streaming response. A 30s total timeout is fundamentally incompatible with streaming.

The same codebase already understands this: the MCP proxy (`mcp_proxy.rs:15-20`) uses a separate `reqwest::Client` with **only `connect_timeout`** and no request timeout, with the comment `// No request timeout — SSE streams are long-lived`.

Additionally, the upstream response includes `set-cookie` from Cloudflare (`_cfuvid=...`). This must **not** be forwarded to the browser — it is a Cloudflare infrastructure cookie scoped to `api.anthropic.com`, and forwarding it to the browser would pollute the client's cookie jar with a cookie that sets `Domain=api.anthropic.com` (browsers will reject it since the origin is `localhost`, but it is noise and a minor security concern).

## Two Problems to Fix

### Problem 1 — reqwest total timeout kills streaming at 30s (root cause of ERR_INCOMPLETE_CHUNKED_ENCODING)

`crates/services/src/ai_apis/ai_api_client_factory.rs:20,73`:
```rust
const DEFAULT_TIMEOUT_SECS: u64 = 30;  // ← this kills streams

SafeReqwest::builder()
  .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))  // total request timeout
  .allow_private_ips()
  .build()?
```

`reqwest`'s `.timeout()` is a **total request timeout** — it fires if the complete response (including body) doesn't finish within the window. For a streaming response that can run for minutes, this will always cut the stream at 30s.

The correct configuration for a streaming proxy is:
- Keep a **connect timeout** (guards against hanging DNS / TCP connects)
- Remove the **total request timeout** (let streams run as long as needed)

### Problem 2 — `set-cookie` from upstream (Cloudflare) should not be forwarded

The upstream Anthropic response sends `set-cookie: _cfuvid=...` (a Cloudflare infrastructure cookie). This must not be forwarded to clients. It should be added to the hop-by-hop / infrastructure header strip list.

## Fix

### Step 1 — Split the timeout into connect-only in `ai_api_client_factory.rs`

Change `DefaultAiApiClientFactory::with_db` and `::new` to use only `connect_timeout`, mirroring the MCP proxy pattern:

```rust
// Before
SafeReqwest::builder()
  .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
  .allow_private_ips()
  .build()?

// After
SafeReqwest::builder()
  .connect_timeout(Duration::from_secs(DEFAULT_CONNECT_TIMEOUT_SECS))
  .allow_private_ips()
  .build()?
```

Rename the constant:
```rust
const DEFAULT_CONNECT_TIMEOUT_SECS: u64 = 30;
```

Check whether `SafeReqwest::builder()` exposes `connect_timeout` — if not, add it (see Step 1b).

### Step 1b — Add `connect_timeout` to `SafeReqwest` if missing

`crates/services/src/shared_objs/safe_reqwest.rs` — check if `SafeReqwestBuilder` has `connect_timeout`. If it only has `timeout`, add `connect_timeout` field and wire it through to `reqwest::ClientBuilder::connect_timeout(...)`.

### Step 2 — Add `set-cookie` to the hop-by-hop strip list in `provider_shared.rs`

`set-cookie` is not an RFC hop-by-hop header but it is an **infrastructure/upstream-scoped** response header that must not leak through a proxy. Upstream providers (Anthropic via Cloudflare, others) may set cookies scoped to their own domains; forwarding them is meaningless and potentially confusing.

Extend `is_hop_by_hop` in `crates/services/src/ai_apis/provider_shared.rs`:

```rust
fn is_hop_by_hop(name: &str) -> bool {
  matches!(
    name.to_ascii_lowercase().as_str(),
    "connection"
      | "keep-alive"
      | "proxy-authenticate"
      | "proxy-authorization"
      | "te"
      | "trailers"
      | "transfer-encoding"
      | "upgrade"
      | "set-cookie"    // ← add: upstream infrastructure cookies must not leak
  )
}
```

## Files to Modify

| File | Change |
|------|--------|
| `crates/services/src/ai_apis/ai_api_client_factory.rs` | Replace `DEFAULT_TIMEOUT_SECS` + `.timeout()` with `DEFAULT_CONNECT_TIMEOUT_SECS` + `.connect_timeout()` in both `with_db` and `new` |
| `crates/services/src/shared_objs/safe_reqwest.rs` | Add `connect_timeout` field + builder method if not present; wire to `reqwest::ClientBuilder::connect_timeout` |
| `crates/services/src/ai_apis/provider_shared.rs` | Add `"set-cookie"` to `is_hop_by_hop` matcher |

## Verification

### Confirm `SafeReqwestBuilder` has or needs `connect_timeout`

Before implementing, read `safe_reqwest.rs` to verify the builder API. If `connect_timeout` already exists as a separate field from `timeout`, the factory change is straightforward. If not, add it.

### Unit test updates

- Update `test_ai_api_forward.rs` `test_forward_response_strips_hop_by_hop_headers` to also assert `set-cookie` is stripped.
- Add/update a test that the factory client is built with `connect_timeout` only (or at minimum, document that the 30s constant is now connect-only).

### Gate checks

```bash
cargo test -p services --lib
cargo test -p routes_app --lib
make test.backend
```

### Manual verification

Start the server (`make app.run`) and make a streaming request that takes >30s. Confirm the stream completes without `ERR_INCOMPLETE_CHUNKED_ENCODING`.
