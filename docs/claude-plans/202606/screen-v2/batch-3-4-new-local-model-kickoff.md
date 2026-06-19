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

## Carry-forward from 3-3 (Model Router rebuild — read `batch-3-3-new-fallback-model-retro.md`)
- **3-3 set the form-REBUILD precedent** (vs 3-2's chrome-only): decompose the prod form into
  `-components/` sub-pieces, reuse the real mutations + request contract, V2-only no-flag. 3-4 likely
  follows the same shape (richest form).
- **ONE `useShellChrome` publisher per screen.** Two publishers (route breadcrumb + form rail) clobber
  each other (the setter replaces the whole slots object). Publish breadcrumb + rail together from the
  form (breadcrumb as a prop); use a separate breadcrumb-only publisher only in loading/error branches
  that don't also mount the form.
- **Keep PRODUCTION submit-gating** unless the user says otherwise — render the prototype's stricter
  validation as **display-only** (red borders / status badge), never feed it to the disabled button.
  Lock with explicit RTL contract tests.
- **cmdk combobox + multi-row strict-mode:** if a form renders multiple comboboxes each listing the same
  options, `getByRole('option',{name})` matches >1. Lift open-state so only one is open at a time AND
  scope the page object to the open popover (`[data-radix-popper-content-wrapper] [data-state="open"]`)
  + type-to-filter. The **single-row test won't catch it — only multi-row E2E does.**
- **Factor shared renderers:** 3-3 extracted `RoutingChainPreview` (shared with the My-Models detail
  rail). Look for the same reuse with the local-model card/metadata pieces.
- **E2E ops:** sweep `ports kill 41135 51135 53000 5517x 5518x` before a Playwright run (it starts ALL
  configured servers regardless of `--project`); filter specs by `-g "<describe title>"` (this Playwright
  parses a bare positional after `--project` as another project name). model-router.spec is a required,
  always-running gate (no `.skip`; throws on missing key) — same expectation for the local-model spec.
