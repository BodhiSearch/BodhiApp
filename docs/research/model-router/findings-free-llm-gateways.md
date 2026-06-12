# Fallback Routing & Health-Tracking Patterns in Free-LLM Gateways

Research to inform BodhiApp's "fallback API model" feature. Two repos studied because they
match the exact target use case: aggregate multiple FREE AI API providers and fail over
between them to dodge per-provider rate limits / token cost.

- `tashfeenahmed/freellmapi` — TypeScript / Express, SQLite-backed.
- `MrFadiAi/free-llm-gateway` — Python / FastAPI, YAML-config + SQLite logs.

Both converged on **nearly identical** in-memory passive-failover designs (same penalty
constants, same decay math). That convergence is itself a signal of what the minimal correct
design looks like.

---

## Repo 1 — `tashfeenahmed/freellmapi` (TypeScript / Express / SQLite)

Key files:
- `server/src/routes/proxy.ts` — request entry, retry loop, error classification, streaming, headers.
- `server/src/services/router.ts` — candidate selection, dynamic penalty tracker.
- `server/src/services/ratelimit.ts` — sliding-window rate limits + cooldown store.
- `server/src/services/health.ts` — background key validation / auto-disable.
- `server/src/routes/fallback.ts` — CRUD API for the ordered chain.

### 1. Fallback chain config
- Stored in a SQLite `fallback_config` table: columns `model_db_id`, `priority`, `enabled`
  (`router.ts:28-32`, query at `router.ts:138-142`).
- The chain is **a list of models** (each model row carries `platform`, `model_id`, rate
  limits, `intelligence_rank`, `speed_rank`); keys are separate (`api_keys` table).
- Runtime editing via REST: `GET/PUT /api/fallback` full-replace with
  `{modelDbId, priority, enabled}[]` (`fallback.ts:57-84`), plus
  `POST /api/fallback/sort/:preset` to re-rank by `intelligence | speed | budget`
  (`fallback.ts:88-114`).

### 2. Failure classification (`isRetryableError`, `proxy.ts:207-228`)
Matches on the lowercased error message (stringly-typed). **Retryable → fall to next entry:**
- 429 / "rate limit" / "too many requests" / "quota" / "resource_exhausted"
- "aborted" / "timeout" / "etimedout" / "econnrefused" / "econnreset" (connection/timeout)
- 503 / "unavailable", 500 / "internal server error" (5xx)
- 413 / "payload too large" (deliberately retryable — another provider may have a bigger limit)
- 404 / "not found" / "no endpoints found" (deprecated model → rotate away)
- `"api error 400"` ONLY (one provider rejects params another accepts)

**Non-retryable → returned immediately as HTTP 502:** 401/403 auth, generic 400 validation
(bare "400"), content-policy. Note: a bare schema-validation 400 short-circuits at the entry
(`proxy.ts:246-255`) before any provider is tried; an explicit user-requested model that's
unknown/disabled returns 400 `code: model_not_found` (`proxy.ts:315-325`).

### 3. Health / cooldown state — **in-memory + thin SQLite mirror**
Two independent mechanisms:

**(a) Dynamic penalty** (`router.ts:47-106`), pure in-memory `Map<modelDbId, {count, lastHit, penalty}>`:
- `PENALTY_PER_429 = 3`, `MAX_PENALTY = 10`.
- `recordRateLimitHit()` adds 3 (capped at 10); `recordSuccess()` subtracts 1 (deletes at 0).
- Time decay: `getPenalty()` subtracts `DECAY_AMOUNT=1` per `DECAY_INTERVAL_MS=2min` elapsed
  since `lastHit` (`router.ts:88-106`). This is **passive** — penalty is recomputed lazily on
  read, no background timer. Recovery is passive half-open: a demoted model is still tried once
  its effective priority floats back up, and a success clears the penalty.

**(b) Per-(platform,modelId,keyId) cooldown** (`ratelimit.ts:208-303`):
- In-memory `Map<key, expiryMs>` **mirrored** to SQLite `rate_limit_cooldowns(platform,
  model_id, key_id, expires_at_ms)` so it survives restart (`setCooldown`/`isOnCooldown`).
- **Escalating duration** keyed off a rolling 24h hit count (`getNextCooldownDuration`,
  `ratelimit.ts:219-234`): `[2min, 10min, 1h, 24h]` — so a daily-quota-exhausted key gets
  quarantined for the day instead of looping the 2-min cooldown 20x/request.
- `isOnCooldown` is **passive half-open**: it just checks `now > expiry`, deletes on expiry,
  no active probe.

**(c) Key validation health** (`health.ts`): a separate background `setInterval` every
`5min` calls `provider.validateKey()`. In-memory `Map<keyId,count>` of consecutive failures;
after `CONSECUTIVE_FAILURES_TO_DISABLE=3` confirmed 401/403, sets `api_keys.enabled=0`.
Transport errors set `status='error'` but do NOT count toward disable (`health.ts:42-50`).

