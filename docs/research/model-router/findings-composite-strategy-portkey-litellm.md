# Pluggable Routing Strategy — Portkey & LiteLLM (composite strategy research)

Focused study of how two production LLM gateways abstract a **pluggable routing strategy** over a
shared set of targets. Goal: inform BodhiApp's `model-router` alias — a composite alias holding an
ordered list of `targets` (referenced alias + pinned model) plus a serde-tagged `strategy` enum
(`Fallback` is v1; round-robin / weighted / load-balance / latency must slot in later as additive
variants **without** duplicating the forwarding/health code).

Two headline insights up front:

1. **Strategy is data, dispatch is a switch/registry.** Both gateways name the strategy as a small
   string/enum on the config node, and a single dispatch point maps that name to behavior. Adding a
   strategy = adding one enum arm + one selection function. The forwarding/execution code is shared.
2. **Load-balance and fallback are two different axes, not two values of one enum** — except in
   Portkey, which deliberately *unifies* them as sibling `mode` values and lets them **compose by
   nesting**. LiteLLM keeps them as **separate orthogonal mechanisms** (a `routing_strategy` that
   picks one deployment *within* a group, and a `fallbacks` map that retries *across* groups). This
   divergence is the single most important design decision for BodhiApp to make consciously.

---

## 1. Portkey-AI/gateway

Portkey models the whole router as a **recursive config tree**. Every node is a `Targets` object
that may carry a `strategy` and a `targets[]` array; leaf nodes are concrete providers.

### 1a. Strategy as data — the `mode` enum

`src/types/requestBody.ts:22-39`

```ts
export enum StrategyModes {
  LOADBALANCE = 'loadbalance',
  FALLBACK = 'fallback',
  SINGLE = 'single',
  CONDITIONAL = 'conditional',
}

interface Strategy {
  mode: StrategyModes;
  onStatusCodes?: Array<number>;   // which HTTP codes trigger fallback (strategy-level, not per-target)
  conditions?: { query: {...}; then: string }[];  // conditional-only
  default?: string;
}
```

Note: `loadbalance` and `fallback` are **sibling values of one enum** — Portkey treats them as the
same kind of thing (a way to choose among `targets`).

Validated at request time by a Zod schema. Crucially the schema is **recursive** — `targets` is
`z.array(z.lazy(() => configSchema))`, which is what enables nesting:

`src/middlewares/requestValidator/schema/config.ts:14-79`

```ts
strategy: z.object({
  mode: z.string().refine(v =>
    ['single','loadbalance','fallback','conditional'].includes(v), {...}),
  on_status_codes: z.array(z.number()).optional(),
  conditions: z.array(z.object({ query: z.object({}), then: z.string() })).optional(),
  default: z.string().optional(),
}).optional(),
...
weight: z.number().optional(),                       // per-target
targets: z.array(z.lazy(() => configSchema)).optional(),  // RECURSIVE
```

### 1b. Shared targets + per-target params

`weight` lives **per target** (used only in loadbalance mode); `onStatusCodes` lives **on the
strategy node** (used by fallback). Both `Options` (leaf provider) and `Targets` (node) carry
`weight`, `retry`, `overrideParams`, `cache`, `index`, `originalIndex`.

`src/types/requestBody.ts:45-225`

```ts
export interface Targets {
  name?: string;
  strategy?: Strategy;      // present => this is a routing node
  weight?: number;          // per-target, used by parent's loadbalance
  retry?: RetrySettings;
  overrideParams?: Params;
  cache?: CacheSettings | string;
  targets?: Targets[];      // children => nesting
  originalIndex?: number;
  // ...all provider creds inline...
}
```

The same `targets[]` array is consumed by **every** mode — the difference is *how the mode walks the
array*, not the array's shape. That is the key reuse lever.

### 1c. Dispatch / polymorphism — `tryTargetsRecursively`

There is **no per-strategy class**. A single recursive function holds one `switch (strategyMode)`.
Each arm decides *which child(ren)* to recurse into; recursion bottoms out at the `default` arm,
which does the actual `tryPost` provider call. This is the entire dispatch table.

