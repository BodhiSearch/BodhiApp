# Plan: Store ApiModel with Metadata + AIProviderClient Strategy Pattern

**Status: IMPLEMENTED**

## Context

`ApiAlias.models` and `ApiAlias.models_cache` were `JsonVec` (`Vec<String>`) — only model IDs stored. This prevented `/anthropic/v1/models` from returning rich metadata. The `fetch_models` implementation had provider-specific auth/header logic scattered inline.

## User Decisions
- **Missing model IDs on create/update**: Return error (`ModelNotFoundAtProvider`)
- **ApiAliasResponse.models**: `Vec<ApiModel>` (rich metadata in CRUD responses)
- **Serde tag field**: `provider` (`#[serde(tag="provider")]`) — per-variant `#[serde(rename)]` (`openai`/`anthropic`), NOT `rename_all = "kebab-case"` (which produced `open-a-i`)
- **AnthropicModel types**: Live in `services` (upstream); used directly, not re-exported through `routes_app`
- **Anthropic routes**: Separate `anthropic/` module in `routes_app` (not merged in `oai/`)
- **Shared provider utils**: In `providers/` module (not embedded in `oai/`)

---

## Phase 1 — New Types in `services` crate [DONE]

### 1a. `services/src/models/anthropic_model.rs` (new)

Full `ModelInfo` schema from Anthropic OpenAPI: `AnthropicModel`, `AnthropicModelCapabilities`, `CapabilitySupport`, `ThinkingCapability`, `ThinkingTypes`, `ContextManagementCapability`, `EffortCapability`. All derive `Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema`.

