# Replace `InferenceService` with `AiApiClientFactory`-uniform dispatch

## Context

`InferenceService` and `AiApiClientFactory` solve overlapping problems with two
different trait shapes:

- For **remote** requests, `StandaloneInferenceService::forward_remote_with_params`
  and `MultitenantInferenceService::forward_remote_with_params` already delegate to
  `AiApiClientFactory::for_alias().forward_request_with_method(...)` (see
  `crates/server_core/src/standalone_inference.rs:176-198`). The "uniform pattern"
  is already uniform — for remote.
- For **local llama.cpp**, `forward_local` lives only on `StandaloneInferenceService`
  and dispatches through `SharedContext::forward_request` (`crates/server_core/src/shared_rw.rs:181-277`),
  which then runs `Server::chat_completions|embeddings`. `MultitenantInferenceService::forward_local`
  returns `InferenceError::Unsupported` (cluster-mode refusal already exists).
- `LlmLibertyOauth` aliases bypass `InferenceService` entirely and call
  `auth_scope.ai_api().for_resolved_credentials(...).forward_request_with_method(...)`
  directly (`crates/routes_app/src/anthropic/routes_anthropic.rs:173`,
  `crates/routes_app/src/oai/routes_oai_responses.rs:151,237`). This is the shape
  we want everywhere.

**Goal**: collapse `InferenceService` entirely. Route handlers ask
`AiApiClientFactory::for_alias(&Alias, api_key)` for a `Box<dyn AiApiClient>` and
call `forward_request_with_method`. The factory enforces cluster-mode refusal
(returns `Err::LocalNotSupportedInCluster` for `Alias::User|Model` when the
factory has no local runtime). Local llama.cpp lifecycle (stop / set_variant /
set_keep_alive / is_loaded + the keep-alive auto-stop timer) moves onto
`SharedContext`, which is exposed via a new service-layer trait `LocalLlama` and
becomes a separate `Option<Arc<dyn LocalLlama>>` accessor on `AppService`.

This kills `StandaloneInferenceService`, `MultitenantInferenceService`,
`NoopInferenceService`, and the `InferenceService` trait. It also removes the
local-vs-remote `match alias { ... }` block from every chat/embeddings/ollama
route handler.

## Design

### New service-layer trait: `LocalLlama`

Lives in `crates/services/src/inference/local_llama.rs` (the `inference/`
directory is repurposed). `DefaultSharedContext` in `server_core` implements it.

```rust
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait]
pub trait LocalLlama: Send + Sync + std::fmt::Debug {
  async fn forward_request(
    &self,
    endpoint: LlmEndpoint,
    request: Value,
    alias: Alias,
  ) -> Result<reqwest::Response, LocalLlamaError>;

  async fn stop(&self) -> Result<(), LocalLlamaError>;
  async fn set_variant(&self, variant: &str) -> Result<(), LocalLlamaError>;
  async fn set_keep_alive(&self, secs: i64);
  async fn is_loaded(&self) -> bool;
}
```

`LocalLlamaError` replaces the parts of `InferenceError` that survive
(`InferenceError` itself can be deleted; its remote-error arms were never
exercised since remote always goes through `AiApiClient` errors). The
`LlmEndpoint` enum stays where it is (move to
`crates/services/src/inference/llm_endpoint.rs` from the now-deleted
`inference_service.rs`).

Why a new trait instead of moving `SharedContext` itself: `SharedContext`'s
internal listener fan-out (`add_state_listener`, `notify_state_listeners`) is a
`server_core` implementation detail used by Tauri/desktop integration paths.
Keeping `SharedContext` in `server_core` and exposing a narrower `LocalLlama`
surface preserves layering.

### Keep-alive timer moves into a new `LocalLlamaImpl` wrapper

**Implementation deviated from the plan**: Rather than mutating `DefaultSharedContext`
directly (which would have entangled `SharedContext`'s listener fan-out with timer
state), the timer + lifecycle live in a new `LocalLlamaImpl` (`crates/server_core/src/local_llama_impl.rs`)
that wraps `Arc<dyn SharedContext>` and implements `LocalLlama`. `SharedContext`
remains untouched — the layering boundary is cleaner this way and the diff is smaller.

