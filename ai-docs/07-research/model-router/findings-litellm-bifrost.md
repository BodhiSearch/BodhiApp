# Fallback Routing & Health/Cooldown Patterns: LiteLLM & Bifrost

Research to inform BodhiApp's "fallback API model" feature. Target design: **in-memory health state only, passive failover with cooldown + half-open recovery, no background probes, no stickiness.** Each finding is tagged **[BORROW]** or **[SKIP for v1]**.

Repos studied (source-read + DeepWiki):
- `BerriAI/litellm` (Python) — `litellm/router.py`, `litellm/router_utils/`, `litellm/utils.py`, `litellm/constants.py`
- `maximhq/bifrost` (Go) — `core/bifrost.go`, `core/utils.go`, `core/schemas/`

---

## 1. LiteLLM

LiteLLM's Router has **two nested loops**: retries (same model group, re-pick a deployment) wrapped by fallbacks (different model group). Cooldown is a separate, orthogonal layer that *filters the candidate pool* before each selection.

### 1.1 Fallback config model

`fallbacks` is a list of single-key dicts mapping a **model group** to an ordered list of fallback **model groups** (not individual deployments):

```python
fallbacks = [{"gpt-3.5-turbo": ["gpt-4"]}, {"gpt-4o": ["gpt-3.5-turbo"]}]
```
- Lookup: `_get_fallback_model_group_from_fallbacks()` — `router.py:7024-7047`. Linear scan, first-match wins.
- `"*"` wildcard entry = default fallbacks (`_get_first_default_fallback`, `router.py:7049-7060`).
- Three separate lists: `fallbacks` (general), `context_window_fallbacks`, `content_policy_fallbacks` — read in `async_function_with_fallbacks`, `router.py:6603-6609`.
- Underlying `model_list` entries carry `litellm_params` (incl. `weight`/`rpm`/`tpm`) used only by selection, not fallback ordering.

### 1.2 Failure classification (retry vs fallback vs immediate error)

**Default retryable status codes** — `_should_retry(status_code)` in `utils.py:6844-6870`:
- `408` (timeout), `409` (lock conflict), `429` (rate limit), `>=500`.
- Everything else in 400-499 (400/401/403/404) is **not** retryable by default.

**`should_retry_this_error()`** — `router.py:6944-7014` — decides retry-same-group vs raise-to-fallback:
- `ContextWindowExceededError` + `context_window_fallbacks` set -> raise (go to fallback).
- `ContentPolicyViolationError` + `content_policy_fallbacks` set -> raise (go to fallback).
- status not in `_should_retry` **and not 401/403** -> raise immediately.
- `NotFoundError` -> raise.
- `RateLimitError` + **no healthy deployments left in group** + fallbacks available -> raise (fall back).
- `AuthenticationError` -> retry **only if >1 deployment** in the group, else raise.
- `_num_healthy_deployments <= 0` -> raise.
- otherwise -> `True` (retry on another deployment in same group).

Key insight: **401/403 are special-cased** — normally terminal, but retried across deployments if the group has more than one. Retry backoff via `_time_to_sleep_before_retry` honors `Retry-After` header.

### 1.3 Cooldown / health mechanics (the core to copy)

State lives in `CooldownCache` (`router_utils/cooldown_cache.py`) wrapping a `DualCache` (in-memory **or** Redis; Redis only for multi-worker HA).

**Cooldown entry** (`CooldownCacheValue`, `cooldown_cache.py:24-28`): `{exception_received, status_code, timestamp, cooldown_time}`. Key = `deployment:{model_id}:cooldown`. **The TTL on the cache entry *is* the cooldown** — `add_deployment_to_cooldown` sets `ttl=cooldown_time` (`cooldown_cache.py:94-98`). Recovery is purely passive TTL expiry; an entry simply stops being returned by `get_active_cooldowns`. **No background probe, no explicit "recovering" state.**

**Should-cooldown decision** — `_should_cooldown_deployment`, `cooldown_handlers.py:166-257` (v2 logic):
1. `429` and group has >1 deployment -> cooldown.
2. `percent_fails == 1.0` AND `total_requests >= SINGLE_DEPLOYMENT_TRAFFIC_FAILURE_THRESHOLD (1000)` -> cooldown.
3. `percent_fails > DEFAULT_FAILURE_THRESHOLD_PERCENT (0.5)` AND `total_requests >= DEFAULT_FAILURE_THRESHOLD_MINIMUM_REQUESTS (5)` AND not single-deployment group -> cooldown.
4. `litellm._should_retry(status) is False` (i.e. a terminal error like 401/404) -> cooldown.

