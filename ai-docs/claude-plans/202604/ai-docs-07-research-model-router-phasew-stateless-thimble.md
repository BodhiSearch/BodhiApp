# Phase 3 — Health-Aware Skipping & Automatic Recovery (model-router)

> Status: PLAN READY — decisions settled (see Decisions section). Awaiting approval to start.

## Context

The model-router (`Alias::ModelRouter`) fronts an ordered list of targets and, since Phase 2,
performs in-request fall-through: it tries enabled targets in declared order, returns the first
Success/Terminal verbatim, and on exhaustion returns the last upstream response. **But it
re-evaluates the whole chain from the top on every request** — a down or quota-exhausted primary
is retried (and fails) first on *every* request before falling through. There is no cross-request
memory.

Phase 3 adds **in-memory, per-process health memory**: a failed target is put in a **cooldown**
window and **skipped during selection** on later requests; when the window elapses the next real
request that would select it is a **half-open trial** (no background probes); success clears its
health (return-to-primary), failure re-cools it. Health is keyed by the **underlying target**
`(tenant, target.alias = alias_name() identity, model)` so two routers sharing a free provider share its cooldown.
The three resilience knobs (`cooldown_secs`, `honor_retry_after`, `max_attempts`) — persisted since
Phase 1, with `max_attempts` already honored — get surfaced in the router create/edit UI.

## Current behaviour (verified against code)

- **Strategy loop**: `crates/services/src/models/router/fallback.rs:19-61` iterates
  `targets.iter().filter(|t| t.enabled)` in **raw declared order**, classifies via
  `classify_status` (`classify.rs:20-26`: 2xx=Success, 400/422=Terminal, else Retryable), returns
  first Success/Terminal verbatim, holds last Retryable `Response` and last `Err`, and on exhaustion
  returns last response (preferred) else last error else `EmptyChain`. `max_attempts` caps the loop.
- **Forward primitive**: `RouterContext::forward_one` (`strategy.rs:34-91`) resolves the inner alias
  by `alias_name()` via `list_aliases`, rejects nested routers / unsupported formats / dangling refs
  with structural `ModelRouterError`s, pins `req["model"] = target.model`, forwards. Upstream
  `Response` carries all headers (so `resp.headers().get("retry-after")` is reachable).
- **RouterContext** (`strategy.rs:19-27`): `tenant_id, user_id, request, query_params, data_service,
  db_service, ai_api` — **no clock, no health**. Built in `routes_app/src/oai/routes_oai_chat.rs:157-165`.
- **Config** (`model_objs.rs:1084-1106`): `FallbackConfig { cooldown_secs(30), max_attempts(0),
  honor_retry_after(true) }`, serde-tagged `RoutingStrategyConfig::Fallback`, JSON-stored. Round-trips
  through DB + API + TS client already. `cooldown_secs`/`honor_retry_after` are **persisted-but-unused**.
- **Time house style**: `TimeService` trait (`db/time_service.rs`: `utc_now()`), real
  `DefaultTimeService`, test `FrozenTimeService` (`test_utils/db.rs`, defaults 2025-01-01). Exposed on
  `AppService::time_service()`, injected into services via `derive_new` constructor `time_service.clone()`.
- **Process-global in-memory state house style**: `MokaCacheService` behind `CacheService` on
  `AppService` (`utils/cache_service.rs`), and `REFRESH_LOCKS: Lazy<Mutex<HashMap<…>>>` in
  `ai_apis/llm_liberty/refresh.rs`. AppService accessor pattern: trait method + `Arc<dyn>` struct field +
  clone impl + builder `get_or_build_*` (`lib_bodhiserver/src/app_service_builder.rs`).
- **Frontend**: `ModelRouterForm.tsx:109` hardcodes
  `strategy: { strategy:'fallback', cooldown_secs:30, max_attempts:0, honor_retry_after:true }`. Plain
  `useState` form (no react-hook-form/zod), shadcn components, strategy `<Select>` disabled (only Fallback).
- **Served-target indicator**: the chat UI does **NOT** surface which provider served a response
  (`grep` found no use of `x-bodhi-routed-alias` in `crates/bodhi/src`). The E2E spec only asserts a
  non-empty reply. **Gap** for the Phase-3 E2E acceptance gate — resolved as a `data-test*` attribute
  (Decision 1).

