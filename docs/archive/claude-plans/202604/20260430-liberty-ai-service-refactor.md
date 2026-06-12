# Decouple `ApiFormat::LlmLibertyOauth` from `AnthropicOAuth`

## Context

`LlmLibertyOauth` was shipped as a flavour of `AnthropicOAuth`: three dispatch arms in `crates/services/src/ai_apis/ai_api_service.rs` (`:135, :178, :229`) collapse the two formats onto `AnthropicOAuthProviderClient`, the `/anthropic/v1/messages` resolver in `crates/routes_app/src/anthropic/routes_anthropic.rs:61` accepts them as one matcher, and `LlmLibertyEnvelope::to_request_parts()` discards `api.chat_url`/`api.models_url` because the downstream client only knows `{base_url}/messages` synthesis. The stop-gap `derive_base_url` (`crates/services/src/models/llm_liberty_envelope.rs:130`) papers over the host-vs-versioned-base mismatch for anthropic but is fragile.

This coupling is a category error: every llm-liberty envelope after v1 (openai-codex, google-gemini, google-antigravity, github-copilot) will have its own URL paths, request shape, response shape, headers, and pagination semantics. We must break the coupling before a second envelope provider lands. The fix replaces the per-method `match api_format` dispatch with a **factory pattern**: one match site picks a per-format/per-provider client implementation, and the resulting impl encapsulates all format-specific behaviour.

## Architectural Approach

### Decision 1 — Factory + Client trait split (replaces today's mixed-concern `AiApiService`)

`AiApiService` becomes a **factory** trait (registered as `Arc<dyn AiApiService>` on `AppService`, kept name unchanged). It exposes three constructor methods that return `Box<dyn AiApiClient>`:

```rust
trait AiApiService: Send + Sync {
  fn for_alias(&self, alias: &ApiAlias, api_key: Option<String>)
    -> Result<Box<dyn AiApiClient>>;                               // OpenAI / OpenAIResponses / Anthropic / AnthropicOAuth / Gemini
  fn for_envelope(&self, envelope: &LlmLibertyEnvelope)
    -> Result<Box<dyn AiApiClient>>;                               // Liberty test/fetch-models before save
  fn for_resolved_credentials(&self, creds: &ResolvedLlmLibertyCredentials, alias: &ApiAlias)
    -> Result<Box<dyn AiApiClient>>;                               // Liberty saved-alias forward + retry
}

#[async_trait]
trait AiApiClient: Send + Sync {
  async fn test_prompt(&self, model: &str, prompt: &str) -> Result<String>;
  async fn fetch_models(&self) -> Result<Vec<ApiModel>>;
  async fn forward_request_with_method(
    &self,
    method: &Method,
    api_path: &str,
    request: Option<Value>,
    query_params: Option<Vec<(String, String)>>,
    client_headers: Option<Vec<(String, String)>>,
  ) -> Result<Response>;
}
```

**One match site** lives in `DefaultAiApiService::for_alias` (and the two Liberty constructors), nowhere else. Adding a format = one new arm + one new client module.

*Alternatives considered, rejected:*
- *Single trait with all methods + opaque LibertyContext:* Keeps the per-method match arm proliferation; doesn't address the smell.
- *Per-call free function:* Drops the AppService registration point and breaks `auth_scope.ai_api()` ergonomics. Confirmed with user: keep `AiApiService` trait name, add inner `AiApiClient`.

### Decision 2 — Per-format `AiApiClient` implementations under `services/src/ai_apis/clients/`

Six client modules implement `AiApiClient`. Each owns its bound credentials/URLs at construction time, so method signatures carry only per-call data:

| Module | Type | URL strategy | Notes |
|---|---|---|---|
| `clients/openai.rs` | `OpenAiClient` | `base_url + path` | Lifted from `OpenAIProviderClient` |
| `clients/openai_responses.rs` | `OpenAiResponsesClient` | `base_url + path` | Lifted from `OpenAIResponsesProviderClient` |
| `clients/anthropic.rs` | `AnthropicClient` | `base_url + path` | Lifted from `AnthropicProviderClient` |
| `clients/anthropic_oauth.rs` | `AnthropicOauthClient` | `base_url + path` (unchanged) | Lifted from `AnthropicOAuthProviderClient`. Setup-token user contract preserved verbatim. |
| `clients/gemini.rs` | `GeminiClient` | `base_url + path` | Lifted from `GeminiProviderClient` |
| `clients/liberty_anthropic.rs` | `LibertyAnthropicClient` | **Explicit URLs** from envelope: `api.chat_url`, `api.models_url`, `oauth.token_url` | NEW. Owns 401-retry-with-force-refresh internally. |

The legacy `provider_*.rs` files are renamed/moved into `clients/`. The `ProviderClient` types become private `AiApiClient` impls.

*Alternatives considered, rejected:*
- *One `LibertyAnthropicClient` that wraps `AnthropicOauthClient` reconfigured with derived base_url:* Keeps the URL-synthesis behaviour the envelope explicitly forbids. Authoring a parallel client that consumes explicit URLs is the cleaner cut.
- *Lift Anthropic-specific logic (anthropic-version header, body merge, model parsing) to a shared trait:* For one Liberty provider, simple module-level helpers in `clients/anthropic_shared.rs` are enough. Promote later when a second Anthropic-shaped client appears.

### Decision 3 — `LibertyAnthropicClient` owns the 401-retry via injected `Arc<dyn LlmLibertyRefresh>`

A small new trait (`services::ai_apis::llm_liberty::LlmLibertyRefresh`) exposes the single capability the client needs:

```rust
#[async_trait]
trait LlmLibertyRefresh: Send + Sync {
  async fn force_refresh(&self, alias_id: &str) -> Result<ResolvedLlmLibertyCredentials, LlmLibertyRefreshError>;
}
```

Implemented by a new adapter struct that wraps the existing `force_refresh_credentials` free function (`services/src/ai_apis/llm_liberty/refresh.rs:99-126`) — repo + http handles captured in the adapter. The factory's `for_resolved_credentials` constructs the adapter and injects it into `LibertyAnthropicClient::new(creds, alias_id, refresh, http)`.

