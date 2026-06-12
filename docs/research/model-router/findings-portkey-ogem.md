# Model Router Research: Portkey AI Gateway & Ogem

Research date: 2026-05-28. Goal: inform a SIMPLE "fallback API model" feature in BodhiApp
(in-memory health state, passive failover with cooldown + half-open recovery, no background
probes, no stickiness). For each gateway, note what is essential vs over-engineered for a minimal v1.

Repos:
- `Portkey-AI/gateway` (TypeScript) — `/Users/amir36/Documents/workspace/src/github.com/Portkey-AI/gateway`
- `yanolja/ogem` (Go) — `/Users/amir36/Documents/workspace/src/github.com/yanolja/ogem`

---

## 1. Portkey AI Gateway (TypeScript)

Portkey is a **stateless, per-request, declarative router**. The entire fallback/loadbalance/conditional
behavior is driven by a JSON `Config` object attached to the request (header `x-portkey-config` or gateway
config store). There is **no persistent per-target health state** in the OSS core — every request walks the
target tree fresh. (A `healthyTargets`/`isOpen` circuit-breaker concept exists but is an enterprise hook,
see below.)

### 1.1 Declarative config schema

Defined as a recursive Zod schema in
`src/middlewares/requestValidator/schema/config.ts:12-191`. The tree is self-similar: every node is a
`Config`, and `targets` is `z.array(z.lazy(() => configSchema))` (line 79) — so strategies nest arbitrarily.

Key fields (a leaf node is a provider; an internal node is a strategy + targets):

```jsonc
{
  "strategy": {
    "mode": "fallback",            // single | loadbalance | fallback | conditional  (line 16-27)
    "on_status_codes": [429, 500], // optional number[]; codes that trigger moving to next target (line 28)
    "conditions": [ { "query": {}, "then": "targetName" } ], // conditional mode only (line 29-36)
    "default": "targetName"        // conditional fallback (line 37)
  },
  "targets": [                     // array of nested Configs (line 79)
    {
      "provider": "openai",        // leaf (line 40-47)
      "api_key": "...",            // (line 48)
      "weight": 1,                 // loadbalance weight (line 77)
      "override_params": { "model": "gpt-4o" },
      "retry": {                   // per-target retry, runs BEFORE fallback (line 67-76)
        "attempts": 5,
        "on_status_codes": [429, 500, 502, 503, 504],
        "use_retry_after_header": true
      },
      "cache": { "mode": "simple", "max_age": 3600 }, // (line 53-66)
      "request_timeout": 30000     // ms (line 80)
    },
    { "provider": "anthropic", "api_key": "...", "override_params": { "model": "claude-3-5-sonnet" } }
  ]
}
```

Config is also valid as a single provider (`provider` + `api_key`) with no strategy — the validator's
top-level `.refine` (lines 128-166) requires either `provider+api_key` OR `strategy+targets` OR
`cache`/`retry`/`request_timeout`. So "no fallback" is just a degenerate config.

### 1.2 Fallback trigger — which codes move to the next target

The core is `tryTargetsRecursively` in `src/handlers/handlerUtils.ts:476`. The `FALLBACK` branch
(lines 663-691) iterates `targets` in array order and **breaks** (stops fallback) when any of:

```ts
const codes = currentTarget.strategy?.onStatusCodes;
const gatewayException = response?.headers.get('x-portkey-gateway-exception') === 'true';
if (
  (Array.isArray(codes) && !codes.includes(response?.status)) || // on_status_codes given, status NOT in list
  (!codes && response?.ok) ||                                    // no on_status_codes, response is 2xx
  gatewayException                                               // internal gateway error, don't mask it
) {
  break; // stop, keep this response
}
// else: continue to next target
```

Semantics:
- **Default (no `on_status_codes`):** fall back on **any non-2xx** (`!response.ok`). Simple "anything that
  isn't success triggers next provider."
- **With `on_status_codes`:** fall back **only** on the listed codes; any other status (including some
  errors) is treated as final. This is the lever for "fail over on 429/5xx, but surface 400 verbatim."
