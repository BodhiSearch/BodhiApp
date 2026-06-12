# Holistic refactor: remove `LlmEndpoint`, consolidate `AiApiClientFactory`, address I1/I2

## Context

Three refactors land together because they tighten the same dispatch-and-construction surface and share the same upstream-to-downstream layer walk. After the `8926f448` refactor unified all dispatch through `AiApiClientFactory`, several pieces of the old plumbing are now redundant or inconsistent — the user asked to clean them up holistically.

### 1. Remove `LlmEndpoint` (user-driven, primary)

After unification through `AiApiClientFactory::for_alias(...).forward_request_with_method(method, api_path, ...)`, `LlmEndpoint` is dead weight:

- **Routes already know `(Method, path)`** at construction time — they build `LlmEndpoint::ChatCompletions`, then immediately call `endpoint.http_method()` and `endpoint.api_path()` to recover the same method+path they just encoded.
- **`LocalLlamaClient::forward_request_with_method`** receives `api_path: &str`, reverse-maps it into `LlmEndpoint::ChatCompletions | Embeddings` (`crates/services/src/ai_apis/clients/local_llama.rs:45–54`), then calls `LocalLlama::forward_request(endpoint, ...)`.
- **`SharedContext::forward_request`** receives `LlmEndpoint`, forwards to `dispatch_local`, which only ever cares about `ChatCompletions` vs `Embeddings` (every other variant is `ContextError::Unreachable`).
- **Smoking gun**: `dispatch_responses_op` in `routes_oai_responses.rs:128–157` already takes `(native_endpoint: LlmEndpoint, liberty_method: Method, liberty_path: String)` — same function, asymmetric abstraction, Liberty arm proves the enum is unnecessary.

### 2. Consolidate `AiApiClientFactory` method surface (user-driven)

The trait currently exposes **4 methods** with overlapping concerns:

| Method | Used for | Key property |
|---|---|---|
| `for_alias(alias, api_key)` | All non-Liberty paths (local + key-based remote) | Standard |
| `for_envelope(envelope)` | Liberty pre-persistence validation flow | **Stateless**: `NoOpRefresh`, empty `alias_id` ⇒ 401 surfaces directly (no DB call) |
| `for_resolved_credentials(creds, alias, tenant_id, user_id)` | Liberty per-request flow | **Stateful**: `DefaultLlmLibertyRefresh`, populated `alias_id` ⇒ 401 triggers `force_refresh + retry-once` |
| `safe_http_client()` | OAuth refresh path (`providers/mod.rs:44`) | Connection-pool reuse |

**Audit findings** (from explore agent + code reading):
- The two Liberty methods exist for **genuine** reasons. The 401-retry guard in `LibertyAnthropicClient::forward_request_with_method` is `if response.status() != UNAUTHORIZED || self.alias_id.is_empty() { return Ok(response); }` — collapsing them by passing a synthetic `alias_id` would fire DB refresh on a non-existent row during validation. The two distinct lifecycle stages must remain semantically distinguishable.
- However, the **API surface** can be unified: instead of 2 methods, expose 1 method that takes a `LibertySource<'_>` enum with two variants (`Envelope` and `Resolved`). The lifecycle stage becomes type-encoded at the call site, and the impl matches once.
- `safe_http_client()` is justified by Arc-shared `SafeReqwest` connection pooling — removing it would either lose pooling or require a bigger refactor (move `ensure_fresh_credentials` into the factory). **Keep with documentation.**

**Result**: trait surface drops from 4 methods to 3 (`for_alias`, `for_liberty`, `safe_http_client`).

### 3. Close out review findings I1 + I2

- **I1** — `LocalLlamaImpl::forward_request` calls `on_request_completed()` on both `Ok` and `Err`. The deleted `StandaloneInferenceService::forward_local` only ran it after `?`. **User direction**: keep the new behavior — even an errored response involved touching the model, and the keep-alive timer should debounce from any user activity, not only successes. **Document the intent via a 3-line comment.**
- **I2** — Zero tests for the keep-alive timer. **User direction**: evaluate value vs cost.

