# BodhiApp — Model-Router Alias (Composite Model): Implementation Proposal

**Status:** Design proposal (v1). No code written yet.
**Companion research:** [`00-consolidated-research.md`](./00-consolidated-research.md) and the per-gateway `findings-*.md`.

A **model-router** is a new kind of alias that does not point at a backend directly. It fronts an ordered set of **targets** (each = an existing alias + a pinned model) and routes between them via a **pluggable strategy**. v1 ships one strategy — **fallback** — but the abstraction is built so round-robin / weighted / latency / load-balance / parallel-hedging strategies are *additive* later, with **no duplication** of forwarding or health code.

Guiding use case (à la `freellmapi`): stack several **free** vendor APIs and fall through them to stretch free quotas before paying.

## Locked design decisions
Confirmed with the user during a `/grill-me` + `/grill-with-docs` interview:
- **Term:** `model-router` (`source = "model-router"`). Fallback is a *strategy*, not the feature name.
- **Strategy = serde-tagged enum**, `Fallback` only in v1; new strategies are additive variants.
- **Strategy owns its execution loop**, calling **shared primitives** — keeps the door open to future parallel/hedging strategies a pure `select()` interface would foreclose.
- **Resilience *config* is per-strategy**; **failure classification** and the **health *state store*** are **shared**.
- **Health state: in-memory only**, passive cooldown + half-open, **no background probes**. Per-process (resets on restart; each Docker replica learns independently — accepted).
- **No sticky sessions** in v1.
- **Targets at alias level**, optional per-target `weight` reserved for future strategies.
- **v1 endpoint scope:** `/v1/chat/completions` only; resolver/executor built endpoint-agnostic.
- **Fall-through aggressively:** `429/5xx/timeout` **and** `401/403/404` fall through; only `400/422`/content-policy/context-window are terminal.
- **Deliverable:** this report only — no code this session.

---

## 1. Domain model (`crates/services/src/models/model_objs.rs`)

