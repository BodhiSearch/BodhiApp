# Composite routing strategy survey: bifrost, ogem, freellmapi, free-llm-gateway

Focus: do these gateways abstract MULTIPLE routing strategies over a shared target set
(selectable strategy), or hardcode a single fallback/load-balance behavior? Informs the
BodhiApp `model-router` alias: ordered `targets` (referenced alias + pinned model) plus a
pluggable serde-tagged `strategy` (v1 = `Fallback` only; round-robin/weighted/latency additive
later without duplicating forwarding/health code).

Verdict up front:
- **ogem** — REAL strategy abstraction. `RoutingStrategy` enum + per-strategy methods behind
  one dispatch + one fallback-strategy. Closest match to the BodhiApp design.
- **bifrost** — Strategy configurable but as *weighted-random load-balance + auto-fallback*,
  not a swappable enum of algorithms. Balance and fallback are ONE mechanism (the weight-sort
  produces both the pick and the fallback order).
- **freellmapi** (tashfeenahmed) — Single hardcoded algorithm (priority + dynamic penalty +
  round-robin keys + sticky). "Sort presets" only re-order the persisted priority column; they
  are not runtime-selectable selection algorithms.
- **free-llm-gateway** (MrFadiAi) — Single hardcoded ordered-fallback + round-robin keys.
  Not pluggable.

---

## 1. maximhq/bifrost (Go)

### Strategy abstraction?
No swappable algorithm enum. The selection algorithm is fixed: **weighted-random selection**,
configured by per-target weights. What varies is data (weights, allowed models, routing rules),
not the algorithm. Three layers stack in precedence: CEL routing rules -> governance
weighted-random -> (enterprise) adaptive load balancer. There is no `Strategy` field you set to
"round_robin" vs "latency".

### Candidates & per-target params
- Fallback targets are a list on the request: `Fallbacks []Fallback`
  (`core/schemas/chatcompletions.go:19`, `core/schemas/bifrost.go:408`; `GetRequestFields` /
  `SetFallbacks` at `core/schemas/bifrost.go:477,724`). A `Fallback` is a provider+model pair.
- Governance enumerates provider configs allowed for a virtual key, filters by
  `allowed_models`/budget/rate limits, then weighted-random over survivors:
  `weightedConfigs`, `totalWeight`, `randomValue := rand.Float64() * totalWeight`, weighted walk
  (`plugins/governance/main.go:834-862`). **Per-target weight is the only knob.**

### Selection state
- In-memory, shared across requests. Provider lists/queues/mutexes are `atomic.Pointer` and
  `sync.Map` on the `Bifrost` struct (`core/bifrost.go:66-74`). Governance rules cached
  in-memory via `LocalGovernanceStore` (`sync.Map`).
- Notably bifrost's primary balance is **stateless weighted-random** (`rand.Float64`), so it
  needs NO cross-request cursor on its main path — randomness replaces a counter.

### Load-balance vs fallback: SAME path
After weighted-random picks the primary, the *remaining* weighted configs become the fallback
chain (sorted by weight, highest first): `fallbackConfigs := weightedConfigs`
(`plugins/governance/main.go:898`). One weight-sort yields both the pick and the ordered
fallbacks. The enterprise adaptive LB does the same: score, pick best, remaining sorted-by-score
become fallbacks.

### Lesson for BodhiApp
Weight is the single per-target param and load-balance == "pick by weight, fallback = rest by
weight." If BodhiApp picks random-weighted, no cursor is needed. But bifrost conflates balance
and fallback; BodhiApp wants them as separate strategy variants, so do NOT copy the "balance and
fallback are the same sort" coupling.

---

## 2. yanolja/ogem (Go) — the closest analog

### Strategy abstraction? YES — explicit; this is the model to study
`routing/router.go:20-46` defines `type RoutingStrategy string` with constants:
`latency`, `cost`, `round_robin`, `weighted_round_robin`, `least_connections`,
`random_weighted`, `performance_based`, `adaptive`.

