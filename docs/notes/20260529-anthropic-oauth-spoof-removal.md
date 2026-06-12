# Tech Debt: Remove Claude Code "spoof" hardcoding for anthropic_oauth

**Status:** Open tech-debt — deferred cleanup map (execute later; upstream requirement already gone).

**Date**: 2026-05-29

## Context

The `anthropic_oauth` API format ("Anthropic Setup Token") was built around a now-obsolete
requirement of Anthropic's API. The OAuth setup token (`sk-ant-oat01...`, obtained via
`npx ... login`) historically required spoofing **Claude Code** client identity, otherwise the
upstream API returned `401`. BodhiApp carried this spoof as hardcoded default `extra_headers` /
`extra_body` on the alias, prefilled by the frontend when the user selected the format.

**Anthropic has since relaxed this requirement.** Verified live (2026-05-29) against a real
`sk-ant-oat01...` token with **no spoofing headers or body** — only `Authorization: Bearer <token>`
and `anthropic-version: 2023-06-01`:

| Request | Result |
|---|---|
| `GET /v1/models` (bare) | **HTTP 200** |
| `POST /v1/messages` non-streaming (bare) | **HTTP 200**, valid completion |
| `POST /v1/messages` streaming (bare) | **HTTP 200**, SSE stream |

This was discovered because the E2E test
`clearing anthropic_oauth extras causes backend metadata fetch to fail on save`
(`crates/lib_bodhiserver/tests-js/specs/api-models/api-models-extras.spec.mjs`) began failing.
That test cleared the alias extras on edit and expected the backend `fetch_models` metadata fetch
to 401 → "Failed to Update API Model" toast. With the upstream requirement gone, the save now
succeeds and the toast never appears. **The test has been removed** (its premise is invalid); see
the git history for that spec. This doc captures the remaining cleanup so it can be executed later
without re-investigating.

### What was hardcoded (the "Claude Code spoof")

1. `anthropic-beta: claude-code-20250219,oauth-2025-04-20`
2. `user-agent: claude-cli/2.1.80 (external, cli)`
3. System prompt: `"You are Claude Code, Anthropic's official CLI for Claude."`
4. `max_tokens: 4096` default body

## Scoping Constraints

- **llm_liberty stays untouched.** The whole point of `llm_liberty` is that it *owns the envelope*
  (provider protocol — headers/body). If the working envelope changes, we update `llm_liberty` to
  emit the most up-to-date one; BodhiApp dumb-forwards it. Do **not** remove anything on the
  `LlmLibertyOauth` / `LibertyAnthropicClient` path.
- **Only remove hardcoding that lives in BodhiApp itself** — the `anthropic_oauth` Claude Code
  spoof defaults baked into BodhiApp's own frontend/backend.

## Surface Area (BodhiApp-owned — candidate for removal)

### Frontend — source of the prefill

- `crates/bodhi/src/schemas/apiModel.ts:33-46` — `API_FORMAT_PRESETS.anthropic_oauth.defaultHeaders` / `defaultBody` hold the spoof constants.
- `crates/bodhi/src/components/api-models/hooks/useApiModelForm.ts:164-184` — `handleApiFormatChange` prefills `extra_headers` / `extra_body` from the preset on format select.
- `crates/bodhi/src/components/api-models/hooks/useApiModelForm.ts:331-337` — `showExtras` computed value: extras editor renders **only** when a preset has `defaultHeaders` / `defaultBody` → today only `anthropic_oauth`.
- `crates/bodhi/src/components/api-models/form/ExtrasSection.tsx` — the two JSON textareas.
- `crates/bodhi/src/components/api-models/ApiModelForm.tsx:119-128` — conditional render of `ExtrasSection`.
- `crates/bodhi/src/schemas/apiModel.ts:62-96,176-177` — extras JSON validation + forbidden-auth-header rule; `:250-251,295-296,314-315` — extras wired into create/update/convert.

### Backend — plumbing that carried the spoof

