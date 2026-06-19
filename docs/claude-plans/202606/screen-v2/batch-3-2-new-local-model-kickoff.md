# Kick-off — Batch 3-2: New Local Model (form)

> Load shared context via @common-prompt.md (framework skills: `tanstack-router`, `tanstack-query`,
> `vercel-react-best-practices`, `web-design-guidelines`), then run the per-batch loop
> (@process.md §"The per-batch loop"). **Read @batch-3-1-models-retro.md + @batch-2-settings-retro.md
> first.** Batch 3 was split into sub-phases (@batch-3-1-models-retro.md); 3-1 (All Models list) is ✅.

## Scope — the New Local Model form (1 screen, 2 routes share it)
- `routes/models/alias/new/` **+** `…/edit/` — one `AliasForm`, create vs edit mode.
- Design source: `Create New Local Model v4.html` (walk it live on http://localhost:8000 — GATE A).
- **Decided in 3-1 planning: "V2 styling, keep real fields."** Port the prototype's *visual shell*
  (the `bf-*` form CSS, section layout, the right-side "click-to-add" flag/param catalogs — those ARE
  real: they map to `context_params` + `request_params`), but keep the data model exactly as the
  existing production `AliasForm` (`routes/models/alias/-components/AliasForm.tsx`): repo/filename/
  snapshot comboboxes, `context_params`, `request_params`. **Drop** the BPW column, "recommended"
  badge, preset pills, and "Save & test" (no backing data). `api_format` is N/A here (local only).

## What 3-1 + Batches 0–2 already give you (don't rebuild)
- AppShell + `useShellChrome` (breadcrumb/sidebar/rail), `ShellFilterTabs`, `useCollapsibleSearch`,
  `LinkRow`, `useViewTransition`, `list.css`/`api-keys.css`. The `models` nav + flags exist.
- The **All Models V2 list** (`ModelsScreenV2`) — its rail's "New Local Model" path + the row "Edit"
  CTA will point here once this lands.
- Existing real hooks/schema: `useCreateModel`/`useUpdateModel`/`useGetModel` (`hooks/models`),
  `createAliasFormSchema` + `convertFormToApi`/`convertApiToForm` (`schemas/alias.ts`),
  `ComboBoxResponsive`, `useListModelFiles` for repo/filename options.

## This batch's prerequisite (the one new thing)
- **Port `bodhi-form.css` (`bf-*`)** scoped to a per-screen root (`.new-local-model-screen` or similar),
  per Batch-1/2 CSS discipline (scope generic names, rename collisions, drop global `:root`/`mark`).
  Batch 1 deliberately skipped it; 3-1 didn't need it (list screen). This is the first form port.

## Loop
0. **GATE A** — walk `Create New Local Model v4.html` interactively (repo combobox, quant table,
   preset pills, flag catalog, request-defaults catalog, footer). Record behaviors; mark which map to
   real fields vs prototype-only.
1. **Explore** — `routes/models/alias/` (new/edit + `-components/AliasForm.tsx`), `schemas/alias.ts`,
   the `hooks/models` create/update/get, the colocated RTL tests + the `AliasFormPage` e2e page object.
2. **Prereq** — port `bf-*` CSS scoped; confirm no backend change (the create/update contract is
   unchanged from 3-1).
3. **Plan → approve** (`batch-3-2-new-local-model-plan.md`) — form structure (dialog→page already done
   in prod), real-data gaps, the flag-branch (`useUiV2Flag('new-local-model')`), test list, risks.
4. **Implement** behind `new-local-model` flag (branch inside the page; `AppInitializer` outside).
   Reuse `convertFormToApi`/`convertApiToForm`; keep `data-testid` + ARIA verbatim.
5. **Migrate tests + e2e** — RTL via the `ShellSlotsProvider` harness; e2e via `navViaShell('models',
   'new-local-model')` (or direct nav — see 3-1 retro), `reducedMotion:'reduce'`, set the flag via
   `addInitScript`.
6. **GATE B** — create + edit a real alias live; light + dark + responsive; console-clean.
7. **All gates green** → retire `new-local-model` flag + delete V1 form → commit → retro → 3-3 kickoff.

## Carry-forward gotchas (from 3-1 + Batch-1/2 retros)
- Route tests render the page directly → wrap in `ShellSlotsProvider` + slots-consumer.
- `routeTree.gen.ts` is git-ignored; commit route files only.
- Scope generic CSS under a per-screen root; strip prototype idioms (`lucide.createIcons`/`window.*`/
  `ReactDOM.createRoot`/`data-theme`/`TweaksPanel`).
- Real-data-only — omit prototype elements with no backing field.
- The router-nav `InvalidStateError` on route entry is pre-existing (techdebt.md), not yours.