`src/handlers/handlerUtils.ts:476-834` (called from every endpoint handler: `chatCompletionsHandler`,
`proxyHandler`, `messagesHandler`, etc.)

```ts
export async function tryTargetsRecursively(c, targetGroup, request, headers, fn, method, jsonPath, inheritedConfig = {}) {
  const currentTarget = { ...targetGroup };
  const strategyMode = currentTarget.strategy?.mode;

  // ... merge inheritedConfig <- currentTarget (retry, cache, guardrails, customHost, timeout) ...
  // ... circuit-breaker prefilter: drop targets where t.isOpen ...

  switch (strategyMode) {
    case StrategyModes.FALLBACK:           // 663
      for (const [i, target] of currentTarget.targets.entries()) {
        response = await tryTargetsRecursively(c, target, ..., currentInheritedConfig); // RECURSE
        const codes = currentTarget.strategy?.onStatusCodes;
        const gatewayException = response?.headers.get('x-portkey-gateway-exception') === 'true';
        if ((Array.isArray(codes) && !codes.includes(response?.status)) ||  // status not in trigger list
            (!codes && response?.ok) ||                                     // default: stop on success
            gatewayException) break;                                        // stop on hard error
      }
      break;

    case StrategyModes.LOADBALANCE:        // 693  — pick ONE child by weight
      currentTarget.targets.forEach(t => { if (t.weight === undefined) t.weight = 1; });
      let total = currentTarget.targets.reduce((s,p)=>s+p.weight,0);
      let r = Math.random() * total;
      for (const [i, provider] of currentTarget.targets.entries()) {
        if (r < provider.weight) {
          response = await tryTargetsRecursively(c, provider, ..., currentInheritedConfig); // RECURSE into one
          break;
        }
        r -= provider.weight;
      }
      break;

    case StrategyModes.CONDITIONAL:        // 725 — ConditionalRouter.resolveTarget() then recurse into one
    case StrategyModes.SINGLE:             // 767 — recurse into targets[0]
    default:                               // 781 — leaf: tryPost(...) actual provider call + circuit breaker hook
  }
  return response;
}
```

**How a new strategy is added in Portkey:** add an enum value + a `case` arm that selects which child
to recurse into. The execution primitive (`tryPost`) and config inheritance are untouched.

### 1d. Load-balance vs fallback — they COMPOSE by nesting

Because every arm recurses back into `tryTargetsRecursively`, a `fallback` target can itself be a
`loadbalance` node and vice versa. Example: top-level `fallback` over two `loadbalance` clusters —
load-balance within a cluster, fail over to the next cluster on the configured status codes. The
recursion + `currentInheritedConfig` merge (`handlerUtils.ts:491-644`) is what makes arbitrary depth
work; child nodes inherit parent retry/cache/timeout/guardrails but can override.

### 1e. Health / circuit breaker

Strategy-agnostic. Before the switch, a circuit-breaker prefilter drops any target flagged
`isOpen` from `currentTarget.targets` (`handlerUtils.ts:646-658`), and the leaf `default` arm calls
`handleCircuitBreakerResponse` after the provider call (`:792-800`). All modes share this — the
strategy never reads circuit-breaker state directly; it just operates on the already-filtered list.

### 1f. Streaming + failover

Streaming is opaque to the strategy. `tryPost` returns a `Response` (possibly a streaming body); the
fallback arm only inspects `response.status` / `response.ok` / the `x-portkey-gateway-exception`
header to decide whether to continue. There is no mid-stream failover — once a target starts
streaming, that target owns the response. Failover is a *pre-first-byte* decision based on the
response status.

---

## 2. BerriAI/litellm

LiteLLM splits the problem along **two explicit axes** that BodhiApp's `mode` enum collapses:

- **`routing_strategy`** — picks *one* deployment from the healthy deployments *within a model
  group*. This is load balancing.
- **`fallbacks` / `context_window_fallbacks` / `content_policy_fallbacks`** — a *separate* wrapper
  layer that retries *across model groups* when the selected deployment fails.

