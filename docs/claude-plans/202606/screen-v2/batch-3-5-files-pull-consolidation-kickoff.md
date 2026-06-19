# Kick-off — Batch 3-5: Models · files / pull consolidation

> Load shared context via @common-prompt.md, then run the per-batch loop (@process.md). **Read
> @batch-3-4-new-local-model-retro.md + @batch-3-1-models-retro.md first.** Depends on 3-4 (the local
> model form must be V2 before its download/pull behavior + quant table fold in).

## Scope — consolidate `/models/files/` + `/models/files/pull/` into the Models flow
Per @screen-coverage.md §B/§E (user-confirmed in 3-1: **keep the current APIs**; only the UI consolidates):
- **`/models/files/pull/` → folds into the New Local Model form (3-4)**: entering an undownloaded
  `<org>/<repo>` + selecting a quant → creating the alias downloads it. This is where the prototype's
  **quant table** (real downloaded-vs-needs-pull status from `useListModelFiles`/`useListDownloads`,
  progress polling) lands — deferred from 3-4. Wire the real `usePullModel` + `useListDownloads` polling.
- **`/models/files/` (local files browser)** — **confirm during Explore** whether it needs any surface
  now that My Models (3-1) lists local files as rows, or is fully absorbed/retired.

## Loop (abbrev)
0. **GATE A** — re-walk the local-model prototype's quant/download area; design the real
   downloaded-vs-needs-pull status + progress UX inside the form.
1. **Explore** — `routes/models/files/` + `files/pull/` (+ `PullForm`), the download hooks + polling,
   their tests + page objects.
2. **Plan → approve** (`batch-3-5-...-plan.md`) — the fold-in design, the `/models/files/` decision
   (retire vs keep-a-surface), what V1 code deletes, test list, risks (polling, progress UI).
3. **Implement → tests → GATE B** — download a real model end-to-end (progress → completion → alias).
4. **All gates green** → delete dead `/models/files*` per the decision → commit → retro → **3-6
   (Local Models discovery) kickoff**.

## Carry-forward gotchas
- Keep the download/pull **APIs** (user-confirmed); only the UI consolidates.
- Reuse `useListDownloads({ enablePolling })`; don't reintroduce a manual interval.
- Real-data-only; scope CSS; `ShellSlotsProvider` RTL harness; settle-waits + `keepPreviousData` for
  any list (3-1 retro). `design/` files user-owned — exclude from commit.
