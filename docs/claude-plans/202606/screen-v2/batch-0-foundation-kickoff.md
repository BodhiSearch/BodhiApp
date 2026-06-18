# Kick-off — Batch 0: Foundation

> Entry point for the first batch of the UI V2 migration. First load the shared context via
> @common-prompt.md, then run the per-batch loop (@process.md §"The per-batch loop"). This is
> **Batch 0** — no screen is redesigned; it lays the shell + flag + token + backend foundation every
> later batch needs (full scope in @process.md §"Batch 0 — Foundation").

## Loop step 1 — Explore
1. Read @context.md, @design-reference.md, @architecture.md, @reference-api.md, @process.md.
2. View the prototype visually (server on :8000 + Claude-in-Chrome) per @design-reference.md.
3. Inspect the current production code fresh (it may have drifted from these docs — trust the code):
   - the root layout + current top-header chrome under `crates/bodhi/src/`,
   - the theming/token surface (`globals.css` + `tailwind.config.ts` + the ThemeProvider),
   - the AppShell port sources in `design/` (`bodhi-app-shell.jsx` + `.css`, `colors_and_type.css`),
   - the backend auth/session/user/app-info/settings areas for the id_token + reference-endpoint work.

## Loop step 2 — Prerequisites
Batch 0 has **no reference-API prerequisite** (those are per-batch — @reference-api.md). Its own
deliverables ARE the prerequisites for Batches 1–5: shell components, per-screen flag mechanism,
token merge, reference-API client scaffold, backend id_token + reference-endpoint + regen.

## Loop step 3 — Plan → refine → approve
Write `batch-0-foundation-plan.md` in this folder using the Batch 0 scope in @process.md, broken
into ordered, individually-verifiable steps (suggested order: backend first, upstream→downstream →
ts-client regen → shell port → token merge → root-layout + flag wiring → wrap each nav-section
route's existing content in the shell behind a default-old flag). Include the test list and the exit
criteria below. Present it, refine with the user, get approval **before** implementing.

## Loop steps 4–9
Implement → keep/migrate tests green → run ALL gate checks (`npm run test`, `make test.backend`,
`make build.ts-client`, `make test.e2e`) → commit per batch → write `batch-0-foundation-retro.md`
→ write `batch-1-api-keys-kickoff.md`.

## Batch 0 exit criteria (from @process.md)
- Every nav-section route renders inside AppShell with correct `section`/`subPage` highlight; old
  content unchanged behind a default-old per-screen flag.
- Light/dark works (no ThemeProvider change expected; the design dark selector covers `.dark`).
- Backend surfaces the OAuth `id_token` to the frontend and a configurable reference-API endpoint;
  OpenAPI + ts-client regenerated and committed.
- **All existing RTL + e2e tests still pass** — the shell wrapper must be transparent to old testids.
- Adoption boundary respected: setup/login/request-access/auth render bare; the app-initializer
  stays outside the shell and its redirects still fire.

## Carry-forward watch-outs
@process.md §"Carry-forward risks" — esp. the global `--primary` purple→pink flip at token merge
(eyeball every screen once), the strip-on-port rule, and trailing-slash routing.
