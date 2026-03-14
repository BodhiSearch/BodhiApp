# Plan: Models Routes & UI Reorganization

## Context

BodhiApp has three model types (Model/GGUF files, User aliases, API proxy models) but the routes and UI are organized inconsistently — `api-models/` as a flat sibling, `models/` handling both common and alias-specific operations, `modelfiles/` and `pull/` as separate sections. This reorganization groups everything under a unified `/models/` namespace with clear sub-sections: `/models/alias/`, `/models/api/`, `/models/files/`.

**Key decisions:**
- Clean break, no backwards compatibility
- Routes + UI reorganization; service layer mostly unchanged
- Remove `ApiModelOutput` type, merge `has_api_key` into `ApiAliasResponse`
- Remove separate API model list endpoint (use unified `GET /models/`)
- Polymorphic `GET /models/{id}` deferred to follow-up
- Auth boundaries stay the same

---

## Route Mapping (old → new)

| Old Endpoint | New Endpoint | Notes |
|---|---|---|
| `GET /bodhi/v1/models` | **stays** | Unified list (all 3 types) |
| `GET /bodhi/v1/models/{id}` | **stays** | Common show (user-alias-only for now) |
| `POST /bodhi/v1/models` | `POST /bodhi/v1/models/alias` | Alias create |
| `PUT /bodhi/v1/models/{id}` | `PUT /bodhi/v1/models/alias/{id}` | Alias update |
| `DELETE /bodhi/v1/models/{id}` | `DELETE /bodhi/v1/models/alias/{id}` | Alias delete |
| `POST /bodhi/v1/models/{id}/copy` | `POST /bodhi/v1/models/alias/{id}/copy` | Alias copy |
| `GET /bodhi/v1/api-models` | **REMOVED** | Use unified list |
| `GET /bodhi/v1/api-models/{id}` | `GET /bodhi/v1/models/api/{id}` | API model show |
| `POST /bodhi/v1/api-models` | `POST /bodhi/v1/models/api` | API model create |
| `PUT /bodhi/v1/api-models/{id}` | `PUT /bodhi/v1/models/api/{id}` | API model update |
| `DELETE /bodhi/v1/api-models/{id}` | `DELETE /bodhi/v1/models/api/{id}` | API model delete |
| `GET /bodhi/v1/api-models/api-formats` | `GET /bodhi/v1/models/api/formats` | API formats |
| `POST /bodhi/v1/api-models/test` | `POST /bodhi/v1/models/api/test` | Test credentials |
| `POST /bodhi/v1/api-models/fetch-models` | `POST /bodhi/v1/models/api/fetch-models` | Fetch remote models |
| `POST /bodhi/v1/api-models/{id}/sync-models` | `POST /bodhi/v1/models/api/{id}/sync-models` | Sync cache |
| `GET /bodhi/v1/modelfiles` | `GET /bodhi/v1/models/files` | Local GGUF files |
| `GET /bodhi/v1/modelfiles/pull` | `GET /bodhi/v1/models/files/pull` | List downloads |
| `POST /bodhi/v1/modelfiles/pull` | `POST /bodhi/v1/models/files/pull` | Start download |
| `GET /bodhi/v1/modelfiles/pull/{id}` | `GET /bodhi/v1/models/files/pull/{id}` | Show download |
| `POST /bodhi/v1/models/refresh` | **stays** | Metadata refresh |
| `GET /bodhi/v1/queue` | **stays** | Queue status |

---

## Execution: Phased Sub-Agent Approach

Each phase is executed by a specialized sub-agent, with gate checks and a local commit before handing off to the next phase. The prior phase's git diff summary is fed as context to the next sub-agent.

---

### Phase 1: Services Crate — Type Changes

**Sub-agent scope:** `crates/services/` only

**Tasks:**
1. Add `has_api_key: bool` to `ApiAliasResponse` (`crates/services/src/models/model_objs.rs` ~line 1484)
   - Update `From<ApiAlias> for ApiAliasResponse`: default `has_api_key: false`
   - Add builder: `pub fn with_has_api_key(mut self, v: bool) -> Self`
2. Remove `ApiModelOutput` struct (~line 1148) and its `from_alias` method
3. Remove `PaginatedApiModelOutput` struct
4. Update `ApiModelService` trait (`crates/services/src/models/api_model_service.rs`):
   - Remove `list` method
   - Change `create`, `update`, `get`, `sync_cache` return types → `ApiAliasResponse`
   - Update `DefaultApiModelService` impl: `ApiAliasResponse::from(alias).with_has_api_key(has_key)`
5. Update `AuthScopedApiModelService` (`crates/services/src/models/auth_scoped_api_models.rs`):
   - Match trait changes, remove `list`
