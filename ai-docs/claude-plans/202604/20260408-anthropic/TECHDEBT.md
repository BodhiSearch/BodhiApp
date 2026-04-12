# Anthropic Messages API — Technical Debt Register

**Status**: Phase 1 shipped (commits `ef590f80` → `933fb595`), opaque-proxy
rethink applied. Anthropic `/v1/chat/completions` (OpenAI-compat endpoint)
supported via opaque proxy — external API clients use Bearer token against
`/v1/chat/completions`; chat UI routes through `/anthropic/v1/messages` (pi-ai).

**Related plans**:
- Phase 1 plan: `ai-docs/claude-plans/202603/transient-puzzling-hoare.md`
- Consolidated AI-gateway research: `ai-docs/claude-plans/202604/20260407-responses-api/12-anthropic-consolidated-recommendation.md`
- This rethink: `ai-docs/claude-plans/202604/transient-puzzling-hoare.md`

This document tracks items explicitly deferred from Phase 1 of the Anthropic
Messages API support. Each entry has:

- **Severity**: how much it limits real users
- **Root cause**: why we're not fixing it yet
- **Fix path**: the smallest plausible change to address it

---

## 1. 3rd-party Anthropic-compatible providers (Bedrock / Vertex)

**Severity**: Medium — blocks users on AWS/GCP-managed Claude deployments.

Phase 1 supports `api.anthropic.com` direct only via the `ApiAlias` base URL +
stored `x-api-key`. The following are not supported:

- **AWS Bedrock** — Anthropic-on-Bedrock requires SigV4 request signing,
  region-specific endpoints (e.g. `bedrock-runtime.us-east-1.amazonaws.com`),
  and an IAM principal rather than a long-lived API key.
- **Google Vertex AI** — Anthropic-on-Vertex uses Google IAM tokens (refreshed
  via service account credentials) and a publisher/region path layout
  (`/v1/projects/{project}/locations/{region}/publishers/anthropic/models/{model}:rawPredict`).

**Fix path**: Add new `ApiFormat` variants (or per-alias provider sub-fields)
for each provider, plug provider-specific signers/token-fetchers into
`AiApiService::forward_request_with_method`, and stand up either real or
mocked integration tests against each. Bedrock/Vertex pull in significant
SDK surface.

## 2. `/anthropic/v1/models` returns synthetic stubs (no `ModelInfo` metadata)

**Severity**: Low — clients that only need model IDs work fine; clients that
expect display names or capability flags get sparse data.

**Current behavior**: `anthropic_models_list_handler` aggregates IDs from
`alias.models ∪ alias.models_cache` across the user's Anthropic-format
aliases and emits `{id, type: "model"}` stubs. The Anthropic native
`ModelInfo` struct has many more fields:

```
display_name:        String
created_at:          String   // RFC 3339
max_input_tokens:    Option<i64>
max_tokens:          Option<i64>
capabilities:        Option<ModelCapabilities>  // batch, citations,
                                                // code_execution, image_input,
                                                // pdf_input, structured_outputs,
                                                // thinking, effort,
                                                // context_management
```

None of these are stored anywhere in BodhiApp.

**Root cause**: `ApiAlias.models` and `ApiAlias.models_cache` are typed
`JsonVec<String>` in `crates/services/src/models/model_objs.rs:74`. They
hold model IDs only.

**Fix path**:
1. Add a sibling field `models_meta: JsonVec<AnthropicModelInfo>` (or a
   provider-agnostic metadata variant) to `ApiAlias`.
2. During `api_models_create` / `api_models_sync`, when the alias format is
   `anthropic`, hit upstream `GET /v1/models` and persist the full response.
3. Update `anthropic_models_list_handler` to read from `models_meta`.
4. Ship a SeaORM migration to add the column on existing aliases (backfill
   to empty array; lazily populate on next sync).

## 3. Chat UI hardcoded `contextWindow` / `maxTokens` defaults

**Severity**: Low — chat works for Anthropic aliases; only the local
limit-calculation UI shows OpenAI-shaped numbers.

`crates/bodhi/src/stores/agentStore.ts::buildModel` hardcodes:

```ts
contextWindow: 128000,
maxTokens: 4096,
```

These values are passed to pi-ai's `Model<TApi>` builder regardless of
provider. For Claude Sonnet 3.5 the actual ceilings are `200_000` context
tokens and `8192` max output tokens. The numbers feed pi-agent-core's
local token-budget calculations and the chat settings panel.

**Fix path**: Read the per-model values from the alias's cached metadata
once item 2 above lands, and look up the entry by model id when
constructing the pi-ai `Model`.

## 4. Anthropic OpenAPI spec refresh automation

**Severity**: Low — manual refresh works; only matters if the upstream spec
churns frequently.

`crates/routes_app/resources/openapi-anthropic.json` is a manual copy from
`~/Documents/workspace/src/github.com/BodhiSearch/anthropic-types/generated/anthropic/filtered-openapi.json`.
Refresh procedure documented at `crates/routes_app/resources/README.md`.
There is no CI check or `make` target to detect drift.