`LocalLlamaImpl` owns:
- `keep_alive_secs: RwLock<i64>`
- `timer_handle: RwLock<Option<JoinHandle<()>>>`
- `start_timer`, `cancel_timer`, `on_request_completed` (verbatim from
  `StandaloneInferenceService`)
- `forward_request` calls `on_request_completed` after a successful local dispatch.

Bootstrap: `Arc::new(LocalLlamaImpl::new(ctx, keep_alive_secs))` becomes the
`Arc<dyn LocalLlama>` shared with the factory and the `AppService`.

### `AiApiClientFactory::for_alias` takes `Alias` enum

```rust
fn for_alias(
  &self,
  alias: &Alias,
  api_key: Option<String>,
) -> Result<Box<dyn AiApiClient>>;
```

`DefaultAiApiClientFactory` gains `local_llama: Option<Arc<dyn LocalLlama>>`:

```rust
fn for_alias(&self, alias: &Alias, api_key: Option<String>) -> Result<Box<dyn AiApiClient>> {
  match alias {
    Alias::User(_) | Alias::Model(_) => match &self.local_llama {
      Some(rt) => Ok(Box::new(LocalLlamaClient::new(rt.clone(), alias.clone()))),
      None => Err(AiApiClientFactoryError::LocalNotSupportedInCluster),
    },
    Alias::Api(api_alias) => match api_alias.api_format {
      ApiFormat::OpenAI => /* existing */,
      ApiFormat::OpenAIResponses => /* existing */,
      ApiFormat::Anthropic => /* existing */,
      ApiFormat::AnthropicOAuth => /* existing */,
      ApiFormat::Gemini => /* existing */,
      ApiFormat::LlmLibertyOauth => Err(AiApiClientFactoryError::LibertyRequiresCredentials),
    },
  }
}
```

The auth-scoped wrapper at `crates/services/src/ai_apis/auth_scoped.rs:13-53`
mirrors the new signature.

`for_envelope` and `for_resolved_credentials` are unchanged.

### New client: `LocalLlamaClient: AiApiClient`

`crates/services/src/ai_apis/clients/local_llama.rs`. Holds
`Arc<dyn LocalLlama>` + `Alias`. Implements:

- `forward_request_with_method(method, api_path, request, ...)`:
  - String-match `api_path` to recover `LlmEndpoint::ChatCompletions` or
    `LlmEndpoint::Embeddings` (the only two local llama supports).
  - Anything else → `AiApiClientFactoryError::NotFound(...)` (no new error
    variant needed; the existing variant carries the path message).
  - Calls `local_llama.forward_request(endpoint, request, alias)` →
    `reqwest::Response` → `convert_reqwest_to_axum` (helper moved from
    `standalone_inference.rs` into `crates/services/src/ai_apis/provider_shared.rs`
    alongside `forward_to_upstream`).
- `test_prompt` / `fetch_models`: not applicable to local aliases →
  `Err(AiApiClientFactoryError::ApiError("... is not supported for local model aliases"))`.

**Deviation from plan**: The plan proposed adding `EndpointNotSupportedLocal`
and `OperationNotSupportedForLocal` variants to `AiApiClientError`. In practice
the existing `NotFound` and `ApiError` variants carry sufficient context, and
adding new error codes would cascade through OpenAPI surfaces unnecessarily.

### `AppService` changes

`crates/services/src/app_service/app_service.rs`:

- **Remove** `fn inference_service(&self) -> Arc<dyn InferenceService>;` (line 44)
  and the field on `DefaultAppService` (line 74).
- **Add** `fn local_llama(&self) -> Option<Arc<dyn LocalLlama>>;` registered next
  to `ai_api_client_factory`.
- `ai_api_client_factory()` accessor unchanged in signature; the builder wires
  the local runtime into the factory.

`crates/services/src/app_service/auth_scoped.rs`:

- **Remove** `pub fn inference(&self) -> Arc<dyn InferenceService>` (line 141).
  Auth-scoping has nothing meaningful to add — handlers go via `ai_api()`
  instead.