`RoutingConfig` (`routing/router.go:50-80`) carries `Strategy` + `FallbackStrategy` (a SECOND
strategy used only if the primary errors) + per-strategy params (`EndpointWeights map[string]float64`
at `:73`, weight factors for performance routing).

Dispatch is a single switch in `RouteRequest` (`routing/router.go:263-278`):
```
switch strategy {
case StrategyCost:               routeByCost(...)
case StrategyRoundRobin:         routeRoundRobin(...)
case StrategyWeightedRoundRobin: routeWeightedRoundRobin(...)
case StrategyLeastConnections:   routeLeastConnections(...)
case StrategyRandomWeighted:     routeRandomWeighted(...)
case StrategyPerformanceBased:   routePerformanceBased(...)
default:                         routeByLatency(...)
}
```
Each strategy is one method taking the SAME `[]*EndpointStatus` and returning one pick. **This is
exactly the BodhiApp seam: one candidate list in, pluggable selection, shared forwarding around
it.** Note `FEATURE_COMPARISON.md` marks advanced routing "not implemented" — the code is ahead
of the docs; the abstraction exists and is unit-tested (`routing/router_test.go:693-751`).

### Candidates & per-target params
- The proxy enumerates endpoints matching provider/region/model via `sortedEndpoints`
  (`server/server.go:2246`), default-sorted by `latency` ascending (`sort.Slice` at `:2279`).
- `intelligentRouteRequest` (`server/server.go:2289`) converts those to `routing.EndpointStatus`
  and hands the WHOLE list to `s.router.RouteRequest`; if the router is nil or errors it falls
  back to `allEndpoints[0]` (latency-first). The router is an optional overlay on a default
  latency ordering (`server/server.go:2308-2324`).
- Per-target weight lives in config (`EndpointWeights` map keyed by endpoint), not on the
  endpoint struct.

### Selection state — the important part for BodhiApp
The `Router` struct (`routing/router.go:151-160`) holds, in-memory and shared across requests:
- `endpointMetrics map[string]*EndpointMetrics` — latency/success/connections per endpoint
  (the "latency table" + circuit-breaker state), updated by `RecordRequestResult`
  (`routing/router.go:313`) using an EWMA.
- `roundRobinIndex int` — a single mutable cursor reused by BOTH `routeRoundRobin`
  (`:459`, `r.roundRobinIndex = (r.roundRobinIndex+1) % len(endpoints)` at `:467`) AND
  `routeWeightedRoundRobin` (`:472`, `:495-496`).
- `adaptiveState *AdaptiveState` — current sub-strategy + history for the `adaptive` meta-mode.
- `mutex sync.RWMutex` guarding all of it.
`EndpointMetrics.ActiveConnections` is the `least_connections` state, incremented at selection
(`incrementActiveConnections`) and decremented in `RecordRequestResult`.

### Load-balance vs fallback: separate but composed
Primary strategy is the load-balance pick. If it errors, a *different* `FallbackStrategy` runs
over the same list (`routing/router.go:281-295`). Distinct from the proxy-level failover: the
proxy ALSO retries the next endpoint in the sorted list and disables a failing endpoint for ~1
minute (rate-limit/429 handling in the `generateChatCompletion` retry loop). So ogem has two
fallback levels: (a) strategy-fallback inside the router, (b) endpoint-failover in the proxy.

### Lesson for BodhiApp
Near-exact blueprint: a `Strategy` enum, one `RouteRequest(endpoints) -> pick` dispatch, strategy
state (cursor, latency/metrics table, active-connections, adaptive sub-state) all held in one
in-memory mutex-guarded struct shared across requests, decoupled from the forwarding/retry loop.
The cursor being a single global `int` shared across round-robin and weighted is a smell — see
seams below (per-router-key cursor, not global).

---

## 3. tashfeenahmed/freellmapi (TypeScript/Node)

