# Model-Router — Phase 1: Foundation & Pass-Through Routing

## Context

We are implementing the **model-router** feature: a new composite alias kind (`source = "model_router"`)
that fronts an **ordered list of targets** (each = an existing alias + a pinned model) and routes a chat
request through them. The guiding use case is freellmapi-style stacking of free vendor APIs to stretch
quotas before paying, with reliability via failover.

The feature is being built in 4 phases (see `ai-docs/07-research/model-router/phasewise-impl/`). **This plan
covers Phase 1 only.** Phase 1 makes the model-router a first-class alias: it persists, validates, lists, is
selectable in chat, and **forwards a chat request end-to-end to its first enabled target** — returning that
target's response verbatim. There is **no failover, no health tracking, no probe** in Phase 1; with a single
healthy target the behavior is indistinguishable from the finished feature.

Design source of truth: `ai-docs/07-research/model-router/bodhiapp-model-router-implementation-proposal.md`
(domain §1, storage §2, strategy §3, forward §4, resolution §5, errors §7, API §8, frontend §9). All of the
proposal's `file:line` references were verified against current code during planning and are accurate.

### Decisions locked in this planning session
1. **Thin strategy seam now (YAGNI).** Introduce the `RoutingStrategy` trait + a minimal `RouterContext`,
   with `FallbackConfig::execute` that forwards to the **first enabled target only**. Do **not** build
   `HealthRegistry`, failure classification, `order_by_health`, `AttemptOutcome` buffering, or the retry loop
   — those are Phase 2/3. Phase 2 grows the loop in place inside `execute`.
2. **Resilience config persisted with defaults; no UI knobs.** `FallbackConfig` is stored with defaults
   (`cooldown_secs=30`, `max_attempts=0`, `honor_retry_after=true`); the Phase 1 form shows only the strategy
   selector + targets. The knobs get UI in Phase 3 (whose brief says "config knobs in UI").
3. **Add `enabled: bool` (default true) to `RouterTarget`.** Phase 1 requires a per-target enable/disable
   toggle (deselect without delete); the original proposal struct lacked it.
4. **Shared `ApiFormat::supports_chat_completions()`** in `model_objs.rs`, used by the existing chat guard and
   the new router validation. The broader capability-method refactor of the other 5 surfaces is a noted,
   optional follow-up — out of Phase 1 scope.
5. **Keep `weight: Option<u32>` on `RouterTarget`** as a reserved seam (locked design decision; stored in the
   JSON blob, so free to use later with no migration). Ignored by Fallback.
6. **Serialization is `snake_case`**, never kebab. The new `source` tag is `model_router`; the strategy tag is
   `fallback`. (`AliasSource` already serializes `snake_case`; the `Alias` enum's container-level
   `rename_all = "kebab-case"` is a no-op because every variant overrides it with an explicit single-word
   rename — the new variant follows suit with an explicit `model_router` rename.)

---

## Implementation (layered upstream → downstream)

Follow BodhiApp's layered methodology (`crates/CLAUDE.md`) and **write the gating tests first** at each layer.
Use the `test-services` and `test-routes-app` skills for the backend test patterns.

### 1. `services` crate

**1a. Domain types — `crates/services/src/models/model_objs.rs`**
- Add `ModelRouter(ModelRouterAlias)` to the `Alias` enum (`:929`) with an explicit `#[serde(rename = "model_router")]`
  (matches the existing per-variant rename style — `"user"`/`"model"`/`"api"`).
- Add `ModelRouter` to `AliasSource` (`:592`); it inherits the enum's `snake_case` serde + strum, so it serializes
  as `model_router` automatically — no per-variant override needed. Both tags land on `model_router`.
- New structs:
  - `ModelRouterAlias { id, alias, targets: Vec<RouterTarget>, strategy: RoutingStrategyConfig, created_at, updated_at }`
  - `RouterTarget { alias: String, model: String, enabled: bool, weight: Option<u32> }` (`enabled` defaults true via serde default)
  - `RoutingStrategyConfig` — `#[serde(tag = "strategy", rename_all = "snake_case")]` with one variant `Fallback(FallbackConfig)` (tag value `fallback`)
  - `FallbackConfig { cooldown_secs: u32, max_attempts: u32, honor_retry_after: bool }` + `impl Default` (30 / 0 / true)
- Extend `Alias` helper methods (`:943-970`): `alias_name()` → `router.alias`; `source()` → `ModelRouter`;
  `can_serve(model)` → `model == router.alias` (exact match, no prefix).
- Add `ModelRouter(ModelRouterResponse)` to the **untagged** `AliasResponse` enum (`:1938`). Place it so its
  unique `targets`/`strategy` fields disambiguate untagged deserialization (put it first).
- Request/response types: `ModelRouterRequest { alias, targets: Vec<RouterTargetRequest>, strategy }`
  (`#[derive(Validate, ToSchema)]`), `RouterTargetRequest { alias, model, enabled, weight }`,
  `ModelRouterResponse { id, alias, source, targets, strategy, created_at, updated_at }` with `From<ModelRouterAlias>`.

