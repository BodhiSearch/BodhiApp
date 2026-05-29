# Phase 3 — Implementation Notes (handoff to Phase 4+)

> Written after Phase 3 landed. Records what was actually built vs. the brief, the seams
> Phase 4 plugs into, and callouts. **The code is the source of truth** — verify before relying.

## 1. What Phase 3 built (deltas vs. spec)

- **`HealthRegistry` is an in-memory `AppService` accessor**, not a DB service.
  `crates/services/src/models/router/health.rs`: `trait HealthRegistry`
  (`is_cooled`/`cooled_until`/`cooldown`/`record_success`, `#[automock]`) +
  `DefaultHealthRegistry { RwLock<HashMap<String, EndpointHealth>> }`. `EndpointHealth` holds
  only `cooldown_until: Option<DateTime<Utc>>` — binary cooled/not, no penalty scoring (per spec).
  Wired onto `AppService` (`app_service/app_service.rs`), the builder
  (`lib_bodhiserver/src/app_service_builder.rs` defaults to `DefaultHealthRegistry::default()`),
  `AppServiceStub` (`test_utils/app.rs`), and a passthrough on `AuthScopedAppService`
  (`auth_scoped.rs::health_registry()`).
- **Health key = `target_key(tenant, target.alias, target.model)`** — a free fn returning
  `"{tenant}:{alias}:{model}"`. `target.alias` IS the `alias_name()` identity (api id / local name),
  so no `forward_one` signature change was needed. Two routers sharing a provider share its cooldown;
  tenant component isolates tenants.
- **`order_by_health` + `cooldown_for` are free fns** (not strategy methods), so future strategies
  reuse them. `order_by_health(targets, registry, tenant, now)`: not-cooled in declared order first,
  then cooled by soonest `cooldown_until`; **never drops a target** (all-cooled yields all → never-starve).
  `cooldown_for(headers, cfg, now)`: `now + max(cooldown_secs, retry_after?)` when `honor_retry_after`,
  else `now + cooldown_secs`. `Retry-After` parsed as delta-seconds or best-effort RFC2822 HTTP-date.
- **`RouterContext` gained `time_service: Arc<dyn TimeService>` + `health: Arc<dyn HealthRegistry>`**
  (`strategy.rs`). Threaded in at the chat handler (`routes_app/src/oai/routes_oai_chat.rs`) via
  `auth_scope.time_service()` / `auth_scope.health_registry()`.
- **`FallbackConfig::execute` is the single behavioral change point** (`fallback.rs`): orders enabled
  targets by health *before* applying `max_attempts`; on `Success` → `record_success(key)`; on
  `Terminal` → leave health untouched; on `Retryable` → `cooldown(key, cooldown_for(resp.headers,…))`;
  on `Err` → `cooldown` ONLY if `e.is_transport_failure()` (i.e. `ModelRouterError::Forward`), NOT for
  structural skips (dangling/nested/unsupported). Exhaustion behavior unchanged from Phase 2.
- **`ModelRouterError::is_transport_failure()`** (new, `error.rs`) distinguishes a genuine transport
  failure (`Forward`) from a structural skip — drives the "don't cool structural problems" rule.

## 2. Test seams

- **`services::test_utils::TestTimeService`** (new, `test_utils/db.rs`): `Arc<RwLock<DateTime<Utc>>>`
  with `new`/`set`/`advance`; `Clone` shares the cell so a clone handed to a service and the test's
  copy observe the same advance. Use for any cooldown/recovery test. `FrozenTimeService` (immutable)
  stays for non-advancing tests.
- **Live server with a controllable clock**: `start_test_live_server_with_time()` (returns
  `(TestLiveServer, TestTimeService)`) + `setup_test_app_service_with_time(temp_dir, time_service)` in
  `server_app/tests/utils/live_server_utils.rs`. The cool-then-recover live test advances this clock
  to expire the cooldown — no sleeps.
- Unit coverage: `models/router/test_health.rs` (registry + `order_by_health` + `cooldown_for`) and the
  Phase-3 block at the bottom of `test_fallback_strategy.rs` (cooldown/skip, half-open recover &
  return-to-primary, re-cool on failed trial, Retry-After extension, shared-health, never-starve,
  disabled-wins, structural-not-cooled, transport-cooled). The fixture now supports a shared registry +
  clock and **mutable** per-alias outcomes (`Outcomes = Arc<Mutex<HashMap>>`) so a target can recover
  mid-test.

## 3. Served-target observability — important constraint for Phase 4

The chat UI streams via pi-ai `streamSimple` (`bodhi/src/stores/agentStore.ts`). The provider sets the
assistant message's `model` to the **request** model (`model.id`), NOT the upstream-echoed served model
(the streamed chunks carry `chunk.model` but the SDK ignores it). So the SDK cannot tell us which target
served. Consequences:
- `ChatMessage.tsx` exposes `data-served-model` from `metadata.model` (= the router alias for a router
  chat) — useful generally but **not discriminating between targets**.
- The Phase-3 E2E therefore distinguishes targets by **reply content** (the only black-box signal), and
  proves the user-visible **skip** behaviour (repeated sends keep succeeding through a cooled-primary
  router). It does NOT advance the server clock (black-box can't), so deterministic cooldown-expiry /
  return-to-primary is covered by the service-unit + live `server_app` layers.
- **Phase 4 spec update suggested:** if Phase 4 wants a UI-visible "served by X" indicator, it must
  surface the `x-bodhi-routed-alias`/`-routed-model` response header — which requires either a custom
  pi-ai stream function that reads response headers, or reading `chunk.model` from the stream. Note this
  in the Phase-4 test/probe spec.

## 4. Untouched / deferred
- No DB-persisted or cross-replica health; no penalty/latency ranking; no background probes (all out of
  scope, confirmed).
- `RouterTarget.weight` and per-router `SelectionState` remain unbuilt (round-robin/weighted concern).
- The Phase-4 router **test/probe** capability is where structural problems get surfaced to the user.