### Drift between design docs and code (callouts)

- Proposal §6 shows `forward_one` keying health by the *resolved* inner alias id (`inner_alias_id`),
  but `forward_one` currently **does not return** the resolved alias — it returns only `Response`.
  Resolved (Decision 2): `RouterTarget.alias` already **is** the `alias_name()` identity (`api.id` for
  api, name for local — `model_objs.rs:970`), and resolution matches on exactly that. So
  `(tenant, target.alias, target.model)` is the underlying-target key with no `forward_one` refactor.
- Proposal §6 sketches a `HealthRegistry` struct with `RwLock<HashMap>` directly; the **house style**
  is an `AppService` accessor returning `Arc<dyn Trait>` (mockable). Plan follows house style.
- Phase-2 notes are otherwise accurate to the code.

## Decisions (settled with user)

1. **Served-target observability** → the chat uses pi-ai `streamSimple` (`agentStore.ts:77`), which
   owns the fetch and does **not** expose the `x-bodhi-routed-alias` HTTP *response header* to our code
   — only the parsed body (content, usage, upstream `model`). So we surface the **served model** as the
   signal: the router already pins `req["model"] = target.model` and the upstream echoes it back, so the
   response body's `model` differs per target. Map that upstream `model` into `MessageMetadata.model`
   (currently dropped in `agentMessageToLegacy`, `ChatUI.tsx:208-216`) and render it as a **`data-test*`
   attribute** on the assistant message (non-visible). The Playwright E2E reads that attribute — still
   black-box (a UI-rendered attribute, no server internals). E2E uses two targets with distinguishable
   pinned models so the attribute value identifies which target served. No product-visible badge.

2. **Health key** → `(tenant_id, target.alias, target.model)`. Rationale: `Alias::alias_name()`
   (`model_objs.rs:970`) returns `api.id` for Api aliases and the `.alias` name for User/Model aliases;
   `RouterTarget.alias` is stored as exactly this `alias_name()` identity (Phase 1), and `forward_one`
   resolves targets via `.find(|a| a.alias_name() == target.alias)`. So `target.alias` **is** the
   resolved underlying-target identity — it cannot diverge from the resolved inner alias id. Keying on
   it needs **no `forward_one` signature change** and still gives shared-cooldown across routers + tenant
   isolation. (If a future alias kind makes `alias_name()` non-identity, revisit — noted for Phase 4.)

3. **Controllable clock in the live test** → add a settable **`TestTimeService`** to
   `services::test_utils` (interior-mutable `Arc<RwLock<DateTime<Utc>>>` with `set(instant)` /
   `advance(Duration)` plus `utc_now()`), and inject it into the `server_app` live harness so cooldown
   expiry is advanced deterministically — no sleeps. Today only `FrozenTimeService` (immutable) and
   mockall's `MockTimeService` (return-sequence) exist; neither supports advance-in-place cleanly. Unit
   tests can use `FrozenTimeService` per-instant or the new `TestTimeService`.

4. **UI knob defaults/limits** → `cooldown_secs=30`, `max_attempts=0` (= try whole chain), 
   `honor_retry_after=on` (mirror persisted defaults). Validation: non-negative integers; soft upper
   guard on `cooldown_secs`; `max_attempts=0` labelled as "try all targets". Prefill from response on edit.

## Proposed change (upstream → downstream)

### 1. services — HealthRegistry (new shared primitive)
- New module `crates/services/src/models/router/health.rs`:
  - `TargetKey(String)` = `format!("{tenant_id}:{alias}:{model}")`.
  - `EndpointHealth { cooldown_until: Option<DateTime<Utc>> }` (binary cooled/not; consecutive-fail
    tracking is out of scope per spec — no penalty scoring).
  - `trait HealthRegistry: Send + Sync + Debug` with `is_cooled(&self, key, now) -> bool`,
    `cooldown(&self, key, until: DateTime<Utc>)`, `record_success(&self, key)`. `#[mockall::automock]`.
  - `DefaultHealthRegistry { inner: RwLock<HashMap<TargetKey, EndpointHealth>> }`.
  - Free fn `order_by_health(targets: &[&RouterTarget], reg, tenant, now) -> Vec<&RouterTarget>`:
    enabled non-cooled in declared order first, then cooled by soonest `cooldown_until`. (Enabled-only
    filter stays in the strategy; never-starve = all-cooled still yields all.)
  - `cooldown_for(headers, cfg, now) -> DateTime<Utc>`: `base = cfg.cooldown_secs`; if
    `cfg.honor_retry_after` and a parseable `Retry-After` (delta-seconds; HTTP-date best-effort),
    `secs = max(cooldown_secs, retry_after)`; `now + secs`.
