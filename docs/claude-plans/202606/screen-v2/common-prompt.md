# UI V2 Migration — Common Prompt

> Paste/load this at the **start of any session** working on the UI V2 migration. It brings you up
> to date on the task by pulling in the reusable context docs, then points you at the right batch.

You are working on the **UI V2 migration**: porting all 14 BodhiApp screens to the new left-sidebar
**AppShell** design, **one nav section per batch**, via an **iterative, manually-coordinated,
exploratory** process. Read the context below before doing anything.

## Load the context (read these first, in order)

- @context.md — what the task is, goal/end-state, approach, and the doc map.
- @design-reference.md — the `design/` hi-fidelity prototype: view it visually on
  http://localhost:8000 with Claude-in-Chrome; reference it **visually** but build to **our**
  conventions (it is not code to copy).
- @architecture.md — orientation to the production frontend + backend (pointers, not a file map).
- @screen-coverage.md — which app screens have a V2 design, which consolidate, and which are
  design-pending (a live checklist; auth/setup/login/Keycloak are pending).
- @reference-api.md — the external `api.getbodhi.app` reference API + the OAuth id_token mechanism +
  how reference-API needs are handled as per-batch prerequisites.
- @process.md — the per-batch loop, the per-screen recipe, e2e/testid discipline, batch sequence,
  Batch 0 scope, and the carry-forward risks.
- @glossary.md — shared vocabulary.
- @techdebt.md — tracked failing tests + deferred fixes carried across batches.

## Then orient to the current batch

1. Check what's already migrated: list this folder and read the most recent
   `batch-*-retro.md` (learnings carried forward) and the latest `batch-*-kickoff.md`.
2. Open the kick-off for the batch you're starting (the first one is
   @batch-0-foundation-kickoff.md) and follow it — it routes you into the per-batch loop
   (@process.md §"The per-batch loop") starting at **Explore**.

## Operating rules for this task

- **Exploratory, not prescriptive.** The code changes as we progress — read the current tree and
  propose, don't trust snapshots. These docs give the goal and where to look, not a fixed recipe.
- **One section per batch.** Migrate behind a per-screen flag; e2e + tests migrate **within** the
  batch; retire the flag + delete old code as part of the batch; commit per batch.
- **Plan before code.** Each batch: explore → ensure prerequisites → write `batch-N-<section>-plan.md`
  here → get approval → implement → all gate checks green → commit → retro → next-batch kick-off.
- **HARD GATE A — analyze the design interactively before planning.** Do NOT plan from screenshots or
  `*-app.jsx` source alone. Walk each prototype live in Claude-in-Chrome (server on :8000): click
  rows, open detail rails, toggle search/controls, collapse the sidebar, switch light/dark, narrow
  the viewport. Capture the **behaviors** as requirements. (Batch-1 rework came from skipping this.)
- **HARD GATE B — validate live before a screen is "done".** RTL/E2E are necessary but NOT sufficient
  (they miss browser-only runtime errors and visual/theme/responsive regressions). Drive the running
  app in Claude-in-Chrome and confirm: interactions work; **light AND dark**; **responsive**; and
  **console is clean (0 errors/exceptions)** on load and on key interactions. See @process.md + the
  batch-1 retro §"Insights".
- **Build to our conventions.** Vite + TanStack Router/Query + shadcn/ui + `lucide-react` +
  `ThemeProvider`; strip prototype-only idioms on port (see @process.md recipe). Follow root
  `CLAUDE.md`, `crates/CLAUDE.md`, `crates/bodhi/src/CLAUDE.md`.
- **Consult the framework skills on every screen port** (they encode the idioms we build to):
  `tanstack-router` (file-based routing, trailing-slash, pathless/bare layouts, `staticData`),
  `tanstack-query` (query keys, invalidation, derived/`select` data, master-detail from cache),
  `vercel-react-best-practices` (re-render/derived-state/no-inline-component rules), and
  `web-design-guidelines` (accessibility/UX review of the ported UI).
- **View transitions are React-18 native** (this app is React 18, NOT 19). Use `useViewTransition()`
  + TanStack Router `defaultViewTransition` per @view-transitions.md and the
  `web-animation-view-transitions` skill — do NOT use React 19's `<ViewTransition>` component (it
  requires a React upgrade and is not available here).
- **Selectable rows are links.** Every master-detail list/grid row ships the shared `LinkRow` anchor
  as its first child so keyboard / link-hint tools (Vimium) and screen readers can target it — part of
  "done" for any list screen. See the per-screen recipe step in @process.md.
- **Never skip e2e** before commit; keep `data-testid`/ARIA stable across restructures.

If anything in the context is stale or contradicts the current code, trust the code and flag the
drift (update the relevant context doc as part of the batch).
