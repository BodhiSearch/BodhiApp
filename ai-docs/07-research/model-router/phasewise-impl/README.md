# Model-Router (Composite Alias) — Phasewise Functional Implementation Plan

This folder contains an **iterative, incremental, evolutionary** plan to implement the **model-router** feature in BodhiApp. The plan is **functional**: it states *what the feature must do and how we verify it*, not *how to code it*. Technical design (domain model, DB schema, trait shape, error types, file locations) lives in the companion proposal and research — each phase references the relevant sections.

## How to use this plan
Each phase file is a **self-contained brief for one work session**. In a fresh session, hand a single phase file to the coding assistant and ask it to:
1. **Explore & analyze** the current codebase and the referenced design docs.
2. **Ask clarifying questions** before coding.
3. **Present its own technical implementation plan** for that phase.
4. Implement **test-first** — write the gating tests, then make them pass.

A phase is **done only when its test gates pass** (see each phase's "Acceptance gates"). Phases build on each other and must be done in order.

## Source context (read these for the "how")
- **Implementation proposal:** [`../bodhiapp-model-router-implementation-proposal.md`](../bodhiapp-model-router-implementation-proposal.md) — domain model (§1), storage (§2), strategy abstraction (§3), forwarding + classification (§4), routing hook (§5), health (§6), errors (§7), API surface (§8), frontend (§9), tests (§10), build order (§11), reserved seams (§12).
- **Consolidated research:** [`../00-consolidated-research.md`](../00-consolidated-research.md) — gateway analysis, the fallback core (§3), failure classification (§4), cooldown mechanics (§5), streaming rule (§6).
- **Per-gateway deep dives:** the `../findings-*.md` files.

## What we are building (scope)
A **model-router** is a new alias kind that fronts an **ordered list of targets**. Each target references an existing alias (a local model alias or a remote API-model alias) and **pins a concrete model**. A chat request addressed to the model-router is routed through its targets using the **fallback strategy**: try the first available target; on a retryable failure, fall through to the next; return to the primary automatically once it recovers.

Primary use case: stack several **free** vendor APIs and fall through them to stretch free quotas before paying (freellmapi-style).

### Non-goals (explicit)
- **Only the fallback strategy is implemented.** No round-robin, weighted, latency, load-balance, or hedging strategies. (The design keeps the strategy *pluggable* so these are possible later, but **this plan implements none of them.**)
- No sticky sessions.
- No active background health probes.
- No strategy nesting (a target may not reference another model-router).
- v1 request surface is **`/v1/chat/completions` only**.
- Health is **in-memory, per-process** (resets on restart; not shared across replicas).

## Global invariants (true in every phase once introduced)
- **Targets are tried in declared order.** First success wins.
- **Enable/disable per target:** every target carries a flag to include/exclude it from the active sequence **without deleting it** (preserves its config and position). A **disabled target is never attempted** — even if every enabled target is failing or cooled down. Disabling is explicit user intent, distinct from a transient health cooldown.
- **All-disabled is allowed to be saved.** A request to a router whose active (enabled) set is empty returns a clear error **at request time**, not at save time.
- **Failover happens before the first byte.** Once response bytes are streamed to the client, no re-routing; a mid-stream failure surfaces as a broken stream and is never silently retried.
- **Exhaustion is verbatim.** When the active chain is exhausted (or a target returns a terminal/client error), the **last upstream response is returned unchanged** (status + body). Structural problems (empty active set, all targets unresolved) return a typed router error.
- **Observability headers** identify which target served the response, the attempt count, and the strategy.

## Phase index
| Phase | Functional outcome | File |
|---|---|---|
| 1 | Define, manage, and use a model-router; requests forward to the first enabled target end-to-end (no failover yet). Full management UI. | [`phase-1-foundation-and-passthrough.md`](./phase-1-foundation-and-passthrough.md) |
| 2 | In-request fallback: on a retryable failure, fall through to the next enabled target; terminal errors stop; exhaustion returns the last upstream response verbatim. | [`phase-2-in-request-fallback.md`](./phase-2-in-request-fallback.md) |
| 3 | Health-aware skipping + automatic recovery: a failed target is skipped for a cooldown window on later requests, then retried; traffic returns to primary on recovery. Config knobs in UI. | [`phase-3-health-and-recovery.md`](./phase-3-health-and-recovery.md) |
| 4 | Router test capability: validate the config and live-probe each target, reporting per-target reachability in the UI. | [`phase-4-router-test-capability.md`](./phase-4-router-test-capability.md) |

## Test-first expectation (all phases)
Follow BodhiApp's layered methodology and testing conventions (`crates/CLAUDE.md`). Each phase must land tests at **every layer it touches**: service/unit, route/integration, frontend component, and Playwright E2E (black-box, UI-only). Write the gating tests first; the phase is complete when they pass and no existing tests regress.
