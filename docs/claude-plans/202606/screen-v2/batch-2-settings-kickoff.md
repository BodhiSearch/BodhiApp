# Kick-off — Batch 2: Settings

> Load the shared context first via @common-prompt.md (it now lists the framework skills —
> `tanstack-router`, `tanstack-query`, `vercel-react-best-practices`, `web-design-guidelines` —
> consult them on every port), then run the per-batch loop (@process.md §"The per-batch loop").
> Read @batch-1-api-keys-retro.md before starting — it carries forward the reusable patterns and the
> gotchas. Scope reminder (@screen-coverage.md): the migration covers the 13 shell app screens + the
> bare Access Request review (now done).

## What Batches 0–1 already give you (don't rebuild)
- **AppShell + chrome** in `crates/bodhi/src/components/shell/` (import from `@/components/shell`).
- **`useShellChrome({ breadcrumb, headerActions, sidebar, rail, railDefaultOpen })`** — publish rich
  chrome to the single root `<AppShell>`. Pass **stable nodes** (memoized), never inline component
  defs (vercel `rerender-no-inline-components`). **Temporary scaffolding** (techdebt.md).
- **`BareLayout`** + `BARE_PREFIXES` for standalone screens (Settings screens are **shell**, so you
  won't need this — but know it exists).
- **Ported CSS**: `api-keys.css` (scoped under `.api-keys-screen`) + `list.css` (`l-*` list / `dp-*`
  detail-panel primitives). **Reuse `list.css` for Manage Users** (it's another list screen). The
  generic page primitives (`page-header`, `filter-tabs`, `empty-state`, `btn-accent`) live in
  `api-keys.css` scoped to `.api-keys-screen` — for Settings, either reuse that class on the screen
  root or port a `settings.css` with the same scoping discipline (decide in the plan).
- **Per-screen flag**: `useUiV2Flag('app-settings' | 'manage-users')` from `@/hooks/useUiV2Flag`.
- **Helper CSS vars** (fonts/shadows/surfaces/easings) are now defined in `globals.css` — safe to use.

## This batch — Settings section (2 screens)
| Screen | design source | prod route | section / subPage | notes |
|---|---|---|---|---|
| App Settings | `Bodhi App Settings.html` + its app jsx | `routes/settings/index.tsx` | settings / app-settings | settings list/editor; **`reference_api_url` is NOT editable** (env/default only) |
| Manage Users | `Manage Users.html` + its app jsx | `routes/users/index.tsx` | settings / manage-users | user list + role management; **shares `list.css`** with the API-Keys lists |

> `/users/pending/` is **retired** (consolidated into `/users/access-requests/`, migrated in Batch 1).
> Confirm during Explore whether `/users/pending/` route + `UserManagementTabs` can now be deleted
> (UserManagementTabs is still used by `/users/` and `/users/pending/` — once both are handled, it
> and the pending route can go).

## Loop steps
1. **Explore** — re-read the current code: `routes/settings/` (+ `-components/`), `routes/users/`,
   the `hooks/settings` + `hooks/users` hooks, colocated tests + the `UsersManagementPage` /
   settings page objects. View the prototypes visually (server on :8000, Claude-in-Chrome). Check
   the real data shape for both screens — **real data only, do NOT increase scope** (the same rule
   that governed Batch 1: if a prototype element has no backing data, omit it).
2. **Prerequisites** — port any remaining shared CSS the Settings screens need (a `settings.css`
   scoped to a `.settings-screen` root if the design uses generic class names; reuse `list.css` for
   Manage Users). No reference-API, no backend changes expected (confirm).
3. **Plan → refine → approve** — write `batch-2-settings-plan.md` (screens, `useShellChrome` props
   per screen, reused hooks, real-data-only gaps, the `UserManagementTabs`/`/users/pending/` cleanup,
   test list incl. the `ShellSlotsProvider` RTL harness + e2e page-object updates, risks). Present,
   refine, get approval before coding.
4. **Implement** behind the flags (recipe @process.md §"Per-screen migration recipe"). Use
   `useShellChrome` for breadcrumb/headerActions; reuse the `ShellSlotsProvider`+slots-consumer RTL
   harness established in Batch 1 (route tests render the page directly, so chrome slots only appear
   inside that harness).
5. **Migrate tests + e2e** — preserve `data-testid`/ARIA; update the settings + users page objects
   (nav → `navViaShell('settings', 'app-settings' | 'manage-users')`); update specs.
6. **All gates green** → retire flags + delete old Settings code (and, if now-dead, the
   `/users/pending/` route + `UserManagementTabs`) → commit → retro → Batch 3 (Models) kickoff.

## Carry-forward gotchas (from @batch-1-api-keys-retro.md + @process.md)
- **Route tests render the page directly** → wrap in `ShellSlotsProvider` + a slots-consumer to test
  published headerActions/rail (the Batch-1 harness).
- **`routeTree.gen.ts` is git-ignored** — committed route files only; regenerate via the Vite plugin
  (or `@tanstack/router-generator`'s `Generator`) when adding routes; e2e picks it up via live Vite.
- **Generic CSS class names collide** — scope ported screen CSS under a per-screen root class.
- **Filter/tab text can collide with content text** in RTL queries — prefer testids over `getByText`.
- **Real-data-only** — no scope increase; omit prototype elements with no backing data.
- Start Docker (PostgreSQL containers up) before E2E; **run the full E2E matrix once** for Settings
  since the shared shell/CSS is global. The 2 pre-existing backend live-test failures are NOT yours.
- Strip-on-port greps (no `lucide.createIcons`/`window.*`/`ReactDOM.createRoot`/`data-theme`).