- `crates/services/src/ai_apis/clients/anthropic_oauth.rs` — `AnthropicOauthClient` is a separate client from `AnthropicClient` **solely** because it stores + merges `extra_headers` / `extra_body`. `apply_auth` injects extra_headers; `test_prompt` / `forward_request_with_method` merge extra_body (the latter only for `/messages`).
- `crates/services/src/ai_apis/ai_api_client_factory.rs:147-154` — only the `AnthropicOAuth` arm passes `extra_headers` / `extra_body` to the client constructor.
- `crates/services/src/ai_apis/provider_shared.rs:63-98` — `merge_extra_body` (system-prompt prepend semantics), used only by `AnthropicOauthClient`.
- `crates/services/src/ai_apis/clients/anthropic_shared.rs:11-49` — `apply_bearer_auth_and_version` (extra_headers injection + default `anthropic-version`). **NOTE:** also used by `LibertyAnthropicClient` — keep for the Liberty path.
- `ApiFormat::AnthropicOAuth` enum + match arms in `crates/services/src/models/model_objs.rs` (enum `:784-786`; request/test/fetch arms `:1230-1231,1498-1499,1642-1643`; treated identically to `Anthropic` in getters / `as_default`).
- `crates/routes_app/src/anthropic/routes_anthropic.rs:49-106` — `resolve_anthropic_alias`: `Anthropic` and `AnthropicOAuth` both take the `Native` arm (identical). The Liberty arm is separate — leave alone.
- DB: `api_model_aliases.extra_headers` / `extra_body` columns. Existing rows may hold stored spoof values.

### Tests that assume the spoof (update when cleanup happens)

- Frontend: `crates/bodhi/src/components/api-models/ApiModelForm.extras.test.tsx` (~11 tests: prefill, visibility, validation, round-trip).
- Backend services: `crates/services/src/ai_apis/test_ai_api_anthropic_oauth.rs` (5 tests: merge/inject behaviour).
- Backend routes: `crates/routes_app/src/models/api/test_api_models_create.rs:193` (stores extras), `test_api_models_sync.rs:303` (passes extras), `test_api_models_validation_basic.rs:348`, `crates/routes_app/src/anthropic/test_anthropic_oauth_routing.rs:55`.
- E2E: `crates/lib_bodhiserver/tests-js/specs/api-models/api-models-extras.spec.mjs`, fixtures `apiModelFixtures.mjs:105-135`, page object `pages/components/ApiModelFormComponent.mjs` (`expectExtrasVisible`, `expectExtrasPrefilledFor`, `fillExtra*`, `expectExtra*Error`), and `specs/api-models/api-live-upstream.spec.mjs` (loops all formats incl. anthropic_oauth).

### Out of scope — do NOT touch (llm_liberty owns its envelope)

- `crates/services/src/ai_apis/clients/liberty_anthropic.rs`, `liberty_codex.rs`, `llm_liberty/`, `models/llm_liberty_envelope.rs`, and the `Liberty` arm of `resolve_anthropic_alias`.

## Options for the Cleanup (decide when picked up)

Three escalating options:

1. **Drop spoof defaults only** — remove `defaultHeaders` / `defaultBody` from the frontend preset and the prefill; keep the generic `extra_headers` / `extra_body` editor and `AnthropicOauthClient` plumbing as a user escape hatch. Smallest change.
2. **Remove extras entirely** — drop the editor, validation, `AnthropicOauthClient` extras fields, `merge_extra_body` usage, and the DB columns; `anthropic_oauth` becomes a thin Bearer-auth variant of `anthropic`.
3. **Collapse into `anthropic`** — eliminate the `AnthropicOAuth` format; fold Bearer-token auth into the single `Anthropic` client/format (DB enum migration + proxy resolver simplification). Most aggressive.

### Sub-questions to resolve

- Keep the generic `extra_headers` / `extra_body` editor as a power-user escape hatch, or was it always just scaffolding for the spoof?
- Tolerance for existing stored rows in `api_model_aliases` (ignore/drop the stale extras vs keep them working unchanged)?
- Fate of `merge_extra_body`'s system-prompt-prepend semantics (anthropic_oauth-specific today)?
- Is `anthropic_oauth` a genuinely distinct format worth keeping, or just "anthropic + setup-token auth scheme (`Authorization: Bearer` vs `x-api-key`)"?
