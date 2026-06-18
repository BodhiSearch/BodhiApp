# Kick-off — Batch 3: Models

> Load the shared context first via @common-prompt.md (it lists the framework skills —
> `tanstack-router`, `tanstack-query`, `vercel-react-best-practices`, `web-design-guidelines` —
> consult them on every port), then run the per-batch loop (@process.md §"The per-batch loop").
> Read @batch-2-settings-retro.md and @batch-1-api-keys-retro.md before starting — they carry the
> reusable patterns + gotchas. Scope reminder (@screen-coverage.md): the migration covers the 13
> shell app screens + the bare Access-Request review (done). Batches 0–2 are ✅ (see @tracker.md).

## What Batches 0–2 already give you (don't rebuild)
- **AppShell + chrome** (`components/shell/`, import from `@/components/shell`).
- **`useShellChrome({ breadcrumb, headerActions, sidebar, rail, railHeader, railDefaultOpen })`** —
  publish chrome to the single root `<AppShell>`. **`sidebar` is now a slot** (added Batch 2 for App
  Settings' scroll-spy nav) — use it if a Models screen wants a page-body sub-nav. Pass **stable
  memoized nodes**, never inline component defs. Temporary scaffolding (techdebt.md).
- **List primitives**: `list.css` (`l-*`/`dp-*`), `api-keys.css` (generic page primitives scoped to
  `.api-keys-screen`). Reusable toolbar pieces: **`ShellFilterTabs`** (themed filter pills with
  counts), **`useCollapsibleSearch`** (search button → row, collapses on empty blur). `AppShell`
  auto-opens the rail when a screen publishes rail content.
- **`useViewTransition()`** for rail/selection transitions (now hardened to swallow async `ready`
  rejections — Batch 2). Router cross-fade via `defaultViewTransition`.
- **`BareLayout`** + `BARE_PREFIXES` for standalone screens (Models are all **shell**).
- **Per-screen flags**: `useUiV2Flag('models' | 'new-local-model' | 'new-api-model' |
  'new-fallback-model')` from `@/hooks/useUiV2Flag` (default-old). Branch **inside** the page
  component, keep the `AppInitializer` wrapper outside.

## This batch — Models section (4 screens, all `models` nav)
| Screen | design source | prod route(s) | subPage | flag |
|---|---|---|---|---|
| All Models | `Bodhi Models (All Models)` | `routes/models/index.tsx` | `all-models` | `models` |
| New Local Model | `Create New Local Model` | `routes/models/alias/new/` **+** `…/edit/` | `new-local-model` | `new-local-model` |
| New API Model | `Create API Model` | `routes/models/api/new/` **+** `…/edit/` | `new-api-model` | `new-api-model` |
| New Fallback Alias | `Create Fallback Model` | `routes/models/router/new/` **+** `…/edit/` | `new-fallback-model` | `new-fallback-model` |

> **`/new/` and `/edit/` share one design** (a single form, create vs edit mode). The 3 "new" screens
> are **forms** (a different shape from the Batches-1/2 list+rail screens) — expect `bodhi-form.css`
> (`bf-*`) to need porting (Batch 1 deliberately skipped it). **Consolidations to confirm during
> Explore** (@screen-coverage.md §B/§E): `/models/files/pull/` folds into New Local Model (enter
> `<org>/<repo>` → creating the alias downloads it); `/models/files/` (local files browser) is
> absorbed — **confirm whether it needs any surface** before deleting. The `api_format` field is
> **read-only on edit** (see `crates/bodhi/src/CLAUDE.md` — server enforces it too).

## ⛔ The two HARD GATES still apply (blocking, not optional)
**GATE A — interactive design analysis in Claude-in-Chrome BEFORE planning.** Walk all 4 Models
prototypes live on http://localhost:8000: open the forms, change `api_format`, toggle local-vs-API,
exercise the model/MCP-access pickers, the fallback strategy/targets UI, collapse the sidebar, switch
light/dark, narrow the viewport. **Real-data-only:** if a prototype element has no backing data, omit
it (Batch 1/2 rule — e.g. the New API Model pickers were dropped in Batch 1 when `CreateTokenRequest`
had no fields for them; check what `CreateAlias`/`UpdateAlias`/the router-alias request actually
carry).

**GATE B — live validation before a screen is "done".** RTL + E2E are necessary but NOT sufficient —
Batch 2 shipped a duplicate-React-key warning and an async view-transition `InvalidStateError` that
**every test missed**; only driving the live app + reading the console caught them. For each migrated
screen confirm in Claude-in-Chrome (`make app.run.live`, log in): interactions (form submit, edit
prefill, `api_format` lock-on-edit, fallback target add/remove, pickers); **light AND dark**;
**responsive**; **console clean (0 errors)** on load and on each key interaction.

## Loop steps
0. **Design analysis (GATE A)** — walk all 4 Models prototypes interactively; record behaviors.
1. **Explore** — re-read `routes/models/` (+ `-components/`, `alias/`, `api/`, `router/`, `files/`),
   the `hooks/models/*` hooks + schemas (`schemas/`), the alias/api-model/router request+response
   types from `@bodhiapp/ts-client`, colocated tests + the Models page objects. **Real data only.**
2. **Prerequisites** — port `bodhi-form.css` (`bf-*`) scoped to a per-screen root (Batch-1/2 CSS
   discipline: scope generic class names, rename collisions like `.badge`→`.s-badge`, drop global
   `:root`/`mark`). Port the model/MCP-access picker as production TSX **only if it has real backing
   data** (it didn't in Batch 1 — re-check for Models). No backend/OpenAPI changes expected (confirm).
3. **Plan → refine → approve** — write `batch-3-models-plan.md` (screens, `useShellChrome` props per
   screen, reused hooks/schemas, real-data-only gaps, the `/models/files*` consolidation decision,
   the dialog/page form structure, test list incl. the `ShellSlotsProvider` RTL harness + e2e
   page-object updates, risks). Present, refine, get approval **before** coding.
4. **Implement** behind the flags. **Reuse Batch-1/2 components** — `ShellFilterTabs`,
   `useCollapsibleSearch`, `useShellChrome` (incl. `sidebar` if needed), `useViewTransition`. Forms:
   react-hook-form + zod (existing `schemas/`), wire the **real** create/update mutations; keep
   `api_format` disabled on edit; reuse `convertFormToApi`/`convertApiToForm`.
5. **Migrate tests + e2e** — preserve `data-testid`/ARIA; nav → `navViaShell('models', '<subPage>')`;
   for rail/transition screens set `reducedMotion: 'reduce'` in the page object (Batch-2 fix) and
   wait for the mutation to settle before asserting/reloading. Set the flag via `addInitScript` until
   it's retired.
6. **Validate live (GATE B)** — each screen + interactions in light + dark + responsive, console-clean.
7. **All gates green** (RTL + E2E + GATE B) → retire flags + delete old Models code (+ any now-dead
   `/models/files*` per the consolidation decision) → commit → retro → Batch 4 (MCP) kickoff.

## Carry-forward gotchas (from the Batch-1/2 retros + @process.md)
- **Route tests render the page directly** → wrap in `ShellSlotsProvider` + a slots-consumer to test
  published chrome (extend the consumer to read `sidebar` if a Models screen publishes one).
- **`routeTree.gen.ts` is git-ignored** — regenerated by the Vite plugin; commit route files only.
- **Generic CSS class names collide** — scope ported screen CSS under a per-screen root; rename
  globals (`.badge`/`mark`/`:root`).
- **Be resilient to duplicate/odd backend data** (Batch 2: backend returned a setting twice → dedup).
- **Gate editability/locks on explicit allow-sets**, never infer from data (`api_format` lock,
  editable settings).
- **E2E flakiness is usually OAuth login latency**, not the screen — re-run on a healthy auth server;
  set `reducedMotion: 'reduce'` to kill view-transition detach races.
- **Real-data-only** — omit prototype elements with no backing data; don't ship dead UI.
- Start Docker before E2E; **run the full E2E matrix once** for Models (shared shell/CSS + the
  `sidebar` slot + the `useViewTransition` change are app-wide). The pre-existing chat-resize +
  backend live-test failures are NOT yours (@techdebt.md).
- Strip-on-port greps (no `lucide.createIcons`/`window.*`/`ReactDOM.createRoot`/`data-theme`/
  `TweaksPanel`/`useTweaks`).