### 2a. Strategy as data — `routing_strategy` literal/enum

`litellm/router.py:312-321` (constructor) and the `RoutingStrategy` enum:

```python
routing_strategy: Literal[
  "simple-shuffle", "least-busy", "usage-based-routing",
  "latency-based-routing", "cost-based-routing",
] = "simple-shuffle"
```

`RoutingStrategy` enum members: `LEAST_BUSY`, `LATENCY_BASED`, `COST_BASED`,
`USAGE_BASED_ROUTING`, `USAGE_BASED_ROUTING_V2`. The constructor accepts string OR enum and
normalizes via `_normalize_strategy` (`router.py:849-857`); validation is a membership check
(`_validate_routing_strategy`, `:859-877`).

Modern LiteLLM also supports **`routing_groups`**: named subsets of model names, each with its own
`routing_strategy` + args; everything else lands in an implicit `"default"` group
(`router.py:363-365`). This is LiteLLM's analogue of "different strategies for different target
sets."

### 2b. Strategy = a class implementing a common interface (registry)

Unlike Portkey's single switch, LiteLLM makes **each non-trivial strategy a class** under
`litellm/router_strategy/`:

```
least_busy.py            -> LeastBusyLoggingHandler
lowest_latency.py        -> LowestLatencyLoggingHandler
lowest_cost.py           -> LowestCostLoggingHandler
lowest_tpm_rpm.py        -> LowestTPMLoggingHandler        (usage-based v1)
lowest_tpm_rpm_v2.py     -> LowestTPMLoggingHandler_v2     (usage-based v2)
simple_shuffle.py        -> simple_shuffle()  (plain fn, no class — random/weighted pick)
base_routing_strategy.py -> BaseRoutingStrategy (shared redis-batching base)
```

All class-based strategies inherit `CustomLogger` (and the stateful ones `BaseRoutingStrategy`).
This dual role is the clever bit: a strategy is **both a callback logger and a selector**. As a
logger it observes `log_pre_api_call` / `log_success_event` to accumulate the metric it routes on
(busy count / latency / tpm / cost) into the shared cache; as a selector it answers
`(async_)get_available_deployments(...)`.

The **registry** is a dict mapping strategy name -> instance attribute:

`litellm/router.py:841-847`

```python
_DEFAULT_SELECTOR_ATTR_BY_STRATEGY = {
  "least-busy":            "leastbusy_logger",
  "usage-based-routing":   "lowesttpm_logger",
  "usage-based-routing-v2":"lowesttpm_logger_v2",
  "latency-based-routing": "lowestlatency_logger",
  "cost-based-routing":    "lowestcost_logger",
}
```

Construction is a `match` factory, `_build_strategy_selector` (`router.py:879-927`) — each arm news
up the handler with the **shared** `self.cache` and registers it on litellm's global callback list.
`simple-shuffle` returns `None` (no selector; handled inline).

### 2c. Dispatch at request time

`async_get_available_deployment` (`router.py:10861-10970`) is the single funnel:

1. pre-routing hooks (may rewrite model) -> `:10895`
2. `strategy, selector = self._get_routing_context(model)` — resolve group -> strategy -> selector
   (`:10910`, def at `:1059-1088`)
3. `healthy_deployments = await self.async_get_healthy_deployments(...)` — **shared** list build +
   health-check filter + **cooldown filter** + blocked filter (`:10912`; filter at `:10773-10804`)
4. `if strategy == "simple-shuffle": return simple_shuffle(self, healthy_deployments, model)`
   (`:10931`)
5. else `deployment = await self._select_deployment_async(strategy=, selector=, ...)`
   (`:10937`, def `:1090-1141`) — a `match strategy:` that calls the selector's
   `async_get_available_deployments(...)`.

So the dispatch is: **registry lookup -> cooldown-filtered shared list -> selector call**. The
selector interface is essentially `get_available_deployments(model_group, healthy_deployments,
[messages, input, request_kwargs]) -> deployment`.

### 2d. Per-target params — `weight`

In LiteLLM, per-target params live under each deployment's `litellm_params`. `simple-shuffle` reads
`weight` (and falls back to `rpm`/`tpm`) for a weighted random pick:

