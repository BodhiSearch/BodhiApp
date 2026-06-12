# Gemini — Eliminate `baseModelId`, align with live upstream (Re-plan)

## Context

Previous plan addressed Gemini cleanup (split provider clients, opaque
`.models()`, forbid `x-goog-api-key`, parameterized tests). Phases 1–5
committed (`7522364b`, `75386a81`, `f471508a`, `9962e67a`). Phase 6 E2E
surfaced 6 failing specs — all Gemini.

**Root-cause confirmed via live curl** (50 models fetched from
`https://generativelanguage.googleapis.com/v1beta/models`):

- 0 of 50 models returned include `baseModelId` (spec marks it required).
- [Google AI Developers Forum issue filed 2024-12-16](https://discuss.ai.google.dev/t/basemodelid-is-not-available-in-api-response/55268)
  confirms this is a known, un-acknowledged Google spec deviation.
- All 50 models include `name` (as `"models/{id}"`), `version`,
  `displayName`. `thinking`, `topK`, `topP`, `maxTemperature`,
  `temperature`, `inputTokenLimit`, `outputTokenLimit`,
  `supportedGenerationMethods`, `description` appear on most.

Phase-1 attempted a defensive "derive `base_model_id` from `name`" fallback
— works but leaves two identifiers in our code (`name` + `base_model_id`)
for the same thing. The re-plan eliminates `baseModelId` entirely
(Rust, TS, storage, routes, frontend). `name: "models/{prefix}{modelId}"`
becomes the sole identifier.

Live sample saved at `/tmp/gemini-models-live.json` (50 models, 797 lines)
— we'll commit a trimmed representative subset as a repo fixture.

**Design decisions (from clarifications, 2026-04-13):**

- `GeminiModel` keeps typed fields for every property Google's spec
  publishes — consistent with `OpenAIModel`/`AnthropicModel` shapes. Drop
  only `baseModelId`.
- `AiApiService::fetch_models` still returns `Vec<ApiModel>` (unchanged
  signature); the HTTP adapter `api_models_fetch_models` continues to
  return `Vec<String>` of bare model ids via `ApiModel::id()`.
- `ApiModel::id()` for Gemini derives bare modelId from `name` (strips
  `"models/"` prefix). Aligns with OpenAI/Anthropic semantics.
- No new indexing mechanism on `ApiModelVec` — follow existing
  OpenAI/Anthropic pattern (iter/find at call sites).

## Uncommitted state to handle

```
M crates/lib_bodhiserver_napi/tests-js/fixtures/apiModelFixtures.mjs
M crates/lib_bodhiserver_napi/tests-js/specs/api-models/api-live-upstream.spec.mjs
M crates/lib_bodhiserver_napi/tests-js/specs/api-models/api-models-no-key.spec.mjs
M crates/services/src/ai_apis/provider_gemini.rs
M crates/services/src/models/gemini_model.rs
```

- **Keep**: fixture `primaryEndpoints` as a function; `api-live-upstream.spec.mjs` callsite update — unrelated to baseModelId, still needed.
- **Keep + refine**: `api-models-no-key.spec.mjs` mock now includes `version` (matches live shape); good. Will also extend to include representative metadata fields from the live sample.
- **Discard**: `provider_gemini.rs` base_model_id fallback — dead once field is removed.
- **Supersede**: `gemini_model.rs` `#[serde(default)]` on base_model_id — field is being deleted.

These will fold into Phase B naturally (no separate cleanup commit).

---

## Execution Model

Five phases, each a dedicated sub-agent with self-contained prompt.
Parent runs gate checks then commits locally per phase (per
`feedback_layered_refactors.md` feature-rollout variant). On E2E
failures iterate within Phase E without restarting earlier phases.

---

## Phase A — OpenAPI filter: strip `baseModelId` from `Model` schema

**Files:**

- `ts-client/scripts/sync-gemini-openapi.mjs` — after the `openapi-format`
  filter step and before the stub-patching block, add a `Model`-schema
  post-processor:
  - Delete `components.schemas.Model.properties.baseModelId`.
  - Filter `components.schemas.Model.required` to exclude `"baseModelId"`.
  - Log what was removed.
- `ts-client/openapi-gemini.json` — will be overwritten by the next
  `npm run sync:gemini` run. For this PR: manually apply the same edit
  in-place so TS type regen picks it up without needing a network call.
  (Script still runs if user refreshes later.)
- `ts-client/src/types-gemini/types.gen.ts` — regenerated via
  `npm run generate:types-gemini` after the JSON edit.
- `ts-client/src/openapi-typescript/openapi-schema-gemini.ts` —
  regenerated similarly (openapi-typescript flavor).

**Test:** no new tests; regen output is the test.

**Gate:**
```
cd ts-client && npm run generate:types-gemini
cd ts-client && npm run build
```

**Commit:** `chore(ts-client): remove baseModelId from Gemini Model schema (Google live endpoint omits it)`

---

## Phase B — Rust services: drop `base_model_id`

**Files:**

- `crates/services/src/models/gemini_model.rs`
  - Remove field `pub base_model_id: String` and its `#[serde(default)]`.
  - Remove the `base_model_id` field comment about live omission.
  - Add method:
    ```rust
    impl GeminiModel {
      pub fn model_id(&self) -> &str {
        self.name.strip_prefix("models/").unwrap_or(&self.name)
      }
    }
    ```
  - Update `test_gemini_model_serde_roundtrip_keeps_both_fields`: rename
    to `test_gemini_model_serde_roundtrip`. Remove `baseModelId` from
    input JSON; remove the baseModelId assertions. Keep `name`,
    `version`, `displayName`, etc.
  - Add `test_gemini_model_model_id_strips_prefix` covering `name:
    "models/gemini-2.5-flash"` → `model_id() == "gemini-2.5-flash"`, and
    the edge case `name: "gemini-2.5-flash"` (no prefix) → returns as-is.

- `crates/services/src/models/model_objs.rs`
  - `ApiModel::id()` for Gemini: `ApiModel::Gemini(m) => m.model_id()`.

- `crates/services/src/models/api_model_service.rs`
  - Prefix-baking (lines ~160-161 create, ~269-270 update):
    ```rust
    if let ApiModel::Gemini(ref mut m) = model {
      let bare = m.name.strip_prefix("models/").unwrap_or(&m.name);
      m.name = format!("models/{}{}", prefix, bare);
    }
    ```
  - No `base_model_id` line. One mutation only.

- `crates/services/src/ai_apis/provider_gemini.rs`
  - Remove the `if m.base_model_id.is_empty() { … }` fallback block.
  - `.models()` becomes pure:
    ```rust
    arr.iter()
      .filter_map(|v| serde_json::from_value::<GeminiModel>(v.clone()).ok())
      .map(ApiModel::Gemini)
      .collect()
    ```

- `crates/services/src/test_utils/fixtures.rs`
  - Update `gemini_model(id)` fixture: construct with
    `name: format!("models/{}", id)`, `version: "001".to_string()`,
    `display_name: None`, all others default. No `base_model_id`.

- **Fixture data**: create
  `crates/services/src/ai_apis/test_data/gemini_models_upstream_sample.json`
  with a trimmed subset of `/tmp/gemini-models-live.json` — ~3 models
  (`gemini-2.5-flash`, `gemini-2.5-pro`, `gemini-embedding-001`)
  verbatim from live response (no baseModelId; all other fields as
  returned). Load via `include_str!` in tests.

- `crates/services/src/ai_apis/test_ai_api_gemini.rs`
  - Replace hand-rolled mock JSON with the new fixture file. Every test
    asserts `m.model_id() == expected_bare_id`. No `baseModelId`
    anywhere in asserts or mock inputs.
  - Drop the `test_fetch_models_gemini_preserves_display_name` if it was
    asserting via `baseModelId`; rewrite to assert `display_name`
    preserved on the `ApiModel::Gemini(m)` returned.

- `crates/services/src/ai_apis/test_ai_api_provider_matrix.rs`
  - Any case that builds Gemini mock data — switch to the fixture file
    or remove `baseModelId` from inline json.

- `crates/services/src/models/test_api_alias_repository.rs`
  - Update DB roundtrip test: build `GeminiModel` with
    `name: "models/gmn/gemini-2.5-flash"` (prefix already baked in name
    only). Assert roundtrip preserves `name`. Drop any `baseModelId`
    assertion.

- `crates/services/src/models/test_api_model_service.rs`
  - `test_create_gemini_bakes_prefix_into_base_model_id_and_name` →
    rename to `test_create_gemini_bakes_prefix_into_name`. Assert
    `m.name == "models/gmn/gemini-2.5-flash"` and
    `m.model_id() == "gmn/gemini-2.5-flash"`. No baseModelId assert.
  - `test_update_gemini_bakes_prefix` → same rename/adjust.
  - The two `test_create/update_forward_all_stores_all_models` cases
    updated in Phase 1 — re-verify they still pass (they compared IDs;
    `m.id()` now returns stripped name — behaviorally equivalent).

**Gate:**
```
cargo check -p services 2>&1 | tail -5
cargo test -p services --lib 2>&1 | grep -E "test result|FAILED" | tail -5
```

**Commit:** `refactor(services): drop GeminiModel.base_model_id; derive id from name`

---

## Phase C — routes_app

**Files:**

- `crates/routes_app/src/gemini/routes_gemini.rs`
  - `gemini_models_list` (line ~97): dedup by `m.model_id()` instead of
    `m.base_model_id`. Response shape unchanged (serializes `GeminiModel`).
  - `gemini_models_get` (line ~121): compare `m.model_id() == model_id`
    (instead of `m.base_model_id == model_id`).
  - `resolve_gemini_alias` (line ~27): compare `gm.model_id() == model`.
  - Keep `strip_alias_prefix` and `gemini_action_handler` unchanged —
    their logic works on the path-param string, not stored struct.

- `crates/routes_app/src/gemini/test_gemini_routes.rs`
  - Update any `seed_gemini_alias_with_prefix` helper to set only
    `name` (no `base_model_id`). Reflect the Phase-B fixture pattern.
  - `test_gemini_models_list_returns_prefixed_name` (added Phase 2) —
    assert response JSON has `name == "models/{prefix}{id}"`; drop any
    `baseModelId` assertion.
  - `test_gemini_action_forwards_alt_sse_query` — unchanged.

**Gate:**
```
cargo check -p routes_app 2>&1 | tail -5
cargo test -p routes_app --lib 2>&1 | grep -E "test result|FAILED" | tail -5
cargo test -p services --lib 2>&1 | grep -E "test result|FAILED" | tail -5   # upstream regression
```

**Commit:** `refactor(routes_app): Gemini lookups use name-derived model_id; drop base_model_id`

---

## Phase D — OpenAPI regen + frontend

**Commands:**
```
cargo run --package xtask openapi
make build.ts-client
```

**Files updated by regen:**
- `openapi.json` (ApiModel::Gemini / GeminiModel schema shrinks)
- `ts-client/src/openapi-typescript/openapi-schema.ts`
- `ts-client/src/types/types.gen.ts`

**Manual frontend edits:**

- `crates/bodhi/src/schemas/apiModel.ts`
  - `getApiModelId`:
    ```ts
    export const getApiModelId = (m: ApiModel): string => {
      if ('id' in m) return m.id;
      // Gemini: name is "models/{prefix}{modelId}"; return bare modelId.
      return m.name.startsWith('models/') ? m.name.slice('models/'.length) : m.name;
    };
    ```
  - Remove the "baseModelId" narrative comment.

**Tests:**

- `crates/bodhi/src/components/api-models/ApiModelForm.extras.test.tsx`
  — any test that seeds Gemini model data: switch from `{baseModelId}`
  to `{name: "models/..."}`. Parameterized tests may exercise this only
  implicitly; adjust as needed.
- Grep all `*.test.tsx` for `baseModelId` references → remove/update.

**Gate:**
```
make ci.ts-client-check
cd crates/bodhi && npm run lint 2>&1 | tail -10
cd crates/bodhi && npm test -- --run 2>&1 | tail -10
```

**Commit:** `chore(openapi): regen after GeminiModel.baseModelId removal; frontend strips models/ prefix in getApiModelId`

---

## Phase E — E2E

Keep the already-uncommitted `primaryEndpoints` function-form change
and the `api-live-upstream.spec.mjs` callsite update.

**Files:**

- `crates/lib_bodhiserver_napi/tests-js/specs/api-models/api-models-no-key.spec.mjs`
  - Mock `/v1beta/models` handler already omits `baseModelId`. Extend
    the mock response to match the live sample more faithfully (add
    `inputTokenLimit`, `outputTokenLimit`, `thinking: false`, etc.) so
    deserialization exercises realistic data. Optional — minimal
    change is what's there today.

- Any `specs/*.mjs` that hard-codes `baseModelId` or fetches it from
  API response — grep and remove. Likely none, since frontend API
  already hides it via `getApiModelId`.

**Build + run:**
```
make build.ui-rebuild
make test.napi.standalone 2>&1 | tail -80
```

**Iterate:** per `feedback_run_all_gate_checks.md`, fix each failing
spec by root-cause → re-run only that spec → when green, re-run full
suite. Stop points:

- `api-gemini-embeddings.spec.mjs` — previously failing because filter
  dropped embed-only models (fixed Phase 1). Should pass now; confirm.
- `api-live-upstream.spec.mjs` (both API-token + OAuth-app-token
  variants) — fixtures fix already applied; confirm green.
- `api-models-no-key.spec.mjs` `[gemini]` both cases — previously
  failing because mock omitted baseModelId and Phase-1 strict schema
  rejected all models. After Phase B removes `baseModelId` entirely,
  stock deserialize path works.
- `chat-gemini.spec.mjs` — same root cause as above, same fix.

**Commit (only if changes beyond what's already unstaged):**
`test(e2e): Gemini specs — align with baseModelId-free model shape`

---

## File Index (cross-phase, updated)

**Phase A (ts-client):** `scripts/sync-gemini-openapi.mjs`,
`openapi-gemini.json`, `src/types-gemini/types.gen.ts`,
`src/openapi-typescript/openapi-schema-gemini.ts`.

**Phase B (services):** `models/gemini_model.rs`,
`models/model_objs.rs`, `models/api_model_service.rs`,
`ai_apis/provider_gemini.rs`, `ai_apis/test_ai_api_gemini.rs`,
`ai_apis/test_ai_api_provider_matrix.rs`,
`ai_apis/test_data/gemini_models_upstream_sample.json` (new),
`models/test_api_alias_repository.rs`,
`models/test_api_model_service.rs`, `test_utils/fixtures.rs`.

**Phase C (routes_app):** `gemini/routes_gemini.rs`,
`gemini/test_gemini_routes.rs`.

**Phase D (regen + frontend):** `openapi.json`, `ts-client/src/**` (generated),
`crates/bodhi/src/schemas/apiModel.ts`,
`crates/bodhi/src/**/*.test.tsx` (any baseModelId refs).

**Phase E (E2E):** `tests-js/specs/api-models/api-models-no-key.spec.mjs`
(optional richer mock), any stale fixtures.

**Memory updates:** reinforce
`.claude/projects/…/memory/feedback_plan_verification.md` — "Verify
upstream shape via live curl before trusting published OpenAPI specs.
Google's `baseModelId` is Required per spec, absent from 100% of live
responses."

---

## Verification (final)

```
make test.backend 2>&1 | grep -E "test result|FAILED|Running "
cd crates/bodhi && npm test -- --run 2>&1 | tail -10
make build.ui-rebuild
make test.napi.standalone 2>&1 | tail -30
```

Expected: all green. Report back; user handles final consolidation / push.

## Out of scope

- DB migration for old rows containing `baseModelId` in their stored
  `models` JSON — per project policy (no backwards compat) — recreate
  aliases after upgrade. Old JSON will deserialize fine since serde
  ignores unknown fields by default.
- Rewriting `matchable_models()` on `ApiAlias` — left untouched; Gemini
  routes don't call it.
- Re-introducing Vec→HashMap indexing on `ApiModelVec` — user chose
  "no new mechanism; follow the openai/anthropic pattern."