- Wire onto `AppService`: accessor `health_registry() -> Arc<dyn HealthRegistry>`, struct field, impl,
  builder `get_or_build_health_registry()` defaulting to `Arc::new(DefaultHealthRegistry::default())`
  (`app_service/app_service.rs`, `lib_bodhiserver/src/app_service_builder.rs`). Add to `AppServiceStub`
  (`test_utils/app.rs`) with a default.

### 2. services — RouterContext gains clock + health + tenant-keyed selection
- Add fields to `RouterContext` (`strategy.rs:19-27`): `time_service: Arc<dyn TimeService>`,
  `health: Arc<dyn HealthRegistry>`.
- `FallbackConfig::execute` (`fallback.rs`):
  - Filter enabled (unchanged) → `EmptyChain` if empty.
  - `let now = ctx.time_service.utc_now();`
  - Order via `order_by_health(&enabled, &ctx.health, &ctx.tenant_id, now)` **before** applying `cap`.
  - In the loop, on `Disposition::Success` → `ctx.health.record_success(&key)`; on `Terminal` → leave
    health untouched; on `Retryable` → `ctx.health.cooldown(&key, cooldown_for(resp.headers(), self, now))`;
    on structural `Err` (dangling/nested/unsupported) → **do NOT cool** (skip only). Key built from
    `(ctx.tenant_id, target.alias, target.model)`.
  - Exhaustion behaviour unchanged (last response verbatim; structural errors never cooled).

### 3. routes_app — thread services into RouterContext
- `routes_oai_chat.rs:157-165`: add `time_service` and `health` to the `RouterContext { … }`
  construction. Confirm the accessor path during impl: `time_service` is on `AppService`
  (`auth_scope.time_service()` or the underlying app-service handle); `health_registry()` is the new
  `AppService` accessor (§1). The served-model header (`x-bodhi-routed-model`) is already attached by
  `with_obs_headers` — no routes change needed for observability beyond what Phase 2 emits.

### 4. Frontend — surface the three knobs + served-model attribute
- `ModelRouterForm.tsx`: replace the hardcoded strategy object (`:109`) with controlled state for
  `cooldown_secs` (number, default 30, min ≥ 0), `max_attempts` (number, default 0 = whole chain),
  `honor_retry_after` (Switch, default true). Render under the (disabled) strategy `<Select>`.
  Validation: non-negative integers; soft upper guard on cooldown; `max_attempts=0` labelled "try all
  targets". Prefill from `ModelRouterResponse.strategy` on edit. Types already exist in
  `@bodhiapp/ts-client` (no OpenAPI change unless field docs/limits change).
- Served-model signal: in `agentMessageToLegacy` (`ChatUI.tsx:204-217`) set
  `metadata.model = assistantMsg.model` (pi `AssistantMessage` carries the upstream model). Add `model?`
  to `MessageMetadata` if not already present (`types/chat.ts:10`). In `ChatMessage.tsx`, render the
  served model as a non-visible `data-testid="served-model"` (or `data-served-model={metadata.model}`)
  on the assistant message block. No visible UI change.

## Test plan (mapped to acceptance gates, controllable clock everywhere)

### New test util — settable `TestTimeService` (`services/src/test_utils`)
Add `TestTimeService { inner: Arc<RwLock<DateTime<Utc>>> }` with `new(instant)` / `set(instant)` /
`advance(chrono::Duration)` and `impl TimeService` (`utc_now()` reads the cell). Used by both the unit
and live tests to expire cooldowns deterministically (no sleeps). `FrozenTimeService` (immutable) and
mockall `MockTimeService` (return-sequence) remain for cases that don't need advance-in-place.

