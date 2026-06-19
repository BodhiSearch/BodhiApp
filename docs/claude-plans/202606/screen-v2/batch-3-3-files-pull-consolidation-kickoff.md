# Kick-off — Batch 3-3: Models · files / pull consolidation

> Load shared context via @common-prompt.md, then run the per-batch loop (@process.md). **Read
> @batch-3-1-models-retro.md + @batch-3-2-new-local-model-retro.md first.** Depends on 3-2 (the local
> model form must be V2 before its download/pull behavior is folded in).

## Scope — consolidate `/models/files/` + `/models/files/pull/` into the Models flow
Per @screen-coverage.md §B/§E:
- **`/models/files/pull/` → folds into the New Local Model form**: entering an undownloaded
  `<org>/<repo>` and creating the alias downloads the referenced model (the quant table / download
  status the 3-2 form deferred). Wire the **real** `usePullModel` + the `useListDownloads` polling.
- **`/models/files/` (local files browser)** — **confirm during Explore** whether it needs any surface
  at all now that All Models (3-1) lists local files as rows, or is fully absorbed/retired. The user
  said in 3-1: "keep the current APIs" — so the **endpoints stay**; the question is the UI surface.

## What earlier phases give you
- All Models V2 list (3-1) already surfaces local-file rows with `size`. New Local Model V2 form (3-2).
- Real hooks: `usePullModel`, `useListDownloads` (`hooks/models/useDownloads.ts`), `useListModelFiles`.
- The existing `PullForm` + `/models/files/pull` route + the files browser route (V1) to fold/retire.

## Loop (abbrev — full loop in @process.md)
0. **GATE A** — re-walk the local-model prototype's quant/download area; decide the download UX inside
   the form (real downloaded-vs-needs-pull status from `useListModelFiles`/`useListDownloads`).
1. **Explore** — `routes/models/files/` + `files/pull/` (+ `PullForm`), the download hooks + polling,
   their tests + page objects.
2. **Plan → approve** (`batch-3-3-...-plan.md`) — the fold-in design, the `/models/files/` decision
   (retire vs keep-a-surface), what V1 code deletes, test list, risks (polling, progress UI).
3. **Implement** — fold pull into the local-model form; handle the `/models/files/` decision.
4. **Tests + e2e + GATE B** — download a real model end-to-end (progress → completion → alias created).
5. **All gates green** → delete the now-dead `/models/files*` per the decision → commit → retro → 3-4.

## Carry-forward gotchas
- Keep the download/pull **APIs** (user-confirmed in 3-1); only the UI consolidates.
- Polling: reuse `useListDownloads({ enablePolling })`; don't reintroduce a manual interval.
- Real-data-only; scope CSS; strip prototype idioms; `ShellSlotsProvider` RTL harness; settle-waits +
  `keepPreviousData` for any list (3-1 retro).