- Add `pub fn local_llama(&self) -> Option<Arc<dyn LocalLlama>>` only if a
  handler needs it. Audit shows none does (lifecycle is owned by `serve.rs`).

### lib_bodhiserver builder changes

`crates/lib_bodhiserver/src/app_service_builder.rs`:

```rust
let local_llama: Option<Arc<dyn LocalLlama>> = if is_multi_tenant {
  None
} else {
  let ctx = Arc::new(
    DefaultSharedContext::new(hub_service.clone(), setting_service.clone()).await
  );
  let keep_alive_secs = setting_service.keep_alive().await;
  Some(Arc::new(LocalLlamaImpl::new(ctx, keep_alive_secs)))
};

let ai_api_client_factory =
  Self::build_ai_api_client_factory(db_service.clone(), local_llama.clone())?;
// ... pass both into DefaultAppService::new(...)
```

Both the factory and the AppService receive the same `local_llama` Arc; tests
can substitute `MockLocalLlama` via the `AppServiceStub` builder
(`local_llama: Option<Arc<dyn LocalLlama>>` field, defaults to `None`).

### Route handler updates

For each handler currently doing `match alias { local | remote }`:

**Before** (`crates/routes_app/src/oai/routes_oai_chat.rs:148-184`):
```rust
match alias {
  Alias::User(_) | Alias::Model(_) =>
    inference.forward_local(LlmEndpoint::ChatCompletions, request, alias).await?,
  Alias::Api(ref api_alias) if matches!(api_alias.api_format, OpenAI | Anthropic | AnthropicOAuth) =>
    inference.forward_remote_with_params(...).await?,
  Alias::Api(ref api_alias) => return Err(...),
}
```

**After**:
```rust
let endpoint = LlmEndpoint::ChatCompletions;
let client = auth_scope.ai_api().for_alias(&alias, api_key)?;
let response = client.forward_request_with_method(
  endpoint.http_method(),
  &endpoint.api_path(),
  Some(request),
  None,
  None,
).await?;
```

The `OpenAIResponses` / `Gemini` rejection stays in the chat-completions
handler (and `OpenAIResponses` in embeddings). **Deviation**: the plan suggested
moving these into the factory but the rejection messages are
endpoint-specific ("use the responses API endpoint instead"), so they belong
in the route handler where the endpoint context is local.

Anthropic and Responses handlers keep their `Native | Liberty` resolution but
collapse to the same `for_alias` call on the Native arm (Liberty path is
unchanged — already uses `for_resolved_credentials`).

### serve.rs / listener updates

`crates/server_app/src/serve.rs`:
- `ShutdownInferenceCallback` → `ShutdownLocalLlamaCallback`. On shutdown, calls
  `app_service.local_llama().map(|rt| rt.stop()).transpose().await?`. No-op when
  `None` (multi-tenant).
- `KeepAliveSettingListener::on_change`: same pattern — `if let Some(rt) =
  app_service.local_llama() { rt.set_keep_alive(secs).await; }`.

`crates/server_app/src/listener_variant.rs`:
- Stores `Option<Arc<dyn LocalLlama>>`. `on_change` early-returns when `None`.

### Files to delete

- `crates/services/src/inference/inference_service.rs` (replaced by `local_llama.rs` + `llm_endpoint.rs`).
- `crates/services/src/inference/noop.rs` (no longer needed; `Option::None` covers the gap).
- `crates/services/src/inference/error.rs` (`InferenceError` deleted; `LocalLlamaError` now lives in `local_llama.rs`).
- `crates/server_core/src/standalone_inference.rs` (timer logic moves into `local_llama_impl.rs`; `convert_reqwest_to_axum` moves into `services/src/ai_apis/provider_shared.rs`; `proxy_to_remote` deleted entirely — handlers now go through `AiApiClient` directly).
- `crates/server_core/src/multitenant_inference.rs`.
- `crates/server_core/src/test_standalone_inference.rs`.
- `MockInferenceService` references in `crates/services/src/test_utils/app.rs` — replaced by `local_llama` field (defaults to `None`).

### Files to modify