**`_is_cooldown_required`** (`cooldown_handlers.py:40-95`) gates *which statuses* even count: cools 429, 401, 408, 404; **ignores** other 4xx; ignores `APIConnectionError` strings; cools all 5xx.

**Counters** (per-deployment, per-current-minute, in-memory `InMemoryCache`): `deployment_id:successes`, `deployment_id:fails`. Percent-fail is computed from these.

**Legacy v1 path** (only active if `allowed_fails` / `allowed_fails_policy` explicitly set) — `should_cooldown_based_on_allowed_fails_policy`, `cooldown_handlers.py:398-430`: simple counter — `failed_calls[deployment] += 1`; if `updated_fails > allowed_fails` -> cooldown. The fail counter itself has `ttl=cooldown_time`, so it self-resets. `allowed_fails_policy` lets you set per-exception-type thresholds (e.g. different count for `TimeoutError`).

Constants (`constants.py`, all env-overridable):
- `DEFAULT_COOLDOWN_TIME_SECONDS = 5`
- `DEFAULT_ALLOWED_FAILS = 3`
- `DEFAULT_FAILURE_THRESHOLD_PERCENT = 0.5`
- `DEFAULT_FAILURE_THRESHOLD_MINIMUM_REQUESTS = 5`
- `SINGLE_DEPLOYMENT_TRAFFIC_FAILURE_THRESHOLD = 1000`

`cooldown_time` can be overridden dynamically from a provider's `Retry-After` header.

