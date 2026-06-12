# LLM Liberty OAuth — `provider = "openai-codex"`

## Context

Three commits (`f8c94cfb` → `f111b2a0`) shipped the LLM Liberty OAuth provider system with `provider = "anthropic"` as the only supported provider. The architecture was deliberately built to admit additional providers without churn:

- `AiApiClientFactory::for_envelope` / `for_resolved_credentials` dispatch on `creds.provider.as_str()`
- Each Liberty provider owns its own `clients/liberty_<provider>.rs` impl of `AiApiClient`
- `LlmLibertyRefresh` trait abstracts the refresh loop; `do_refresh` uses RFC 6749 generic shape
- The factory's `Err(LibertyProviderUnsupported(other))` arm is the seam for new providers

This plan adds `provider = "openai-codex"`. Per `data/local/TOKEN_REFRESH.md` the refresh wire-shape is the **same RFC 6749 grant** Anthropic uses (`{grant_type, client_id, refresh_token}` JSON, no client_secret) — so `do_refresh` is reused unchanged.

### Curl-verified upstream contract (2026-04-30, against the live token in `data/local/codex.json`)

**`POST https://chatgpt.com/backend-api/codex/responses`** — request and SSE response are **fully OpenAI-Responses-shaped**:
- Request body: `{model, input, stream:true, store, instructions}` — identical to OpenAI `/v1/responses`
- Response: SSE events `response.created` / `response.in_progress` / `response.output_item.added` / `response.output_text.delta` / `response.output_text.done` — identical to OpenAI Responses streaming
- Required headers (envelope-supplied): `Authorization: Bearer`, `ChatGPT-Account-ID`, `originator`, `User-Agent`, `OpenAI-Beta`
- Required body fields (envelope-supplied): `instructions`, `store`, `stream` (Codex 400s without `stream:true`)

**Routing decision (Q1 confirmed by curl):** Wire-compat is total. Reuse the existing `/v1/responses` route by making it format-aware (mirror `/anthropic/v1/messages`). No new route group; OpenAPI types stay stable.

**Body-merge decision (Q2 confirmed):** Mirror Anthropic — merge envelope `body` only when `api_path == "/responses"`. Skip on `/models*`.

**`GET https://chatgpt.com/backend-api/codex/models?client_version=0.0.1`** returns a Codex-specific shape, **NOT** OpenAI `{data: [...]}`:
```json
{ "models": [
    { "slug": "gpt-5.2", "display_name": "gpt-5.2", "context_window": 272000,
      "supports_parallel_tool_calls": true, "input_modalities": ["text","image"], ... },
    ...
] }
```
**Models parser decision (Q3 resolved by curl):** Custom Codex parser. Read `models[].slug` as model id; synthesize `ApiModel::OpenAI { id: slug }` rows (no Anthropic-style `AnthropicModel` here). The endpoint requires `client_version` query param — the client appends it.

**Client structure (Q4 confirmed):** Parallel `LibertyCodexClient` next to `LibertyAnthropicClient`. Two new factory arms.

---

## Decisions Summary

| # | Decision |
|---|----------|
| 1 | Extend `/v1/responses` route with format-aware resolution (`ResponsesAliasResolution` enum) — mirror `routes_anthropic.rs::AnthropicAliasResolution`. Only `responses_create_handler` gains a Liberty arm; the other 4 handlers (retrieve / list / delete / cancel) stay native-only. |
| 2 | New `clients/liberty_codex.rs::LibertyCodexClient` mirrors `LibertyAnthropicClient` structurally. Owns 401-retry-with-force-refresh via injected `Arc<dyn LlmLibertyRefresh>`. |
| 3 | `validate_supported` whitelist becomes `provider in {"anthropic","openai-codex"}`. |
| 4 | Factory dispatches `"openai-codex"` to `LibertyCodexClient` in both `for_envelope` and `for_resolved_credentials`. |
| 5 | Refresh layer (`do_refresh`) unchanged — RFC 6749 shape works for both providers. |
| 6 | Frontend (`agentStore.ts`) routes `llm_liberty_oauth + openai-codex` → `api: "openai-responses"`, `provider: "openai"`, `baseUrl: ${origin}/v1`. |
| 7 | `LlmLibertyEnvelopeInput` Zod schema accepts the codex provider literal. |
| 8 | New E2E spec `api-llm-liberty-codex.spec.mjs` (gated on `BODHI_E2E_LOCAL=1`) mirrors `api-llm-liberty-anthropic.spec.mjs`. |

