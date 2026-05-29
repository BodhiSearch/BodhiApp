# Phase 3 — Health-Aware Skipping & Automatic Recovery

> Read [`README.md`](./README.md) first. Builds on Phase 2. Technical design context: proposal §6 (health tracking), §3 (per-strategy resilience config), consolidated research §5 (cooldown mechanics).
> **Read [`phase-2-in-request-fallback-notes.md`](./phase-2-in-request-fallback-notes.md)** — it records what Phase 2 actually built and the exact seams this phase plugs into (the fallback loop iterates raw declared order today; `cooldown_secs`/`honor_retry_after` are persisted-but-unused; `RouterContext` has no clock/health yet; the form hardcodes the strategy config). The code is the source of truth — validate against it.

## Goal
A target that fails is **temporarily skipped on subsequent requests** (a cooldown window) instead of being retried from the top every time — so a known-bad/quota-exhausted provider stops receiving traffic. When the cooldown expires the target is **automatically retried** (half-open), and if it succeeds, **traffic returns to it**. This is what makes "return to the primary once it recovers" work, with **no background probes** — recovery is driven entirely by real traffic.

## Functional requirements

### Cross-request health memory
- When a target fails with a **retryable upstream error** (a retryable HTTP status) or a **genuine transport failure** (connection refused, timeout), it is placed in a **cooldown** for a configurable window. While cooled, it is **skipped during target selection** on later requests (as if temporarily not in the sequence).
- A **structural skip** (the referenced alias is dangling/missing, references another router, or has a format that doesn't support chat completions) is **not a transient failure** and must **not** be cooled — cooling it would only delay the inevitable. Such a target is skipped this request (Phase 2 behavior) but its eligibility next request is unchanged; the Test capability (Phase 4) is where structural problems get surfaced to the user.
- Health is tracked **in memory, per process** (no database, no external store). It **resets on restart**, and in multi-replica deployments each replica learns independently. (Accepted limitation.)
- Health is keyed by the **underlying target** (tenant + the referenced alias + pinned model), so if the **same provider is used by multiple routers**, a cooldown discovered via one router is honored by all of them — valuable for shared free-tier quotas — while the tenant component preserves multi-tenant isolation.

### Passive half-open recovery (no probes)
- A cooldown is simply a **time window**. When it expires, the target becomes **eligible again** and the **next real request that would select it is the trial**.
- If the trial **succeeds**, the target's health is **cleared** and it returns to normal priority (so traffic returns to the primary automatically).
- If the trial **fails**, it is **cooled down again**.
- There is **no background timer or ping** — eligibility is evaluated lazily at request time.

### Never starve
- Cooldown applies only among **enabled** targets. If **every enabled target is currently cooled**, the router must still **attempt** them (ordered by soonest recovery) rather than returning an instant synthetic failure — the user gets a real upstream result.
- **Disabled targets remain excluded** regardless of health (explicit user intent wins over health).

### Configurable resilience (per router, surfaced in UI this phase)
- **Cooldown duration** — how long a failed target is skipped.
- **Honor upstream `Retry-After`** — when a provider returns a `Retry-After`, use the larger of it and the configured cooldown.
- **Max attempts per request** — cap how many targets a single request will try (default: the whole enabled chain). (Honored since Phase 2; its UI lands here.)
- These settings appear in the router create/edit UI with sensible defaults.

### Interaction with prior phases
- Selection order is now: **enabled & not-cooled targets first (in declared order), then enabled-but-cooled targets (soonest-recovery first)**; disabled targets never included.
- Failure classification and verbatim-exhaustion behavior from Phase 2 are unchanged; this phase only adds *which targets are eligible* and *remembering failures across requests*.

## Out of scope (this phase)
- No active/background health checks or probes.
- No persisted (DB) health; no cross-replica sharing.
- No penalty-score/latency ranking — only binary cooled / not-cooled.
- No router test/probe capability (Phase 4).

## Acceptance gates (test-first)
Use a controllable/fake clock so cooldown timing is deterministic.

**Cooldown & recovery (service/unit, fake clock):**
- After a target returns a retryable error, it is marked cooled; a subsequent request **skips** it and selects the next eligible target.
- After the cooldown window elapses (advance the clock), the target is eligible again; the next request **tries it** (half-open). On success, its health clears and it is selected first again on the following request (return-to-primary).
- A half-open trial that fails re-cools the target.
- `Retry-After` from the upstream extends the cooldown to the larger value.
- **Shared health:** two routers referencing the same underlying target both skip it once either observes a failure.
- **Never starve:** when all enabled targets are cooled, a request still attempts them (ordered by soonest recovery) and returns a real upstream result.
- **Disabled wins:** a disabled target is never selected even when all enabled targets are cooled.

**Config (integration):**
- Cooldown duration, honor-Retry-After, and max-attempts settings are persisted and take effect.

**Frontend (component):**
- The router form exposes the resilience settings with defaults and validation.

**End-to-end (Playwright, UI-only):**
- With a router whose primary is failing, repeated chat sends are served by the secondary (primary skipped). After the primary is restored and the cooldown elapses, a later send is again served by the primary. Verified via UI (e.g. the served-target indicator / response content differing per provider).

## Demo script
Create a router primary→secondary. Break the primary; send several messages — all served by secondary (primary cooled, not retried each time). Fix the primary; before cooldown expiry it's still skipped; after expiry the next send is served by the primary again. Point a second router at the same broken provider — it also skips it without having to fail first.