### 4. Selection algorithm (`routeRequest`, `router.ts:134-236`)
1. Load chain ordered by base `priority`.
2. Compute `effectivePriority = priority + getPenalty(modelDbId)`, re-sort (`router.ts:144-148`).
3. **Sticky session**: if `preferredModelDbId` set, splice it to the front (`router.ts:150-157`).
4. For each model in order: skip if disabled / no provider / no healthy keys.
5. Among that model's keys, **round-robin** (`roundRobinIndex` Map, `router.ts:186-212`),
   skipping keys in `skipKeys` (failed this request), `isOnCooldown`, or over rate limit.
6. First key passing all checks wins. If chain exhausts → throw 429.
   Retry loop in proxy caps at `MAX_RETRIES = 20` (`proxy.ts:112, 334`).

### 5. Streaming (`proxy.ts:357-412`)
- Headers/body are set **lazily on first chunk** (`streamStarted` flag, `proxy.ts:371-378`).
- **Before first byte**: error bubbles to the outer retry loop → normal fallback (`proxy.ts:410-411`).
- **Mid-stream** (headers already sent): cannot retry. Emits an `error` SSE frame +
  `data: [DONE]`, logs full error, returns (`proxy.ts:397-409`). Client sees a generic
  `stream_error`, not provider internals.

### 6. Observability (`proxy.ts`)
- `X-Routed-Via: <platform>/<modelId>` on every success (`proxy.ts:375, 424`).
- `X-Fallback-Attempts: <n>` only when `attempt > 0` (`proxy.ts:376, 425`).
- Per-request row inserted into SQLite `requests` table (platform, model_id, key_id, status,
  input/output tokens, latency_ms, error) via `logRequest` (`proxy.ts:476-495`).
- Console log on each fallback hop (`proxy.ts:452`).

### 7. Data model
- `RouteResult` (`router.ts:34-42`): `{provider, modelId, modelDbId, apiKey, keyId, platform, displayName}`.
- Penalty record: `{count, lastHit, penalty}` keyed by `modelDbId`.
- Cooldown record: `expires_at_ms` keyed by `(platform, model_id, key_id)`; escalation hits
  tracked as `cooldownHits: Map<key, number[]>` (timestamps).

---

## Repo 2 — `MrFadiAi/free-llm-gateway` (Python / FastAPI / YAML)

Key files:
- `models.yaml` — declarative chain config.
- `config.py` — `ModelFallback`, `ModelConfig`, `ProviderConfig` dataclasses + YAML loader.
- `router.py` — `Router.route_request`, `PenaltyTracker`, candidate selection, retry/backoff.
- `rate_tracker.py` — `PerKeyRateTracker` (RPM/RPD/TPM/TPD + cooldown).
- `health.py` — `HealthChecker` (provider ping + per-key validation, circuit-breaker-ish).
- `sticky_sessions.py` — `StickySessionManager`.
- `providers.py` — `ProviderError(status, message, retry_after)`.
- `main.py` — FastAPI endpoint, streaming wrap, headers.

### 1. Fallback chain config — **declarative YAML**
- `models.yaml`: each unified model name maps to `capabilities` + an ordered `fallbacks:` list
  of `{provider, model}` (and optional per-entry `enabled`, `rpm_limit/rpd_limit/tpm_limit/tpd_limit`).
  Example: `llama-3.3-70b` → openrouter, then nvidia, then cerebras.
- Loaded into `ModelConfig{unified_name, fallbacks: list[ModelFallback], capabilities,
  intelligence_rank, speed_rank, monthly_token_budget, ...}` (`config.py:97-107`).
- `ModelFallback{provider, model, enabled=True, rpm/rpd/tpm/tpd_limit=0}` (`config.py:79-87`).
- Editing = edit YAML + reload (there is an `auto_update.py` / `sync_providers.py` to
  auto-sync from `awesome-free-llm-apis`). Less runtime-dynamic than repo 1.

### 2. Failure classification (`router.py:331-429`) — **status-code based** (cleaner than repo 1)
Errors arrive as typed `ProviderError(status, message, retry_after)` (`providers.py:19-32`;
raised per status at `providers.py:95-99` etc.). In `route_request`:
- `status == 0` (timeout): break to **next provider immediately** (`router.py:336-345`).
- `status == 429`: if `retry_after` present and attempts remain → `asyncio.sleep(retry_after)`
  and **retry same provider**; else rotate key, `penalty_tracker.record_hit`,
  `set_cooldown`, move to next (`router.py:348-386`).
- `status in (500,502,503)`: **retry same provider** with exponential backoff
  `RETRY_BACKOFF_BASE * 2**attempt`, up to `RETRY_MAX_ATTEMPTS=2` (`router.py:389-405`).