### Strategy abstraction? NO — one hardcoded algorithm
`routeRequest` in `server/src/services/router.ts:134` implements a single fixed algorithm: sort
the persisted fallback chain by `effectivePriority = base_priority + dynamic_penalty`
(`:144-148`), optional sticky-session move-to-front (`:150-157`), iterate models, and for each
model round-robin over its keys (`:185-222`). No strategy selector at request time.

The "sort presets" (`/api/fallback/sort/:preset` -> `intelligence`/`speed`/`budget`) only rewrite
the persisted `priority` column in the DB; they re-order the ONE chain, they do not swap selection
algorithms. So: configurable ordering, fixed algorithm.

### Candidates & per-target params
- Candidates = rows of `fallback_config` (`model_db_id, priority, enabled`) joined to `models`
  and `api_keys` (`router.ts:28-32, 138-142`).
- Per-target params: `priority` (base), plus `intelligence_rank`/`speed_rank` on the model used by
  sort presets. Dynamic penalty is computed, not configured.

### Selection state
- Persistent: `fallback_config`, `models`, `api_keys` in better-sqlite3.
- In-memory, module-level (shared across requests, single process):
  - `roundRobinIndex = new Map<string, number>()` keyed `platform:model_id` (`router.ts:45`,
    advanced at `:187,212,226`).
  - `rateLimitPenalties = new Map<number, {count,lastHit,penalty}>()` (`router.ts:49`) — the
    penalty/decay table (`recordRateLimitHit` `:60`, time-based `getPenalty` decay `:88-106`).
  - `stickySessionMap` (proxy layer) maps conversation hash -> model_db_id.

### Load-balance vs fallback: same loop
Fallback (iterate models) and load-balance (round-robin keys within a model) are nested in the one
`routeRequest` loop. Not separable.

### Lesson for BodhiApp
Concretely shows the in-memory state a stateful strategy needs: a `Map<targetKey, cursor>` for
round-robin and a `Map<targetKey, penaltyEntry>` for penalty/decay, both keyed per logical target
and held at process scope. Confirms the cursor must be per-target, not a single global int.

---

## 4. MrFadiAi/free-llm-gateway (Python)

### Strategy abstraction? NO — hardcoded ordered-fallback + round-robin
`router.py` `Router.route_request` (`router.py:281`) and `_select_provider` (`router.py:164`)
implement one fixed algorithm: build the ordered fallback candidate list, walk it with
retry/backoff. Selection order is the YAML fallback order, adjusted by penalty sort and a
round-robin rotation among equal-penalty candidates (`router.py:231-242`). No strategy enum, no
runtime selector. `models.yaml` defines per-model `fallbacks` (provider+model list).

### Candidates & per-target params
- Candidates from `config.models[model].fallbacks` (`get_fallbacks` `:151`), each a
  `ModelFallback` provider+model (+ optional per-model rpm/rpd/tpm/tpd limits).
- `_select_provider` filters rate-limited/unhealthy/on-cooldown providers, then sorts by penalty
  and rotates (`:231-242`). No configured weight; ordering = YAML order + penalty + round-robin.

### Selection state (all in-memory, single gateway instance)
- `Router._rr_index: dict[str,int]` — round-robin cursor per model (`router.py:149`, used
  `:240-241`).
- `Router.penalty_tracker` = `PenaltyTracker` with `_penalties: dict[str, {count,last_hit,penalty}]`
  keyed `provider:model` (`router.py:36-100`); same +3/-1/decay scheme as freellmapi.
- `ProviderConfig._key_index` — round-robin key rotation inside a provider (`config.py`,
  `rotate_key`, called on 429/auth at `router.py:367-368, 408-409`).
- Cooldown in `per_key_rate_tracker`/`RateLimiter`; health/cooldown in `HealthChecker`.
- "Sticky" is a separate module (`sticky_sessions.py`), not part of router selection.

### Load-balance vs fallback: same path
Round-robin (across equal candidates / across keys) and fallback (walk the chain on failure) are
the same loop in `route_request`. Not pluggable.