**1b. Shared format helper — `model_objs.rs`**
- `impl ApiFormat { pub fn supports_chat_completions(&self) -> bool { matches!(self, OpenAI | Anthropic | AnthropicOAuth) } }`

**1c. Entity + migration**
- `crates/services/src/models/model_router_entity.rs`, table `model_router_aliases`: columns `id` (PK ULID),
  `tenant_id`, `user_id`, `alias`, `targets` (JsonBinary), `strategy` (JsonBinary), `created_at`, `updated_at`.
  **No encryption columns, no health columns.** Mirror `api_model_alias_entity.rs`; timestamps via `TimeService`.
- Migration `crates/services/src/db/sea_migrations/m20250101_000022_model_router_aliases.rs` (next number after
  `_000021`, confirmed current highest). Partial unique index on `(tenant_id, user_id, alias)`; Postgres
  `ENABLE/FORCE ROW LEVEL SECURITY` + `tenant_isolation` policy using `current_tenant_id()`. Register in
  `sea_migrations/mod.rs`.

**1d. Repository — `crates/services/src/models/model_router_repository.rs`**
- `trait ModelRouterRepository` impl'd on `DefaultDbService`, mirroring `ApiAliasRepository`:
  `create`, `get`, `update`, `delete`, `list`, `check_alias_exists`. All mutations via `with_tenant_txn`.

**1e. Strategy seam — `crates/services/src/models/router/` (new module)**
- `RoutingStrategy` trait: `async fn execute(&self, ctx: &mut RouterContext) -> Result<Response, ModelRouterError>`
  + `fn validate(&self, targets: &[RouterTarget]) -> Result<(), ObjValidationError>`.
- `RouterContext` (minimal): holds `targets`, `request`, and the resolution/forward primitives it needs
  (`find_alias`, api-key resolver, `ai_api.for_alias`). **No `HealthRegistry` field in Phase 1.**
- `forward_one(target)` (Phase 1 = single, simple path): resolve inner alias (skip/error if missing or a
  router), apply `supports_chat_completions()` guard, resolve api key, clone request and set
  `request["model"] = target.model`, `for_alias(...).forward_request_with_method(POST, "/chat/completions", ...)`,
  return the axum `Response` **verbatim**.
- `impl RoutingStrategy for FallbackConfig::execute`: iterate `targets` in declared order, pick the **first
  `enabled` target**, call `forward_one`, attach observability headers, return. If **no enabled target** →
  `ModelRouterError::EmptyChain`. If the chosen target's referenced alias is **deleted/dangling** → typed error
  (Phase 1 surfaces it; Phase 2 introduces fall-through). `weight`/cooldown fields ignored this phase.
- `RoutingStrategyConfig::behavior()` → `match` to `&dyn RoutingStrategy` (one arm).

**1f. Errors — `crates/services/src/models/model_router_error.rs`**
- `#[derive(ErrorMeta)]` enum (mind the `errmeta` gotchas — Display, error codes): `EmptyChain` (BadRequest),
  `ReferencedAliasNotFound { alias }` (NotFound), `NestedRouterNotAllowed { alias }`, `SelfReference`,
  `InvalidPinnedModel { alias, model }`, `TargetFormatUnsupported { alias, api_format }` (all BadRequest).
  Upstream failures are **never wrapped** (verbatim).

**1g. Service — `crates/services/src/models/model_router_service.rs`**
- `ModelRouterService` trait (create/update/delete/get) mirroring `ApiModelService`. Validation:
  - alias name non-empty + **unique across all alias kinds** (DB unique index + pre-check via `find_alias`/`check_alias_exists`).
  - each `target.alias` resolves to a user/model/api alias (`ReferencedAliasNotFound`), is not a router
    (`NestedRouterNotAllowed`), is not self (`SelfReference`).
  - each pinned model accepted by the referenced alias via the existing primitive **`inner_alias.can_serve(&target.model)`**
    (handles local = name match, API forward_all = prefix, API selected = `matchable_models`) → else `InvalidPinnedModel`.
  - referenced API alias `api_format.supports_chat_completions()` → else `TargetFormatUnsupported`.
  - **zero / all-disabled targets is allowed to save** (validation must not block it; empty active set errors at request time).

**1h. Resolution — `crates/services/src/models/data_service.rs`**
- `list_aliases` (`:91`): also load `model_router_aliases` into the aggregate list.
- `find_alias` (`:137`): add model-router lookup; priority `user → model-router → model → api(prefix)`.

Run `cargo test -p services` (capture output to file per the slow-command convention).

### 2. `routes_app` crate
- CRUD handlers under `crates/routes_app/src/models/router/` mirroring `models/api/routes_api_models.rs`
  (session-only, `ResourceRole::User`):
  `POST /bodhi/v1/models/router`, `GET/PUT/DELETE /bodhi/v1/models/router/{id}`. (Router `test` endpoint is Phase 4.)
- Endpoint constants via `make_ui_endpoint!` in `shared/openapi.rs`; register routes in `routes.rs`
  (`user_session_apis` group); register paths/op_ids + tag in the OpenAPI derive.