- `status in (401,403)`: rotate to next key if multiple, then fall through to next provider
  (`router.py:408-413`).
- All others: log, move to next provider (`router.py:415-429`).
- If every failure was a 429 → raises `AllRateLimitedError` (→ request queue), else `RuntimeError`.

Note: this design distinguishes **retry-same-provider** (429 w/ Retry-After, 5xx backoff)
from **fall-to-next-provider** (timeout, exhausted 429, auth, other) — a two-axis policy repo 1
collapses into one.

### 3. Health / cooldown state — in-memory, three layers

**(a) `PenaltyTracker`** (`router.py:36-100`) — identical design/constants to repo 1:
`PENALTY_PER_429=3`, `MAX_PENALTY=10`, `DECAY_INTERVAL_S=120`, `DECAY_AMOUNT=1`. Keyed by
`"provider:model"`, fields `{count, last_hit, penalty}`, lazy decay on read. Passive recovery.

**(b) `PerKeyRateTracker`** (`rate_tracker.py`) — in-memory, thread-safe (`Lock`):
- `RateBucket{requests, tokens, timestamps[], token_timestamps[]}` sliding windows
  (MINUTE=60, DAY=86400). Key = `provider:model:key_suffix(last 8 chars)` (`rate_tracker.py:90-94`).