### 1b. `ApiModel` enum in `services/src/models/model_objs.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema, FromJsonQueryResult)]
#[serde(tag = "provider")]
pub enum ApiModel {
  #[serde(rename = "openai")]
  OpenAI(async_openai::types::models::Model),
  #[serde(rename = "anthropic")]
  Anthropic(AnthropicModel),
}
```

`ApiModelVec` newtype with `Deref`, `DerefMut`, `From<Vec<ApiModel>>`, `FromIterator`.

Updated: `ApiAlias.models`/`models_cache` → `ApiModelVec`, `ApiAliasResponse.models` → `Vec<ApiModel>`, `FetchModelsResponse.models` → `Vec<ApiModel>`, `get_models() -> &[ApiModel]`, `matchable_models()` extracts `.id()`.

---

## Phase 2 — AIProviderClient Strategy Pattern [DONE]

### `services/src/ai_apis/ai_provider_client.rs` (new)

`AIProviderClient` trait + `OpenAIProviderClient` (Bearer auth, single GET `/models`) + `AnthropicProviderClient` (x-api-key + anthropic-version, paginated with `before_id`/`has_more`). Factory: `make_provider_client(api_format, api_key, base_url, client)`.

### `services/src/ai_apis/ai_api_service.rs`

`fetch_models` returns `Vec<ApiModel>`, delegates to `make_provider_client(...).models().await`. `status_to_error` moved to `AiApiServiceError` (in `error.rs`) for shared use.

---

## Phase 3 — DB Layer Updates [DONE]

- `api_model_alias_entity.rs`: Entity/view `models`/`models_cache` fields → `ApiModelVec`
- `api_alias_repository.rs`: `update_api_model_cache` takes `Vec<ApiModel>`
- `test_utils/db.rs`: `MockDbService`/`TestDbService` signatures updated

---

## Phase 4 — Service Layer Updates [DONE]

### `api_model_service.rs`

**`create()`**: When `forward_all_with_prefix = false`: extracts api_key from form, calls `fetch_models`, validates all selected IDs exist (returns `ModelNotFoundAtProvider` if any missing), filters provider results to selected models, stores `Vec<ApiModel>`. When `true`: models empty, background cache refresh stores full metadata.

**`update()`**: Same fetch+validate+filter. Resolves api_key from: updated key if `Set`, existing key from DB if `Keep`.

**`sync_cache()`/`spawn_cache_refresh()`**: `fetch_models` returns `Vec<ApiModel>` → passed directly to `update_api_model_cache`.

**New error**: `ModelNotFoundAtProvider(String)` — `ErrorType::BadRequest`.

---

## Phase 5 — Route Handler Updates + Module Restructure [DONE]

### Module restructure (diverged from original plan)

Original plan renamed `shared/anthropic_error.rs` → `shared/anthropic_objs.rs` and kept Anthropic routes in `oai/`. Actual implementation:

- **New `routes_app/src/anthropic/` module**: `anthropic_api_schemas.rs` (error types, moved from `shared/anthropic_error.rs`), `routes_anthropic.rs` (handlers, moved from `oai/`), tests
- **New `routes_app/src/providers/` module**: `resolve_api_key_for_alias` (shared utility, moved from `oai/routes_oai_responses.rs`, visibility changed from `pub(super)` to `pub(crate)`)
- **`oai/mod.rs`**: Removed anthropic constants/routes/re-exports. `routes_oai_responses` reverted to private.
- **`lib.rs`**: Added `pub mod anthropic` and `mod providers`, plus `pub use anthropic::*`

### `anthropic_models_list_handler`

Returns full metadata from `ApiModel::Anthropic`: `type`, `id` (with prefix), `display_name`, `created_at`, plus optional `capabilities`, `max_input_tokens`, `max_tokens`. Skips `ApiModel::OpenAI` variants.

### `oai/routes_oai_models.rs`

Cache refresh passes `Vec<ApiModel>` directly (no change needed — types flow through).

---

## Phase 6 — Test Updates [DONE]

### services (866 tests pass)
- `test_api_alias_repository.rs`: `Vec<ApiModel>` in cache tests
- `test_api_model_service.rs`: Mock `fetch_models` returns `Vec<ApiModel>`, updated `times()` expectations (create/update now call fetch_models for validation)
- `test_data_service.rs`, `test_model_objs.rs`: `ApiAlias::new()` with `Vec<ApiModel>`
- `test_ai_api_service.rs`: Mock responses include required OpenAI/Anthropic fields
- `test_utils/fixtures.rs`: `create_test_api_model_alias` signatures → `Vec<ApiModel>`

### routes_app (695 tests pass)
- `test_api_models_crud.rs`, `test_api_models_prefix.rs`, `test_api_models_isolation.rs`: Added `MockAiApiService` with `fetch_models` expectations via `.ai_api_service()` on builder
- `test_anthropic.rs`, `test_chat.rs`, `test_models.rs`, `test_oai_responses.rs`: `ApiAlias` builders with `ApiModel` objects
- `test_api_models_prefix.rs`: JSON assertions use `"provider": "openai"` (not `"open-a-i"`)
- `test_anthropic_api_schemas.rs`: Import path updated after module move

All test files use `openai_model(id)` and `anthropic_model(id)` helper functions.

---

## Phase 7 — Frontend Update [DONE]

### OpenAPI + TS client
Generated types: `ApiModel = (Model & {provider:'openai'}) | (AnthropicModel & {provider:'anthropic'})`.

### Frontend files updated (14 files, 881 tests pass)
- **Schema/hooks**: `convertApiToForm()`, `convertApiToUpdateForm()` extract `.id` from `ApiModel[]`. `useFetchModels` extracts IDs before setting state.
- **Display components**: `ModelTableRow`, `ModelActions`, `ModelPreviewModal` iterate with `m.id` instead of raw string.
- **Chat routing**: `AliasSelector` uses `apiModel.id` in forEach/map.
- **Test fixtures**: `createMockApiAlias`, MSW handlers, all test assertions updated for `ApiModel` objects.

---

## Files Changed (51 files, +1168 -1548)

| File | Change |
|------|--------|
| `services/src/models/anthropic_model.rs` | **New**: AnthropicModel + capability types |
| `services/src/models/model_objs.rs` | ApiModel enum, ApiModelVec, updated ApiAlias/ApiAliasResponse/FetchModelsResponse |
| `services/src/models/api_model_alias_entity.rs` | Entity/view: JsonVec → ApiModelVec |
| `services/src/models/api_alias_repository.rs` | `update_api_model_cache`: Vec<String> → Vec<ApiModel> |
| `services/src/models/api_model_service.rs` | create/update fetch+validate+filter, ModelNotFoundAtProvider |
| `services/src/ai_apis/ai_provider_client.rs` | **New**: AIProviderClient trait + OpenAI/Anthropic impls |
| `services/src/ai_apis/ai_api_service.rs` | fetch_models → Vec<ApiModel>, delegate to provider client |
| `services/src/ai_apis/error.rs` | `status_to_error` moved here from DefaultAiApiService |
| `services/src/ai_apis/mod.rs` | Expose `pub mod ai_provider_client` |
| `services/src/test_utils/db.rs` | Mock/TestDbService signature update |
| `services/src/test_utils/fixtures.rs` | Fixture signatures → Vec<ApiModel> |
| `routes_app/src/anthropic/` | **New module**: routes, error schemas, tests (moved from oai/ and shared/) |
| `routes_app/src/providers/` | **New module**: resolve_api_key_for_alias (moved from oai/) |
| `routes_app/src/oai/mod.rs` | Removed anthropic routes/constants |
| `routes_app/src/oai/routes_oai_responses.rs` | Removed resolve_api_key_for_alias (moved to providers/) |
| `routes_app/src/shared/mod.rs` | Removed anthropic_error module |
| `routes_app/src/lib.rs` | Added anthropic, providers modules |
| `routes_app/src/routes_dev.rs` | Test alias uses ApiModel |
| Test files (services: 6, routes_app: 10) | Vec<String> → Vec<ApiModel>, mock expectations |
| Frontend (14 files) | Extract .id from ApiModel[], test fixtures |
| `openapi.json` | ApiModel, AnthropicModel, ApiModelVec schemas |
| `ts-client/` (2 files) | Generated types for ApiModel discriminated union |
| `services/CLAUDE.md`, `routes_app/CLAUDE.md` | Updated for new modules and patterns |

---

## Verification Results

1. **Unit tests**: 866 services + 695 routes_app + 8 server_app + 881 frontend = all pass
2. **OpenAPI spec**: ApiModel, AnthropicModel, ApiModelVec schemas present. FetchModelsResponse.models and ApiAliasResponse.models reference ApiModel[]
3. **App startup**: Compiles, starts on port 1135, routes registered, ping responds
4. **TS client**: `provider: 'openai' | 'anthropic'` discriminated union generated correctly
