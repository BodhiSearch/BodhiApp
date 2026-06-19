# Kick-off — Batch 3-3: New Fallback Alias (form)

> Load shared context via @common-prompt.md, then run the per-batch loop (@process.md). **Read
> @batch-3-2-new-api-model-retro.md + @batch-3-1-models-retro.md first.** Its own sub-phase
> (commit + retro), after 3-2 (API Model form).
>
> **⚠ Corrected by the 3-2 retro (the original framing here is stale):**
> - **3-2 did NOT port `bf-*` CSS.** The user directed: the hi-fi prototype is *indicative, not
>   constraining* — implement V2 UX with **what production already uses** (shadcn `Card` + existing
>   field components), do **not** replace production styling with the prototype's `bf-*` system, do
>   **not** restyle shared form components. **Same rule for 3-3:** the prod `ModelRouterForm` is reused
>   as-is; the port is **V2 chrome (breadcrumb + centered container + the right info `rail`) + flag**,
>   not a `bf-*` rebuild.
> - **Flag is 3-3's call at plan time; do NOT delete V1 / migrate specs in this batch.** 3-2 (API form)
>   ended up shipping **V2-only with NO flag** — the chrome was purely additive over the same routes.
>   3-3 has a real **info rail** on top of the chrome, so re-decide at plan time: no-flag-always-on (if
>   the rail is also purely additive) vs a `new-fallback-model` flag. Either way, do **not** delete the
>   V1 router form or migrate the model-router E2E specs here — the **Models V1-list retirement** (the
>   `models` list flag + V1 `ModelsPageContent` + the V1-flow delete/edit specs) is a **separate
>   dedicated iteration**; the forms are independent of it. See @techdebt.md + tracker §"Active
>   per-screen flags".
> - **3-3 is heavier than 3-2:** the prototype adds a real **right-hand info rail** (ROUTING CHAIN /
>   HOW IT WORKS / TIPS) → that's genuine new chrome (a published, memoized `rail`), unlike 3-2 which
>   was breadcrumb + container only. The form *fields* are still reused from production.

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
6. **All gates green** (RTL + targeted model-router E2E reused as-is + GATE B live) → commit (flag-or-
   not per the plan-time decision; do **not** delete V1 or migrate the model-router specs here) → retro
   → **3-4 (New Local Model form) kickoff**.

## Carry-forward gotchas
- Right-rail is a published `rail` — memoize it; depends on form state, so memo on that.
- Don't drop real fields the prototype omits (resilience — `cooldown_secs`/`max_attempts`/`honor_retry_after`).
- **Reuse production `ModelRouterForm` unchanged** (no `bf-*`, no restyle) — the port is chrome + the
  rail + flag. `ShellSlotsProvider` RTL harness for the published rail/breadcrumb.
- Prefer direct nav over `navViaShell` in the page object (3-1 retro).
- `design/` files are user-owned — exclude from commit. The model-router E2E specs stay on the V1 flow
  and migrate in the dedicated Models-flag-retirement iteration (NOT this batch).
