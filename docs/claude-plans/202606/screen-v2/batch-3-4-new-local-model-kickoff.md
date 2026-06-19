# Kick-off — Batch 3-4: New Local Model (form)

> Load shared context via @common-prompt.md, then run the per-batch loop (@process.md). **Read
> @batch-3-3-new-fallback-model-retro.md + @batch-3-1-models-retro.md first.** Reuses the `bf-*` form
> CSS ported in 3-2. The **richest form** — sequenced AFTER the simpler API + Fallback forms.

## Scope — the New Local Model form (1 screen, 2 routes share it)
- `routes/models/alias/new/` **+** `…/edit/` — one `AliasForm`, create vs edit. Flag `new-local-model`.
- Design source: `Create New Local Model v4.html` (walk it live — **GATE A**).
- **Decided in 3-1 planning: "V2 styling, keep real fields."** Port the prototype's *visual shell*
  (`bf-*` CSS, section layout, the right-side "click-to-add" flag/param catalogs — those ARE real: they
  map to `context_params` + `request_params`), but keep the data model as the existing prod `AliasForm`
  (`routes/models/alias/-components/AliasForm.tsx`): repo/filename/snapshot comboboxes, `context_params`,
  `request_params`. **Drop** the BPW column, "recommended" badge, preset pills, "Save & test" (no data).
  The **quant-table + download flow folds in at 3-5** (files/pull) — 3-4 keeps the existing combobox
  file selection; don't build the quant table here.
- `alias` is **disabled on edit**. Real hooks: `useCreateModel`/`useUpdateModel`/`useGetModel`; schema
  `schemas/alias.ts` (`createAliasFormSchema`, `convertFormToApi`/`convertApiToForm`).

## Loop (abbrev)
0. **GATE A** — walk the prototype (repo combobox, quant table, preset pills, flag catalog, request-
   defaults catalog, footer). Mark real (context_params/request_params catalogs) vs prototype-only
   (BPW/recommended/presets/Save-&-test/quant-table → 3-5).
1. **Explore** — `routes/models/alias/` (new/edit + `AliasForm`), `schemas/alias.ts`, the create/update/
   get hooks, RTL tests, the `AliasFormPage` e2e page object.
2. **Plan → approve** (`batch-3-4-new-local-model-plan.md`) — form structure, the flag-context_params/
   request_params catalogs (real), what's dropped vs deferred-to-3-5, flag-branch
   (`useUiV2Flag('new-local-model')`), test list, risks.
3. **Implement** behind `new-local-model`; `alias` disabled on edit; reuse `convertFormToApi`/
   `convertApiToForm`; keep `data-testid`/ARIA verbatim.
4. **Tests + e2e + GATE B** — create + edit a real alias live; light + dark + responsive; console-clean.
5. **All gates green** → retire `new-local-model` flag + delete V1 form → migrate the model-alias E2E
   specs → commit → retro → **3-5 (files/pull consolidation) kickoff**.

## Carry-forward gotchas
- `alias` read-only on edit. Reuse the `bf-*` root from 3-2. Scope CSS; strip prototype idioms.
- The quant-table/download UX is **3-5**, not here. `ShellSlotsProvider` RTL harness; direct nav in e2e.
- `design/` files user-owned — exclude from commit. Migrate V1 alias specs with the form.
