# Kick-off — Batch 3-3: New Fallback Alias (form)

> Load shared context via @common-prompt.md, then run the per-batch loop (@process.md). **Read
> @batch-3-2-new-api-model-retro.md + @batch-3-1-models-retro.md first.** Reuses the `bf-*` form CSS
> ported in 3-2. Its own sub-phase (commit + retro), after 3-2 (API Model form).

## Scope — the New Fallback Alias form (1 screen, 2 routes share it)
- `routes/models/router/new/` **+** `…/edit/` — one form, create vs edit mode. Flag `new-fallback-model`.
- Design source: `Create Fallback Model.html` (walk it live on http://localhost:8000 — **GATE A**).
  This form has a **right-hand info rail** in the prototype (Ready-to-create status + ROUTING CHAIN
  preview + HOW IT WORKS + TIPS) — port it as a published `rail` (memoized).
- The My Models list rail's **"Edit fallback alias" CTA** + the "New Fallback Alias" nav point here.

## Real-data map (confirmed in 3-1 exploration)
The production `ModelRouterForm` (`routes/models/router/-components/`):
- **IDENTITY**: alias name (lowercase/digits/dashes — becomes the `model` clients send).
- **FALLBACK SEQUENCE**: ordered `targets[] {alias, model, enabled, weight?}` — each step a card with
  enable toggle + up/down/remove; API-model steps get a "ROUTE TO MODEL (pre-configured only)"
  sub-select. Strategy is hardcoded `fallback`.
- **RESILIENCE config** (`FallbackConfig`: `cooldown_secs` / `max_attempts` / `honor_retry_after`) —
  **the prototype OMITS these; the backend + prod form HAVE them.** Real-data-only cuts both ways:
  **default to KEEPING resilience settings** (3-1 precedent: don't drop real fields the prototype
  skipped). Confirm at plan time.
- Real hooks: `useCreateModelRouter` / `useUpdateModelRouter` / `useGetModelRouter`. Targets resolve by
  alias identity (`list_aliases`). Form is plain React state today (no react-hook-form).

## Loop (abbrev — full loop in @process.md, mirror 3-2)
0. **GATE A** — walk `Create Fallback Model.html`: add/remove/reorder/disable steps, the API-step
   ROUTE-TO-MODEL sub-select, the right-rail info panel. Record behaviors.
1. **Explore** — `routes/models/router/` (new/edit + `ModelRouterForm`), hooks, schema, tests, the
   `ModelRouterFormPage` e2e page object.
2. **Plan → approve** (`batch-3-3-new-fallback-model-plan.md`) — **resolve the resilience-settings
   question**, the published right `rail` (memoized), the step add/remove/reorder UX, flag-branch
   (`useUiV2Flag('new-fallback-model')`), test list, risks.
3. **Implement** behind `new-fallback-model`. Reuse the real mutations; keep `data-testid`/ARIA
   verbatim. Right-rail = published `rail`; `useViewTransition` only for in-page.
4. **Tests + e2e** — RTL via `ShellSlotsProvider` harness; e2e via direct nav + `addInitScript` flag +
   `reducedMotion:'reduce'`; target add/remove/reorder/disable; create + edit a real router.
5. **GATE B (live)** — create + edit a fallback alias live (chain, toggles, resilience if kept);
   light + dark + responsive; console-clean. Rebuild the binary only if a backend change crept in.
6. **All gates green** → retire `new-fallback-model` flag + delete V1 router form → migrate the
   model-router E2E specs → commit → retro → **3-4 (New Local Model form) kickoff**.

## Carry-forward gotchas
- Right-rail is a published `rail` — memoize it; depends on form state, so memo on that.
- Don't drop real fields the prototype omits (resilience).
- Scope CSS (reuse the 3-2 `bf-*` root); strip prototype idioms; `ShellSlotsProvider` RTL harness.
- Prefer direct nav over `navViaShell` in the page object (3-1 retro).
- `design/` files are user-owned — exclude from commit. Migrate the V1 router specs with the form.
