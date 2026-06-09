# Plan: Add Mandatory `name` Field to API Models

## Context

API models (`api_model_aliases` table) are identified only by an auto-generated ULID `id` (e.g. `01jc8x...`). In the UI's "Name" column this ULID is what users see — opaque and unmemorable. There is no way to give an API model a human-readable label.

This change adds a **mandatory, user-provided `name`** to API models — a free-text UTF-8 string, `length(min=1, max=255)`, **not** unique (the ULID `id` stays the unique key, like MCP instance `name`). It is required on create and update, returned in all responses, and shown as the primary label in the models list (replacing the ULID). Existing rows are backfilled with their `id` so the NOT-NULL/NOT-EMPTY contract holds.

### Decisions (confirmed with user)
- **Format**: Free-text UTF-8, `#[validate(length(min = 1, max = 255))]`. No charset regex.
- **Uniqueness**: None. Duplicates allowed; `id` remains the unique identifier.
- **Backfill**: Existing rows → `name = id`.
- **Both request types**: `DefaultApiModelRequest` AND `LlmLibertyApiModelRequest` get `name`.
- **List display**: "Name" column shows `name` only (replaces the ULID for API aliases).
- **Form position**: Name input is the first field, above API Format.

### Scope reality (from exhaustive call-site census)
Adding a **required** field to `ApiAlias` touches a lot of call sites: ~6 production + ~41 test calls to `ApiAlias::new(...)`, ~26 struct literals, ~35 `ApiAliasBuilder` usages, plus ~48 `DefaultApiModelRequest` and 8 `LlmLibertyApiModelRequest` constructions. The strategy below minimizes churn by leaning on the builder's `test_default()` and the `Default` derive that already exist under the `test-utils` feature.

---

## Strategy to Contain Blast Radius

1. **`ApiAlias.name` is a plain `String` field** added to the struct. The `ApiAliasBuilder` (derive_builder) auto-generates a `.name(...)` setter. `build_with_time` and `ApiAliasBuilder::test_default()` set a sensible default so the ~35 builder-based tests need **zero** changes (or one-line `test_default` edit).
2. **`ApiAlias::new(...)` gains a `name` parameter.** This is the unavoidable churn — every `::new` caller adds one arg. Production callers pass the real name; tests pass a literal. (Alternative considered: keep `new` arity and add `with_name()` — rejected because a *required* field passed via optional setter invites bugs where name silently stays empty.)
3. **`DefaultApiModelRequest` / `LlmLibertyApiModelRequest` derive `Default` under `#[cfg_attr(test/test-utils)]`** — so test literals using `..Default::default()` keep compiling; only literals that spell out every field need `name` added.
4. **Repository `update` uses `..Default::default()` on the SeaORM `ActiveModel`** — meaning a new column defaults to `NotSet` and would be **silently skipped on edit**. We MUST explicitly add `name: Set(model.name.clone())` to the update ActiveModel, or edits won't persist the name. This is the single highest-risk spot.

---

## Implementation (upstream → downstream)

### Phase 1 — Migration (`services`)

**New file**: `crates/services/src/db/sea_migrations/m20250101_000023_api_alias_name.rs`
Follow `m20250101_000020_api_alias_extra_fields.rs` structure.

- `up()`:
  1. `ADD COLUMN name` as `.text().not_null().default("")` (so the ALTER succeeds on existing rows).
  2. Raw SQL backfill: `UPDATE api_model_aliases SET name = id` (works on both SQLite & PostgreSQL).
  3. PostgreSQL only: `ALTER COLUMN name DROP DEFAULT` (SQLite can keep the harmless default — app-layer validation enforces non-empty).
- `down()`: `DROP COLUMN name`.

**Register** in `crates/services/src/db/sea_migrations/mod.rs`:
- `mod m20250101_000023_api_alias_name;`
- Append `Box::new(m20250101_000023_api_alias_name::Migration)` to the vec.

---

### Phase 2 — Entity + Domain Types (`services`)

**`crates/services/src/models/api_model_alias_entity.rs`**
- Add `pub name: String` to `Model` (after `id`).
- Add `pub name: String` to `ApiAliasView` (after `id`).
- In `From<ApiAliasView> for ApiAlias`: set `name: v.name`.

