# Composite Model Routing in Open-Source AI Gateways — Consolidated Research

This consolidates code-level research across the strongest reference gateways for building a **composite model alias** in BodhiApp — a single alias that fronts an ordered set of *targets* (each = a referenced alias + a pinned model) and routes between them via a **pluggable strategy**. The first (and only v1) strategy is **fallback**; the abstraction is designed so round-robin / weighted / latency / load-balance strategies slot in later as additive variants without duplicating the forwarding or health-tracking code.

> **Terminology:** the feature is a **model-router** alias (`source = "model-router"`). "Fallback" is now one *strategy* under it, not the feature name.

Feeds the BodhiApp design in [`bodhiapp-model-router-implementation-proposal.md`](./bodhiapp-model-router-implementation-proposal.md).

Source deep-dives (with `file:line` references):
- **Strategy/composite abstraction** (primary for this revision):
  - [`findings-composite-strategy-portkey-litellm.md`](./findings-composite-strategy-portkey-litellm.md) — Portkey `strategy.mode`, LiteLLM `routing_strategy` vs `fallbacks`
  - [`findings-composite-strategy-bifrost-ogem-free.md`](./findings-composite-strategy-bifrost-ogem-free.md) — Ogem's selectable `RoutingStrategy` enum, Bifrost weight-based, free-gw hardcoded
- **Fallback mechanics** (still valid, underpins the v1 strategy):
  - [`findings-free-llm-gateways.md`](./findings-free-llm-gateways.md) — `freellmapi`, `MrFadiAi/free-llm-gateway`
  - [`findings-litellm-bifrost.md`](./findings-litellm-bifrost.md) — LiteLLM cooldown, Bifrost failover
  - [`findings-portkey-ogem.md`](./findings-portkey-ogem.md) — Portkey declarative config, Ogem health loop
- Original landscape survey: [`fallback-routes.md`](./fallback-routes.md)

---

## 1. The composite abstraction — how gateways model "pluggable strategy over shared targets"