- `AuthScope::model_routers()` accessor (mirror `api_models()` in `auth_scoped.rs`).
- **Chat hook — `crates/routes_app/src/oai/routes_oai_chat.rs`** (after `find_alias` `:138`, before the existing
  single-alias path): `if let Alias::ModelRouter(router) = alias { build RouterContext; return router.strategy.behavior().execute(...).await.map_err(OaiApiError::from); }`.
- Replace the inline chat guard (`:146`) with `api_alias.api_format.supports_chat_completions()` (shared helper).
- Run `cargo test -p routes_app` incl. OpenAPI snapshot (`cargo test -p routes_app -- openapi`).

### 3. `server_app` — live integration test (multi-turn, `#[serial_test::serial(live)]`)
- Chat to a router whose first enabled target points at a stubbed upstream returning success → client gets the
  stubbed response + observability headers report that target, strategy `fallback`, attempts `1`.

### 4. `make test.backend`

### 5. Regenerate types
- `cargo run --package xtask openapi && make build.ts-client`

### 6. Frontend — `crates/bodhi/src/`
- Hook `hooks/models/useModelRouters.ts` (`useGetModelRouter`/`useCreateModelRouter`/`useUpdateModelRouter`/
  `useDeleteModelRouter`) + endpoint constants & query keys in `hooks/models/constants.ts` (mirror `useModelsApi.ts`).
- Routes `routes/models/router/new/index.tsx`, `routes/models/router/edit/index.tsx`, shared
  `routes/models/router/-components/ModelRouterForm.tsx` (mirror `components/api-models/ApiModelForm.tsx`).
- `ModelRouterForm`: strategy selector (only **Fallback** enabled), ordered reorderable target list; per target
  an enable/disable toggle and a model picker that adapts to the referenced alias shape (local → fixed; API
  `forward_all_with_prefix` → free-text; API selected → dropdown of that alias's `models`; routers excluded from
  the alias dropdown). **No resilience-config fields** this phase.
- Aggregate list (`routes/models/index.tsx`) and chat picker (`routes/chat/-components/settings/AliasSelector.tsx`)
  pick up routers automatically once resolution returns them; verify the `AliasSelector` flat-map handles a
  router entry (single entry keyed by router alias). Import types from `@bodhiapp/ts-client`.
- `cd crates/bodhi && npm run test`.

### 7. E2E — `crates/lib_bodhiserver/tests-js/` (black-box, UI-only)
- Create a router via the UI with one enabled target, select it in chat, send a message, receive a completion;
  add a second target + disable the first, send again (served by second); disable both, send (clear error).
  Assert through the UI only — no `page.evaluate`/direct fetch. Throw in `beforeAll` if required env is missing
  (no `test.skip`). Run `make build.dev-server` then `make test.e2e` from `lib_bodhiserver/tests-js`.

### 8. Docs
- Update `crates/services/CLAUDE.md`, `crates/routes_app/CLAUDE.md`, `crates/bodhi/src/CLAUDE.md` for the new module.

---

## Observability headers (added by the strategy to the returned response)
- `x-bodhi-routed-alias: <referenced alias name>`
- `x-bodhi-routed-model: <pinned model>`
- `x-bodhi-router-strategy: fallback`
- `x-bodhi-router-attempts: 1` (always 1 this phase)

---

## Acceptance gates (test-first — phase done when these pass, nothing regresses)

**services (unit, `#[rstest]`, sqlite+postgres via `#[values]`):**
- `ModelRouterRepository` CRUD + tenant isolation (pattern: `test_api_alias_repository_isolation.rs`).
- Validation: duplicate alias name (any kind) rejected; missing-ref / nested-router / self-ref rejected with
  the matching typed error; invalid pinned model rejected; chat-unsupported format rejected; **zero/all-disabled
  targets succeed**.
- `FallbackConfig::execute` (mocked clients): success on first enabled target; first disabled + second enabled →
  served by second; all disabled → `EmptyChain` typed error; upstream error returned verbatim (no fall-through).
- A created router appears in `list_aliases` and resolves by name via `find_alias`.

**routes_app (`tower::oneshot`):** CRUD happy/validation paths; OpenAPI snapshot updated.

**server_app (live):** chat to a router → served by first enabled target + obs headers correct.

**frontend (component, vitest + MSW):** form validates required fields, add/reorder/remove targets, per-target
enable/disable, model picker adapts to referenced alias type.

**E2E (Playwright, black-box):** create → select in chat → completion; disabled-skip; all-disabled error.

---

## Verification
- `cargo test -p services` → `cargo test -p routes_app` → `make test.backend` (tee output to a file).
- `cargo run --package xtask openapi && make build.ts-client`; `cd crates/bodhi && npm run test`.
- `make build.dev-server` then `make test.e2e` (from `crates/lib_bodhiserver/tests-js`).
- Manual: `make app.run.live`, create `my-stack` router with one working target, confirm it shows in the model
  list + chat picker, send a chat and get a reply; add a 2nd target, disable the 1st, send (served by 2nd);
  disable both, send (clear error).
