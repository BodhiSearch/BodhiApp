# Gemini API Format — Tech Debt

Items uncovered while wiring up `ApiFormat::Gemini` end-to-end. Tracked here so the routing/serialization fix lands clean and the smaller cleanups can be picked up independently.

## maxOutputTokens default fallback (frontend, hot-fixed)

**Status**: workaround landed in `crates/bodhi/src/stores/agentStore.ts` — `buildModel()` sets `maxTokens: 32000` instead of `0`.

**Background**: pi-ai's google provider (`node_modules/@mariozechner/pi-ai/dist/providers/simple-options.js`) computes:

```js
maxTokens: options?.maxTokens || Math.min(model.maxTokens, 32000)
```

When the user has the chat settings slider OFF (`options.maxTokens` undefined) AND `model.maxTokens === 0`, the result is `0`. Pi-ai then emits `generationConfig.maxOutputTokens: 0`, which Google rejects with `INVALID_ARGUMENT: max_output_tokens must be positive`.

**Why 32000**: matches the value pi-ai already uses as its Anthropic ceiling, and is a sane cap for large-context Gemini models without surprising users who haven't touched the slider.

**Followups**:
- The same `maxTokens: 0` is in `buildModel()` for ALL formats — only Gemini surfaced the bug because it errors instead of silently dropping the field. Audit whether OpenAI/Anthropic code paths quietly truncate to 0 in any edge case.
- Consider sourcing per-model defaults from the `outputTokenLimit` field already returned by `/v1beta/models` (Gemini) instead of a flat constant — would let us cap at e.g. 8192 for `gemini-2.5-flash` and 65k for `gemini-2.5-pro`.
- Push a fix upstream to pi-ai: `options?.maxTokens ?? Math.min(model.maxTokens, 32000)` (use `??` instead of `||`) so an explicit `0` from the caller doesn't silently fall through to the model default.

## Chat-UI CSP `unsafe-inline` violations

Console error observed during Gemini chat:
```
Refused to execute inline script because it violates the following Content Security Policy directive: 'script-src 'self' 'unsafe-eval''
```

Not Gemini-specific — pre-existing chat shell issue. Tracked under `project_security_remediation.md` memory; out of scope for this fix.

## MCP `405 Method Not Allowed` on chat init

```
GET http://0.0.0.0:1135/bodhi/v1/apps/mcps/<id>/mcp net::ERR_ABORTED 405
```

Chat shell calls GET on an endpoint that only accepts POST. Pre-existing; not Gemini-related. Surface separately.

## `models/` prefix handling — root cause and remediation

**Root cause**: the original `GeminiModel` struct overloaded its serde shape to be both the upstream Google wire format (`name: "models/X"`) AND the internal storage / API surface format. This caused `/bodhi/v1/models` to leak `name: "models/gemini-flash-latest"` to the frontend, which the chat selector concatenated with the alias prefix, producing model ids like `gem/models/gemini-flash-latest` — instead of the intended `gem/gemini-flash-latest`.

**Fix landed**:
- `GeminiModel` now mirrors Google's `Model` schema (`openapi-gemini.json:5423-5494`): `baseModelId` is the bare model id and the source of truth; `name` is always derived as `models/{baseModelId}` and is never stored. Added missing schema fields (`baseModelId`, `maxTemperature`, `thinking`).
- All `models/` prefix handling is collapsed into a single explicit translation point:
  - **Inbound** (`GeminiProviderClient::models` in `crates/services/src/ai_apis/ai_provider_client.rs`): converts upstream `name: "models/X"` to internal `baseModelId: "X"` before deserialize. Third-party Gemini-compatible hosts that emit `name` without the `models/` prefix work too — `strip_prefix` is a no-op.
  - **Outbound** (`gemini_wire_format` in `crates/routes_app/src/gemini/routes_gemini.rs`): synthesizes both `name: "models/{prefix}{baseModelId}"` and `baseModelId: "{prefix}{baseModelId}"` so the @google/genai SDK can introspect responses unchanged.
- `ApiModel::id()` for the Gemini variant returns `&base_model_id` — same external contract, internal field renamed from `id`.
- Frontend `getApiModelId` reads `m.baseModelId` for Gemini variants.

**Followups**:
- The `gemini_wire_format()` helper is small enough to live next to the handlers, but if a third Gemini-shaped surface emerges (e.g. embeddings list, tuned models), promote it into a `GeminiWireModel` newtype.
- `GeminiProviderClient::models` does the JSON-level rewrite manually; if more upstream-translation cases appear, pull this into a typed adapter (`UpstreamGeminiModel { name: String } -> GeminiModel`).

## Wildcard route and `validate_model_id` permissiveness

**Root cause**: `/v1beta/models/{id}` used Axum's single-segment matcher. Prefixed aliases (e.g. `gem/`) produce paths like `/v1beta/models/gem/gemini-flash-latest:streamGenerateContent`, which 404'd because `{id}` only matches `gem`. The `@google/genai` SDK does not URL-encode `/`, so the production behavior diverged from the test behavior (which used `%2F`).

**Fix landed**: route changed to `/v1beta/models/{*model_path}` (Axum 0.8 wildcard). New regression test `test_action_handler_accepts_literal_slash_in_prefixed_alias` asserts the literal-slash path matches.

**Followups**:
- `validate_model_id` already permits `/`, but that means a malicious caller can submit `/v1beta/models/../../etc/passwd:generateContent`. The handler resolves via `find_alias` so directory traversal can't escape, but tighten the validator to forbid `..` segments and consecutive slashes for defense in depth.
- The wildcard catches everything under `/v1beta/models/` for both GET and POST. If we add other Gemini sub-resources later (e.g. `/v1beta/models/{model}/operations/{op}`), the dispatch logic in `gemini_action_handler` needs to learn to peel off action vs sub-path.

## Test masking from URL-encoded paths

The pre-existing `test_generate_content_strips_alias_prefix` used `%2F` in its request URL, so the single-segment route matched and the test passed — but real browser/SDK traffic uses literal `/` and 404'd. New `test_action_handler_accepts_literal_slash_in_prefixed_alias` uses an unencoded `/` to lock down the production behavior.

**Followup**: audit other route tests that assert on prefixed model ids — anywhere they use `%2F`, add a sibling case with literal `/`.