**Single-deployment groups are deliberately NOT cooled down** for percent-based reasons (you'd cool down your only option). Worth noting for BodhiApp.

### 1.4 Selection algorithm

Cooldown deployments are excluded from the candidate set *before* the routing strategy runs (`_async_get_cooldown_deployments` feeds the healthy-deployment filter, `router.py:6276-6279`). Strategies live in `router_strategy/`: simple-shuffle (default), least-busy, lowest-latency, lowest-cost, lowest-tpm-rpm.

**Simplest = `simple-shuffle`** (`router_strategy/simple_shuffle.py`): if no weights, `random.choice(healthy_deployments)`; if `weight`/`rpm`/`tpm` present, weighted random pick. ~70 lines, no state. This is the model BodhiApp should copy (or even simpler: ordered list, first non-cooled).

### 1.5 Recovery

**Purely passive.** Cooldown entry expires via cache TTL; no `/health` endpoint polling, no background task in the routing hot path. (LiteLLM has separate, opt-in health-check endpoints for the admin UI, unrelated to routing.) There is no explicit half-open state — the *first request after TTL expiry* implicitly probes; if it fails again, the cooldown logic re-applies. This is effectively half-open-by-default.

### 1.6 Streaming fallback

Streaming goes through the same `async_function_with_fallbacks` wrapper, so fallback/retry apply identically — **but only to errors raised before the stream starts**. Once tokens flow, mid-stream failures are not re-routed.

### 1.7 Observability

Response headers via hidden params (`router_utils/add_retry_fallback_headers.py`):
- `x-litellm-attempted-retries`, `x-litellm-max-retries`
- `x-litellm-attempted-fallbacks` (intentionally no max-fallbacks, to keep headers small).

---

## 2. Bifrost

Bifrost's open-source core is **strikingly minimal** — and the most relevant comparison for BodhiApp's "deliberately simple" goal. The fancy health/circuit-breaker ("Healthy/Degraded/Failed/Recovering" states, adaptive scoring, smart-exploration probes) is an **Enterprise/closed feature** — only described in docs, **not in the open-source Go code**. The OSS core has **no cooldown and no circuit breaker at all.**

### 2.1 Fallback config model

`Fallback` struct (`core/schemas/bifrost.go:408-411`) — a flat ordered list on the request itself:
```go
type Fallback struct {
    Provider ModelProvider `json:"provider"`
    Model    string        `json:"model"`
}
```
Each fallback names an explicit `(provider, model)` pair. Tried in array order — no scoring in OSS.

### 2.2 Failover orchestration

`handleRequest` (`core/bifrost.go:4526+`): try primary via `tryRequest`; if it errors and `shouldTryFallbacks` is true, loop fallbacks in order, calling `tryRequest` on each; return first success or the **primary** error if all fail.

`shouldTryFallbacks` (`bifrost.go:4386-4416`): stop if no error, request cancelled, `AllowFallbacks == &false` (plugin short-circuit), or no fallbacks configured. `AllowFallbacks == nil` => treated as **true** (fail-open). `shouldContinueWithFallbacks` (`4506-4520`) skips further fallbacks on cancellation / explicit no-fallback.

### 2.3 Retry vs fallback (failure classification)

Per-attempt retries (same provider) live in `executeRequestWithRetries` (`bifrost.go:5230-5585`). Loop bound: `attempts <= NetworkConfig.MaxRetries`. **`DefaultMaxRetries = 0`** (`schemas/provider.go:13`) — so **OSS Bifrost does no same-provider retry unless explicitly configured**; failover-to-next is the primary resilience mechanism.

Retryable set (`core/utils.go:23-29`):
```go
retryableStatusCodes = {500, 502, 503, 504, 429}
```
Plus network/transport errors (`ErrProviderDoRequest`, `ErrProviderNetworkError`) -> retry. Rate-limit detection is **dual**: status `429` OR message/type/code matching `rateLimitPatterns` (`"rate limit"`, `"quota exceeded"`, `"throttled"`, `"tpm exceeded"`, ..., `utils.go:32+`) — this catches providers that return 200 + SSE error or non-standard codes.

Terminal (no retry, break loop): `IsBifrostError` (validation/internal), `RequestCancelled`, or any status not in the retryable set.

### 2.4 Key rotation (the one clever-but-simple bit)

`executeRequestWithRetries` distinguishes two retry causes (`bifrost.go:5258-5260`, `5561-5569`):
- **Rate-limit (429/pattern)** -> mark current key used (`usedKeyIDs[currentKey.ID]=true`), **rotate to a different key** next attempt (a different key may have capacity). When all keys exhausted, the set resets for a fresh weighted round.
- **Network/5xx** -> **keep the same key** (transient server issue, not per-key).

Key selection: `core/keyselectors/weightedrandom.go` — weighted random, excluding `usedKeyIDs`. This is the OSS-level "load balancer"; the adaptive scorer is Enterprise.

### 2.5 Health / cooldown / recovery

**None in OSS.** No persistent per-provider/per-key health record, no cooldown TTL, no circuit breaker. Failover is **stateless and per-request**: every request re-evaluates the ordered fallback list from scratch. A provider that just failed is tried again on the very next request. Recovery is therefore trivially immediate (and naive — a down provider is retried every request, paying the primary-failure latency each time).

### 2.6 Streaming fallback

Handled explicitly (`bifrost.go:5456-5466`): on an HTTP-200 stream, Bifrost inspects the **first chunk** via `CheckFirstStreamChunkForError`; if it's an embedded error (e.g. rate-limit-as-SSE), it's promoted to a `bifrostError` so retry/fallback logic applies. After the first good chunk, mid-stream failures are not re-routed (same limitation as LiteLLM).

### 2.7 Observability

`attempt_trail` — a `[]KeyAttemptRecord` ({Attempt, KeyID, KeyName, FailReason}) accumulated in context across every attempt (`bifrost.go:5306-5315`, `5544-5555`), plus OTel span attributes per attempt: `fallback.index`, retry count, selected key id/name. Surfaced in observability logs, not response headers.

---

## 3. Minimal subset worth borrowing vs. skip for BodhiApp v1

BodhiApp wants: in-memory passive cooldown + half-open recovery, ordered fallback, no probes, no stickiness. The sweet spot sits **between** the two: LiteLLM's cooldown TTL idea + simple counter, with Bifrost's flat ordered list and stateless simplicity.

### BORROW (essential, cheap, directly maps to the spec)

1. **Cooldown = TTL on an in-memory entry** (LiteLLM `cooldown_cache.py:94-98`). One `HashMap<ModelId, Instant /* cooldown_until */>`. A model is "in cooldown" iff `now < cooldown_until`. **Expiry == recovery == half-open** — the first request after expiry naturally probes; if it fails, re-cooldown. This is exactly BodhiApp's "passive failover + half-open" with zero extra machinery.
2. **Simple fail-counter threshold, not percentage** (LiteLLM v1 `should_cooldown_based_on_allowed_fails_policy`, `cooldown_handlers.py:420-428`): `consecutive_fails >= allowed_fails (default 3)` -> cooldown. Counter TTL'd / reset on success. Skip the percent-fails-per-minute machinery (needs success+fail counters and traffic minimums — over-engineered for v1).
3. **Failure classification table** (union of both, conservative):
   - **Cooldown + fall to next**: `429`, `5xx` (500/502/503/504), connection/timeout errors, `401`/`403` (terminal auth — bad key, no point retrying soon).
   - **Immediate error, no cooldown, no fallback**: `400`, `404`, `422`, context-window/content-policy (client's fault — failing over won't help). Mirrors LiteLLM `_is_cooldown_required` (`cooldown_handlers.py:70-91`) and Bifrost `retryableStatusCodes`.
   - Treat connection-refused / DNS / timeout as cooldown-worthy (Bifrost does; LiteLLM ignores `APIConnectionError` for cooldown — BodhiApp targeting *local + remote* models should cool a dead upstream).
4. **Rate-limit message-sniffing fallback** (Bifrost `rateLimitPatterns`, `utils.go:32+`): when a provider returns 200/non-standard code but the body says "rate limit"/"quota exceeded", treat as 429. Cheap, high-value for heterogeneous providers.
5. **Ordered, explicit fallback list** (Bifrost `Fallback{provider, model}`, flat array). Simpler than LiteLLM's model-group indirection. First non-cooled entry wins. No weighted/stochastic selection.
6. **Observability**: response header `x-bodhi-attempted-fallbacks` + chosen model id (LiteLLM headers pattern). Optionally an in-memory attempt trail for logs (Bifrost `KeyAttemptRecord`).
7. **`cooldown_time` from `Retry-After`** when present (LiteLLM) — trivial and respectful of upstream.
8. **Don't cooldown the only/last option** (LiteLLM single-deployment guard): if all fallbacks are cooled, still attempt the primary rather than hard-fail.

### SKIP for v1 (over-engineered)

- **Redis / distributed cooldown** (LiteLLM `DualCache`) — BodhiApp v1 is single-process; in-memory only.
- **Percent-fail-per-minute + traffic thresholds** (`DEFAULT_FAILURE_THRESHOLD_PERCENT`, `SINGLE_DEPLOYMENT_TRAFFIC_FAILURE_THRESHOLD=1000`) — needs success counters and meaningful traffic; pointless at low volume. Use a flat consecutive-fail count.
- **Active health-check endpoints / background probes** — neither uses them in the routing hot path; recovery is passive TTL. Skip entirely.
- **Adaptive/scored load balancing, "Degraded/Recovering" states, smart-exploration probes** (Bifrost Enterprise) — not even in Bifrost OSS. Pure over-engineering for v1.
- **Multiple routing strategies** (least-busy, lowest-latency, lowest-cost) — ordered list is enough.
- **Stickiness / session affinity** — explicitly unwanted.
- **`allowed_fails_policy` per-exception-type thresholds** (LiteLLM) — premature; one global `allowed_fails` is fine.
- **Same-deployment retry loop with backoff** — optional. Bifrost OSS defaults `MaxRetries=0` and relies on failover; BodhiApp can do the same (fall straight to next model) for a simpler v1, adding at most 1 same-model retry on 429/5xx if desired.
- **Weighted key rotation** (Bifrost) — only relevant with multiple keys per provider; defer.

### Suggested minimal data model for BodhiApp

```rust
struct HealthState {            // one per fallback target, in-memory
    consecutive_fails: u32,
    cooldown_until: Option<Instant>,
}
// config: ordered Vec<ModelTarget>; allowed_fails (default 3); cooldown (default ~5-30s, or Retry-After)
// on failure(classified as cooldown-worthy): fails+=1; if fails>=allowed_fails { cooldown_until = now+cooldown; fails=0 }
// on success: reset state
// selection: first target where cooldown_until is None or expired; if none, use primary anyway
```