`litellm/router_strategy/simple_shuffle.py:42-71`

```python
for weight_by in ["weight", "rpm", "tpm"]:
    weight = healthy_deployments[0].get("litellm_params").get(weight_by)
    if weight is not None:
        weights = [m["litellm_params"].get(weight_by, 0) for m in healthy_deployments]
        total = sum(weights)
        if total <= 0: continue
        weights = [w/total for w in weights]
        i = random.choices(range(len(weights)), weights=weights)[0]
        return healthy_deployments[i]
item = random.choice(healthy_deployments)   # uniform if no weights
```

### 2e. Load-balance vs fallback — SEPARATE, orthogonal layers

This is the explicit two-axis model:

- **Within a group (load balance):** `routing_strategy` selects one deployment. State (tpm, latency,
  busy) is tracked per-deployment in the shared cache.
- **Across groups (fallback):** `fallbacks=[{"gpt-4": ["claude"]}]`, plus `context_window_fallbacks`
  and `content_policy_fallbacks`, are handled by a **wrapper** — `async_function_with_fallbacks`
  (and the streaming variants `stream_with_fallbacks`, `router.py:2147/2542/2675`). On failure it
  re-enters `async_get_available_deployment` for a *different* model group.

How they **compose**: fallback wraps selection. A request first load-balances within the primary
group; if that deployment errors (and the error type matches), the fallback layer picks the next
group and load-balances within *it*. `enable_weighted_failover` additionally re-picks within the
same group before crossing to another group.

### 2f. Shared cooldown / health store

Confirmed strategy-agnostic. `CooldownCache` is owned by the `Router`, not by any strategy.
Cooldowns are written on failure (`_set_cooldown_deployments`, `router.py:7336/7621`) and read by the
**shared** `_filter_cooldown_deployments` step (`:10787`) that runs *before* the strategy ever sees
the list. Strategies receive an already-cooldown-filtered `healthy_deployments` and never query
cooldown state themselves. `_get_cooldown_deployments` / `_async_get_cooldown_deployments`
(`router.py:104-107`) are the shared accessors.

### 2g. Streaming + failover

Same pre-first-byte model as Portkey but at the fallback layer: `stream_with_fallbacks` wrappers
catch the failure of starting a stream and retry against the next group. Once tokens flow, that
deployment owns the stream; no mid-stream switching.

---

## 3. Synthesized pluggable-strategy interface — what BodhiApp's `RoutingStrategy` should look like

### Decide the axis question first

Portkey = **one enum, compose by nesting**; LiteLLM = **two orthogonal axes** (select-within vs
retry-across). For BodhiApp's `model-router` alias, the cleanest fit given the brief (one `targets[]`
+ one serde-tagged `strategy`) is the **Portkey unified-enum** shape:

```jsonc
{ "strategy": { "fallback": { "on_status_codes": [429,500,502,503] } },
  "targets": [ { "alias": "gpt-4o", "model": "gpt-4o" },
               { "alias": "claude", "model": "claude-3-5-sonnet" } ] }
```

`Fallback` is v1; `RoundRobin`, `Weighted`, `LoadBalance`, `Latency` become additive serde-tagged
variants. Each variant only customizes **which target(s) to try and in what order** — never the
forwarding or health code. Keep the door open to LiteLLM-style nesting later by making a target able
to *be* another model-router alias (recursion), which gives you load-balance-within + fallback-across
for free, exactly as Portkey does.

### The trait

The minimal interface both gateways converge on is: *given the candidate targets and shared health
state, decide the ordered set of targets to attempt; the engine executes and reports back outcomes.*
Two viable shapes:

**(a) Selector / iterator (recommended)** — strategy only chooses order; engine owns execution,
retries, streaming, health writes. Maps cleanly to Fallback (full ordered list), Weighted/RoundRobin
(one pick, or a randomized order), Latency/LeastBusy (sort by a metric the engine feeds in).