- `x-portkey-gateway-exception` always halts fallback (don't retry an internal gateway bug across providers).

Default retry status codes (the retry layer, distinct from fallback): `RETRY_STATUS_CODES = [429, 500, 502,
503, 504]` in `src/globals.ts:38`.

### 1.3 Retry (inner loop, before fallback)

`src/handlers/retryHandler.ts:65` `retryRequest(...)` uses `async-retry` with exponential backoff.
- Retries while `statusCodesToRetry.includes(response.status)` (line 103).
- `429 + use_retry_after_header` honors `Retry-After`/provider rate-limit headers
  (`POSSIBLE_RETRY_STATUS_HEADERS`), capped by `MAX_RETRY_LIMIT_MS` (lines 108-152).
- Connection-unreachable -> synthesized `503`; unknown error -> `500` (lines 182-212).

Ordering: **retry exhausts on a single target first, then fallback advances to the next target.**

### 1.4 Health state

OSS core is **stateless**. The only stateful hook: in `tryTargetsRecursively` (lines 646-658), if an
`id` is present (`isHandlingCircuitBreaker`), it filters `currentTarget.targets` to those without
`isOpen`, and after a leaf request calls `c.get('handleCircuitBreakerResponse')?.(...)` (lines 792-800).
But `handleCircuitBreakerResponse`, `isOpen`, and `cbConfig` are **injected externally** (enterprise/host
app) — there is no circuit-breaker state machine, timer, or store in this repo. Confirmed: `grep` finds no
`CircuitBreaker` class, only these reference points in `handlerUtils.ts`.

So: per-request fallback is the whole OSS reliability story; durable health is left to the host.

### 1.5 Selection algorithm

- `fallback`: strict **array order**, first target first (lines 664-690).
- `loadbalance`: **weighted random** — sum `weight` (default 1), pick `Math.random()*total` (lines 693-722).
  No latency, no health, no stickiness.
- `conditional`: JMESPath-style rules on metadata/params via `ConditionalRouter` (lines 725-765).
- `single`: always `targets[0]` (lines 767-779).

### 1.6 Streaming fallback

Streaming is handled inside `tryPost`/stream handlers, not a special fallback path. Because fallback keys
off the **response status before the body is consumed**, a stream that returns a non-2xx status can still
fall over. Once a 2xx stream has started, Portkey commits to it (no mid-stream re-route). For v1 this maps
to: decide failover on the response status/headers, never mid-stream.

### 1.7 Observability

`src/handlers/services/responseService.ts:99-124` `updateHeaders(...)` appends on every response:
- `x-portkey-last-used-option-index` — which target index served the request (`this.context.index`).
- `x-portkey-trace-id`.
- `x-portkey-retry-attempt-count` — retries used.
- `x-portkey-cache-status`, and provider name (`HEADER_KEYS.PROVIDER`).

Header keys in `src/globals.ts:19,31-35`. This "which target won + how many tries" pattern is cheap and
worth copying.

---

## 2. Ogem (Go)

Ogem is a multi-provider/multi-region proxy. It has **in-memory endpoint state**, but the state is only
**latency** (for sorting), refreshed by an **optional active ping loop**. Failover itself is **reactive**:
the per-request loop tries the next endpoint when one errors or is rate-limited. There is **no up/down
boolean and no cooldown** — a "down" endpoint just keeps a stale latency and is still attempted.

### 2.1 Config schema

`config/config.go:19-77`, loaded from `config.yaml` or env. Top-level fields:

```yaml
valkey_endpoint: ""          # optional Redis/Valkey for distributed rate-limit + cache (else in-memory)
retry_interval: 1m           # wait before re-scanning when ALL endpoints are rate-limited (ParseDuration)
ping_interval: 1h            # health/latency ping cadence; 0 = DISABLED (shipped config.yaml uses 0)
port: 8080
providers:                   # ogem.ProvidersStatus = map[provider] -> ProviderStatus
  openai:
    base_url: ...
    protocol: openai
    api_key_env: OPENAI_API_KEY
    regions:
      openai:                # region "default" = provider-wide settings
        models:
          - name: "gpt-4o"
            rpm: 10000
            tpm: 30000000
routing: {}                  # optional advanced/intelligent router (metrics-driven)
```

In-memory status structs (`ogem.go`):
- `ProvidersStatus = map[string]*ProviderStatus` (line 9)
- `ProviderStatus{ BaseUrl, Protocol, ApiKeyEnv, Regions map[string]*RegionStatus }` (11-25)
- `RegionStatus{ Models, Latency time.Duration, LastChecked time.Time }` (27-38) — **state = latency +
  last-checked timestamp only. No "healthy" flag.**

### 2.2 Fallback chains — comma-separated models

In `server/server.go` `HandleChatCompletions`, the request `model` is split on commas
(`models := strings.Split(openAiRequest.Model, ",")`, line 307) and tried **in order** (lines 319-331):

```go
for index, model := range models {
    openAiResponse, err = s.generateChatCompletion(ctx, &req, index == lastIndex)
    if err != nil { lastError = err; continue }       // model failed -> next model in chain
    if openAiResponse.Choices[0].FinishReason == "stop" { break }
}
if openAiResponse == nil { handleError(w, lastError) } // all failed -> last error
```

So the fallback CHAIN (across model names) is purely positional, request-scoped, last-error-wins. This is
exactly BodhiApp's desired "ordered fallback list" shape — and it lives entirely in the request, no config
object needed.

### 2.3 Endpoint selection & failover (within a single model name)

`generateChatCompletion` calls `sortedEndpoints(provider, region, model)`
(`server/server.go:2246-2287`): collects all endpoints serving that model, then
`sort.Slice` by **ascending latency** (lines 2279-2281). Optional `intelligentRouteRequest` (2289-2325)
can override with a metrics router, else falls back to lowest-latency / first endpoint.

Inside the request, the loop (e.g. streaming at 403-484) walks the sorted endpoints, asks the rate
limiter `Allow(...)` per endpoint; if rate-limited it records the shortest wait and tries the next; if
**all** are unavailable it `time.Sleep(s.retryInterval)` and re-scans (lines 484-485). Endpoint errors ->
move to next endpoint. **Failover trigger = error or rate-limit, not a status-code allowlist.**

### 2.4 Health-check / ping loop (the active part)

`StartPingLoop` (`server/server.go:2027-2047`):
```go
if s.pingInterval <= 0 { return }          // 0 disables it entirely
ticker := time.NewTicker(s.pingInterval)
s.pingAllEndpoints(ctx)                     // prime immediately
for { select { case <-ctx.Done(): return; case <-ticker.C: s.pingAllEndpoints(ctx) } }
```
`pingAllEndpoints` (2336-2345) calls each provider's `Ping(ctx)` (a minimal completion), and
`updateEndpointStatus` (2347-2356) writes `regionStatus.Latency` + `LastChecked` under a `sync.RWMutex`.

Crucial: **the ping result only updates latency used for sorting.** A failing ping logs a warning and
`continue`s — it does **not** mark the endpoint down or remove it from rotation. There is no cooldown,
no half-open, no eviction. The shipped `config.yaml` even sets `ping_interval: 0`, i.e. the active loop
is off by default and the system relies entirely on reactive per-request failover.

### 2.5 Recovery model

Passive/reactive: a failed endpoint is simply skipped for that request; on the next request it is tried
again. The ping loop, when enabled, is purely informational (latency for ordering) — it is **not** a
recovery mechanism, because there is no "down" state to recover from. Net: Ogem proves you can ship
useful failover with **zero** health-gating state.

### 2.6 Streaming fallback

`handleStreamingChatCompletions` (345-383) mirrors the non-stream chain: iterate models, call
`generateStreamingChatCompletion(...)`; if it errors before a stream is established, `continue` to the
next model; once `success` (stream established), `break`. Errors after the stream starts are written as an
SSE error event then `[DONE]`. Same rule as Portkey: fail over only before the stream commits.

### 2.7 Observability

Structured zap logs only: "Selected endpoint" (sortedEndpoints, 2283-2285), per-model warnings on
fallback ("Failed to get chat completions", 323), "No available endpoints / waiting" before retry-sleep.
No response headers exposing the chosen endpoint or attempt count (weaker than Portkey here).

---

## 3. Declarative config ideas: borrow vs skip for BodhiApp v1

BodhiApp target: in-memory passive cooldown + half-open, no active probes, no stickiness, ordered list.

### Borrow

- **Ordered fallback list as the primary mental model** (Ogem comma-chain, Portkey `mode:"fallback"` +
  array-order `targets`). Strict positional order, no weights — matches "primary then backups."
- **`on_status_codes` allowlist as the failover trigger** (Portkey `config.ts:28`,
  `handlerUtils.ts:676-689`). Decision rule for v1: fail over on a configurable set defaulting to
  `[429, 500, 502, 503, 504]` (Portkey's `RETRY_STATUS_CODES`, `globals.ts:38`) **plus** transport
  errors/timeouts; treat 4xx (esp. 400/401/403/404) as terminal and surface verbatim. Cleaner than
  Ogem's "any error" because it avoids burning the whole chain on a malformed request.
- **Decide failover on response status/headers BEFORE consuming the stream; never re-route mid-stream**
  (both gateways). Keeps streaming simple and correct.
- **Observability headers** (Portkey `responseService.ts:99-124`): emit "which model served" + "attempt
  count" (e.g. `x-bodhi-last-used-model`, `x-bodhi-fallback-attempts`). Cheap, high debugging value.
- **Per-request request_timeout** (Portkey `config.ts:80`) so a hung primary fails over promptly instead
  of hanging the whole request.

### Adapt (BodhiApp goes slightly beyond both)

- **In-memory passive cooldown + half-open.** Neither gateway has this. Portkey is fully stateless;
  Ogem keeps state but never gates on it. BodhiApp's improvement: on a failover-triggering failure, mark
  the model "open" for a cooldown window so subsequent requests skip it without paying a timeout each
  time; after cooldown, allow one "half-open" trial; success closes, failure re-opens. This is the
  circuit-breaker Portkey leaves to the host (`isOpen`/`handleCircuitBreakerResponse`,
  `handlerUtils.ts:646-658,792-800`) — implement it natively, but minimally (a `HashMap<ModelId,
  {state, opened_at}>` behind a `Mutex`/`RwLock`, mirroring Ogem's `sync.RWMutex` + `endpointStatus`).

### Skip for v1

- **Active ping / health-check loop** (Ogem `StartPingLoop`, `pingAllEndpoints`, 2027-2047, 2336-2345).
  Not worth it: it only feeds latency-sorting, doesn't gate health, costs a background goroutine + an
  upstream call per endpoint per interval (real token cost on paid providers), and Ogem itself ships it
  **disabled** (`ping_interval: 0`). Passive cooldown gives recovery without any background probes.
- **Latency-based sorting** (Ogem `sortedEndpoints`, 2279-2281) — needs the ping data we're skipping;
  ordered list is simpler and predictable.
- **Weighted load-balancing / conditional routing** (Portkey loadbalance/conditional, 693-765) —
  out of scope for a fallback feature; no stickiness wanted anyway.
- **Recursive/nested strategy trees** (Portkey `z.lazy` recursion, `config.ts:79`) — powerful but
  overkill; a flat ordered list covers v1.
- **Distributed state (Valkey/Redis)** (Ogem `valkey_endpoint`) — keep health in-process per the brief;
  revisit only for multi-tenant Docker cluster mode later.
- **`Retry-After`-aware retry backoff layer** (Portkey `retryHandler.ts:108-152`) — nice but a second
  loop on top of fallback; for v1, treat 429 as a failover trigger (open + cooldown) instead of an
  inner retry-with-backoff. Can add later.

### One-line takeaway

Adopt **Portkey's declarative `strategy:fallback` + ordered `targets` + `on_status_codes`** as the config
shape and trigger, take **Ogem's reactive per-request "try next, last-error-wins"** loop as the execution
model, and add the **small in-memory cooldown + half-open** that both gateways omit — while **skipping
Ogem's active ping loop entirely** (it's latency-only, costs upstream calls, and ships disabled).
