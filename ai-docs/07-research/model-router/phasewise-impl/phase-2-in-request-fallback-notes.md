# Phase 2 — Implementation Notes (handoff to Phase 3+)

> Written after Phase 2 landed (commit `feat(model-router): in-request fallback (Phase 2)`).
> Purpose: record what was actually built vs. the spec, the concrete seams the next
> phases must touch, known gaps, and callouts that will shape Phase 3/4 plans.
> **The code is the source of truth** — verify every path below before relying on it.

## 1. What Phase 2 actually built (deltas vs. spec)

- **Classification lives in its own module.** `crates/services/src/models/router/classify.rs` exposes
  `Disposition { Success, Retryable, Terminal }` and `classify_status(StatusCode) -> Disposition`.
  Kept separate from `fallback.rs` so future strategies reuse it (the proposal's "shared primitives").
  The items are `pub` inside a private `mod classify;` (reachable in-crate via `super::classify::`, not re-exported).
- **Classification is status-only and encoded as a complement.** The rule is literally
  *"2xx = Success, 400/422 = Terminal, everything else = Retryable."* This means `401/403/404/408/429/5xx`,
  unknown 4xx, and unknown 5xx are all Retryable by construction. Content-policy / context-window are
  treated Terminal **only because providers surface them as 400/422** — there is no body inspection.
  If a provider ever returns context-window as a non-400/422 status, that's a future refinement, not a bug today.
- **No body buffering.** A retryable response's body is never read; we hold the whole `Response` object
  (status + lazy body stream) and either drop it (a later attempt commits) or return it verbatim on exhaustion.
  This preserves the "decide before first byte" and opaque-proxy conventions.
- **Exhaustion returns the last upstream `Response` verbatim.** The loop tracks
  `last_resp: Option<(&RouterTarget, Response)>` and `last_err: Option<ModelRouterError>`. On exhaustion:
  `last_resp` wins if present (real upstream status + body, served-target header reports its producer),
  else the typed `last_err` surfaces, else `EmptyChain` (unreachable since `cap >= 1`). A real upstream
  response is always preferred over a transport/structural error when both occurred.
- **`max_attempts` is honored now.** `cap = max_attempts == 0 ? enabled.len() : min(max_attempts, enabled.len())`.
  Disabled targets are filtered out *before* the loop, so they are never counted as attempts.
- **Observability headers semantics:** `x-bodhi-routed-alias` / `-routed-model` report the **target that
  actually produced the returned response**; `x-bodhi-router-attempts` reports the **total number of forwards
  made** (so a success on the 3rd enabled target reports attempts=3). `x-bodhi-router-strategy` = `"fallback"`.
  Helper: `with_obs_headers(resp, target, strategy, attempts)` in `strategy.rs`.

Key files: `classify.rs` (new), `fallback.rs` (the loop), `strategy.rs` (`RouterContext`, `forward_one`,
`route_chat_completion`, `with_obs_headers`), tests in `test_fallback_strategy.rs`,
`crates/server_app/tests/test_live_model_router.rs`, and the E2E in
`crates/lib_bodhiserver/tests-js/specs/models/model-router.spec.mjs`.

## 2. Concrete hook points Phase 3 must touch

- **`RouterContext` has no clock and no health state yet.** Today its fields are: `tenant_id`, `user_id`,
  `request`, `query_params`, `data_service`, `db_service`, `ai_api` (`strategy.rs`). Phase 3's cooldown logic
  needs a `TimeService` (never `Utc::now()`) and the shared health registry reachable from inside
  `RoutingStrategy::execute` — `ctx` is the natural carrier. Both must be threaded in at the construction
  site: `crates/routes_app/src/oai/routes_oai_chat.rs` (where `services::RouterContext { … }` is built from
  `auth_scope` accessors).
- **`FallbackConfig.cooldown_secs` (default 30) and `honor_retry_after` (default true) are persisted but
  UNUSED.** Phase 3 activates them. They already round-trip through the DB (`strategy` JSON column) and the
  request/response types — no migration or API-shape change needed to start *reading* them.
- **The strategy gets per-request config via `&self` but no shared state.** `impl RoutingStrategy for
  FallbackConfig` — `self` is the config, `ctx` is the per-request carrier. Shared, cross-request health must
  ride on `ctx` (which holds `Arc`s), not on `self`.
- **`Retry-After` is already reachable.** `forward_to_upstream` (`provider_shared.rs`) copies *all* upstream
  headers onto the returned axum `Response`, so the strategy can read `resp.headers().get("retry-after")`
  before deciding the cooldown. No forwarder change required.
- **Health key components are already in hand.** `target.alias` is the referenced-alias identity
  (`alias_name()`: id for api, name for local) and `target.model` is the pinned model. The proposal keys
  health by `format!("{tenant_id}:{referenced_alias_id}:{model}")` — all three are available without
  re-resolving the alias (tenant from `ctx.tenant_id`). Note `forward_one` still resolves the inner alias for
  forwarding; if Phase 3 wants to key on the *resolved* id rather than `target.alias`, decide whether to key
  before or after resolution (they're the same string for api/local today, but resolution also catches
  dangling/nested — see gaps).
- **Frontend strategy config is hardcoded.** `crates/bodhi/src/routes/models/router/-components/ModelRouterForm.tsx`
  builds the request body with `strategy: { strategy: 'fallback', cooldown_secs: 30, max_attempts: 0,
  honor_retry_after: true }` — literal constants, no form controls. Phase 3 adds the three knobs to the form
  with defaults + validation; the request/response types and OpenAPI already carry the fields, so the TS
  client already knows them.
- **AppService wiring pattern for a new shared registry.** Precedents: `MokaCacheService` (process-global
  in-memory KV behind `CacheService`) and the `REFRESH_LOCKS` static `Lazy<Mutex<HashMap<…>>>` in
  `ai_apis/llm_liberty/refresh.rs`. Adding a `HealthRegistry` accessor touches the `AppService` trait +
  `DefaultAppService` struct/impl (`app_service/app_service.rs`) and the builder
  (`lib_bodhiserver/src/app_service_builder.rs`). `TimeService` is already an `AppService` accessor and is
  injected into services via constructor (`time_service.clone()`); mirror that.

## 3. Known gaps / tech-debt from Phase 2

- **`max_attempts` is unit-tested only.** There is no live `server_app` or E2E test exercising the cap. If
  Phase 3 surfaces `max_attempts` in the UI, add at least one integration assertion that a UI-set cap takes
  effect end-to-end.
- **Retryable set is a complement, not an allowlist.** Any unmapped status (e.g. a bizarre `418`, or a `5xx`
  we didn't enumerate) is Retryable. This is intentional for free-tier stacking but is worth a one-line note
  if Phase 4's probe wants finer-grained categories (unauthorized vs rate-limited vs unreachable) — the probe
  will likely need richer classification than the binary status-only `Disposition`.
- **Served-target vs. attempt-count headers can look surprising.** On exhaustion the `routed-alias` header
  names the last target that returned a *response*, which may not be the last target *attempted* (a trailing
  transport error doesn't overwrite an earlier retryable response). Documented here so Phase 4's per-target
  report doesn't assume "attempts == index of routed-alias".
- **Structural skips are folded into the same fall-through path as transport errors.** Dangling reference,
  nested router, and unsupported format all return `Err` from `forward_one` and are treated like a retryable
  transport failure (skip to next). Phase 3 must decide whether a *structural* skip should also be cooled
  down (probably **not** — a dangling alias isn't a transient quota issue; cooling it would just delay the
  inevitable). Recommendation: only cool **upstream retryable responses + genuine transport errors**, not
  structural errors. Phase 4's validate-vs-probe split makes this distinction explicit.

## 4. Callouts that will shape future plans

- **Health should be keyed by the underlying target, not the router.** Confirmed by the proposal: keying by
  `(tenant, referenced_alias_id, model)` is what lets two routers sharing a free provider share its cooldown.
  Keep `tenant_id` in the key for isolation. Phase 3's "shared health" and "never starve" gates depend on this.
- **`order_by_health` is a free function, not strategy logic.** The proposal puts target ordering
  (non-cooled in declared order first, cooled last by soonest expiry) as a shared free function so every
  future strategy reuses it. Phase 3 should add it next to the registry, and `FallbackConfig::execute` should
  iterate the *ordered* enabled set instead of raw declared order. The Phase-2 loop currently iterates raw
  declared order — that's the single behavioral change point.
- **Never-starve interacts with the Phase-2 loop directly.** When every enabled target is cooled,
  `order_by_health` must still yield them all (soonest-recovery first) so the existing loop attempts them and
  returns a real upstream result. No new "all cooled" error path — `EmptyChain` stays reserved for the
  truly-empty enabled set.
- **Half-open is implicit in the loop.** There is no probe/timer. A cooled target becoming eligible just means
  `order_by_health` stops filtering it; the next real request that reaches it *is* the trial. Success →
  `record_success` clears it; retryable → `cooldown` re-cools it. The Phase-2 commit/skip branches are exactly
  where `record_success` / `cooldown(key, cooldown_for(headers, cfg))` should be called.
- **`cooldown_for` is where `honor_retry_after` lives.** `if honor_retry_after && Retry-After present →
  max(cooldown_secs, retry_after) else cooldown_secs`. Reads the header off the held `Response`.
- **Reserved seam for stateful strategies is untouched and fine.** `RouterTarget.weight` and any per-router
  `SelectionState` remain unbuilt. Phase 3 adds only the shared `HealthRegistry`; do not introduce per-router
  mutable selection state (that's a later round-robin/weighted concern).
- **Tests use a controllable clock.** Phase 3's cooldown/recovery gates need `FrozenTimeService` (advance the
  clock to expire a cooldown). `RouterContext` getting a `TimeService` makes the strategy unit-testable with a
  frozen clock the same way the rest of `services` is.