**Fix path**: Add `make sync.anthropic-spec` (or similar) that copies the
file and runs `cargo check -p routes_app`. Optionally add a CI assertion
that the checked-in file matches the upstream when the upstream repo is
also a build dependency.

**Related quirk**: The upstream filtered file omits `info.version`, which
strict OpenAPI 3.x parsers reject. BodhiApp patches it at boot (see
`routes.rs` — `openapi_anthropic` block) by injecting `"version": "1.0.0"`
into `info` if absent.

## 5. Live integration test gaps (SSE streaming, upstream errors)

**Severity**: Low — main proxy paths are covered; two edge-case code paths
have no live test counterpart:

- **SSE streaming pass-through**. The streaming proxy uses the same
  `response.bytes_stream()` plumbing as Responses API (which has
  streaming coverage in `test_oai_responses.rs`), but no Anthropic-
  specific live test covers it explicitly.
- **Upstream error pass-through**. If upstream Anthropic returns a 429 with
  its own native error envelope, BodhiApp forwards the body verbatim. Not
  asserted in any live test.

**Fix path**: Extend `crates/server_app/tests/test_live_anthropic.rs` with
streaming and error-forwarding cases.

## 6. Remove `AiApiService` wrapper — expose `AIProviderClient` directly

**Severity**: Low — code works, just an unnecessary indirection layer.

`AiApiService` trait (`crates/services/src/ai_apis/ai_api_service.rs`) is now
a thin wrapper that delegates every method to `AIProviderClient`. Each method
is 2-4 lines: construct `AIProviderClient::new(...)`, call the corresponding
method. The only logic it retains is the `test_prompt` length check.

Downstream code (routes, `server_core`) mocks `AiApiService` via
`mockall::automock`. Removing `AiApiService` requires updating all call sites
and mock setups to work with `AIProviderClient` directly.

**Fix path**:
1. Move `PromptTooLong` validation into callers or a standalone function.
2. Replace `Arc<dyn AiApiService>` in `SharedContext` / service containers
   with direct `AIProviderClient` construction at call sites.
3. Update mockall mocks in `routes_app` and `server_app` tests.
4. Delete `ai_api_service.rs` and the `AiApiService` trait.

---

## 7. `anthropic_oauth` E2E coverage incomplete (Phase 11c + 11d)

**Severity**: Medium — backend + UI shipped and manually verified, but
multi-format E2E loops have not been run end-to-end with the new variant.

**Status**: Plan at `ai-docs/claude-plans/202604/sorted-tinkering-naur.md`
Phases 1–6b shipped (commits `b1fffe66` → `1a3fd1c9`). Remaining:

- **Phase 11d** — `specs/api-models/api-models-no-key.spec.mjs` loops
  `Object.entries(ApiModelFixtures.API_FORMATS)`. Fixture now includes
  `anthropic_oauth` (commit `cfdca640`), so the iteration should run
  automatically, but the suite has NOT been run end-to-end since. A
  sub-agent attempted this and got stuck on an SSE streaming issue in
  the Anthropic SDK chat UI flow (unclear whether it's a real bug or a
  test-setup artifact). The agent was stopped without making changes.

- **Phase 11c** — `specs/api-models/api-live-upstream.spec.mjs` also loops
  over `API_FORMATS`. With `INTEG_TEST_ANTHROPIC_OAUTH_TOKEN` present in
  `.env.test`, the `anthropic_oauth` iteration should exercise:
  - Form UI with pre-filled extras → save alias
  - Primary endpoints `/v1/messages` and `/anthropic/v1/messages`
  - Universal `/v1/chat/completions` routing
  - Chat UI with OAuth token
  Not yet verified.

**Fix path**:
1. Run `npm run test:playwright:standalone -- --grep "api-models-no-key"`
   locally. Triage the `anthropic_oauth` iteration failures — most likely
   the mock server needs to recognise Bearer auth (currently it ignores
   auth, but cross-format routing might send wrong headers), or the
   chat UI's model-selection flow needs to handle the new display name.
2. Run `npm run test:playwright:standalone -- --grep "api-live-upstream"`
   with a live network to verify the OAuth token round-trip.
3. If the streaming SSE issue turns out to be a real bug (the stopped
   sub-agent suspected it was in the byte-stream passthrough in
   `convert_reqwest_to_axum`), file a separate TECHDEBT entry.

**Blocked by**: Manual reviewer follow-up; no automated gate at the
moment.

## 8. Extras fields typed as `unknown` in generated TypeScript

**Severity**: Low — frontend works via string-based JSON editors,
but consumers of `@bodhiapp/ts-client` lose type safety on the
`extra_headers`/`extra_body` fields.

**Status**: Backend uses `Option<serde_json::Value>` which utoipa
translates to an open-schema object. Generated types emit
`extra_headers?: unknown` / `extra_body?: unknown`.

**Fix path**: If we ever need typed access, define per-format structs
(e.g. `AnthropicOAuthExtras { headers: HashMap<String, String>,
body: AnthropicOAuthBody }`) behind a tagged enum keyed on `api_format`.
For now the open shape is deliberate — it keeps the wire format
extensible without schema migrations as providers add/change header
requirements.
