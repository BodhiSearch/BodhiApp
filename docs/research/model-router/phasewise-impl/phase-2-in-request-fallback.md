# Phase 2 — In-Request Fallback

> Read [`README.md`](./README.md) first. Builds on Phase 1. Technical design context: proposal §3 (strategy abstraction), §4 (forwarding + failure classification), consolidated research §3 (fallback core), §4 (failure classification), §6 (streaming rule).

## Goal
When a target fails with a **retryable** error, the request **falls through to the next enabled target** within the same request, until one succeeds or the active chain is exhausted. **Terminal** (client-caused) errors stop immediately. This phase makes the router actually resilient within a single request. There is still **no cross-request memory** — every request re-evaluates the chain from the top.

## Functional requirements

### Fall-through behavior
- The request tries enabled targets **in declared order**. The first **success** is returned (streamed) and routing stops.
- On a **retryable** failure from a target, routing moves to the **next enabled target**.
- **Disabled targets are never tried** (Phase 1 invariant continues).
- A target whose referenced alias is **missing/dangling** or otherwise structurally unusable is **skipped** (treated like a retryable failure), so one broken target never kills an otherwise-working chain.

### Failure classification (the core rule — see proposal §4)
- **Retryable → fall through to next target:** transport/connection errors, timeouts, `408`, `429`, `500`, `502`, `503`, `504`, **and `401`, `403`, `404`** (a bad key or missing model on one vendor must not fail the request if another vendor works — this is the free-tier stacking rule).
- **Terminal → stop and return verbatim:** `400`, `422`, content-policy violations, context-window-exceeded (the request itself is the problem; another vendor won't help).
- **Success → stream back and stop:** any `2xx`.

### Exhaustion
- If every enabled target is tried and all fail with retryable errors, the **last upstream response is returned verbatim** (its real status and body) — not a synthesized wrapper error.
- If the router has **no enabled targets at all**, return the typed "no active target" router error (Phase 1 behavior).

### Streaming
- The decision to fall through is made **before any response bytes reach the client** (based on the upstream status). Once streaming to the client begins, there is no re-routing; a later upstream failure surfaces as a broken stream.

### Observability
- The response headers report the target that **actually served** the response and the **number of attempts** made (e.g. served by target #3 after 2 prior retryable failures).
- An optional `max attempts` limit is honored if configured (default: try the whole enabled chain). (The configurable knob's UI is introduced in Phase 3 alongside the other fallback settings; this phase honors the limit if present.)

## Out of scope (this phase)
- No cooldown / cross-request skipping / recovery (Phase 3) — a target that failed this request is tried again from the top on the next request.
- No router test/probe capability (Phase 4).
- No new strategies.

## Acceptance gates (test-first)

**Classification & fall-through (integration, stubbed upstreams):**
- Primary returns `503`, secondary returns `200` → client gets the secondary's `200`; headers report secondary served, attempts = 2.
- Parameterized over the retryable set (`408/429/500/502/503/504`, transport error/timeout, and `401/403/404`): each causes fall-through to the next target.
- Parameterized over the terminal set (`400/422`, content-policy, context-window): the first such response is returned verbatim and **no** further target is tried.
- All enabled targets return retryable errors → the **last** upstream response is returned verbatim (correct status + body), attempts = number of enabled targets.
- A target with a dangling/missing referenced alias is skipped and the next target serves.
- Disabled targets are never counted as attempts.
- With `max attempts` set below the chain length, routing stops after that many attempts and returns the last attempted response.

**Streaming (integration):**
- A streaming-capable success on the first enabled target streams through unchanged (no buffering of the success path).
- A retryable failure before first byte falls through; a success then streams.

**End-to-end (Playwright, UI-only):**
- A router with a deliberately-broken primary (points at an unreachable/erroring provider) and a working secondary serves a chat completion via the secondary; the user sees a normal reply.

## Demo script
Create a router with target A (a provider configured to fail, e.g. bad endpoint) followed by target B (working). Send a chat message — get a reply from B. Swap so the working one is primary — still works, served by primary. Configure both to fail — observe the real upstream error surfaced verbatim.