The central question for BodhiApp is *not* "how to do fallback" (that's well-trodden, see §5) but **how to make the routing behavior pluggable so it can evolve**. Three repos answer this directly:

| Gateway | Strategy named as data | Dispatch | Each strategy is… | Load-balance vs fallback |
|---|---|---|---|---|
| **Portkey** | `strategy.mode` enum: `single \| loadbalance \| fallback \| conditional` (`requestBody.ts:22`) | one recursive `tryTargetsRecursively` with a `switch(mode)` (`handlerUtils.ts:476-834`) | a `case` arm that picks *which child target(s)* to recurse into | **Unified** — sibling enum values that **compose by nesting** (`z.lazy` recursive schema; a fallback node can wrap loadbalance nodes) |
| **LiteLLM** | `routing_strategy` enum + separate `fallbacks` map (`router.py:312`) | `_build_strategy_selector` factory (`:879`) → `_select_deployment_async` (`:1090`) | a **class** in `router_strategy/*` (LeastBusy/LowestLatency/LowestCost…) registered name→obj | **Separated** — `routing_strategy` selects ONE deployment *within* a group; `fallbacks` is a *wrapper* that retries *across* groups |
| **Ogem** | `RoutingStrategy` enum: `latency, cost, round_robin, weighted_round_robin, least_connections, random_weighted, performance_based, adaptive` (`routing/router.go:20-46`) | one `switch` in `RouteRequest(endpoints) -> pick` (`:263-278`) | one **method** over the same candidate list | **Separated but adjacent** — strategy picks; a distinct `FallbackStrategy` runs on error (`:281-295`) |

**Ogem is the closest precedent to BodhiApp's model-router:** a single selectable strategy enum, each strategy a method over a shared candidate list, with failover as an adjacent concern. Portkey contributes the cleanest *data* model (strategy-as-enum on a node holding shared `targets[]`).

### The load-balance ⇄ fallback axis (key insight)
These are conceptually two axes:
- **Selection**: *which* target to try first (order, round-robin, weight, latency).
- **Failover**: *what to do when a try fails* (move to the next remaining target).

LiteLLM keeps them orthogonal (two mechanisms). Portkey unifies them (nesting). **BodhiApp collapses them into the strategy itself**: because each strategy *owns its execution loop*, a strategy decides both the selection order *and* the fall-through behavior. Fallback = declared order + fall-through. A future round-robin = rotated start + fall-through. This is simpler than LiteLLM's split and avoids Portkey's recursive-nesting complexity, while still composing the two behaviors.

### Why "strategy owns the loop" beats "strategy only selects"
LiteLLM/Portkey strategies mostly *select a candidate set* and a shared engine runs the attempt loop. That interface **cannot express parallel hedging / first-response-wins** (fire two targets, take whichever returns first), because the engine, not the strategy, owns concurrency. Giving the strategy its own `execute()` loop (calling shared primitives) keeps that door open. The cost — each strategy writes a ~5-line `for` loop — is paid back by not foreclosing concurrent strategies. This is the BodhiApp choice.

### Avoiding duplication despite per-strategy loops
The mechanical parts are factored into **shared primitives every strategy calls**, never reimplements:
- a **forwarding primitive** (Portkey `tryPost`, LiteLLM per-deployment call) — BodhiApp: `forward_one(target, req) -> AttemptOutcome` doing forward + failure-classification + health update;
- a **strategy-agnostic health/cooldown store**, read *before* selection and written *after* each attempt (Portkey circuit-breaker `isOpen` prefilter `handlerUtils.ts:646`; LiteLLM `CooldownCache` + `_filter_cooldown_deployments` `router.py:10787` — **confirmed strategies never query it directly**);
- candidate enumeration/filtering.

---

## 2. Seams a v1 (fallback-only) design must reserve for future strategies

From the strategy research, stateful strategies need cross-request state that v1 fallback does not — but the v1 *type signatures* must already accommodate them so adding a strategy needs no signature change and no forwarder change:

| Future strategy | Extra state it needs | Seam to reserve in v1 |
|---|---|---|
| round-robin | per-router cursor (counter) — **keyed per router-alias, not global** (Ogem's global cursor `router.go:154` is a bug-smell) | a per-router mutable `selection_state` slot, behind a lock, held *alongside* (not inside) the health store |
| weighted | cursor + accumulated-weight walk; per-target `weight` | per-target optional `weight` field (additive default-uniform); same `selection_state` slot |
| latency / least-conn / performance | per-target metrics table (EWMA latency, success rate, active conns), updated post-request (Ogem `RecordRequestResult` `:313`) | `AttemptOutcome` already carries timing; a post-attempt hook the forwarder always calls (v1 fallback no-ops it) |

**Reserve, don't build:** v1 stores none of this, but (a) `RouterContext` exposes a `selection_state` accessor, (b) `RouterTarget` has an optional `weight`, and (c) `AttemptOutcome` includes elapsed time. No dead code, no speculative types — just signatures that won't churn.

---

## 3. The fallback strategy core (convergent across all six gateways)

Independent of stack, the gateways converge on the same minimal core for the fallback behavior:

1. **Ordered chain.** Targets tried in array order; first success wins.
2. **Status-driven fall-through.** Move to the next target only on a *retryable* status. The default retryable set is remarkably stable: **`429`, `500`, `502`, `503`, `504`, plus transport timeouts/connection errors**.
3. **One in-memory cooldown map.** `HashMap<target_key, cooldown_until>`. On a retryable failure set `cooldown_until = now + N`; selection skips targets where `now < cooldown_until`.
4. **Passive half-open recovery.** Cooldown is just a TTL; when it expires the target silently becomes eligible and the **next real request is the trial**. No background pinger needed to return traffic to the primary.

---

## 4. Failure classification — the three-way split (BodhiApp tuning)

The single most important runtime rule. BodhiApp's choice (tuned for stacking free-tier providers, where a bad key/model on one vendor must not kill the chain):

| Bucket | Statuses / conditions | Action |
|---|---|---|
| **Retryable** | transport error, timeout, `408`, `429`, `500`, `502`, `503`, `504`, **and `401`, `403`, `404`** | cool down this target, fall through to next |
| **Terminal** | `400`, `422`, context-window-exceeded, content-policy | return verbatim, **do not** fall through (the request itself is the problem) |
| **Success** | `2xx` | stream back, record success, stop |

Notes from code: LiteLLM's `_should_retry` covers `408/409/429/5xx`, `_is_cooldown_required` adds `401/404`. Bifrost adds rate-limit **message-sniffing** (`rateLimitPatterns`) — a body saying "quota exceeded" is treated as `429` even on an odd status (a worthwhile *later* refinement; v1 classifies by status code only).

---

## 5. Cooldown mechanics worth copying (LiteLLM legacy path)

LiteLLM has two cooldown paths; **copy the legacy one, skip the default**:
- **Default (skip):** percent-fails-per-minute — needs success+fail counters, `>0.5` over `>=5` requests, a 1000-request guard. Too heavy at desktop/low volume.
- **Legacy (use):** flat `consecutive_fails >= allowed_fails` → cooldown for `cooldown_time`, entry self-clears via TTL (`cooldown_handlers.py:420-428`; the cache entry's TTL *is* the cooldown, `cooldown_cache.py:94-98`).

For free-API stacking, default `allowed_fails = 1` (one quota hit ⇒ stop sending), small flat cooldown, and **honor upstream `Retry-After`** (`cooldown = max(base, retry_after)`).

---

## 6. Streaming rule (verbatim across all)

Decide failover **before the first byte**. The upstream HTTP status is known before the body stream is consumed, so classification happens on status alone. Once bytes are forwarded, failover is impossible — a mid-stream failure must surface as a broken stream and **never silently truncate or re-route**.

**BodhiApp note:** because the existing forwarder returns an axum `Response` whose body is a *lazy* stream (`provider_shared.rs:155-171`), the executor can read `response.status()` and decide to **stream it** (commit) or **drop+buffer it** (retry) without any forwarder refactor. The status is available without consuming the body. (Transport failures still arrive as `Err` from `send().await?`.)

---

## 7. What to deliberately NOT build for v1

| Feature | Why skip |
|---|---|
| Active health-check / ping loop | Ogem ships it **disabled**; costs a paid upstream call per target per interval; passive half-open already returns traffic to primary. |
| Penalty score + decay re-ranker | A plain ordered chain + binary cooldown-skip is behaviorally equivalent for an ordered strategy. |
| Round-robin / weighted / latency strategies | v1 is fallback-only; seams reserved (§2) so they're additive later. |
| Strategy nesting (Portkey-style recursive targets) | Powerful (router-of-routers) but adds cycle detection + recursion; flat single-strategy covers the requirement. Disallow a target referencing another model-router for v1. |
| Sticky sessions | Conflicts with "return to primary ASAP"; adds a session store. |
| Redis / distributed shared state | In-memory per-process matches BodhiApp's "DB-only dependency, runs on desktop" constraint. |
| Percent-fails-per-minute cooldown | Needs windowed counters; flat consecutive-fail + TTL is simpler and fine at low volume. |

---

## 8. Distilled recommendation for BodhiApp

A **`model-router` alias** = ordered `targets[]` + a serde-tagged `strategy` enum (`Fallback` only in v1). Build:
1. The composite alias + targets in the DB; per-target optional `weight` reserved.
2. A **`RoutingStrategy` trait** with `execute(ctx) -> Response` (strategy owns its loop) + `validate(targets)`. `FallbackStrategy` is the one impl.
3. **Shared primitives** in `RouterContext`: `forward_one()` (forward + classify + health update, returns `AttemptOutcome` incl. elapsed) and a process-global **`HealthRegistry`** keyed by `(tenant, referenced_alias_id, model)` so chains sharing a free provider share its cooldown.
4. The three-way classification (§4), shared default; passive half-open recovery; `Retry-After` honoring.
5. Pre-first-byte failover only; chain-exhausted ⇒ surface the **last upstream response verbatim**.
6. Reserve the seams (§2): `selection_state` accessor, per-target `weight`, timing in `AttemptOutcome` — without building them.
7. Two observability headers: `x-bodhi-routed-alias`, `x-bodhi-router-attempts` (+ `x-bodhi-router-strategy`).

Full BodhiApp-specific design (domain model, DB schema, trait, primitives, APIs, errors, config, routing hook, frontend, tests, build order) in [`bodhiapp-model-router-implementation-proposal.md`](./bodhiapp-model-router-implementation-proposal.md).