- Cooldown: `_cooldowns: dict[key,float]`, `set_cooldown(DEFAULT_COOLDOWN_S=30.0)`,
  `is_on_cooldown` passive expiry check (`rate_tracker.py:159-187`). **Flat 30s, no escalation**
  (simpler than repo 1's `[2m,10m,1h,24h]`).
- `would_exceed_token_limit()` pre-checks TPM/TPD with an *estimated* token count BEFORE
  routing (`rate_tracker.py:189-222`) — proactive avoidance, not just reactive cooldown.
- Hard-coded `PROVIDER_FREE_LIMITS` table of known free-tier RPM/RPD/TPM per provider
  (`rate_tracker.py:315-359`).

**(c) `HealthChecker`** (`health.py`) — closest thing to a real circuit breaker:
- `ProviderHealth{status: up|down|unknown, last_check_time, last_error, consecutive_failures,
  latency_ms}` (`health.py:23-29`).
- `KeyHealth{key_index, status: healthy|invalid|error|rate_limited|unknown,
  consecutive_failures, last_checked, last_error, disabled}` (`health.py:32-40`).
- Background loop `CHECK_INTERVAL=180s`, alternating provider-ping vs per-key-validate cycles
  (`health.py:357-377`). `DOWN_THRESHOLD=2` consecutive failures → status `down`;
  `is_available()` = passive half-open: a `down` provider is retried after
  `COOLDOWN_SECONDS=300` (`health.py:89-98`).
- Per-key: 401/403 → `invalid`, after `CONSECUTIVE_FAILURES_TO_DISABLE=3` → `disabled`
  (added to `provider.disabled_keys`); 429 → `rate_limited` (does NOT count toward disable);
  5xx/transport → `error` (also no disable). **Auto re-enable** when a disabled key validates
  healthy again (`health.py:180-184`) — active-probe recovery for keys.

### 4. Selection algorithm (`_select_provider`, `router.py:164-244`)
1. Walk `fallbacks` in declared order; drop entries where: `not enabled`, no api_key, provider
   `rate_limiter.is_limited()`, `health_checker.is_available()==False`, per-key
   rate-limited, would-exceed-token, or `is_on_cooldown`.
2. Surviving candidates **sorted by `penalty_tracker.get_penalty`** (`router.py:231-237`).
3. **Round-robin** among equal-penalty candidates via `_rr_index[model]` rotation
   (`router.py:240-242`).
4. Sticky: applied in `main.py` BEFORE routing — `sticky_sessions.get(conversation_id)`
   returns a preferred `(provider, model)` (`main.py:358-363`), recorded after success
   (`main.py:382-384`).
5. Retry loop tries `candidates[:MAX_RETRIES=10]`, inner `RETRY_MAX_ATTEMPTS+1` per candidate.

### 5. Streaming (`main.py:421-434`, `providers.py:104-123`)
- Streaming is selected per-request; `route_request` returns an async generator.
- The provider opens `client.stream(...)` and raises `ProviderError` on 429/5xx **at stream
  open, before yielding** (`providers.py:112-117`) — so pre-first-byte failures DO trigger
  router fallback.
- Once chunks flow, `safe_stream()` (`main.py:219-242`) only catches mid-stream breaks and
  emits an SSE `stream_error` frame — **no mid-stream fallback** (same as repo 1).

### 6. Observability (`main.py:415-443`)
- `X-Routed-Via: <provider>/<provider_model>`, `X-Fallback-Attempts`, `X-Sticky-Session`,
  `X-Provider`, `X-Provider-Model` headers on both streaming and JSON responses.
- In-memory ring buffer `_logs` (max 100) + persisted `request_db.log_request` with
  `RequestLog{timestamp, model, provider, provider_model, success, error, latency_ms,
  tokens, attempt}` (`router.py:103-114, 246-261`).

### 7. Data model
- Chain: `ModelConfig.fallbacks: list[ModelFallback{provider, model, enabled, *_limit}]`.
- Penalty: `{count, last_hit, penalty}` keyed `provider:model`.
- Cooldown: `dict[provider:model:key_suffix, expiry_epoch]` (flat 30s).
- Health: `ProviderHealth` + `KeyHealth` dataclasses above.
- `StickySessionManager`: `SessionEntry{conversation_id, provider, model, created_at,
  last_used_at, request_count, ttl=1800s}`, conversation id = hash of first user message
  (`sticky_sessions.py:186-212`).

---

## Patterns worth borrowing for a minimal in-memory passive-failover design

BodhiApp's target is **simple in-memory, passive-cooldown, no-stickiness**. Distilling both
repos to that constraint:

1. **Chain = ordered list of endpoints, config-driven.** Repo 2's declarative YAML
   (`ModelFallback{provider, model}` list per unified name) is the cleaner fit than repo 1's
   DB table when no runtime UI editing is required. For Rust: a `Vec<FallbackEntry>` in the
   model/API-model config, ordered = priority.

2. **Classify on a typed error, not a stringly message.** Repo 2's `ProviderError{status,
   retry_after}` is the model to copy; repo 1's substring matching is fragile. Map to a Rust
   enum. Minimal policy for passive failover:
   - **Fall to next endpoint:** connection error, timeout, 429, 5xx (502/503/500), 404
     model-not-found. (You can skip repo 2's "retry same provider with backoff" entirely for a
     minimal design — just advance to the next entry.)
   - **Return immediately (no fallover):** 401/403 auth, 400 validation, content-policy. These
     won't be fixed by another provider and signal a config/client bug.

3. **One in-memory cooldown map is enough.** `HashMap<EndpointId, Instant /* cooldown_until */>`.
   On a fall-to-next error, `cooldown_until = now + COOLDOWN`. Selection skips any endpoint with
   `now < cooldown_until`. **Passive half-open**: no probe, no background timer — just compare
   `now` on read and let it lapse. Both repos do exactly this; it is the single most important
   primitive. (`ratelimit.ts:282-303`, `rate_tracker.py:174-187`.)

4. **Flat cooldown beats escalation for v1.** Repo 2 uses a flat `30s`; that's the minimal
   correct thing. Repo 1's `[2m,10m,1h,24h]` escalation is a worthwhile *later* refinement
   specifically to handle daily-quota exhaustion without burning every fallback slot — note it
   as a known follow-up, not v1.

5. **Skip the penalty-score reordering for a no-stickiness design.** The
   `penalty + decay` reranker (identical in both repos) only matters when you want a demoted
   endpoint to keep its *relative* position and float back. With a plain ordered chain +
   binary cooldown skip, you get equivalent passive failover with far less state. Drop it for v1.

6. **Drop sticky sessions** — explicitly out of scope per the target. Removing it also removes
   the conversation-hashing and TTL-map machinery (`stickySessionMap`, `StickySessionManager`).

7. **Streaming rule both repos share, and BodhiApp should adopt verbatim:** set response
   headers/status **lazily on first byte**. Fallback is only possible **before the first byte**;
   once a chunk is forwarded, a mid-stream failure can only emit a terminal error SSE frame and
   close — never silently truncate. (`proxy.ts:357-412`, `main.py:219-242`.)

8. **Observability is two cheap headers:** `X-Routed-Via: provider/model` and
   `X-Fallback-Attempts: n`. Both repos expose exactly these; trivially worth adding.

9. **Token/RPM pre-checks are optional polish.** Repo 2's `would_exceed_token_limit` proactive
   skip is nice but orthogonal to failover correctness — defer.

### Suggested minimal Rust shape
```
struct FallbackEntry { provider: ProviderId, model: String }      // ordered Vec = config
enum RouteError { Connection, Timeout, RateLimited, ServerError(u16), ModelNotFound, // -> next
                  Auth, BadRequest, ContentPolicy }                                  // -> return
struct CooldownMap(HashMap<EndpointId, Instant>);  // cooldown_until; skip if now < it; passive lapse
const COOLDOWN: Duration = 30s;
```
Select = first entry not in cooldown; on a "→ next" error set its cooldown and continue;
on a "→ return" error surface upstream status verbatim; success clears nothing (cooldown just
lapses). Headers `X-Routed-Via` / `X-Fallback-Attempts`. Lazy-header streaming, no mid-stream
fallback. That is the entire feature.