---

## File-by-file Plan

### Phase 1 — `services` crate (factory + envelope + new client)

#### 1.1 `src/models/llm_liberty_envelope.rs`
- **Line 75:** Replace `if self.provider != "anthropic"` with `if !matches!(self.provider.as_str(), "anthropic" | "openai-codex")`.
- Update the doc comment at line 46 (`Only "anthropic" is supported in v1.`) to list both providers.
- Update the rejection message: `"Unsupported provider '{}'. Only 'anthropic' and 'openai-codex' are supported in this version."`
- Update the inline tests at line 215+ to add a `validate_supported_accepts_codex_envelope` `#[case]`.

#### 1.2 `src/ai_apis/clients/liberty_codex.rs` — NEW
Mirror `clients/liberty_anthropic.rs` structurally. Differences:

- **Bearer auth + extra headers**: same pattern as `apply_bearer_auth_and_version` minus the anthropic-version default. Inline a `apply_bearer_and_extra_headers` helper local to this file (or factor to a new `clients/liberty_shared.rs` if a third provider is anticipated — for now, inline).
- **`test_prompt(model, prompt)`** — POST `chat_url` with body shaped per OpenAI Responses + envelope merge:
  ```rust
  let base_body = json!({
    "model": model,
    "input": [{"role":"user","content":prompt}],
    "max_output_tokens": 50
  });
  let request_body = match &self.extra_body {
    Some(extra) => merge_extra_body(base_body, extra),  // envelope adds instructions/store/stream
    None => base_body,
  };
  ```
  Parse SSE response for the assistant text. Since `stream:true` is forced by envelope, `test_prompt` must collect SSE events and extract the final `response.output_text.done.text`. Add a private helper `extract_codex_completion_text(sse_body: &str) -> String` that scans for `response.output_text.done` events.
- **`fetch_models()`** — GET `models_url` (must append `?client_version=0.0.1` query param — make it a const `CODEX_CLIENT_VERSION = "0.0.1"`). Parse `body["models"]` as an array; map each `models[i].slug` (string) → `ApiModel::OpenAI { id: slug.to_string() }`. No pagination (single response).
- **`forward_request_with_method`** — same 401-retry-with-force-refresh shape as Anthropic. `resolve_url`:
  ```rust
  fn resolve_url(api_path: &str, chat_url: &str, models_url: Option<&str>) -> Result<String> {
    if api_path == "/responses" || api_path.starts_with("/responses?") {
      return Ok(chat_url.to_string());
    }
    if api_path.starts_with("/models") {
      if let Some(mu) = models_url { return Ok(mu.to_string()); }
      return Err(AiApiClientFactoryError::NotFound(api_path.to_string()));
    }
    Err(AiApiClientFactoryError::NotFound(api_path.to_string()))
  }
  ```
- **Body merge in `forward_once`**: `let is_responses = api_path == "/responses";` mirroring Anthropic's `is_messages`. Merge `extra_body` only on `is_responses`.
- **Constructors**: `from_envelope(envelope, client)` and `from_credentials(creds, alias_id, prefix, tenant_id, user_id, refresh, client)` — identical signature to `LibertyAnthropicClient`.

#### 1.3 `src/ai_apis/clients/mod.rs`
Add `pub(crate) mod liberty_codex;` next to `pub(crate) mod liberty_anthropic;`.

#### 1.4 `src/ai_apis/ai_api_client_factory.rs`
- Import `LibertyCodexClient`.
- Extend `for_envelope` (line 120-130):
  ```rust
  match envelope.provider.as_str() {
    "anthropic" => Ok(Box::new(LibertyAnthropicClient::from_envelope(envelope, self.client.clone()))),
    "openai-codex" => Ok(Box::new(LibertyCodexClient::from_envelope(envelope, self.client.clone()))),
    other => Err(AiApiClientFactoryError::LibertyProviderUnsupported(other.to_string())),
  }
  ```