6. Fix any test files referencing removed types

**Gate check:**
```bash
cargo check -p services
cargo test -p services --lib
```

**Commit:** `refactor: remove ApiModelOutput, add has_api_key to ApiAliasResponse`

---

### Phase 2: Routes App — Module Restructure & Endpoint Changes

**Sub-agent scope:** `crates/routes_app/` only (receives Phase 1 summary)

**Tasks:**

**2a. Update endpoint constants** (`crates/routes_app/src/shared/openapi.rs`):
- Remove: `ENDPOINT_MODEL_FILES`, `ENDPOINT_MODEL_PULL`, `ENDPOINT_API_MODELS`, `ENDPOINT_API_MODELS_TEST`, `ENDPOINT_API_MODELS_FETCH_MODELS`, `ENDPOINT_API_MODELS_API_FORMATS`
- Add: `ENDPOINT_MODELS_ALIAS`, `ENDPOINT_MODELS_API`, `ENDPOINT_MODELS_API_TEST`, `ENDPOINT_MODELS_API_FETCH_MODELS`, `ENDPOINT_MODELS_API_FORMATS`, `ENDPOINT_MODELS_FILES`, `ENDPOINT_MODELS_FILES_PULL`
- Keep: `ENDPOINT_MODELS`, `ENDPOINT_MODELS_REFRESH`, `ENDPOINT_QUEUE`

**2b. Update OpenAPI tags:**
- Remove `API_TAG_API_MODELS`
- Add: `API_TAG_MODELS_ALIAS`, `API_TAG_MODELS_API`, `API_TAG_MODELS_FILES`

**2c. Restructure modules:**
```
routes_app/src/models/
├── mod.rs                          (re-exports)
├── routes_models.rs                (models_index, models_show ONLY)
├── routes_models_metadata.rs       (refresh_metadata_handler — stays)
├── models_api_schemas.rs           (stays)
├── error.rs                        (ModelRouteError — stays)
├── test_metadata.rs
├── api/
│   ├── mod.rs
│   ├── routes_api_models.rs        (from api_models/, remove api_models_index)
│   ├── error.rs                    (ApiModelsRouteError from api_models/)
│   ├── test_routes_api_models_crud.rs
│   ├── test_routes_api_models_auth.rs
│   ├── test_routes_api_models_validation.rs
│   ├── test_routes_api_models_isolation.rs
│   ├── test_routes_api_models_prefix.rs
│   ├── test_routes_api_models_sync.rs
│   ├── test_types.rs
├── alias/
│   ├── mod.rs
│   ├── routes_alias.rs             (extract create/update/destroy/copy from routes_models.rs)
│   ├── test_aliases_crud.rs
│   ├── test_aliases_auth.rs
├── files/
│   ├── mod.rs
│   ├── routes_files.rs             (modelfiles_index from routes_models.rs)
│   ├── routes_files_pull.rs        (from routes_models_pull.rs)
│   ├── test_downloads_isolation.rs
│   ├── test_pull.rs
```
- Delete: `routes_app/src/api_models/` directory, remove `mod api_models` from lib.rs

**2d. Update handlers:**
- Each handler: update `#[utoipa::path(...)]` — path, tag, operation_id
- API model handlers: return `ApiAliasResponse` instead of `ApiModelOutput`
- Remove `api_models_index` handler entirely

**2e. Update route registration** (`crates/routes_app/src/routes.rs`):
- `user_apis`: `modelfiles_index` → `ENDPOINT_MODELS_FILES`
- `user_session_apis`: all `ENDPOINT_API_MODELS*` → `ENDPOINT_MODELS_API*`; remove `api_models_index`
- `power_user_apis`: alias CRUD → `ENDPOINT_MODELS_ALIAS`; pull → `ENDPOINT_MODELS_FILES_PULL`

**2f. Update OpenAPI doc registration:**
- Remove `__path_api_models_index`, `ApiModelOutput`, `PaginatedApiModelOutput` from schemas
- Add new tags, verify `ApiAliasResponse` registered

**2g. Update all test files:**
- Import paths, endpoint constants, response type assertions
- Replace `ApiModelOutput` with `ApiAliasResponse` in assertions

**2h. Add tech debt note** to `crates/routes_app/TECHDEBT.md`:
```
## Models list query filter
GET /bodhi/v1/models returns all 3 types with no filter.
Add query parameter type=alias|api|model to filter by source type.
```

**Gate check:**
```bash
cargo check -p routes_app
cargo test -p routes_app
```

**Commit:** `refactor: restructure models routes into /models/{alias,api,files} sub-modules`

---

### Phase 3: Full Backend Verification & Downstream Fixes

