# Kick-off — Batch 0: Foundation

> Entry point for the first batch of the UI V2 migration. Run the per-batch loop from the charter
> (`docs/claude-plans/202606/we-have-new-designs-swift-pillow.md`, §6). This is **Batch 0** — no
> screen is redesigned; it lays the shell + flag + token + backend foundation every later batch needs.

## Start here (loop step 1 — Explore)
1. Read the charter (`../we-have-new-designs-swift-pillow.md`) end-to-end — especially §6 (loop),
   §7 (per-screen recipe), §8 (Batch 0 scope + exit criteria), §9 (risks).
2. Re-inspect current state before planning (it may have drifted since the charter was written):
   - `crates/bodhi/src/routes/__root.tsx` — current root layout (ThemeProvider → ClientProviders →
     NavigationProvider → AppHeader → Outlet → Toaster).
   - `crates/bodhi/src/components/navigation/AppHeader.tsx`, `AppNavigation.tsx` — old chrome to be
     superseded (deleted only at the very end, Batch 5).
   - `crates/bodhi/src/styles/globals.css` + `tailwind.config.ts` — token surface to merge.
   - `crates/bodhi/src/components/ThemeProvider.tsx` — confirm `.dark`/`.light` class behavior.
   - `design/bodhi-app-shell.jsx` + `bodhi-app-shell.css` + `colors_and_type.css` — port sources.
   - Backend: `crates/services/src/auth/auth_service.rs` (`exchange_auth_code`),
     `crates/services/src/session_keys.rs`, `crates/routes_app/src/users/users_api_schemas.rs`,
     `crates/routes_app/src/setup/setup_api_schemas.rs`, `crates/services/src/settings/*`.

## Loop step 2 — Prerequisites (none external for Batch 0)
Batch 0 has no reference-API prerequisite (those are per-batch, charter §4). Its own deliverables ARE
the prerequisites for Batches 1–5: shell components, per-screen flag mechanism, token merge,
reference-API client scaffold, backend id_token + `reference_api_endpoint` + regen.

## Loop step 3 — Plan → refine → approve
Write `batch-0-foundation-plan.md` in this folder with the concrete scope from charter §8, broken
into ordered, individually-verifiable steps (suggest: backend first — upstream→downstream — then
ts-client regen, then shell port, then token merge, then root-layout + flag wiring, then wrap each
nav-section route's existing content in `<Shell>` behind a default-old flag). Include the test list
and the exact exit criteria. Present it, refine with the user, get approval **before** implementing.

## Loop steps 4–9
Implement → migrate/keep tests green → run ALL gate checks (`npm run test`, `make test.backend`,
`make build.ts-client`, `make test.e2e`) → commit per batch → write `batch-0-foundation-retro.md`
→ write `batch-1-api-keys-kickoff.md`.

## Batch 0 exit criteria (from charter §8)
- Every nav-section route renders inside `<Shell>` with correct `section`/`subPage` highlight; old
  content unchanged behind a default-old per-screen flag.
- Light/dark works (no ThemeProvider change needed; design dark selector covers `.dark`).
- Backend surfaces OAuth `id_token` on `GET /bodhi/v1/user` and `reference_api_endpoint` on
  `GET /bodhi/v1/info`; OpenAPI + ts-client regenerated and committed.
- **All existing RTL + e2e tests still pass** — the shell wrapper must be transparent to old testids.
- Adoption boundary respected: `setup/*`, `login/`, `request-access/`, `auth/*` render bare;
  `AppInitializer` stays outside `<Shell>` and its redirects still fire.

## Carry-forward watch-outs
Charter §9 — esp. the global `--primary` purple→pink flip at token merge (eyeball every screen once),
the `lucide`/`window`/`ReactDOM.createRoot` strip-on-port rule, and trailing-slash routing.