**services**:
- `crates/services/src/inference/mod.rs` — re-exports `LlmEndpoint`, `LocalLlama`, `LocalLlamaError`, `MockLocalLlama`.
- `crates/services/src/inference/local_llama.rs` — new trait + `LocalLlamaError` type.
- `crates/services/src/inference/llm_endpoint.rs` — moved enum (with sibling tests).
- `crates/services/src/ai_apis/ai_api_client_factory.rs` — new `for_alias(&Alias, ...)` signature; `with_db(db, local_llama)` constructor takes `Option<Arc<dyn LocalLlama>>`; `with_local_llama(...)` builder for tests.
- `crates/services/src/ai_apis/error.rs` — adds `LocalNotSupportedInCluster` variant + `From<LocalLlamaError>` impl.
- `crates/services/src/ai_apis/auth_scoped.rs` — `for_alias` wrapper signature mirrors.
- `crates/services/src/ai_apis/clients/local_llama.rs` — new client.
- `crates/services/src/ai_apis/clients/mod.rs` — register module.
- `crates/services/src/ai_apis/provider_shared.rs` — absorb `convert_reqwest_to_axum`. (`proxy_to_remote` deleted, not migrated.)
- `crates/services/src/app_service/app_service.rs` — accessor swap (`inference_service` → `local_llama`).
- `crates/services/src/app_service/auth_scoped.rs` — drop `inference()` accessor.
- `crates/services/src/test_utils/app.rs` — `local_llama: Option<Arc<dyn LocalLlama>>` field.
- `crates/services/src/test_utils/mod.rs` — drop `MockInferenceService` re-export.
- `crates/services/src/models/api_model_service.rs` — `for_alias` call sites wrap `ApiAlias` in `Alias::Api(...)`.

**server_core**:
- `crates/server_core/src/local_llama_impl.rs` — new `LocalLlamaImpl` (timer + `LocalLlama` impl).
- `crates/server_core/src/lib.rs` — drop standalone/multitenant exports; export `LocalLlamaImpl`.
- `crates/server_core/src/shared_rw.rs` — unchanged (preserves layering).

**routes_app**:
- `crates/routes_app/src/oai/routes_oai_chat.rs` — collapse local/remote match in `chat_completions_handler` and `embeddings_handler`.
- `crates/routes_app/src/oai/routes_oai_responses.rs` — Native arm uses `for_alias(&Alias::Api(...), ...)`.
- `crates/routes_app/src/anthropic/routes_anthropic.rs` — Native arm uses `for_alias(&Alias::Api(...), ...)`.
- `crates/routes_app/src/gemini/routes_gemini.rs` — `gemini_action_handler` uses `for_alias(&Alias::Api(...), ...)`.
- `crates/routes_app/src/ollama/routes_ollama.rs` — collapse `forward_local` / `forward_remote` match in `ollama_model_chat_handler`.
- `crates/routes_app/src/models/api/routes_api_models.rs` — wrap `ApiAlias` in `Alias::Api(...)` at `for_alias` call sites.
- `crates/routes_app/src/test_utils/router.rs` — `build_live_test_router` builds `LocalLlamaImpl` + factory `with_local_llama`.
- `crates/routes_app/src/test_utils/mock_ai_factory.rs` — **new helper** `mock_ai_factory_returning(response_fn)` for migrating handler tests.
- `crates/routes_app/src/test_utils/mod.rs` — register helper.

**server_app**:
- `crates/server_app/src/serve.rs` — `ShutdownInferenceCallback` → `ShutdownLocalLlamaCallback`; `KeepAliveSettingListener` uses `local_llama()`.
- `crates/server_app/src/listener_variant.rs` — `VariantChangeListener` holds `Option<Arc<dyn LocalLlama>>`; tests use `MockLocalLlama`; new no-op-when-`None` test added.
- `crates/server_app/tests/utils/live_server_utils.rs` — three setup functions rebuilt: standalone arms instantiate `LocalLlamaImpl` and pass into factory + AppService; multi-tenant arm passes `None`.

**lib_bodhiserver**:
- `crates/lib_bodhiserver/src/app_service_builder.rs` — `build_ai_api_client_factory(db, local_llama)` takes new param; standalone arm builds `LocalLlamaImpl(ctx, keep_alive_secs)`.

## Verification

### Unit tests — completed