- Extend `for_resolved_credentials` (line 132-153) symmetrically.

#### 1.5 `src/ai_apis/clients/test_liberty_codex.rs` — NEW
Mirror `test_liberty_anthropic.rs`. Cases:
- `test_prompt_success_codex` — mockito SSE response with a `response.output_text.done` event; assert extracted text.
- `fetch_models_success_codex` — mockito returns `{"models":[{"slug":"gpt-5.2","display_name":"gpt-5.2"},{"slug":"gpt-5.2-codex","display_name":"gpt-5.2-codex"}]}`; assert two `ApiModel::OpenAI` rows with those ids. Verify request URL contains `client_version=0.0.1`.
- `fetch_models_empty_models_url_returns_empty` — when envelope has `models_url: None`, returns `Ok(Vec::new())`.
- `forward_post_responses_routes_to_chat_url` — POST `/responses` → mockito asserts target URL = `chat_url`, headers include all envelope headers + `Authorization: Bearer`, body merges envelope `instructions/store/stream`.
- `forward_returns_not_found_for_unknown_path` — `/foo` returns `NotFound("/foo")`.
- `forward_retries_on_401_with_force_refresh` — `MockLlmLibertyRefresh` returns rotated creds; first 401 triggers refresh; second call succeeds.
- `forward_propagates_second_401` — refresh runs but second 401 surfaces verbatim.

Use a fixture `test_llm_liberty_envelope_codex()` (added in 1.6).

#### 1.6 `src/test_utils/llm_liberty.rs`
Add two builders mirroring the existing anthropic helpers:
- `test_llm_liberty_envelope_codex()` — provider="openai-codex", chat_url="https://api.example.com/codex/responses", models_url="https://api.example.com/codex/models", headers `{ChatGPT-Account-ID, originator, User-Agent, OpenAI-Beta}`, body `{instructions, store:false, stream:true}`.
- `test_resolved_llm_liberty_credentials_codex()` — same shape, `provider: "openai-codex".into()`.

#### 1.7 `src/ai_apis/test_factory.rs`
Add cases:
- `for_envelope_dispatches_codex_to_liberty_codex_client` — assert `Ok(_)` for codex envelope (no static type assertion needed; `Box<dyn AiApiClient>` is opaque, test instead via observable behaviour: call `fetch_models` against a mockito codex shape and assert it parses correctly).
- `for_resolved_credentials_dispatches_codex_provider` — symmetric.
- Existing `for_envelope_returns_error_for_unsupported_provider` keeps using a synthetic `"unknown-provider"` string.

#### 1.8 `cargo test -p services`
Must be green. The factory test file path registration (`ai_api_client_factory.rs:177`) and `test_factory` are already wired — no module declarations to add for new tests beyond `clients/mod.rs` for `test_liberty_codex` (already gated by the `#[cfg(test)] #[path = "test_liberty_codex.rs"] mod test_liberty_codex;` discipline).

Add the test_liberty_codex registration to `clients/liberty_codex.rs`:
```rust
#[cfg(test)]
#[path = "test_liberty_codex.rs"]
mod test_liberty_codex;
```

---

### Phase 2 — `routes_app` (extend `/v1/responses`)

#### 2.1 `src/oai/routes_oai_responses.rs`
Today (line 55-78):
```rust
async fn resolve_responses_alias(...) -> Result<(ApiAlias, Option<String>), OaiApiError> {
  // accepts only ApiFormat::OpenAIResponses
}
```

Refactor to mirror `AnthropicAliasResolution`:
```rust
enum ResponsesAliasResolution {
  Native { alias: ApiAlias, api_key: Option<String> },
  Liberty { alias: ApiAlias, creds: ResolvedLlmLibertyCredentials },
}

async fn resolve_responses_alias(auth_scope: &AuthScope, model: &str)
  -> Result<ResponsesAliasResolution, OaiApiError>
{
  // alias lookup as today
  match api_alias.api_format {
    ApiFormat::OpenAIResponses => {
      let api_key = providers::resolve_api_key_for_alias(auth_scope, &api_alias.id).await;
      Ok(ResponsesAliasResolution::Native { alias: api_alias, api_key })
    }
    ApiFormat::LlmLibertyOauth => {
      let creds = providers::resolve_llm_liberty_credentials(auth_scope, &api_alias.id).await?;
      Ok(ResponsesAliasResolution::Liberty { alias: api_alias, creds })
      // Provider mismatch (openai-codex expected) is detected by factory → 400
    }
    _ => Err(/* "not OpenAI-Responses-format" 400 */),
  }
}
```