### Lesson for BodhiApp
Second independent implementation reinforcing freellmapi: same penalty/decay map and per-model
round-robin cursor, plus a SECOND cursor inside the provider for key rotation. Two levels of
round-robin state (across-targets and within-target) each need their own cursor map.

---

## Seams a v1 fallback-only design must leave for future strategies

(round-robin cursor, weighted, latency table — all observed above)

1. **Strategy as a dispatch over a shared candidate list, not branching in the forwarder.**
   Mirror ogem `RouteRequest(endpoints) -> pick` (`routing/router.go:263-278`): the strategy
   receives the already-enumerated `targets` and returns the next target; health checks,
   forwarding, and failover live OUTSIDE it. v1 `Fallback` is just "return first healthy in
   order," but it must implement the same `select(&targets, &state) -> target` trait so
   round-robin/weighted/latency slot in without touching forwarding/health code. The single most
   important seam.

2. **A per-router, mutable, cross-request selection-state store, SEPARATE from the health store.**
   Every stateful strategy needs mutable state that survives across requests and is shared:
   - round-robin -> a cursor/counter. ogem `roundRobinIndex int` (`:154`); freellmapi
     `roundRobinIndex: Map<platform:model,int>`; fadi `_rr_index: dict[model,int]`.
   - weighted-round-robin -> cursor + accumulated-weight walk (ogem `:472-510`); or stateless
     random-weighted (bifrost `rand.Float64`, no cursor).
   - latency / least-conn / performance -> a metrics table per target (EWMA latency, success
     rate, active connections): ogem `endpointMetrics map[key]*EndpointMetrics` (`:153`).
   v1 `Fallback` needs none of this, BUT the router struct should already own an opaque
   `selection_state` slot (an enum or per-strategy struct behind a `Mutex`/`RwLock`) keyed per
   router, sitting ALONGSIDE the health store — not folded into it. Health = "is target usable";
   selection-state = "which usable target is next." Keeping them separate (as ogem does: metrics
   in Router vs. endpoint-disable in proxy) lets Fallback ignore selection-state entirely while
   later strategies populate it.

3. **Cursor/counters keyed per logical target set, not a single global.**
   ogem's single global `roundRobinIndex int` shared between two strategies is a smell that breaks
   with concurrent routers/models. freellmapi and fadi both correctly key the cursor by
   `model`/`platform:model`. BodhiApp should key selection state by router-alias (and, for
   key-level rotation, by target) from the start, even though v1 stores nothing.

4. **Per-target params on the target struct, so a `weight`/`priority` field is additive.**
   Weighted strategies need a per-target weight (bifrost weighted-random; ogem `EndpointWeights`).
   v1 `targets` are ordered (priority = position), so no weight field is required yet — but the
   target struct/serde should be shaped so adding an optional `weight` (default uniform) later is
   non-breaking. ogem keeps weights in a side-map keyed by endpoint; an optional field on the
   target itself is cleaner for BodhiApp.

5. **Strategy state mutated UNDER A LOCK and updated post-request (latency/penalty).**
   Latency/performance/penalty strategies update state AFTER the call returns (ogem
   `RecordRequestResult` `:313`; freellmapi `recordRateLimitHit`/`recordSuccess`; fadi
   `penalty_tracker.record_hit/success`). v1 Fallback records nothing, but the forwarding loop
   should already expose a post-request hook (success/failure + latency) that v1 leaves as a
   no-op and stateful strategies later implement — otherwise adding latency routing forces a
   forwarder rewrite. Pair with `RwLock`/`Mutex`: the cursor/table is read on select and written
   on completion concurrently (all four guard with a mutex/sync.Map or rely on single-threaded JS).

6. **Keep `FallbackStrategy`-as-second-strategy optional, don't hardcode it.**
   ogem separates primary `Strategy` from `FallbackStrategy` (`:52-55`). For v1 the strategy IS
   fallback, so this is moot — but the dispatch shape (a strategy can delegate to another on
   empty/error) is worth leaving room for rather than baking a single ordered walk into the router.