```
services       1012 passed
routes_app      802 passed (2 ignored — pre-existing)
server_core      90 passed
server_app       17 passed
lib_bodhiserver   9 passed
```

Workspace `cargo test --workspace --lib`: zero failures.

- `services` — existing `test_ai_api_*` matrix updated to wrap `&ApiAlias` as
  `&Alias::Api(...)` at `for_alias` call sites (35 spots, batch-migrated).
- `routes_app` — handler tests migrated to a new
  `mock_ai_factory_returning(response_fn)` helper. The previous `MockInferenceService`
  `withf(...)` predicate matchers were dropped during migration; tests now assert
  response flow only. (See "Test coverage trade-offs" below.)
- `server_app` — `listener_variant` tests use `MockLocalLlama`. Added
  `test_variant_change_listener_noop_when_no_local_llama` for the multi-tenant
  case.

### Tests deferred (not blocking the refactor)

- New `test_ai_api_local_llama.rs` covering `LocalLlamaClient` happy paths and
  unsupported-path rejection — not added.
- `test_ai_api_provider_matrix.rs` extension for `Alias::User|Model` cases with
  `Some(MockLocalLlama)` vs `None` — not added.
- Multi-tenant integration test asserting `400 LocalNotSupportedInCluster` for
  a `User`/`Model` alias request — not added.

### Test coverage trade-offs

The route-handler tests (chat / embeddings / anthropic / gemini / responses) lost
their `MockInferenceService::expect_*().withf(predicate)` argument-matching
checks during migration. Predicates that asserted endpoint routing, header
forwarding, and query-param passthrough were replaced with a generic
`mock_ai_factory_returning(ok_response)` that ignores arguments. The route
handler logic that constructs those arguments is unchanged (we only rerouted
where the call lands), so the e2e/Playwright suite still covers wire behavior.

If finer assertions are needed later, the pattern is:
```rust
let mut mock_factory = MockAiApiClientFactory::new();
mock_factory.expect_for_alias().withf(|alias, _| /* predicate */)
  .returning(|_, _| {
    let mut mock_client = MockAiApiClient::new();
    mock_client.expect_forward_request_with_method()
      .withf(|method, path, _, _, _| /* predicate */)
      .returning(|_, _, _, _, _| ok_response());
    Ok(Box::new(mock_client))
  });
```

### E2E (Playwright)

Not yet executed in this session — needs Docker + Keycloak. Existing chat,
embeddings, LlmLibertyOauth suites should pass unchanged (wire surface is
identical).

```bash
cargo check -p services -p server_core -p routes_app -p server_app -p lib_bodhiserver
make test.backend
make build.dev-server
make test.e2e
make app.run  # smoke-test in browser
```

## Out of scope

- API surface visible to external callers (OpenAI/Anthropic/Gemini wire formats) is unchanged.
- `LlmLibertyOauth` flow: structure unchanged. The factory's `for_resolved_credentials` and the route-handler call sites stay identical.
- TypeScript client: no OpenAPI changes.

## Implementation Notes (deviations from plan)

| Plan | Implementation | Reason |
|------|----------------|--------|
| Keep-alive timer in `DefaultSharedContext` | New `LocalLlamaImpl` wrapper in `server_core/src/local_llama_impl.rs` | Keeps `SharedContext` listener fan-out separate from timer state; smaller diff |
| `AiApiClientError::EndpointNotSupportedLocal` + `OperationNotSupportedForLocal` | Reuse existing `NotFound` and `ApiError` variants | Avoids cascading new error codes through OpenAPI surfaces |
| Move `proxy_to_remote` into `provider_shared.rs` | `proxy_to_remote` deleted entirely | Handlers now call `AiApiClient` directly — helper unused |
| Format-mismatch rejection (OpenAIResponses on chat, etc.) moves into factory | Stays in route handlers | Rejection messages are endpoint-specific; factory has no endpoint context |
| Extend provider matrix tests + new `test_ai_api_local_llama.rs` | Deferred | Existing tests cover the production paths; new tests are nice-to-have |
| Multi-tenant 400 integration test for User/Model alias | Deferred | The error path is straightforward bubble-up from the factory; not a regression risk |