Update **only** `responses_create_handler` (line 125+) — match on `ResponsesAliasResolution`:
- `Native` → `auth_scope.ai_api().for_alias(&alias, api_key)?` then `client.forward_request_with_method(&Method::POST, "/responses", body, query_params, client_headers).await`. (Or keep the existing `forward_remote_with_params` path if it's already compatible; choose whichever produces minimal diff.)
- `Liberty` → `auth_scope.ai_api().for_resolved_credentials(&creds, &alias, &tenant_id, &user_id)?` then `client.forward_request_with_method(...)`. The factory rejects non-codex providers with `LibertyProviderUnsupported` → maps to 400.

The other four handlers (`responses_retrieve_handler`, `_delete_handler`, `_cancel_handler`, `_list_handler` on lines 189, 234, 279, 324) keep calling `resolve_responses_alias` but will now error out on `Liberty` — so they need a small adapter:
```rust
let (api_alias, api_key) = match resolve_responses_alias(...).await? {
  ResponsesAliasResolution::Native { alias, api_key } => (alias, api_key),
  ResponsesAliasResolution::Liberty { .. } => {
    return Err(OaiApiError::not_supported_for_liberty(...));  // or similar 400
  }
};
```
Add a typed error variant `OaiApiError::LibertyResponsesOpUnsupported` for the four CRUD handlers. (Or simpler: keep `resolve_responses_alias` returning the enum; provide a helper `resolution.into_native_or_400()` for the four handlers.)

#### 2.2 `src/oai/test_oai_responses_success.rs` + `test_oai_responses_errors.rs`
- Existing native tests stay green — the `Native` arm preserves behaviour.
- Add a new test `responses_create_forwards_to_llm_liberty_oauth_codex_alias` parallel to `test_messages_create_forwards_to_llm_liberty_oauth_alias` in `anthropic/test_anthropic_oauth_routing.rs`. Mocks `MockAiApiClientFactory::expect_for_resolved_credentials` returning a `MockAiApiClient` whose `forward_request_with_method` is asserted called with `(POST, "/responses", body)`.
- Add `responses_create_rejects_llm_liberty_non_codex_provider` — when the resolved creds carry `provider: "anthropic"`, factory returns `LibertyProviderUnsupported` → 400.
- Add a test verifying `responses_retrieve` returns a 400 (or appropriate error) for Liberty aliases.

#### 2.3 `src/anthropic/routes_anthropic.rs` — minor doc-only update
Update the `AnthropicAliasResolution` doc comment that says "factory rejects non-anthropic providers" — the assertion now reads "factory rejects providers that don't match the route's expected provider (anthropic for /messages, openai-codex for /responses)". No code change.

#### 2.4 `cargo test -p routes_app`
Must be green.

---

### Phase 3 — `lib_bodhiserver` E2E

#### 3.1 `crates/lib_bodhiserver/tests-js/specs/api-models/api-llm-liberty-codex.spec.mjs` — NEW
Mirror `api-llm-liberty-anthropic.spec.mjs`. Key differences:
- Load envelope from `data/local/codex.json`.
- Format selection still `llm_liberty_oauth`.
- Save alias, route chat through `/v1/responses` (because agentStore now maps `llm_liberty_oauth + openai-codex` → `openai-responses` piApi → `${origin}/v1`).
- Send "Reply with exactly: hello bodhi"; assert streamed response contains `hello bodhi`.

Gating: `test.skip(!process.env.BODHI_E2E_LOCAL, ...)` — this is local-only because it consumes a real OpenAI account token.

---

### Phase 4 — Frontend

#### 4.1 `crates/bodhi/src/stores/agentStore.ts`

**Line 29-30** (`apiFormatToPiApi`):
```typescript
case 'llm_liberty_oauth':
  if (provider === 'anthropic') return 'anthropic-messages';
  if (provider === 'openai-codex') return 'openai-responses';
  return 'openai-completions';
```

**Line 39** (`apiFormatToProvider`):
```typescript
if (apiFormat === 'llm_liberty_oauth' && provider === 'anthropic') return 'anthropic';
if (apiFormat === 'llm_liberty_oauth' && provider === 'openai-codex') return 'openai';
```

**Line 83** (`getBaseUrl`):
```typescript
if (apiFormat === 'llm_liberty_oauth' && provider === 'anthropic') return `${origin}/anthropic`;
if (apiFormat === 'llm_liberty_oauth' && provider === 'openai-codex') return `${origin}/v1`;
```

#### 4.2 `crates/bodhi/src/stores/agentStore.test.ts`
Extend the existing parameterized routing test (rows added in commit `f8c94cfb`) with:
```js
['llm_liberty_oauth', 'openai-codex', 'gpt-5.2'],  // expect api: 'openai-responses', provider: 'openai', baseUrl: /\/v1$/
```

#### 4.3 `crates/bodhi/src/components/api-models/form/LlmLibertyEnvelopeInput.tsx` (and Zod schema)
Search for the provider whitelist (Zod literal `"anthropic"` or string equality). Update to accept the union `"anthropic" | "openai-codex"`. Update `LlmLibertyEnvelopeInput.test.tsx` with a codex envelope fixture case.

If the form has a "preview parsed envelope" UI that shows a provider-specific badge, add a codex variant.

#### 4.4 `crates/bodhi/src/routes/chat/-components/settings/AliasSelector.tsx` and tests
No code change needed (provider extraction already pulls `alias.llm_liberty?.provider` generically). Just add a parameterized test row for `provider: 'openai-codex'` to lock in `setLlmLibertyProvider('openai-codex')`.

#### 4.5 `cd crates/bodhi && npm run test`
Must be green.

---

## Reusable building blocks

| Existing | Reused as-is |
|----------|--------------|
| `LlmLibertyRefresh` trait + `DefaultLlmLibertyRefresh` adapter | Refresh shape is RFC 6749 generic; both providers share. |
| `do_refresh` (`refresh.rs:169`) | `{grant_type, client_id, refresh_token}` JSON body works for both per `TOKEN_REFRESH.md`. |
| `provider_shared::merge_extra_body` | Body merge logic is generic. |
| `provider_shared::forward_to_upstream` | URL forwarding is generic. |
| `AiApiClientFactoryError::LibertyProviderUnsupported(String)` | Error variant already typed; routes already map it to 400. |
| `ResolvedLlmLibertyCredentials` struct | All fields needed by Codex are already present (chat_url, models_url, headers_json, body_json, oauth_token_url, oauth_client_id, refresh_token, access_token). |
| `apiFormatToPiApi` / `apiFormatToProvider` / `getBaseUrl` extension points | Already accept `provider?: string` arg threaded through `AliasSelector` → `chatSettingsStore.llmLibertyProvider`. |
| `api-llm-liberty-anthropic.spec.mjs` E2E shape | Direct template for the codex spec. |

## Critical files to modify

```
crates/services/src/models/llm_liberty_envelope.rs           (1 line + tests)
crates/services/src/ai_apis/clients/liberty_codex.rs         NEW
crates/services/src/ai_apis/clients/test_liberty_codex.rs    NEW
crates/services/src/ai_apis/clients/mod.rs                   (1 line)
crates/services/src/ai_apis/ai_api_client_factory.rs         (2 match arms)
crates/services/src/ai_apis/test_factory.rs                  (2 cases)
crates/services/src/test_utils/llm_liberty.rs                (2 builders)

crates/routes_app/src/oai/routes_oai_responses.rs            (resolver enum + handler dispatch)
crates/routes_app/src/oai/test_oai_responses_success.rs      (codex success case)
crates/routes_app/src/oai/test_oai_responses_errors.rs       (codex 400, CRUD-on-Liberty 400)
crates/routes_app/src/oai/api_error.rs                       (new error variant if needed)

crates/bodhi/src/stores/agentStore.ts                        (3 routing branches)
crates/bodhi/src/stores/agentStore.test.ts                   (1 row)
crates/bodhi/src/components/api-models/form/LlmLibertyEnvelopeInput.tsx  (Zod literal)
crates/bodhi/src/components/api-models/form/LlmLibertyEnvelopeInput.test.tsx (codex case)
crates/bodhi/src/routes/chat/-components/settings/AliasSelector.test.tsx (1 row)

crates/lib_bodhiserver/tests-js/specs/api-models/api-llm-liberty-codex.spec.mjs  NEW
```

No OpenAPI surface change — `ApiFormat` enum already includes `LlmLibertyOauth`, `LlmLibertySummary.provider` is `string`, so no `cargo run --package xtask openapi` regeneration needed. (Confirm by running it; should produce zero diff.)

## Verification

### Unit (Rust)
```bash
cargo test -p services --lib 2>&1 | grep -E "test result|FAILED|failures:"
cargo test -p routes_app --lib 2>&1 | grep -E "test result|FAILED|failures:"
```
Expected: codex factory cases pass; codex client tests pass; `/v1/responses` Liberty arm tests pass; non-codex provider on Liberty creds yields 400; CRUD-on-Liberty handlers return 400.

### Frontend
```bash
cd crates/bodhi && npm run test -- agentStore.test AliasSelector.test LlmLibertyEnvelopeInput.test
cd crates/bodhi && npm run test
```

### Backend gate
```bash
make test.backend
```

### E2E (manual, BODHI_E2E_LOCAL=1)
```bash
make build.dev-server
cd crates/lib_bodhiserver && BODHI_E2E_LOCAL=1 npm run test:playwright -- api-llm-liberty-codex
```

### Manual round-trip
1. `ports kill 1135 && make app.run.live`
2. Open Chrome → `/ui/models/new`, choose API Model, format `llm_liberty_oauth`, paste `data/local/codex.json` envelope.
3. Click "Fetch Models" — expect `gpt-5.2`, `gpt-5.2-codex`, etc. populating the model list (the Codex `/models` shape must parse).
4. Click "Test Connection" with `gpt-5.2` — expect a successful response.
5. Save alias.
6. Open `/ui/chat/`, pick the codex alias, send "Reply with exactly: hello bodhi".
7. Network panel: request goes to `${origin}/v1/responses` (not `/v1/chat/completions`, not `/anthropic/v1/messages`); SSE response stream renders the reply.
8. Force-expire test: edit DB row to push `expires_at` into the past, send another message — confirm 401-retry-with-force-refresh fires inside `LibertyCodexClient` (token rotates, request retries once).

### Regression spot-checks
- Native `OpenAIResponses` alias still hits `/v1/responses` and works.
- Native `OpenAI` chat-completions alias unaffected.
- Anthropic Liberty alias still flows through `/anthropic/v1/messages`.
- `responses_retrieve` / `_delete` / `_cancel` / `_list` on a Liberty alias return 400 (not 500).

## Commit cadence (per layered-refactors preference)

1. **services**: envelope whitelist + LibertyCodexClient + factory dispatch + tests.
2. **routes_app**: `ResponsesAliasResolution` enum + handler dispatch + tests.
3. **frontend**: agentStore routing + Zod schema + unit tests.
4. **e2e**: `api-llm-liberty-codex.spec.mjs`.

Run `make test.backend && cd crates/bodhi && npm test` before each commit; full `make test.e2e` once at the end (codex spec is `BODHI_E2E_LOCAL`-gated so it's skipped in CI by default).

## Out of Scope

- Other Liberty providers (`google-gemini`, `google-antigravity`, `github-copilot`). The TOKEN_REFRESH.md research catalogues differences for those (e.g. Copilot uses GET token-exchange not POST refresh; Gemini requires `client_secret`); supporting them needs the `do_refresh` generalization that the prior plan's "Decision 7" deferred.
- OAuth-bearing CRUD on `/v1/responses` (retrieve/delete/cancel/list of stored responses for Liberty aliases). Codex backend may not expose these for OAuth users; punt until a user need surfaces.
- Embeddings / images / files via Codex.
- Refresh-token rotation telemetry, expiry alarm UI.