#### Cost-vs-value analysis for I2 (decision: skip)

| Test | Cost | Value | Decision |
|---|---|---|---|
| `keep_alive=-1` no-op | Tiny | Low — verbatim from old code, branch is one match arm | Skip |
| `keep_alive=0` immediate stop | Small — needs `tokio::task::yield_now()` to drain spawned task | Small — locks "0 = stop now" semantic | Skip |
| `keep_alive>0` debounced stop | **Medium-High** — `crates/server_core/` has **no `tokio::time::pause()` precedent**; `direct_sse`/`fwd_sse` use real 10ms sleeps and accept flake risk; debounce test would need ≥50ms | Small — verbatim from old code | Skip |
| Wrapper test for I1 (timer fires on error) | Small | Medium — protects intentional design from accidental revert | **Replace with comment** |

A 3-line code comment beside `on_request_completed()` is cheaper than a flaky-prone test and gives equivalent regression protection (a future PR that "fixes" I1 by reverting must also explain why they're deleting the comment).

## Critical Files

### Definition + trait files
- `crates/services/src/inference/llm_endpoint.rs` — **delete** (entire file: enum + 16 variants + `api_path()` + `http_method()` + tests)
- `crates/services/src/inference/mod.rs` — drop `pub use llm_endpoint::LlmEndpoint;` and `mod llm_endpoint;`
- `crates/services/src/inference/local_llama.rs` — change `LocalLlama::forward_request` signature: `endpoint: LlmEndpoint` → `api_path: &str`
- `crates/services/src/ai_apis/ai_api_client_factory.rs` — collapse 3 Liberty entry points to 1 `for_liberty(LibertySource)`; introduce `LibertySource<'_>` enum
- `crates/services/src/ai_apis/auth_scoped.rs` — wrapper passes through; rename `for_resolved_credentials` → `for_resolved`; both Liberty wrappers convert into `LibertySource`
- `crates/services/src/ai_apis/clients/local_llama.rs` — drop the reverse-map; pass `api_path` straight through
- `crates/services/src/ai_apis/clients/liberty_anthropic.rs` + `liberty_codex.rs` — no API changes; the `from_envelope`/`from_credentials` constructors stay (called from inside `for_liberty`)
- `crates/services/src/models/api_model_service.rs` — 4 call sites: `for_envelope` → `for_liberty(LibertySource::Envelope)`, `for_resolved_credentials` → `for_liberty(LibertySource::Resolved {..})`
- `crates/server_core/src/lib.rs:18` — drop `pub use services::inference::LlmEndpoint;`
- `crates/server_core/src/local_llama_impl.rs` — pass through new signature; **add I1 comment**
- `crates/server_core/src/shared_rw.rs` — `SharedContext::forward_request` signature change; `dispatch_local` matches `&str` not enum
- `crates/server_core/src/test_shared_rw.rs` — replace 4× `LlmEndpoint::ChatCompletions` and 1× `LlmEndpoint::Responses` with string literals

### Route handler files (call-site updates)
- `crates/routes_app/src/oai/routes_oai_chat.rs` — 2 LlmEndpoint sites
- `crates/routes_app/src/oai/routes_oai_responses.rs` — 5 LlmEndpoint sites + `dispatch_responses_op` signature simplification
- `crates/routes_app/src/anthropic/routes_anthropic.rs` — 1 LlmEndpoint site; `for_resolved_credentials` → `for_resolved`
- `crates/routes_app/src/gemini/routes_gemini.rs` — 4 LlmEndpoint variants collapse to one `format!`
- `crates/routes_app/src/ollama/routes_ollama.rs` — 1 LlmEndpoint site + stale comment cleanup
- `crates/routes_app/src/models/api/routes_api_models.rs` — 4 call sites (`for_envelope`/`for_resolved_credentials` → wrapper renames)
- `crates/routes_app/src/oai/test_oai_responses_success.rs:66` — stale comment cleanup
- `crates/routes_app/src/anthropic/test_anthropic_messages.rs:198` — stale comment cleanup

## Design Details

### A. LlmEndpoint removal — new signatures

**`services::inference::LocalLlama::forward_request`**:
```rust
async fn forward_request(
    &self,
    api_path: &str,
    request: Value,
    alias: Alias,
) -> Result<reqwest::Response, LocalLlamaError>;
```
Method dropped — local llama is always POST. The `api_path` carries the only discrimination needed (`/chat/completions` vs `/embeddings`).

**`server_core::SharedContext::forward_request`** — same shape:
```rust
async fn forward_request(
    &self,
    api_path: &str,
    request: Value,
    alias: Alias,
) -> Result<reqwest::Response>;
```

**`server_core::shared_rw::dispatch_local`** (private):
```rust
async fn dispatch_local(
    server: &(dyn Server + '_),
    api_path: &str,
    input: &Value,
) -> Result<reqwest::Response> {
    match api_path {
        "/chat/completions" => Ok(server.chat_completions(input).await?),
        "/embeddings" => Ok(server.embeddings(input).await?),
        other => Err(ContextError::Unreachable(format!(
            "path '{}' is not supported for local models", other
        ))),
    }
}
```

**`LocalLlamaClient::forward_request_with_method`** simplification:
```rust
async fn forward_request_with_method(
    &self,
    _method: &Method,
    api_path: &str,
    request: Option<Value>,
    _query_params: Option<Vec<(String, String)>>,
    _client_headers: Option<Vec<(String, String)>>,
) -> Result<Response> {
    if !matches!(api_path, "/chat/completions" | "/embeddings") {
        return Err(AiApiClientFactoryError::NotFound(format!(
            "endpoint '{}' is not supported for local model aliases", api_path
        )));
    }
    let body = request.unwrap_or(Value::Null);
    let resp = self.local_llama.forward_request(api_path, body, self.alias.clone()).await?;
    convert_reqwest_to_axum(resp)
}
```
The early validation stays — without it, a misrouted Gemini/Anthropic path on a local alias would surface as 500 (`ContextError::Unreachable` → `LocalLlamaError::Internal` → `ApiError`) instead of 404 (`NotFound` → BadRequest).

### B. Route call-site pattern

**Standard fixed-path routes** (chat, embeddings, ollama, anthropic, responses-create):
```rust
// Before
let endpoint = LlmEndpoint::ChatCompletions;
client.forward_request_with_method(endpoint.http_method(), &endpoint.api_path(), ...)
// After
client.forward_request_with_method(&Method::POST, "/chat/completions", ...)
```

**Gemini routes** — 4 LlmEndpoint variants collapse to one `format!`:
```rust
// Before
let endpoint = match action {
    "generateContent" => LlmEndpoint::GeminiGenerateContent(stripped_model),
    "streamGenerateContent" => LlmEndpoint::GeminiStreamGenerateContent(stripped_model),
    "embedContent" => LlmEndpoint::GeminiEmbedContent(stripped_model),
    "batchEmbedContents" => LlmEndpoint::GeminiBatchEmbedContents(stripped_model),
    _ => unreachable!(),
};
client.forward_request_with_method(endpoint.http_method(), &endpoint.api_path(), ...)
// After (action already validated against GEMINI_ACTIONS earlier)
let upstream_path = format!("/models/{}:{}", stripped_model, action);
client.forward_request_with_method(&Method::POST, &upstream_path, ...)
```

**`dispatch_responses_op` simplification** — kills the asymmetric arms:
```rust
// Before
async fn dispatch_responses_op(
    auth_scope: &AuthScope,
    resolution: ResponsesAliasResolution,
    native_endpoint: LlmEndpoint,
    liberty_method: Method,
    liberty_path: String,
    query_params: Option<Vec<(String, String)>>,
) -> Result<Response, OaiApiError> { ... }

// After — both arms pass (method, path) directly
async fn dispatch_responses_op(
    auth_scope: &AuthScope,
    resolution: ResponsesAliasResolution,
    method: Method,
    upstream_path: String,
    query_params: Option<Vec<(String, String)>>,
) -> Result<Response, OaiApiError> {
    let client = match resolution {
        ResponsesAliasResolution::Native { alias, api_key } =>
            auth_scope.ai_api().for_alias(&Alias::Api(alias), api_key).map_err(OaiApiError::from)?,
        ResponsesAliasResolution::Liberty { alias, creds } =>
            auth_scope.ai_api().for_resolved(&creds, &alias).map_err(OaiApiError::from)?,
    };
    client.forward_request_with_method(&method, &upstream_path, None, query_params, None)
        .await.map_err(OaiApiError::from)
}
```
Callers drop one redundant arg: `dispatch_responses_op(.., LlmEndpoint::ResponsesGet(id), Method::GET, format!("/responses/{}", id), ..)` → `dispatch_responses_op(.., Method::GET, format!("/responses/{}", id), ..)`.

### C. AiApiClientFactory consolidation

**New trait surface** (`crates/services/src/ai_apis/ai_api_client_factory.rs`):

```rust
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait]
pub trait AiApiClientFactory: Send + Sync + std::fmt::Debug {
    /// Per-request client for non-Liberty alias flows.
    /// - `Alias::User`/`Alias::Model` → `LocalLlamaClient` (or `LocalNotSupportedInCluster`)
    /// - `Alias::Api` with key-based formats → corresponding remote client with `api_key`
    /// - `Alias::Api` with `LlmLibertyOauth` format → `LibertyRequiresCredentials` (caller must use `for_liberty`)
    fn for_alias(&self, alias: &Alias, api_key: Option<String>) -> Result<Box<dyn AiApiClient>>;

    /// Per-request client for an `LlmLibertyOauth` source.
    /// - `LibertySource::Envelope` → stateless validation client (no DB, no refresh, 401 surfaces directly)
    /// - `LibertySource::Resolved` → request-time client (DB-backed `force_refresh` on 401)
    fn for_liberty(&self, source: LibertySource<'_>) -> Result<Box<dyn AiApiClient>>;

    /// Shared `SafeReqwest` for OAuth refresh paths that reuse the same connection pool as model traffic.
    /// Justified by Arc-shared pooling; not an AI-client concern but the cheapest place to expose it.
    fn safe_http_client(&self) -> SafeReqwest;
}

pub enum LibertySource<'a> {
    /// Pre-persistence validation: stateless client, no refresh, 401 surfaces directly.
    Envelope(&'a LlmLibertyEnvelope),
    /// Request-time: stateful client, force-refresh on 401.
    Resolved {
        creds: &'a ResolvedLlmLibertyCredentials,
        alias_id: &'a str,
        prefix: Option<String>,
        tenant_id: &'a str,
        user_id: &'a str,
    },
}
```

**Implementation** (`DefaultAiApiClientFactory::for_liberty`):
```rust
fn for_liberty(&self, source: LibertySource<'_>) -> Result<Box<dyn AiApiClient>> {
    match source {
        LibertySource::Envelope(envelope) => match envelope.provider.as_str() {
            "anthropic" => Ok(Box::new(LibertyAnthropicClient::from_envelope(envelope, self.client.clone()))),
            "openai-codex" => Ok(Box::new(LibertyCodexClient::from_envelope(envelope, self.client.clone()))),
            other => Err(AiApiClientFactoryError::LibertyProviderUnsupported(other.to_string())),
        },
        LibertySource::Resolved { creds, alias_id, prefix, tenant_id, user_id } => match creds.provider.as_str() {
            "anthropic" => Ok(Box::new(LibertyAnthropicClient::from_credentials(
                creds, alias_id, prefix, tenant_id, user_id, self.refresh.clone(), self.client.clone(),
            ))),
            "openai-codex" => Ok(Box::new(LibertyCodexClient::from_credentials(
                creds, alias_id, prefix, tenant_id, user_id, self.refresh.clone(), self.client.clone(),
            ))),
            other => Err(AiApiClientFactoryError::LibertyProviderUnsupported(other.to_string())),
        },
    }
}
```

The two arms have identical provider-dispatch logic — this could be further factored if desired, but it's clear as-is and matches the existing pattern in the deleted methods.

**`AuthScopedAiApiClientFactory` wrapper** (`crates/services/src/ai_apis/auth_scoped.rs`):
```rust
pub fn for_alias(&self, alias: &Alias, api_key: Option<String>) -> Result<Box<dyn AiApiClient>> {
    self.inner.for_alias(alias, api_key)
}

/// Validation-only Liberty client from a freshly-pasted envelope.
pub fn for_envelope(&self, envelope: &LlmLibertyEnvelope) -> Result<Box<dyn AiApiClient>> {
    self.inner.for_liberty(LibertySource::Envelope(envelope))
}

/// Request-time Liberty client from resolved (decrypted) credentials.
/// Auto-injects `tenant_id`/`user_id` from `AuthContext`.
pub fn for_resolved(
    &self,
    creds: &ResolvedLlmLibertyCredentials,
    alias: &ApiAlias,
) -> Result<Box<dyn AiApiClient>> {
    let tenant_id = self.auth_context.require_tenant_id()?;
    let user_id = self.auth_context.require_user_id()?;
    self.inner.for_liberty(LibertySource::Resolved {
        creds,
        alias_id: &alias.id,
        prefix: alias.prefix.clone(),
        tenant_id,
        user_id,
    })
}

pub fn safe_http_client(&self) -> SafeReqwest {
    self.inner.safe_http_client()
}
```

The wrapper keeps thin convenience methods (`for_envelope`, `for_resolved`) so route handlers don't have to construct `LibertySource` manually. The rename `for_resolved_credentials` → `for_resolved` makes it parallel to `for_envelope`.

**Service-layer callers** (`api_model_service.rs`, bare factory) construct `LibertySource` directly:
```rust
// Before
factory.for_envelope(&env).map_err(...)?
factory.for_resolved_credentials(&creds, &api_alias, tenant_id, user_id).map_err(...)?

// After
factory.for_liberty(LibertySource::Envelope(&env)).map_err(...)?
factory.for_liberty(LibertySource::Resolved {
    creds: &creds,
    alias_id: &api_alias.id,
    prefix: api_alias.prefix.clone(),
    tenant_id,
    user_id,
}).map_err(...)?
```

### D. I1 documentation comment

In `crates/server_core/src/local_llama_impl.rs::forward_request`, add immediately above `self.on_request_completed();`:

```rust
// Intentional: reset keep-alive on every completion, including errors. A failed
// forward still touched the model (load attempt or in-flight request); the timer
// should debounce from the latest user activity, not the latest success.
self.on_request_completed();
```

## Out of Scope

Documented separately, will land as follow-up PRs:

- **I3** (dead `LocalLlamaError::ModelNotFound`/`ExecNotFound` variants)
- **I4** (`LocalLlamaClient` happy/error-path unit tests — would benefit from being added once the api_path validation is the only smarts left in the client)
- **I5** (factory `for_alias` cluster-vs-standalone matrix)
- **I6** (restore `.withf(...)` argument matchers in routes_app handler tests)
- **I7** (HTTP 400 mapping coverage for `LocalNotSupportedInCluster`)
- **I8** (live-test setups using `new()` instead of `with_db(...)`)
- **N1–N16** (concurrent `start_timer` race; `RwLock::unwrap()`; `LocalLlamaImpl::stop()` `is_loaded()` symmetry; `set_keep_alive` async unused; `live_server_utils.rs` comment; `ApiError`-vs-`BadRequest` for `test_prompt`/`fetch_models`; `mod tests use super::*`; field placement; `with_local_llama` builder helper; ollama format rejection; `responses_create_handler` Native arm dedup; `errmeta` codes; doc sweeps for stale `InferenceService` references in 5 CLAUDE.md/PACKAGE.md files)

## Verification

Per the layered methodology in `CLAUDE.md`:

```bash
# 1. Most upstream: services compiles + tests pass after trait signature changes
cargo check -p services 2>&1 | tail -5
cargo test -p services --lib 2>&1 | grep -E "test result|FAILED|failures:"

# 2. server_core: compile + existing tests
cargo check -p server_core 2>&1 | tail -5
cargo test -p server_core --lib 2>&1 | grep -E "test result|FAILED|failures:"

# 3. routes_app: compile + tests (verifies all 10 LlmEndpoint sites + 4 factory sites)
cargo check -p routes_app 2>&1 | tail -5
cargo test -p routes_app --lib 2>&1 | grep -E "test result|FAILED|failures:"

# 4. Full backend (catches anything missed in lib_bodhiserver, server_app)
make test.backend 2>&1 | tee /tmp/backend.log | grep -E "test result|FAILED|failures:"
# (per feedback_capture_long_commands — tee, don't re-run)
```

No frontend or E2E impact — Rust-only signature/dispatch refactor; the wire protocol on every endpoint is unchanged.

## Implementation Sequence (upstream → downstream)

Per `feedback_layered_refactors`: refactors defer commits to the end. This is one logical refactor (consolidate dispatch + factory surface) and lands as **a single commit** after `make test.backend` passes — this keeps the trait-signature change atomic and reviewable as a single diff.

1. **services trait + factory** —
   - Change `LocalLlama::forward_request` signature (`endpoint` → `api_path`).
   - Simplify `LocalLlamaClient`.
   - Delete `inference/llm_endpoint.rs`; update `inference/mod.rs`.
   - Replace `for_envelope` + `for_resolved_credentials` with `for_liberty(LibertySource)`; update `LibertySource` enum; update `MockAiApiClientFactory` consumers.
   - Update `AuthScopedAiApiClientFactory` wrapper (rename `for_resolved_credentials` → `for_resolved`; both Liberty methods convert into `LibertySource`).
   - Update `api_model_service.rs` 4 call sites.
   - `cargo test -p services --lib` — expect green.

2. **server_core** —
   - Update `SharedContext::forward_request` signature.
   - Rewrite `dispatch_local` to match `&str`.
   - Update `LocalLlamaImpl::forward_request` pass-through.
   - **Add I1 comment** above `on_request_completed()`.
   - Update `test_shared_rw.rs` (5 sites with string literals).
   - Drop `LlmEndpoint` re-export from `lib.rs:18`.
   - `cargo test -p server_core --lib` — expect green.

3. **routes_app** —
   - Update 10 LlmEndpoint construction sites:
     - `routes_oai_chat.rs` × 2 (chat, embeddings)
     - `routes_oai_responses.rs` × 5 + `dispatch_responses_op` signature simplification
     - `routes_anthropic.rs` × 1
     - `routes_gemini.rs` × 4 → 1 `format!`
     - `routes_ollama.rs` × 1
   - Update factory call sites:
     - `routes_anthropic.rs` `for_resolved_credentials` → `for_resolved`
     - `routes_oai_responses.rs` 2 sites: same rename
     - `routes_api_models.rs` 4 sites: keep `for_envelope` / `for_resolved` (wrapper names)
   - Drop `use services::inference::LlmEndpoint;` imports.
   - Clean 3 stale `InferenceService`/`LlmEndpoint` comments.
   - `cargo test -p routes_app` — expect green.

4. **Full backend** — `make test.backend 2>&1 | tee /tmp/backend.log`. Investigate any failure; do not skip per `feedback_run_all_gate_checks`.

5. **Single commit** with message: `Remove LlmEndpoint; consolidate AiApiClientFactory Liberty methods; document keep-alive intent`.

Estimated effort: ~90–120 minutes including test runs and any compile-error chasing across the trait boundary changes.