**`crates/services/src/models/model_objs.rs`**
- `ApiAlias` struct (line ~810): add `pub name: String` after `id`. Add `#[builder(default)]` so the builder doesn't make it required (keeps `test_default()` callers green).
- `ApiAlias::new(...)` (line ~833): add `name: impl Into<String>` param (place right after `id`); set `name: name.into()` in body.
- `ApiAliasBuilder::build_with_time` (line ~897): add `name: self.name.clone().unwrap_or_default()`.
- `ApiAliasResponse` struct (line ~2054): add `pub name: String` after `id`.
- `From<ApiAlias> for ApiAliasResponse` (line ~2077): set `name: alias.name`.
- `DefaultApiModelRequest` (line ~1327): add `pub name: String` with `#[validate(length(min = 1, max = 255))]`. (Struct derives `Default` under test feature → fine.)
- `LlmLibertyApiModelRequest` (line ~1367): add `pub name: String` with the same validation.
- `ApiModelRequest` impl: add `pub fn name(&self) -> &str` accessor matching the existing `prefix()`/`models()` match arms.

**`crates/services/src/models/api_model_service.rs`**
- `create_default` (line ~376): pass `form.name` into `ApiAlias::new(...)`. The two scratch `ApiAlias::new(...)` calls used only to build a client for `fetch_models` (lines ~356, ~489) can pass `String::new()` / the existing name — name is irrelevant to model fetching.
- `create_llm_liberty` (line ~431): pass `form.name` into `ApiAlias::new(...)`.
- `update_default` (line ~509 block): add `api_alias.name = form.name;` alongside the other field assignments.
- `update_llm_liberty`: add `api_alias.name = form.name;` in the same way.

**`crates/services/src/models/api_alias_repository.rs`** — CRITICAL
- `create_api_model_alias` ActiveModel (line ~113): add `name: Set(alias.name.clone()),`.
- `update_api_model_alias` ActiveModel (line ~182, the one with `..Default::default()`): add `name: Set(model.name.clone()),`. **Without this, edits silently drop the name.**

**`crates/services/src/test_utils/model_fixtures.rs`** (line ~179)
- In `ApiAliasBuilder::test_default()`, add `.name("test-name")` so all 35 builder-based tests get a valid name for free.

Run: `cargo test -p services --lib 2>&1 | grep -E "test result|FAILED|error\["`

---

### Phase 3 — Fix Remaining Rust Call Sites (`services`, `routes_app`, `server_core`)

The census identified every site. Mechanically add `name` to:
- **~41 test `ApiAlias::new(...)` calls** — add a literal name arg (e.g. `"test-name"`).
- **~25 `ApiAlias { ... }` struct literals** (incl. `test_utils/fixtures.rs`, `routes_dev.rs:314`, all `test_ai_api_*.rs`, `test_api_alias_repository*.rs`) — add `name: "test-name".to_string(),`.
- **`routes_dev.rs:314`** (production dev-seed): add `name: "Test API Alias".to_string(),`.
- **`DefaultApiModelRequest` / `LlmLibertyApiModelRequest` literals** that spell out all fields — add `name`. Literals using `..Default::default()` need nothing.

Representative paths (not exhaustive — see census in conversation):
`crates/services/src/models/test_*.rs`, `crates/services/src/ai_apis/.../test_ai_api_*.rs`, `crates/services/src/db/.../test_api_alias_repository*.rs`, `crates/server_core/.../test_shared_rw.rs`, `crates/routes_app/src/routes_dev.rs`.

Run: `cargo check --workspace --all-targets 2>&1 | grep -E "error\[|missing field" | head -50` — iterate until clean. The compiler enumerates every remaining site.

---

### Phase 4 — Route Handler + Service Tests (`routes_app`, `services`)

Handlers in `routes_api_models.rs` need **no logic change** (they pass `ApiModelRequest` straight through). Update tests:

- **`test_api_models_create.rs`**, **`test_api_models_read_update_delete.rs`** (incl. its `create_expected_response()` helper at line ~40), **`test_api_models_prefix.rs`**, **`test_api_models_auth.rs`**, **`test_api_models_isolation.rs`**, **`test_api_models_sync.rs`**, **`test_api_models_llm_liberty.rs`**, **`test_types.rs`**: add `name` to request payloads and to `ApiAliasResponse` assertions.
- **New validation cases** in `test_api_models_validation_basic.rs` / `test_api_models_validation_format.rs`:
  - missing `name` → 400
  - `name = ""` → 400 (length min)
  - `name` > 255 chars → 400 (length max)
  - happy path with a valid name → 200/201, name echoed in response.
- **`test_api_model_service.rs`** (`services`): add `name` to all request builders and response assertions; add a test asserting **update changes the name** (guards the `..Default::default()` ActiveModel trap from Phase 2).

Run: `cargo test -p routes_app --lib 2>&1 | grep -E "test result|FAILED|error\["`

---

### Phase 5 — OpenAPI + TypeScript Client

```bash
cargo run --package xtask openapi
make build.ts-client
```
Adds `name` to `ApiAlias`, `ApiAliasResponse`, `DefaultApiModelRequest`, `LlmLibertyApiModelRequest` schemas and regenerates `ts-client` types. Verify with `make ci.ts-client-check`.

---

### Phase 6 — Frontend (`crates/bodhi/src`)

**Schema** — `src/schemas/apiModel.ts`
- Add `name: z.string().min(1, 'Name is required').max(255, 'Name must be 255 characters or fewer')` to `baseShape`.
- `convertFormToCreateRequest` and `convertFormToUpdateRequest`: include `name: formData.name` in BOTH the default-format return object and the `llm_liberty_oauth` return object.
- `convertApiToForm`: set `name: apiData.name`.