```rust
/// Read-only view the engine hands every strategy.
pub struct RoutingContext<'a> {
    pub targets: &'a [Target],          // shared ordered target list (alias + pinned model)
    pub health: &'a dyn HealthStore,    // shared cooldown/circuit state, strategy-agnostic
    pub request: &'a RoutingRequest,    // model, est. tokens, metadata — for conditional/latency
}

pub trait RoutingStrategy: Send + Sync {
    /// Ordered candidates to attempt. Fallback: all healthy in config order.
    /// Weighted/RoundRobin: a (possibly 1-element) weight-biased order.
    /// Latency/LeastBusy: sorted by the metric in `health`.
    fn plan(&self, ctx: &RoutingContext) -> Vec<TargetRef>;

    /// Whether to advance to the next candidate after an attempt outcome.
    /// Fallback uses on_status_codes; load-balance variants return false (no failover within node).
    fn should_failover(&self, outcome: &AttemptOutcome) -> bool { outcome.is_failure() }
}
```

The engine loop (shared, written once):

```
candidates = strategy.plan(ctx)        // health already filtered out cooled-down targets
for t in candidates:
    outcome = forward(t, request)      // shared forwarding incl. streaming passthrough
    health.record(t, &outcome)         // shared cooldown/circuit write — every strategy benefits
    if !strategy.should_failover(&outcome): return outcome.response
return last_outcome.response
```

**(b) Stateful observer + selector (LiteLLM-style)** — needed only when the metric must be
accumulated over time (latency/tpm/cost). Add an optional second trait the engine calls after every
attempt so metric-based strategies can update the shared store:

```rust
pub trait MetricObserver: Send + Sync {
    fn observe(&self, target: &TargetRef, outcome: &AttemptOutcome);  // writes into shared HealthStore
}
```

For v1 (`Fallback`) and the near-term `RoundRobin`/`Weighted`, trait (a) alone suffices —
none of them need history. Defer (b) until a latency/usage strategy lands.

### Shared services every strategy reuses (do NOT duplicate per strategy)

| Service | Portkey | LiteLLM | BodhiApp equivalent |
|---|---|---|---|
| Forwarding / execution primitive | `tryPost` (leaf arm) | per-deployment call inside fallback wrapper | one `forward(target, req)` the engine owns |
| Health / cooldown store (strategy-agnostic) | circuit-breaker `isOpen` prefilter + `handleCircuitBreakerResponse` | `CooldownCache` + `_filter_cooldown_deployments` (runs *before* strategy) | `HealthStore`: read = pre-filter candidates, write = `record()` after each attempt |
| Candidate list build + filter | `currentTarget.targets` (already filtered) | `async_get_healthy_deployments` | engine builds filtered `targets` before `plan()` |
| Config / param inheritance | `currentInheritedConfig` merge | `litellm_params` per deployment | resolve referenced alias + pinned model into a `Target` |
| Streaming | `tryPost` returns stream; failover is pre-first-byte | `stream_with_fallbacks` pre-first-byte | failover decision before first byte; once streaming, target owns response |

### Key takeaways for BodhiApp

1. **Strategy = data + one dispatch point.** A serde-tagged enum (`{"fallback": {...}}`) plus a
   `match` that returns the right `Box<dyn RoutingStrategy>`. New strategy = new enum arm + new impl;
   forwarding/health untouched. (Both gateways prove this.)
2. **Targets are shared and strategy-shape-independent.** One `targets[]` (alias + pinned model +
   optional `weight`) consumed by all strategies. `on_status_codes` belongs on the *strategy* node
   (Portkey), `weight` on the *target* (both).
3. **Health/cooldown is shared infra read *before* and written *after* strategy selection** — the
   strategy never owns it. This is what lets every strategy benefit from cooldown without code dup.
4. **Decide the load-balance-vs-fallback axis deliberately.** Recommend Portkey's unified enum +
   recursion (a target may itself be a model-router alias) so load-balance and fallback compose,
   rather than LiteLLM's two-mechanism split — it's a better fit for the single-`strategy`-field
   alias shape.
5. **Streaming failover is pre-first-byte only** in both. Don't design for mid-stream switching.