Add a fourth variant to the existing internally-tagged `Alias` enum (tagged on `source`, kebab-case — `model_objs.rs:929-941`). This follows the established alias convention; the `{"v1":{...}}` versioned-envelope pattern is **not** used here (it's reserved for opaque 3rd-party paste-in contracts like `LlmLibertyEnvelope`). Evolution of the routing config comes from the **`strategy` tagged enum**, which is additive by construction.

```rust
#[serde(tag = "source", rename_all = "kebab-case")]
pub enum Alias {
    User(UserAlias),
    Model(ModelAlias),
    Api(ApiAlias),
    ModelRouter(ModelRouterAlias),   // NEW -> {"source":"model-router", ...}
}

pub struct ModelRouterAlias {
    pub id: String,                    // ULID
    pub alias: String,                 // user-facing model name, unique across all aliases
    pub targets: Vec<RouterTarget>,    // ordered; the composition (shared across all strategies)
    pub strategy: RoutingStrategyConfig,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct RouterTarget {
    pub alias: String,                 // name of an existing user/model/api alias (NOT a model-router)
    pub model: String,                 // model string placed into request["model"] for this target
    pub weight: Option<u32>,           // SEAM: ignored by Fallback; used by future weighted strategy
}

// The strategy is DATA (persisted, wire-exposed). Adding a strategy = adding a variant.
#[serde(tag = "strategy", rename_all = "kebab-case")]
pub enum RoutingStrategyConfig {
    Fallback(FallbackConfig),          // {"strategy":"fallback", ...}
    // future (additive): RoundRobin(RoundRobinConfig), Weighted(WeightedConfig), Latency(LatencyConfig), ...
}

pub struct FallbackConfig {            // per-strategy resilience config (user's choice)
    pub cooldown_secs: u32,            // default 30
    pub max_attempts: u32,             // 0 = try the whole chain; default 0
    pub honor_retry_after: bool,       // default true
    // allowed_fails omitted in v1 (== 1: cool down on first retryable failure)
}
```

Extend the three helper methods on `Alias` (`model_objs.rs:943-970`):
- `alias_name()` → `router.alias`.
- `source()` → add `AliasSource::ModelRouter` (extend `model_objs.rs:592-599`).
- `can_serve(model)` → `model == router.alias` (exact match; no prefix semantics).

Add `ModelRouter` to the **untagged** `AliasResponse` enum (`model_objs.rs:1941`) — ordering matters for untagged deserialization; the `targets`/`strategy` fields make it distinctive.

**Why `model` is stored per target:** routing reuses the existing `for_alias` dispatch, which expects `request["model"]` to be a value the referenced alias's `supports_model()` accepts. Storing the resolved model keeps the executor thin and avoids re-deriving prefixes at request time.

---

## 2. Storage (DB)

New tenant-scoped table mirroring `api_model_aliases` (SQLite + Postgres RLS). **No health columns** — health is in-memory (§6).

### Entity — `crates/services/src/models/model_router_entity.rs`, table `model_router_aliases`
| Column | Type | Notes |
|---|---|---|
| `id` | String (PK) | ULID |
| `tenant_id` | String | RLS key |
| `user_id` | String | owner |
| `alias` | String | unique per (tenant, user) |
| `targets` | JsonBinary | `Vec<RouterTarget>` |
| `strategy` | JsonBinary | `RoutingStrategyConfig` (tagged enum incl. per-strategy resilience config) |
| `created_at` / `updated_at` | timestamps | via `TimeService` |

Storing `strategy` as a JSON blob means new strategy variants need **no migration**.

### Migration — `m20250101_000022_model_router_aliases.rs`
Next number after the current highest (`m20250101_000021`). Register in `db/sea_migrations/mod.rs`. Replicate the `api_model_aliases` pattern: partial unique index on `(tenant_id, user_id, alias)`; Postgres `ENABLE/FORCE ROW LEVEL SECURITY` + `tenant_isolation` policy using `current_tenant_id()`.

### Repository — `crates/services/src/models/model_router_repository.rs`
`trait ModelRouterRepository` impl'd for `DefaultDbService`, mirroring `ApiAliasRepository`: `create`, `get`, `update`, `delete`, `list`, `check_alias_exists`. All mutations use `begin_tenant_txn(tenant_id)`. No encryption columns (referenced API aliases own their keys).

---

## 3. The strategy abstraction (the evolvable spine)

Two layers: **config** (data, §1) and **behavior** (the trait below). Mapping config → behavior is a single `match` — additive per strategy.

```rust
#[async_trait]
pub trait RoutingStrategy: Send + Sync {
    /// Run the full routing loop for one request; return the response to stream back.
    /// The strategy OWNS control flow but calls shared primitives on `ctx`.
    async fn execute(&self, ctx: &mut RouterContext<'_>) -> Result<Response, ModelRouterError>;

    /// Validate this strategy's config against the target list at create/update time.
    fn validate(&self, targets: &[RouterTarget]) -> Result<(), ObjValidationError>;
}

// Config dispatches to behavior. Adding a strategy = one arm + one impl. No other code changes.
impl RoutingStrategyConfig {
    fn behavior(&self) -> &dyn RoutingStrategy {
        match self { RoutingStrategyConfig::Fallback(c) => c }   // FallbackConfig: RoutingStrategy
    }
}
```

### `RouterContext` — the shared primitives every strategy calls (no duplication)
```rust
pub struct RouterContext<'a> {
    pub targets: &'a [RouterTarget],
    pub request: Value,                       // the incoming chat-completions body
    health: &'a HealthRegistry,               // shared, strategy-agnostic (§6)
    // SEAM (reserved, not built in v1): per-router mutable selection state for stateful strategies
    // selection_state: &'a mut SelectionState,
    // ... resolver (find_alias), api-key resolver, ai_api factory, time service ...
}

impl<'a> RouterContext<'a> {
    /// THE shared mechanical primitive: resolve target -> forward -> classify -> update health.
    /// Returns either a committable Response (stream it & stop) or a retryable failure (try next).
    async fn forward_one(&self, target: &RouterTarget) -> AttemptOutcome { /* see §4 */ }
}

pub enum AttemptOutcome {
    Commit(Response),                         // 2xx OR terminal 4xx: stream verbatim, stop
    Retry { last: BufferedError, elapsed: Duration },  // retryable: cooled down, try next
    //                            ^^^^^^^ SEAM: elapsed reserved for future latency strategy
}
```

### The one v1 strategy
```rust
#[async_trait]
impl RoutingStrategy for FallbackConfig {
    async fn execute(&self, ctx: &mut RouterContext<'_>) -> Result<Response, ModelRouterError> {
        let order = order_by_health(ctx.targets, ctx.health, now);  // non-cooled first (declared order), cooled last by soonest expiry
        let mut last: Option<BufferedError> = None;
        let mut attempts = 0;
        for target in order {
            if self.max_attempts != 0 && attempts >= self.max_attempts { break; }
            attempts += 1;
            match ctx.forward_one(target).await {
                AttemptOutcome::Commit(resp) => return Ok(with_obs_headers(resp, target, attempts)),
                AttemptOutcome::Retry { last: e, .. } => { last = Some(e); continue; }
            }
        }
        match last {
            Some(e) => Ok(with_obs_headers(e.into_response(), /*exhausted*/)),  // last upstream error, VERBATIM
            None => Err(ModelRouterError::EmptyChain),
        }
    }
    fn validate(&self, targets: &[RouterTarget]) -> Result<(), ObjValidationError> { /* §8 */ }
}
```

**Why this satisfies "evolve, don't duplicate":** the `for`-loop is the only per-strategy code; all mechanics (`forward_one`, classification, health) are shared via `ctx`. A future `RoundRobinConfig` writes its own ~5-line loop (rotating the start index via the reserved `selection_state`) and reuses the exact same primitives. A future hedging strategy can `tokio::select!` over multiple `forward_one` calls — impossible with a `select()`-only interface.

---

## 4. `forward_one`, failure classification, and the streaming insight

**No forwarder refactor is needed.** The existing `forward_to_upstream` (`crates/services/src/ai_apis/provider_shared.rs:100-172`) returns an axum `Response` whose body is a *lazy* stream and whose `status()` is readable immediately. So `forward_one` can inspect status and decide to **stream** (commit) or **drop + buffer** (retry) without consuming the body and without changing the forwarder. Transport failures still arrive as `Err` from `send().await?` (line 150).

```rust
async fn forward_one(&self, target: &RouterTarget) -> AttemptOutcome {
    let key = target_key(tenant_id, &inner_alias_id, &target.model);   // §6
    let inner = match self.find_alias(&target.alias) {
        Some(a) if !a.is_model_router() => a,
        _ => return retry_skip(/*structural: missing or nested*/),     // skip bad target, keep chain alive
    };
    let api_key = self.resolve_api_key(&inner);
    let mut req = self.request.clone();
    req["model"] = target.model.clone().into();                        // pin the concrete model

    let started = self.time.now();
    match self.ai_api.for_alias(&inner, api_key).and_then(|c|
              c.forward_request_with_method(&POST, "/chat/completions", Some(req), params, None).await) {
        Ok(resp) => match classify(resp.status()) {
            Class::Success  => { self.health.record_success(&key); AttemptOutcome::Commit(resp) }
            Class::Terminal => { /* don't touch health */          AttemptOutcome::Commit(resp) }
            Class::Retryable => {
                let cd = cooldown_for(resp.headers(), self.cfg);       // honor Retry-After
                self.health.cooldown(&key, cd);
                AttemptOutcome::Retry { last: buffer(resp).await, elapsed: self.time.now() - started }
            }
        },
        Err(_transport) => {
            self.health.cooldown(&key, self.cfg.cooldown_secs);
            AttemptOutcome::Retry { last: BufferedError::transport(), elapsed: self.time.now() - started }
        }
    }
}
```

### Classification (shared default — §4 of consolidated research)
| Class | Trigger | Action |
|---|---|---|
| `Success` | `2xx` | commit (stream), record success |
| `Retryable` | transport err, timeout, `408`, `429`, `500`, `502`, `503`, `504`, **`401`, `403`, `404`** | cooldown, fall through |
| `Terminal` | `400`, `422`, content-policy, context-window | commit (stream verbatim), stop — failover won't help |

`cooldown_for`: if `honor_retry_after` and a `Retry-After` header is present, `max(cooldown_secs, retry_after)`; else `cooldown_secs`.

**Verbatim exhaustion:** when the chain is exhausted, return the last upstream response (status + body) unchanged — matches BodhiApp's "opaque proxy" convention. `ModelRouterError` is reserved for *structural* problems only (§7), never wrapped upstream failures.

**Resilience:** a missing/nested/misconfigured referenced alias is a `retry_skip` (skip to next), so one bad target never kills an otherwise-working chain. **Never starve:** `order_by_health` still yields all-cooled targets (sorted by soonest expiry) so the user gets a real attempt rather than an instant synthetic error.

---

## 5. Resolution & routing hook

### 5a. Alias resolution — `crates/services/src/models/data_service.rs`
- `list_aliases` (`:91`) — also load `model_router_aliases` into the aggregate list.
- `find_alias` (`:137`) — add model-router lookup by exact name. Priority: `user` → `model-router` → `model` → `api`, so an explicit router name resolves before any prefix-based API match. (Name uniqueness is enforced at create time across kinds; this is the tiebreak safety net.)

### 5b. Chat handler hook — `crates/routes_app/src/oai/routes_oai_chat.rs:125`
After `find_alias` (`:138`), branch before the existing single-alias path:
```rust
if let Alias::ModelRouter(ref router) = alias {
    let mut ctx = RouterContext::new(router, request, &auth_scope, health);
    return router.strategy.behavior().execute(&mut ctx).await.map_err(OaiApiError::from);
}
// ...existing single-alias path unchanged...
```
The existing API-format guard (`:146`) is applied **per resolved target** inside `forward_one`, not on the router alias itself. v1 wires only `/chat/completions`; the executor is endpoint-agnostic so `/v1/responses`, embeddings, and the Anthropic/Gemini proxies are later wiring, not redesign.

---

## 6. Health tracking (shared, in-memory)

One process-global registry on `AppService` as a concrete `Arc` (not a DB service). Interior mutability via `RwLock`; time via injected `TimeService` (never `Utc::now()`). **Strategy-agnostic** — every strategy reads/writes it; each applies its own `cooldown_secs`.

```rust
pub struct HealthRegistry {
    inner: RwLock<HashMap<TargetKey, EndpointHealth>>,
    time: Arc<dyn TimeService>,
}
struct EndpointHealth { cooldown_until: Option<DateTime<Utc>>, consecutive_fails: u32, last_failure_at: Option<DateTime<Utc>> }
type TargetKey = String;  // format!("{tenant_id}:{referenced_alias_id}:{model}")
```
Operations: `is_cooled(key,now)`, `cooldown(key,secs)`, `record_success(key)`, plus a free function `order_by_health(targets, registry, now)` (non-cooled in declared order first; cooled last by soonest expiry).

**Passive half-open recovery:** no timer/probe. When `cooldown_until` passes, the next request that selects the target is the trial; success clears it, failure re-cools it.

**Key rationale:** keying by `(tenant, referenced_alias_id, model)` means if free-provider X is rate-limited, *every* router that includes X skips it (quota saving), while `tenant_id` preserves isolation. Keying by the underlying target (not router-alias+index) is what enables this sharing — matches LiteLLM's single shared cooldown store.

**Reserved seam (not built):** stateful strategies (round-robin cursor, latency metrics) need a per-router mutable `SelectionState` held *alongside* this registry. v1 doesn't create it; `RouterContext` is shaped so adding it later (a sibling `RwLock<HashMap<router_id, SelectionState>>`) needs no signature churn, and `AttemptOutcome.elapsed` already feeds future latency strategies.

**Accepted limitations:** per-process state resets on restart and is not shared across Docker replicas (each rediscovers an exhausted provider once). Promoting health to the DB is a documented future option.

---

## 7. Errors — `ModelRouterError` (services)

`#[derive(ErrorMeta)]` enum (e.g. `models/model_router_error.rs`), auto-converted to HTTP via the blanket `From<T: AppError>`; OAI handlers convert via `OaiApiError`. Codes auto-generate as `model_router_error-<variant>`.

| Variant | ErrorType | When |
|---|---|---|
| `EmptyChain` | BadRequest | no targets (caught at create; defensive at runtime) |
| `ReferencedAliasNotFound { alias }` | NotFound | create/update validation — a target references a non-existent alias |
| `NestedRouterNotAllowed { alias }` | BadRequest | a target references another model-router |
| `SelfReference` | BadRequest | a target references the router being created |
| `InvalidPinnedModel { alias, model }` | BadRequest | pinned model not accepted by the referenced alias |
| `TargetFormatUnsupported { alias, api_format }` | BadRequest | referenced API alias's format has no `/chat/completions` surface |

Upstream failures are **never wrapped** — exhaustion returns the last upstream `Response` verbatim (an all-429 chain returns `429`, not a `model_router_error`). At runtime a target that turns out structurally broken is *skipped* (`retry_skip`), not surfaced as an error.

---

## 8. API surface — `crates/routes_app/src/models/router/`

New CRUD group mirroring API-model routes (session-only, `ResourceRole::User`), plus a `ModelRouterService` (mirroring `ApiModelService`). Endpoint constants via `make_ui_endpoint!`.

| Method + Path | Handler | op_id |
|---|---|---|
| `POST /bodhi/v1/models/router` | `model_router_create` | createModelRouter |
| `GET /bodhi/v1/models/router/{id}` | `model_router_show` | getModelRouter |
| `PUT /bodhi/v1/models/router/{id}` | `model_router_update` | updateModelRouter |
| `DELETE /bodhi/v1/models/router/{id}` | `model_router_destroy` | deleteModelRouter |
| `POST /bodhi/v1/models/router/{id}/test` | `model_router_test` | testModelRouter (optional v1) |

Routers also appear automatically in `GET /bodhi/v1/models` and `GET /bodhi/v1/models/{id}` via the extended resolution (§5a).

### Request / response types (`model_objs.rs`)
```rust
pub struct ModelRouterRequest {                 // Serialize, Deserialize, Validate, ToSchema
    #[validate(length(min = 1))] pub alias: String,
    #[validate(length(min = 1))] pub targets: Vec<RouterTargetRequest>,  // non-empty
    pub strategy: RoutingStrategyConfig,        // tagged enum; defaults applied if omitted -> Fallback defaults
}
pub struct RouterTargetRequest { pub alias: String, pub model: String, pub weight: Option<u32> }

pub struct ModelRouterResponse {                // From<ModelRouterAlias>
    pub id: String, pub alias: String, pub source: AliasSource,  // = ModelRouter
    pub targets: Vec<RouterTarget>, pub strategy: RoutingStrategyConfig,
    pub created_at: DateTime<Utc>, pub updated_at: DateTime<Utc>,
}
```

### Create/update validation (`ModelRouterService` + `strategy.validate`)
- `targets` non-empty; alias name unique across all alias kinds (DB unique index + pre-check).
- each `target.alias` resolves to a `user`/`model`/`api` alias (else `ReferencedAliasNotFound`), is **not** a model-router (`NestedRouterNotAllowed`), not self (`SelfReference`).
- each `target.model` accepted by the referenced alias: local → equals alias name; API selected → in `matchable_models()`; API forward_all → starts with the alias prefix (else `InvalidPinnedModel`).
- referenced API alias `api_format` supports chat completions (else `TargetFormatUnsupported`).
- `strategy.validate(targets)` runs strategy-specific checks (Fallback: none beyond the above; future Weighted: weights present/positive).
- **Dangling references allowed:** deleting a referenced alias is not blocked (no FK); the executor skips the dead target at runtime. UI may warn.

### Observability headers (added by the strategy to the returned response)
- `x-bodhi-routed-alias: <referenced alias name>`
- `x-bodhi-routed-model: <pinned model>`
- `x-bodhi-router-strategy: <strategy tag>`
- `x-bodhi-router-attempts: <n>`

---

## 9. Frontend (`crates/bodhi/src/`)

Mirror the API-model UI:
- Routes: `routes/models/router/new/index.tsx`, `routes/models/router/edit/index.tsx`, shared `routes/models/router/-components/ModelRouterForm.tsx`.
- Hook: `hooks/models/useModelRouters.ts` (`useGetModelRouter`, `useCreateModelRouter`, `useUpdateModelRouter`, `useDeleteModelRouter`), endpoint constants in `hooks/models/constants.ts`.
- Aggregate list (`routes/models/index.tsx`) and chat picker (`routes/chat/-components/settings/AliasSelector.tsx`) pick up routers automatically.

**ModelRouterForm UX** — a **strategy selector** (only "Fallback" enabled in v1; the control exists so future strategies appear as options) plus an ordered, reorderable list of targets. Per target:
1. Select an existing alias (dropdown of user/model/api aliases; routers excluded).
2. Pick the model based on the selected alias's shape (data already on `ApiAliasResponse`): local → fixed; API `forward_all_with_prefix=true` → free-text; API selected → dropdown of that alias's `models`.
3. Reorder to set priority.
Plus Fallback config fields: `cooldown_secs`, `max_attempts`, `honor_retry_after`.

Import types from `@bodhiapp/ts-client` (regenerate after API changes: `cargo run --package xtask openapi && make build.ts-client`).

---

## 10. Testing plan (all layers)

**services (unit, `#[rstest]`, sqlite+postgres via `#[values]`):**
- `ModelRouterRepository` CRUD + **tenant isolation** (pattern: `mcps/test_mcp_repository_isolation.rs`).
- `classify()` table test over every status in §4.
- `HealthRegistry` with `FrozenTimeService`: cooldown set/skip, passive expiry (advance frozen clock → eligible), success-clears, `order_by_health` ordering, all-cooled-still-yields.
- `FallbackConfig::execute` with mocked clients: success on primary; primary `429`/`503` → secondary success; all `429` → returns last `429` verbatim; `400` on primary → returns verbatim, no fall-through; `401` on primary → falls through (free-tier rule); missing referenced alias → skip to next; cooldown shared across two routers via `TargetKey`.
- `strategy.validate`: nesting, self-ref, bad pinned model, format-unsupported.

**routes_app (single-turn `tower::oneshot`):** CRUD happy/validation paths; OpenAPI snapshot (`cargo test -p routes_app -- openapi`).

**server_app (multi-turn live, `#[serial_test::serial(live)]`):** chat to a router whose primary stub returns `503` and secondary returns `200` → client gets `200` + `x-bodhi-routed-alias` = secondary; verify primary recovery after the cooldown window elapses.

**Playwright E2E (`lib_bodhiserver/tests-js`, black-box, UI-only):** create a router via the UI, select it in chat, drive a completion; assert via UI only (no `page.evaluate`/direct fetch). Throw in `beforeAll` if required env is missing (no `test.skip`).

---

## 11. Layered build order (upstream → downstream)
1. `services`: domain types in `model_objs.rs`; entity + migration `m20250101_000022`; `ModelRouterRepository`; `HealthRegistry` + wire into `AppService`; `RoutingStrategy` trait + `RouterContext`/`forward_one` + `classify` + `FallbackConfig` impl; `ModelRouterError`; `ModelRouterService`. `cargo test -p services`.
2. `routes_app`: CRUD handlers + chat-handler hook; OpenAPI registration. `cargo test -p routes_app`.
3. `server_app`: live integration test.
4. `make test.backend`.
5. `cargo run --package xtask openapi && make build.ts-client`.
6. Frontend routes/hook/form + `npm test`.
7. `make build.dev-server` + Playwright E2E.
8. Update crate `CLAUDE.md`/`PACKAGE.md`.

---

## 12. Reserved seams (built later, signatures ready now)
- **Stateful strategies** (round-robin cursor, weighted walk, latency/least-conn metrics): a per-router `SelectionState` registry beside `HealthRegistry`; `RouterContext` exposes the accessor; `AttemptOutcome.elapsed` already feeds latency. Per-target `weight` already on `RouterTarget`.
- **Parallel hedging / first-response-wins:** enabled by `execute()`-owns-loop (a `select()`-only interface could not).
- **Per-target / per-strategy `on_status_codes` override** (Portkey-style), **rate-limit body sniffing** (Bifrost-style).
- **DB-persisted health** (cross-replica + restart-durable).
- **Strategy nesting** (router-of-routers) — needs cycle detection; intentionally disallowed in v1.
- **Broader endpoint scope:** `/v1/responses`, embeddings, Anthropic/Gemini proxies.
- **Active background health probes** (Ogem-style; deliberately omitted — passive half-open suffices).