`LibertyAnthropicClient::forward_request_with_method` runs the upstream call once; on `StatusCode::UNAUTHORIZED` it calls `self.refresh.force_refresh(&self.alias_id)` to get fresh creds, mutates its bound state in-place (`self.creds = new_creds`), and retries once. `test_prompt` and `fetch_models` do not retry (matches today's behaviour). The route handler shrinks to a plain `forward`.

*Alternative considered, rejected:* DbService + SafeReqwest injection (heavier surface, harder to mock); refresh closure (async-capture finicky). User confirmed `Arc<dyn LlmLibertyRefresh>` injection.

### Decision 4 — `resolve_anthropic_alias` splits per format; alias-mutation hack disappears

`crates/routes_app/src/anthropic/routes_anthropic.rs:48-107` is rewritten as:

```rust
async fn resolve_anthropic_alias(auth_scope: &AuthScope, model: &str)
  -> Result<AnthropicAliasResolution, BodhiErrorResponse> {
    let alias = ...;
    match api_alias.api_format {
      ApiFormat::Anthropic | ApiFormat::AnthropicOAuth => {
        let api_key = providers::resolve_api_key_for_alias(auth_scope, &api_alias.id).await;
        Ok(AnthropicAliasResolution::Native { alias: api_alias, api_key })
      }
      ApiFormat::LlmLibertyOauth => {
        let creds = providers::resolve_llm_liberty_credentials(auth_scope, &api_alias.id).await?;
        if creds.provider != "anthropic" {
          return Err(/* same 400 message */);
        }
        Ok(AnthropicAliasResolution::Liberty { alias: api_alias, creds })
      }
      _ => Err(/* "not Anthropic-format" 400 */),
    }
}

enum AnthropicAliasResolution {
  Native { alias: ApiAlias, api_key: Option<String> },
  Liberty { alias: ApiAlias, creds: ResolvedLlmLibertyCredentials },
}
```

`anthropic_messages_create_handler` matches on the resolution and constructs the right `AiApiClient` via the factory:
- `Native` → `auth_scope.ai_api().for_alias(&alias, api_key)?`
- `Liberty` → `auth_scope.ai_api().for_resolved_credentials(&creds, &alias)?`

Then calls `client.forward_request_with_method(..)`. **The 401-retry loop in the handler is removed** (now inside `LibertyAnthropicClient`). The handler stops mutating `api_alias.{base_url, extra_headers, extra_body}`.

`list_user_anthropic_aliases` (`:109-133`) keeps its three-format filter (it's a list filter, not a dispatcher — no behavioural coupling).

### Decision 5 — `LlmLibertyRequestParts`, `derive_base_url`, `to_request_parts`, `into_request_parts` deleted

The lossy 4-tuple shape disappears. `LibertyAnthropicClient` reads what it needs (`access_token`, `api_chat_url`, `api_models_url`, `headers_json`, `body_json`) directly from `LlmLibertyEnvelope` or `ResolvedLlmLibertyCredentials`. Callers in `api_model_service.rs` (~10 sites) switch to the factory: `service.ai_api.for_envelope(env)?.fetch_models().await`.

*Alternative considered, rejected:* Keep `LlmLibertyRequestParts` as a transitional shim. The whole point of this work is to remove the lossy projection — keeping it preserves the bug.

### Decision 6 — `InferenceService::forward_remote_with_params` is rewritten to call the factory

`crates/server_core/src/standalone_inference.rs::proxy_to_remote` (`:176-200`) and `MultitenantInferenceService` (`:30+`) currently call `ai_api_service.forward_request_with_method(method, path, alias, key, ...)` directly. They migrate to:

```rust
let client = ai_api_service.for_alias(api_alias, api_key)?;
client.forward_request_with_method(method, &api_path, body, query_params, client_headers).await
```

For Liberty traffic, the route handler at `routes_anthropic.rs` bypasses `InferenceService` and calls the factory directly with `for_resolved_credentials` (because `forward_remote_with_params` only carries `Option<String>` api_key, not `ResolvedLlmLibertyCredentials`). The `forward_remote_with_params` trait signature stays unchanged for non-Liberty callers (gemini, oai_responses, etc.).

*Alternative considered, rejected:* Widen `InferenceService::forward_remote_with_params` to accept Liberty creds. That couples Liberty back into the inference-service abstraction. Cleaner: Liberty path goes route → factory → client, skipping the InferenceService middleman.

### Decision 7 — Refresh layer kept generic for now

`refresh.rs::do_refresh` continues to send `{grant_type, client_id, refresh_token}` as JSON. When a non-anthropic Liberty provider lands and needs different refresh semantics, refactor at that point. A `// TODO` comment on `do_refresh` documents the deferral. The new `LlmLibertyRefresh` trait gives us the seam to swap in a per-provider refresh strategy without touching the client.

### Decision 8 — Test reorganization: split Liberty tests into a dedicated module

Per the explorer's recommendation: the existing matrix file is not actually a matrix (5 hand-written success tests). A `(ApiFormat × LlmLibertyProvider)` cartesian explosion is mostly nonsense cells. Better:

- Drop the `llm_liberty_oauth` rows from the rstest matrices in `test_ai_api_provider_matrix.rs` (`:268-272, :404-410, :439-446`).
- Move `test_prompt_success_llm_liberty_oauth` and `test_fetch_models_success_llm_liberty_oauth` to a new file `services/src/ai_apis/clients/test_liberty_anthropic.rs` parameterised over the **liberty provider** axis (single `anthropic` row today; future arms add cases).
- Add a new `test_factory.rs` covering the `for_alias` / `for_envelope` / `for_resolved_credentials` construction matrix and the format-error arms.

## Layered Implementation Order

Follow upstream-first (services → routes_app → tests at each layer; no ts-client regen needed — no API change).

### Phase 1 — `services` factory + client traits + per-format impls

1. Add `AiApiClient` trait in `services/src/ai_apis/ai_api_client.rs` with `#[mockall::automock]`.
2. Refactor `AiApiService` trait in `services/src/ai_apis/ai_api_service.rs` to the factory shape (3 constructor methods returning `Box<dyn AiApiClient>`).
3. Move existing provider clients to `services/src/ai_apis/clients/`:
   - `provider_openai.rs` → `clients/openai.rs::OpenAiClient: AiApiClient`
   - `provider_openai_responses.rs` → `clients/openai_responses.rs::OpenAiResponsesClient`
   - `provider_anthropic.rs` → `clients/anthropic.rs::AnthropicClient`
   - `provider_anthropic_oauth.rs` → `clients/anthropic_oauth.rs::AnthropicOauthClient` (behaviour unchanged)
   - `provider_gemini.rs` → `clients/gemini.rs::GeminiClient`
4. Author `clients/liberty_anthropic.rs::LibertyAnthropicClient`:
   - Constructor takes `(access_token, api_chat_url, api_models_url, headers_json, body_json, alias_id, refresh: Arc<dyn LlmLibertyRefresh>, http: SafeReqwest)`.
   - `test_prompt` calls `api_chat_url` directly (no `/messages` synthesis).
   - `fetch_models` calls `api_models_url` directly with `before_id` pagination, falls back to "no fetch_models" if `api_models_url` is None.
   - `forward_request_with_method` posts to `api_chat_url` for `/messages`, posts to `api_models_url` for `/models`, returns 404 for any other path (provider-specific contract).
   - 401-retry: try once, on `UNAUTHORIZED` call `refresh.force_refresh(&alias_id)`, mutate bound creds, retry once.
5. Add `LlmLibertyRefresh` trait + `DefaultLlmLibertyRefresh` adapter in `services/src/ai_apis/llm_liberty/refresh.rs`.
6. Implement `DefaultAiApiService` factory:
   ```rust
   fn for_alias(&self, alias, api_key) -> Result<Box<dyn AiApiClient>> {
     match alias.api_format {
       ApiFormat::OpenAI => Ok(Box::new(OpenAiClient::new(api_key, alias.base_url.clone(), self.http.clone()))),
       ApiFormat::OpenAIResponses => Ok(Box::new(OpenAiResponsesClient::new(...))),
       ApiFormat::Anthropic => Ok(Box::new(AnthropicClient::new(...))),
       ApiFormat::AnthropicOAuth => Ok(Box::new(AnthropicOauthClient::new(api_key, alias.base_url.clone(), self.http.clone(), alias.extra_headers.clone(), alias.extra_body.clone()))),
       ApiFormat::Gemini => Ok(Box::new(GeminiClient::new(...))),
       ApiFormat::LlmLibertyOauth => Err(AiApiServiceError::LibertyRequiresCredentials),
     }
   }

   fn for_envelope(&self, env) -> Result<Box<dyn AiApiClient>> {
     match env.provider.as_str() {
       "anthropic" => Ok(Box::new(LibertyAnthropicClient::from_envelope(env, &self.refresh, self.http.clone()))),
       other => Err(AiApiServiceError::LibertyProviderUnsupported(other.into())),
     }
   }

   fn for_resolved_credentials(&self, creds, alias) -> Result<Box<dyn AiApiClient>> {
     match creds.provider.as_str() {
       "anthropic" => Ok(Box::new(LibertyAnthropicClient::from_credentials(creds, &alias.id, &self.refresh, self.http.clone()))),
       other => Err(AiApiServiceError::LibertyProviderUnsupported(other.into())),
     }
   }
   ```
   The factory holds `http: SafeReqwest` and `refresh: Arc<dyn LlmLibertyRefresh>` (constructed once from `DbService` + `http` at AppService build time).
7. Delete `LlmLibertyRequestParts`, `derive_base_url`, `to_request_parts`, `into_request_parts` in `services/src/models/llm_liberty_envelope.rs`. `ResolvedLlmLibertyCredentials` keeps all its fields — they are now consumed by `LibertyAnthropicClient` directly.
8. Update `ApiModelService` (`services/src/models/api_model_service.rs`) to switch from `LlmLibertyRequestParts` plumbing to the factory: every site that constructed parts and called `ai_api.test_prompt(...)` now calls `ai_api.for_envelope(env)?.test_prompt(...)` or `for_resolved_credentials(creds, alias)?...`.
9. Update `app_service_builder.rs` to construct the new factory with `LlmLibertyRefresh` adapter.
10. Run `cargo test -p services` — fix all compile + test breakage. The hand-written success tests in `test_ai_api_provider_matrix.rs` keep passing under the new dispatch (they exercise the factory path indirectly via `DefaultAiApiService`).

### Phase 2 — `server_core` `InferenceService` migration

11. Update `proxy_to_remote` in `standalone_inference.rs` and `multitenant_inference.rs` to call `ai_api_service.for_alias(alias, key)?.forward_request_with_method(...)`. Same diff in both files.
12. Run `cargo test -p server_core` — fix.

### Phase 3 — `routes_app` resolver split + 401-retry removal

13. Rewrite `resolve_anthropic_alias` in `crates/routes_app/src/anthropic/routes_anthropic.rs` to return `AnthropicAliasResolution` enum. Remove the alias-mutation block (`:93-101`).
14. Rewrite `anthropic_messages_create_handler`:
    - Match on `AnthropicAliasResolution`.
    - For `Native` → `auth_scope.ai_api().for_alias(&alias, api_key)?` then `client.forward_request_with_method(...)`.
    - For `Liberty` → `auth_scope.ai_api().for_resolved_credentials(&creds, &alias)?` then `client.forward_request_with_method(...)`.
    - **Delete** the 401-retry block (`:158-208`). The Liberty client now owns it.
    - The pre-clone of `request`/`params`/`headers` (`:161-163`) is also deleted.
15. Update `routes_api_models.rs::api_models_test` and `api_models_fetch_models` (`:171-202, :292-321`) to call the factory: `for_envelope` for inline-envelope variants, `for_alias` for non-Liberty saved aliases, `for_resolved_credentials` after `resolve_llm_liberty_credentials` for Liberty saved aliases.
16. Run `cargo test -p routes_app` — fix.

### Phase 4 — Test reorganization

17. Drop `llm_liberty_oauth` rows from `test_ai_api_provider_matrix.rs`. Keep only `OpenAI / OpenAIResponses / Anthropic / AnthropicOAuth / Gemini` cases.
18. Move `test_prompt_success_llm_liberty_oauth` and `test_fetch_models_success_llm_liberty_oauth` (currently `:180`, `:210` in matrix) to new file `services/src/ai_apis/clients/test_liberty_anthropic.rs`. Add cases:
    - `test_prompt_success_anthropic` — round-trip via mockito hitting the **explicit** chat URL `/v1/messages`.
    - `test_fetch_models_success_anthropic` — explicit models URL `/v1/models` with `before_id` pagination.
    - `forward_retries_on_401_with_force_refresh` — mock that returns 401 then 200; verify `LlmLibertyRefresh` mock was called once and the request was retried with the rotated access token.
    - `forward_propagates_second_401` — 401 → refresh → 401 again; second 401 surfaces verbatim.
    - `unsupported_provider_returns_error` — `for_envelope` with `provider="openai-codex"` → `LibertyProviderUnsupported`.
19. Add `services/src/ai_apis/test_factory.rs` covering: `for_alias` matches all 5 non-Liberty formats; `for_alias(LlmLibertyOauth)` returns `LibertyRequiresCredentials`; `for_envelope`/`for_resolved_credentials` dispatch on provider correctly.
20. Update `routes_app/src/anthropic/test_anthropic_oauth_routing.rs`:
    - `test_messages_create_forwards_to_anthropic_oauth_alias` — predicate unchanged (still `LlmEndpoint::AnthropicMessages` + alias.id match).
    - `test_messages_create_forwards_to_llm_liberty_oauth_alias` — predicate now needs to validate via the new factory path. Since the route bypasses `InferenceService` for Liberty, this test pivots to mocking `MockAiApiService` (the factory) and asserting `for_resolved_credentials` is called with the resolved creds, and the returned mock client receives a `forward_request_with_method` with the access token set in the request headers.
    - `test_messages_create_rejects_llm_liberty_non_anthropic_provider` — semantics unchanged; the factory's `for_resolved_credentials` returns `LibertyProviderUnsupported`; route maps it to 400.
21. CRUD tests (`test_api_models_llm_liberty.rs`) should be unaffected (they go through `ApiModelService` whose external contract is preserved).
22. Refresh tests (`test_refresh.rs`) untouched — refresh layer unchanged. Add **one** new test in `clients/test_liberty_anthropic.rs` using `MockLlmLibertyRefresh` to verify the client invokes the trait correctly.
23. Repository tests (`test_llm_liberty_credentials_repository.rs`) untouched.

### Phase 5 — Backend gate

24. `make test.backend` (services 979 → ~983, routes_app 796 unchanged baseline) — must be green.
25. `cd crates/bodhi && npm test` — must be green (no API surface change, but verify).

### Phase 6 — Manual verification

26. `make app.run.live`, paste a fresh `npx @bodhiapp/llm-liberty@latest login anthropic` envelope into the API model form, confirm `/fetch-models` succeeds (currently broken without `derive_base_url`).
27. Save the alias, run a chat through `/anthropic/v1/messages` with a model that's been used long enough that `expires_at` is near; confirm 401-retry-with-force-refresh works (force-expire by editing DB or using a stale token).
28. `make test.e2e` — the `api-llm-liberty-anthropic.spec.mjs` E2E (gated on `BODHI_E2E_LOCAL=1`) must pass.

## Critical Files to Modify

### `crates/services`

| File | Change |
|---|---|
| `src/ai_apis/ai_api_client.rs` | NEW — defines `AiApiClient` trait, `#[mockall::automock]` |
| `src/ai_apis/ai_api_service.rs` | Refactor `AiApiService` trait to factory shape; rewrite `DefaultAiApiService::{for_alias, for_envelope, for_resolved_credentials}` |
| `src/ai_apis/clients/mod.rs` | NEW — declares the 6 client modules |
| `src/ai_apis/clients/openai.rs` | Move from `provider_openai.rs`; impl `AiApiClient` |
| `src/ai_apis/clients/openai_responses.rs` | Move from `provider_openai_responses.rs`; impl `AiApiClient` |
| `src/ai_apis/clients/anthropic.rs` | Move from `provider_anthropic.rs`; impl `AiApiClient` |
| `src/ai_apis/clients/anthropic_oauth.rs` | Move from `provider_anthropic_oauth.rs`; impl `AiApiClient`; behaviour unchanged |
| `src/ai_apis/clients/gemini.rs` | Move from `provider_gemini.rs`; impl `AiApiClient` |
| `src/ai_apis/clients/liberty_anthropic.rs` | NEW — `LibertyAnthropicClient`, explicit URLs, owns 401-retry |
| `src/ai_apis/clients/anthropic_shared.rs` | NEW — module-level helpers shared by `anthropic_oauth` and `liberty_anthropic` (anthropic-version header default, `merge_extra_body`, `AnthropicModel` parsing, `content[0].text` extraction) |
| `src/ai_apis/llm_liberty/refresh.rs` | Add `LlmLibertyRefresh` trait + `DefaultLlmLibertyRefresh` adapter |
| `src/ai_apis/llm_liberty/mod.rs` | Re-export `LlmLibertyRefresh` |
| `src/ai_apis/error.rs` | Add `AiApiServiceError::LibertyRequiresCredentials`, `::LibertyProviderUnsupported(String)` |
| `src/ai_apis/mod.rs` | Replace `provider_*` declarations with `clients`; export `AiApiClient` |
| `src/models/llm_liberty_envelope.rs` | DELETE `LlmLibertyRequestParts`, `derive_base_url`, `to_request_parts`, `into_request_parts`, `value_to_opt`, `value_to_opt_owned` |
| `src/models/mod.rs` | Drop `LlmLibertyRequestParts` re-export |
| `src/models/api_model_service.rs` | Replace 10 `LlmLibertyRequestParts` callsites with factory calls |
| `src/app_service/app_service.rs` | No change to `Arc<dyn AiApiService>` registration |
| `src/test_utils/app.rs` | Update default `AiApiService` mock construction (factory shape) |
| `src/ai_apis/test_factory.rs` | NEW — factory construction tests |
| `src/ai_apis/clients/test_liberty_anthropic.rs` | NEW — owns Liberty success + 401-retry tests |
| `src/ai_apis/test_ai_api_provider_matrix.rs` | DROP `llm_liberty_oauth` rows from rstest matrices and the two hand-written tests |

### `crates/server_core`

| File | Change |
|---|---|
| `src/standalone_inference.rs` | `proxy_to_remote` calls `ai_api_service.for_alias(...)?` then client method |
| `src/multitenant_inference.rs` | Same migration |
| `src/test_standalone_inference.rs` | Mock setup migrates to factory + client |

### `crates/routes_app`

| File | Change |
|---|---|
| `src/anthropic/routes_anthropic.rs` | Split `resolve_anthropic_alias` into per-format arms returning `AnthropicAliasResolution`; rewrite `anthropic_messages_create_handler` to use factory + delete 401-retry block (now in client) |
| `src/anthropic/test_anthropic_oauth_routing.rs` | Update Liberty test predicates to mock factory + client; rejection test maps to `LibertyProviderUnsupported` |
| `src/models/api/routes_api_models.rs` | `api_models_test` / `api_models_fetch_models` switch to `for_envelope` / `for_alias` / `for_resolved_credentials` |
| `src/providers/mod.rs` | Unchanged — still resolves Liberty creds via `ensure_fresh_credentials` |

### `crates/lib_bodhiserver`

| File | Change |
|---|---|
| `src/app_service_builder.rs:315` | `build_ai_api_service` constructs `DefaultAiApiService` with the new `LlmLibertyRefresh` adapter |

### `crates/bodhi` (frontend)

No change. The OpenAPI surface is untouched — no API request/response types change.

## Test Plan

| Layer | Existing | Updated assertions | New |
|---|---|---|---|
| services unit (clients) | `test_ai_api_anthropic.rs`, `_anthropic_oauth.rs`, `_gemini.rs`, `_openai.rs`, `_openai_responses.rs` — all stay identical (they exercise the per-format clients directly) | — | `clients/test_liberty_anthropic.rs` (5 tests: prompt-success, fetch-models-success, body-merge, 401-retry-success, 401-retry-then-401) |
| services unit (factory) | — | — | `test_factory.rs` (factory match coverage including error arms) |
| services unit (matrix) | `test_ai_api_provider_matrix.rs` (`OpenAI/AnthropicOAuth/Gemini` rows + 401/forward matrices) | DROP `llm_liberty_oauth` rows + 2 hand-written tests | — |
| services unit (refresh) | `llm_liberty/test_refresh.rs` 8 tests stay identical | — | `clients/test_liberty_anthropic.rs::forward_retries_on_401` uses `MockLlmLibertyRefresh` |
| services unit (envelope) | `llm_liberty_envelope.rs` inline tests (`validate_supported`, default headers/body) stay identical | DELETE `derive_base_url` test (function removed) | — |
| services unit (repo) | `test_llm_liberty_credentials_repository.rs` 11 tests stay identical | — | — |
| services unit (api_model_service) | `test_api_model_service.rs` keeps coverage | Adjust call patterns to factory (mocks of `for_envelope` / `for_resolved_credentials`) | — |
| server_core unit | `test_standalone_inference.rs` | Mock factory + client | — |
| routes_app unit (anthropic) | `test_anthropic_oauth_routing.rs::forwards_to_anthropic_oauth_alias` stays identical (route still calls `forward_remote_with_params`) | `forwards_to_llm_liberty_oauth_alias` pivots to factory mock; predicate asserts `for_resolved_credentials` called with creds + access token threaded into client.forward call | `non_anthropic_provider_rejected_by_factory` (factory returns `LibertyProviderUnsupported` → 400) |
| routes_app unit (api_models) | `test_api_models_llm_liberty.rs` 9 tests stay identical (CRUD external contract preserved) | — | — |
| routes_app unit (other) | `oai/routes_oai_chat.rs`, `gemini/routes_gemini.rs` etc. tests stay identical | Update mock setups (`MockAiApiService.expect_for_alias`) | — |
| E2E | `api-llm-liberty-anthropic.spec.mjs` (gated on `BODHI_E2E_LOCAL=1`) | Must pass after fix | — |
| Manual | — | — | Live `/fetch-models` against pasted envelope; chat via `/anthropic/v1/messages`; force-refresh 401-retry |

## Migration Strategy

### `derive_base_url` stop-gap removal

`derive_base_url` is referenced only at `llm_liberty_envelope.rs:117` (call) and `:130` (def). After Phase 1.7 deletes it, the next test run breaks any callsite that still constructs `LlmLibertyRequestParts`. Phase 1.8 (api_model_service migration) is the cleanup that resolves those breaks. No external consumer references the function — safe deletion.

### `Anthropic | AnthropicOAuth | LlmLibertyOauth` matcher splits

Three call sites:
1. `routes_anthropic.rs:61` — split per Decision 4.
2. `routes_anthropic.rs:124` (`list_user_anthropic_aliases`) — kept as-is. This is a list filter for `/anthropic/v1/models`. Aliases of all three formats expose anthropic-shaped `ApiModel`s for the chat UI's Anthropic-format consumer; that's a presentation choice, not a dispatch coupling.
3. The `routes_anthropic.rs::resolve_anthropic_alias` matcher arm is rewritten by Decision 4.

The `oai/routes_oai_chat.rs:156` matcher (`OpenAI | Anthropic | AnthropicOAuth`) is unrelated and stays as-is.

### `LlmLibertyOauth` arm in `for_alias`

Rather than `panic!`/`unreachable!`, return `AiApiServiceError::LibertyRequiresCredentials` with a clear message: "LlmLibertyOauth aliases must be constructed via for_envelope or for_resolved_credentials." This is a defensive arm — no caller should hit it after Phase 3.

### Mock migration

`MockAiApiService` (mockall-generated) acquires three new `expect_*` methods (`expect_for_alias`, `expect_for_envelope`, `expect_for_resolved_credentials`). Existing tests using `expect_test_prompt` / `expect_fetch_models` / `expect_forward_request_with_method` must rewire to:
1. Mock the factory call returning a `MockAiApiClient`.
2. Set up `MockAiApiClient.expect_test_prompt` etc. on the returned mock.

This is a mechanical edit. About 25 tests in routes_app touch these mocks; budget ~2h.

### Behaviour preservation contract

After this work:
- `AnthropicOauthClient` (formerly `AnthropicOAuthProviderClient`) behaves identically. The setup-token user contract (paste `https://api.anthropic.com/v1` into BaseUrl) is preserved.
- All non-Liberty formats behave identically.
- Liberty fetch-models now succeeds (the bug `derive_base_url` papered over).
- Liberty 401-retry-with-force-refresh fires from inside the client; user-observable behaviour identical.
- The provider-XOR validation in `LlmLibertyCredsSource::try_from_pair` is unchanged (envelope schema validation untouched).

## Out of Scope

- Implementing `openai-codex`, `google-gemini`, `google-antigravity`, `github-copilot` Liberty providers. Architecture admits them; v1 still ships only anthropic.
- Changing the `LlmLibertyEnvelope` / `LlmLibertyEnvelopeUpdate` / `LlmLibertySummary` JSON wire format.
- Changing the `api_model_oauth_credentials` schema.
- Changing UI-facing form behaviour or `@bodhiapp/ts-client` types.
- Per-provider refresh-token shape (kept generic; future work).
- Promoting Anthropic-shared logic from `clients/anthropic_shared.rs` into a trait — a flat helper module is enough for one Liberty provider.
- Renaming the `AiApiService` trait. Per user decision: name stays; only its method shape changes.