### services unit — `test_fallback_strategy.rs` (+ `test_health.rs`) with controllable clock
Inject `TestTimeService` (or `FrozenTimeService` per instant) + a real `DefaultHealthRegistry` into the
test `RouterContext`. Advance via `time.advance(...)` to cross a cooldown boundary.
- Retryable on a target → target cooled → next request skips it, selects next eligible. (gate: cooldown)
- After window elapses (advance clock) → next request **tries** it (half-open); on success health
  clears and it's selected **first** on the following request. (gate: recovery / return-to-primary)
- Half-open trial that fails → re-cooled. (gate)
- `Retry-After` header → cooldown extended to `max(cooldown_secs, retry_after)`. (gate)
- **Shared health**: two routers referencing the same `(tenant, alias, model)` both skip after either
  observes a failure (same registry, same key). (gate)
- **Never-starve**: all enabled cooled → request still attempts (ordered soonest-recovery) and returns a
  real upstream result; no synthetic error. (gate)
- **Disabled wins**: disabled target never selected even when all enabled are cooled. (gate)
- **Structural skip not cooled**: dangling/nested/unsupported skipped this request, eligibility next
  request unchanged. (gate)
- `order_by_health` unit tests: non-cooled declared-order first, cooled soonest-expiry last, all-cooled
  yields all.

### server_app integration — `test_live_model_router.rs` (mockito, `#[serial_test::serial(live)]`)
- Inject the settable `TestTimeService` into the live test server harness (a test-only seam to supply
  the `TimeService` Arc at server build, sharing the same Arc the test holds so it can `advance()`).
  No sleeps.
- Primary mock 503 then 200; second request **does not** re-hit primary (cooldown — assert primary mock
  call count stays 1) → secondary serves; `time.advance(cooldown+1s)` → next request hits primary again
  (recovered). Assert via `x-bodhi-routed-alias` + `x-bodhi-routed-model` + mock call counts.
- Config takes effect: a router created via the API with `max_attempts`/`cooldown_secs` behaves
  accordingly end-to-end (covers the Phase-2 "max_attempts unit-only" gap).

### frontend component — `ModelRouterForm` (Vitest + RTL, MSW)
- The three controls render with defaults; editing updates the submitted request body; validation
  rejects negative numbers; edit-mode prefills from response.

### E2E — `model-router.spec.mjs` (Playwright, UI-only)
- Observe the served target via the `data-test*` served-model attribute on the assistant message
  (Decision via interview): use a primary and secondary with **distinguishable pinned models** so the
  attribute value names which target served. Black-box (UI attribute only).
- Flow: router primary→secondary, break primary; repeated sends served by secondary (primary skipped —
  cooldown means it is NOT re-tried each time, observable as fast, consistent secondary replies). Restore
  primary; before cooldown expiry it's still skipped; after expiry a later send is served by primary
  again. **Note:** the E2E can't advance the server clock (black-box), so it must use a **short
  `cooldown_secs`** set via the router form and a real (bounded) wait for expiry — the only place a real
  wait is acceptable, because a fake clock isn't reachable from a black-box test. Deterministic
  cooldown/recovery timing is fully covered by the unit + live-integration layers with the controllable
  clock; the E2E proves the user-visible behaviour with a short real window.

## Deferred to Phase 4 / future-spec updates
- Router **test/probe** capability (explicitly Phase 4) — structural problems get surfaced there; do
  not build here.
- DB-persisted / cross-replica health; penalty-score/latency ranking; background probes — all out of scope.
- If health keying ever needs the *resolved* inner alias id (vs `target.alias`), revisit `forward_one`
  to return the resolved id — note for Phase 4 specs.

## Verification
- `cargo test -p services --lib` (health + fallback units), then `cargo test -p server_app` (live),
  then `make test.backend`.
- `cargo run --package xtask openapi && make build.ts-client` if any API field metadata changes.
- `cd crates/bodhi && npm run test` for the form component.
- `make build.dev-server` + `make test.e2e` (from `lib_bodhiserver/tests-js`) for the Playwright gate.
