# Phase 4 — Router Test Capability (Validate + Live Probe)

> Read [`README.md`](./README.md) first. Builds on Phase 3. Technical design context: proposal §8 (API surface — `test` action), §4 (forwarding), and the existing API-model "test" capability as a precedent.
> Also read [`phase-2-in-request-fallback-notes.md`](./phase-2-in-request-fallback-notes.md): the validate-side checks (dangling reference, invalid pinned model, unsupported format, empty active set) already exist as `ModelRouterError` variants and as create/update validation in the router service — reuse them rather than re-deriving. The probe side will likely need **richer failure categories** (unauthorized / rate-limited / unreachable / timed-out) than Phase 2's binary status-only `Disposition` — plan to map upstream status → a probe-result category here. The code is the source of truth.

## Goal
A user can **test a model-router** from the UI and get an at-a-glance report of whether it is correctly configured **and** whether each target is actually reachable right now. This turns "did I set this up right?" into a one-click answer and gives operators a diagnostic before relying on the router in chat.

## Functional requirements

### Two-part check
1. **Validate (no upstream calls):** confirm the router is well-formed — every target's referenced alias still exists and is usable, every pinned model is still valid for its alias, referenced formats support chat completions, and report whether the **active (enabled) set is empty**.
2. **Live probe (per target):** send a minimal probe request to **each enabled target** and report its result individually: reachable/succeeded, or failed with the upstream status/category (e.g. unauthorized, rate-limited, unreachable, timed out).

### Reporting
- The result is **per-target**: for each target, show its referenced alias, pinned model, enabled/disabled state, current health (cooled vs available, if known), the **validation outcome**, and the **probe outcome** (with a human-readable reason on failure).
- **Disabled targets** are shown but reported as **skipped** (not probed) — consistent with routing behavior.
- The overall result summarizes whether the router is **usable** (at least one enabled target validates and probes successfully) and flags problems (all-disabled, dangling references, all targets failing, etc.).
- Probing is **diagnostic only** — it must **not** change persisted config. (It *may* update in-memory health like any real call; this should be stated in the report rather than hidden.)

### UI
- A **Test** action on the router (in its detail/edit view), consistent with the existing API-model test experience.
- Results render as a clear per-target list with success/failure indicators and reasons, plus the overall verdict.
- The action communicates that probing **makes real upstream calls** (may consume quota/tokens) before running.

### Cost & safety
- The probe is **minimal** (smallest reasonable request per target) to limit token/quota cost.
- Probes run for **enabled** targets only.

## Out of scope (this phase)
- No scheduled/automatic periodic probing (this is a manual, on-demand action — not a background health loop).
- No change to routing behavior; the test capability is purely diagnostic.
- No new strategies.

## Acceptance gates (test-first)

**Validation reporting (integration, no upstream):**
- A well-formed router reports all targets valid.
- A router with a dangling reference, an invalid pinned model, an unsupported format, or an empty active set reports the specific problem per target and in the overall verdict — **without** making upstream calls for the validation part.

**Live probe (integration, stubbed upstreams):**
- Each enabled target is probed once; a stub returning success is reported reachable; stubs returning `401` / `429` / unreachable are reported with the corresponding failure category and reason.
- Disabled targets are reported as skipped and are **not** probed.
- The overall verdict is "usable" when ≥1 enabled target probes successfully and "not usable" when none do.
- The test action does **not** mutate persisted router config (verify config is unchanged afterward).

**Frontend (component):**
- The test results view renders per-target success/failure with reasons and the overall verdict, and warns before running that real calls are made.

**End-to-end (Playwright, UI-only):**
- A user opens a router, clicks Test, and sees a per-target reachability report (one healthy, one failing target) plus the overall verdict — all asserted through the UI.

## Demo script
Open a router with a working target and a deliberately-broken target, click **Test**, and review the report: working target → reachable; broken target → failed with reason; disabled target → skipped; overall → usable. Disable all targets and re-test → reported not usable with an "empty active set" reason.