**Form hook** — `src/components/api-models/hooks/useApiModelForm.ts`
- Add `name` to all three `defaultValues` branches (edit → `initialData?.name ?? ''`; create/setup → `''`).
- `handleApiFormatChange` does NOT reset `name` (it's format-independent; leave it untouched).

**Form component** — `src/components/api-models/ApiModelForm.tsx`
- Add a `Name` text input as the **first field** in `CardContent`, before `<ApiFormatSelector>`.
- `data-testid="api-model-name-input"`. Wire via `formLogic.register('name')`, show `formLogic.errors.name`.
- Likely a small new `NameInput` component under `src/components/api-models/form/` to match the existing per-field component pattern (`BaseUrlInput`, `PrefixInput`, etc.) — keeps the form composition consistent.

**Models list** — `src/routes/models/-components/ModelTableRow.tsx`
- In the `name`/`name_source`/`combined` cells, for API aliases render `model.name` instead of `model.id` (`isApiAlias(model) ? model.name : model.alias`). The `id` is still used for `data-model-id` and copy/edit actions, so it stays accessible — just not the displayed label. Keep a copyable `id` as secondary muted text if it fits the existing cell layout.

**Preview modal** — `src/routes/models/-components/ModelPreviewModal.tsx`
- Add a `Name` field (`data-testid="preview-api-name"`) in the API metadata section, near `preview-api-format`.

**Frontend tests / fixtures**
- `src/test-fixtures/models.ts`: add `name` to API-alias fixture factories.
- `src/test-utils/msw-v2/handlers/api-models.ts`: include `name` in mocked responses.
- Update `ApiModelForm.test.tsx`, `ApiModelForm.extras.test.tsx`, `ApiModelForm.llm_liberty.test.tsx`, `routes/models/api/edit/index.test.tsx`: fill the name input; assert it round-trips; add a "name required" validation test.

Run: `cd crates/bodhi && npm test`

---

### Phase 7 — E2E (`crates/lib_bodhiserver/tests-js`)

**Fixtures** — `fixtures/apiModelFixtures.mjs`
- `createModelData()` (line ~189): add `name: 'Test API Model'` default.
- Per-format helper `createModelDataForFormat` / `API_FORMATS`: give each a distinct name (e.g. `Test OpenAI`, `Test Anthropic`) so list assertions can target a specific row.
- `validateModelData()`: add `name` to the required-fields check.
- `scenarios.*`: ensure each includes a `name`.

**Page object (form)** — `pages/components/ApiModelFormComponent.mjs`
- Add selector `nameInput: '[data-testid="api-model-name-input"]'`.
- Add `async fillName(name)` and call it from `fillBasicInfo()` (or add a `fillBasicInfoWithName`) so existing flows populate the now-required field.
- `waitForFormReady()`: ensure it waits for the name input too if needed.

**Page object (list)** — `pages/ModelsListPage.mjs`
- Add `nameCell` selector keyed by id.
- Update `verifyApiModelInList(...)` to accept and assert the displayed `name`.

**Helper** — `utils/api-model-helpers.mjs`
- `registerApiModelViaUI` (line ~14): fill `name` (from `modelData.name`) before/with `fillBasicInfo`; return `modelName`/`name`.

**Specs** — `specs/api-models/*.spec.mjs` (`api-models-prefix`, `api-models-extras`, `api-live-upstream`, `api-sdk-compat`, `api-models-no-key`, `api-models-forward-all`)
- Pass `name` through all create/edit flows; assert the name appears in the list and (where relevant) the preview modal.
- Add one spec asserting the name persists across an edit (create → edit name → reload → verify) to cover the Phase-2 update trap end-to-end.

Run: `make build.dev-server && make test.e2e`

---

## Key Files

| File | Change |
|------|--------|
| `crates/services/src/db/sea_migrations/m20250101_000023_api_alias_name.rs` | **New** — add `name` col, backfill `= id` |
| `crates/services/src/db/sea_migrations/mod.rs` | Register migration |
| `crates/services/src/models/api_model_alias_entity.rs` | `name` on `Model`, `ApiAliasView`, `From` impl |
| `crates/services/src/models/model_objs.rs` | `name` on `ApiAlias`(+`new`,+builder), `ApiAliasResponse`(+`From`), both request types, `ApiModelRequest::name()` |
| `crates/services/src/models/api_model_service.rs` | Thread `name` through create/update for both default & llm-liberty |
| `crates/services/src/models/api_alias_repository.rs` | `name: Set(...)` in BOTH create and update ActiveModel (update is the trap) |
| `crates/services/src/test_utils/model_fixtures.rs` | `.name("test-name")` in `test_default()` |
| ~50 Rust test/fixture files + `routes_dev.rs` | Add `name` (compiler-enumerated via Phase 3) |
| `crates/bodhi/src/schemas/apiModel.ts` + form hook + `ApiModelForm.tsx` + new `NameInput` | Name field, validation, conversions |
| `crates/bodhi/src/routes/models/-components/ModelTableRow.tsx` | Show `name` (not id) for API aliases |
| `crates/bodhi/src/routes/models/-components/ModelPreviewModal.tsx` | `preview-api-name` field |
| `crates/bodhi/src/test-fixtures/models.ts`, `test-utils/msw-v2/handlers/api-models.ts`, form tests | Frontend test data + assertions |
| `crates/lib_bodhiserver/tests-js/fixtures/apiModelFixtures.mjs`, `pages/components/ApiModelFormComponent.mjs`, `pages/ModelsListPage.mjs`, `utils/api-model-helpers.mjs`, `specs/api-models/*.spec.mjs` | E2E fixtures, page objects, specs |

---

## Verification

1. **Migration round-trip**: `cargo test -p services --lib` — DB create/read/update tests pass with `name`; the new update-changes-name service test passes (proves the ActiveModel update fix).
2. **Full backend**: `make test.backend` — zero regressions; validation tests confirm missing/empty/>255 name → 400.
3. **TS sync**: `make ci.ts-client-check` — generated types match the spec.
4. **Frontend**: `cd crates/bodhi && npm test` — name renders as the list label, validates, and round-trips through create/edit.
5. **E2E**: `make test.e2e` — all `api-models-*` specs pass; name shows in list/preview and survives an edit.
6. **Manual smoke** (`make app.run.live`): create an API model with a name, confirm the list shows the name (not the ULID), edit the name, confirm it persists after reload.