**Sub-agent scope:** All Rust crates (receives Phase 1+2 summary)

**Tasks:**
1. Fix any compilation issues in downstream crates (`server_app`, `routes_all`, `lib_bodhiserver`)
2. Run full backend test suite

**Gate check:**
```bash
make test.backend
```

**Commit (if needed):** `fix: update downstream crates for models route restructure`

---

### Phase 4: OpenAPI + TypeScript Client Regeneration

**Sub-agent scope:** `xtask`, `ts-client/` (receives Phase 1-3 summary)

**Tasks:**
1. Regenerate OpenAPI spec: `cargo run --package xtask openapi`
2. Rebuild TypeScript client: `make build.ts-client`
3. Verify sync: `make ci.ts-client-check`

**Gate check:**
```bash
make ci.ts-client-check
```

**Commit:** `chore: regenerate OpenAPI spec and TypeScript client for models reorg`

---

### Phase 5: Frontend — File Moves & Import Updates

**Sub-agent scope:** `crates/bodhi/src/` (receives Phase 1-4 summary)

**Tasks:**

**5a. Move UI files:**
```
STAYS at models/:
  page.tsx, page.test.tsx, components/ModelPreviewModal.tsx

MOVE to models/components/:
  ModelTableRow.tsx, ModelActions.tsx, SourceBadge.tsx, tooltips.ts

MOVE to models/alias/:
  AliasForm.tsx → alias/components/AliasForm.tsx
  new/page.tsx (+test) → alias/new/page.tsx (+test)
  edit/page.tsx (+test) → alias/edit/page.tsx (+test)

MOVE to models/api/:
  api-models/new/page.tsx (+test) → api/new/page.tsx (+test)
  api-models/edit/page.tsx (+test) → api/edit/page.tsx (+test)
  components/api-models/* → api/components/*

MOVE to models/files/:
  modelfiles/page.tsx (+test) → files/page.tsx (+test)

MOVE to models/files/pull/:
  pull/page.tsx (+test) → files/pull/page.tsx (+test)
  pull/PullForm.tsx (+test) → files/pull/PullForm.tsx (+test)
```
- Delete empty: `app/ui/api-models/`, `app/ui/modelfiles/`, `app/ui/pull/`

**5b. Update navigation links** in `models/page.tsx`:
- `/ui/api-models/new` → `/ui/models/api/new`
- `/ui/api-models/edit?id=` → `/ui/models/api/edit?id=`
- `/ui/models/new` → `/ui/models/alias/new`
- `/ui/models/edit?id=` → `/ui/models/alias/edit?id=`

**5c. Rename hooks & update endpoints:**
- `hooks/useApiModels.ts` → `hooks/useModelsApi.ts` — update URLs, types (`ApiModelOutput` → `ApiAliasResponse`)
- `hooks/useModels.ts` → rename or split — alias CRUD → `/models/alias/`, modelfiles → `/models/files/`, pull → `/models/files/pull`
- Update query cache keys

**5d. Update schemas** (`schemas/apiModel.ts`): `ApiModelOutput` → `ApiAliasResponse`

**5e. Update navigation** (`hooks/use-navigation.tsx`): Minimal — single 'Models' at `/ui/models/`

**5f. Update setup wizard** (`app/ui/setup/api-models/page.tsx`): import path → `@/app/ui/models/api/components/ApiModelForm`

**5g. Update MSW handlers:**
- `test-utils/msw-v2/handlers/api-models.ts` — endpoint paths
- `test-utils/msw-v2/handlers/models.ts` — alias CRUD endpoints
- `test-utils/msw-v2/handlers/modelfiles.ts` — file/pull endpoints

**5h. Fix all import paths in moved files**

**Gate check:**
```bash
cd crates/bodhi && npm test
```

**Commit:** `refactor: reorganize models UI into /models/{alias,api,files} sub-sections`

---

### Phase 6: E2E Tests

**Sub-agent scope:** `crates/lib_bodhiserver_napi/` (receives Phase 1-5 summary)

**Prerequisite:** `make build.ui-rebuild`

**Tasks:**
1. Update page objects in `tests-js/pages/`
2. Update spec files referencing old endpoint paths or URLs
3. Update setup fixtures if they reference old endpoints

**Gate check:**
```bash
make build.ui-rebuild
make test.napi
```

**Commit:** `test: update E2E tests for models route reorganization`

---

## Deferred (follow-up PRs)

- **Polymorphic `GET /models/{id}`**: Return `AliasResponse` (any type) via `find_alias() → Option<Alias>`
- **Query filters on `GET /models/`**: Add `type=alias|api|model` parameter (noted in TECHDEBT.md)
