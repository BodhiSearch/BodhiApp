# UI V2 Migration — Process Playbook

The migration runs **iteratively, one nav section per batch, manually coordinated**. No frozen
master plan: each batch is **exploratory** — it inspects the current code and what's already
migrated, ensures its prerequisites, then proposes and gets approval for its own plan before coding.
Hand-over between batches happens through this folder.

## Batch sequence

Roadmap risk×traffic order. **Foundation first, Chat last** are fixed; the rest can be re-ranked
between batches if priorities shift.

`Batch 0 Foundation → API Keys → Settings → Models → MCP → Chat`

- **Batch grain = one nav section** (e.g. all of API Keys together), so a section's old code can be
  deleted in one piece when the section lands.
- **Coexistence = per-screen flag.** AppShell is the root layout; each screen renders old or new at
  the **same route** based on a per-screen flag. The flag is retired and the old version deleted when
  the new screen is accepted — **flag cleanup is part of the batch, not "later."**

## The per-batch loop (the core)

Run this for each batch, with manual check-ins between steps:

1. **Explore + interactive design analysis (HARD GATE A)** — load this folder via
   [common-prompt.md](@common-prompt.md); read the prior batch's retro + this batch's kick-off;
   inspect what's already migrated; survey this section's screens (prototype in `design/` ↔ current
   production routes/hooks/tests). **Do NOT plan from screenshots / `*-app.jsx` source alone** —
   *walk each prototype live in Claude-in-Chrome* (server on :8000): click rows, open detail rails,
   toggle search/controls, collapse the sidebar, switch light/dark, narrow the viewport. The
   design's substance is interactive; capture the **behaviors** as requirements. (Batch 1 rework
   came from skipping this — see batch-1 retro §"Insights".)
2. **Prerequisites** — ensure everything this batch depends on is in place **before** implementing:
   shared components/CSS, reference-API interface+spec+stub if any ([reference-api.md](@reference-api.md)),
   backend changes + OpenAPI/ts-client regen if any, per-screen flags wired.
3. **Plan → refine → approve** — write `batch-N-<section>-plan.md` here (screens, AppShell props per
   screen, reused hooks, new pieces, test list, risks). Present it, refine with the user, get
   approval **before** coding.
4. **Implement** — migrate the section's screens behind their flags (recipe below).
5. **Migrate e2e + tests** — update RTL tests and the section's page objects/specs (testid
   discipline below).
6. **Ensure all pass + validate live (HARD GATE B)** — run **all** automated gate checks for what
   changed: `cd crates/bodhi && npm run test` (RTL), `make test.backend` (if Rust changed),
   `make build.ts-client` (if regen), `make test.e2e` (from `crates/lib_bodhiserver/tests-js`). Never
   skip e2e before commit. **Then validate LIVE in Claude-in-Chrome** (`make app.run.live`, log in) —
   RTL/E2E are necessary but NOT sufficient (they miss browser-only runtime errors and
   visual/theme/responsive regressions; Batch 1 shipped an `Illegal invocation` that broke the rail
   while every test passed). For each migrated screen confirm: interactions work (select→rail,
   filter, search, toggles, nav); **light AND dark** render; **responsive** (container-query column
   drops, sidebar collapse, mobile drawers); and **`read_console_messages` shows 0 errors/exceptions**
   on load and on each key interaction. A screen isn't "done" until live validation passes.
7. **Retire flags + commit** — remove this section's flags, delete its old versions/dead code,
   commit per batch (trunk-based, directly on `main`; rebase onto `origin/main` first).
8. **Retrospect** — write `batch-N-<section>-retro.md`: what landed, divergences (folded vs left
   local), surprises, reusable patterns, anything to fix centrally.
9. **Kick-off next batch** — write `batch-(N+1)-<section>-kickoff.md`: an exploratory prompt that
   carries forward learnings + prerequisites so the next session re-enters this loop at step 1.

## Per-screen migration recipe (inside step 4)

Presentation-only — hooks, queries, mutations, fixtures unchanged.

1. Keep all hooks/queries; keep the app-initializer/redirect wrapper outside the shell.
2. Render via the per-screen flag: old version unchanged, **or** the new screen wrapped in AppShell
   (declare `section`/`subPage`/breadcrumb/headerActions, and `sidebar`/`rail`/`contentClass` as the
   design needs).
3. **Dialog → page** where the design dictates (a create flow becomes a full page route under the
   section), wiring the **real** mutation in place of the prototype's sample/fake behavior.
4. Port shared design components as production TSX (e.g. the model/MCP access picker), reusing
   existing primitives.
5. **Strip on every port** (prototype idioms that don't ship): `lucide.createIcons()`,
   `Object.assign(window,…)`/other `window.*` globals, `ReactDOM.createRoot`, `data-theme` setattr
   effects, `TweaksPanel`/`useTweaks`/`EDITMODE` scaffolding. Grep each ported file before "done".
6. **Preserve `data-testid` + ARIA roles verbatim** across the markup restructure.
7. **Selectable rows are links.** Any master-detail row (a list/grid row that opens the detail rail)
   must render the shared `LinkRow` (`@/components/shell` → `components/shell/LinkRow.tsx`) as its
   **first child**: `<LinkRow onActivate={onSelect} label={`Open <entity> ${name}`} />`. This makes
   the row a real `<a>` target so keyboard / link-hint tools (Vimium's `f`) and screen readers can
   reach it. Keep the row's own `onClick={onSelect}` (mouse clicks on static cells). Inner controls
   stay clickable because the control-raising z-index selector in `list.css` (and the `.setting-row`
   copy in `settings.css`) lifts `button/select/input/[role]/[tabindex]` above the stretched link;
   if a row uses a control type not in that selector, extend it. Selection is local state (no URL),
   so the link is `href="#"` + `preventDefault` + `stopPropagation` — `LinkRow` handles all three.
   Non-master-detail rows (e.g. the settings sidebar scroll-spy nav) do **not** get a `LinkRow`.

## E2E migration context

- **Black-box only** — UI interactions, no `page.evaluate`/`page.context` fetch. Suite in
  `crates/lib_bodhiserver/tests-js`; `make test.e2e` (dev-server + Vite HMR, **no ui-rebuild**).
- **E2E is migrated within each batch**, not deferred: update the section's page objects (nav
  switches from old top-nav to the shell nav; any dialog→page renames) and specs; they must pass
  before commit.
- **testid discipline is the linchpin** of presentation-only migration: restructure markup but keep
  `data-testid` + ARIA stable so page objects and RTL queries keep working. Conventions: shell nav
  `data-testid="shell-nav-{section}"` / `shell-sub-{subPage}`; page root
  `data-testid="{domain}-page"` + a `data-pagestatus`.
- **RTL + MSW**: handlers/fixtures reused unchanged (contract identical). Reference-API stubs use the
  external-origin approach in [reference-api.md](@reference-api.md).

## Batch 0 — Foundation (scope summary)

Batch 0 builds what every later batch needs (no screen redesigned). Concrete scope to plan against:
- **Shell**: port `design/bodhi-app-shell.jsx` + its CSS into production TSX (idiomatic to ours).
- **Root layout**: AppShell becomes root; old top-header chrome stops rendering (deleted at the very
  end). **Adoption boundary**: setup/login/request-access/auth routes stay bare; the app-initializer
  stays outside the shell so its redirects still fire.
- **Per-screen flag mechanism**: simple, greppable, default-old; flipped per screen as batches land;
  removed entirely at the end.
- **Token merge**: merge `design/colors_and_type.css` into `globals.css` (preserve prod-only vars
  still referenced). ⚠ this flips the brand `--primary` (purple→lotus-pink) **globally and
  instantly** — do it deliberately and eyeball every screen once for contrast / hardcoded-white-on-
  `bg-primary` regressions.
- **Reference-API client scaffold** (not the APIs): typed `fetch` client + a hook composing base-URL
  (from app-info) + id_token (from current-user). See [reference-api.md](@reference-api.md).
- **Backend**: surface the OAuth id_token to the frontend + add the reference-API endpoint setting +
  regen OpenAPI/ts-client. See [architecture.md](@architecture.md) + [reference-api.md](@reference-api.md).
- **Exit criteria**: every nav-section route renders inside the shell (old content behind a
  default-old flag, correct nav highlight); light/dark works; backend id_token + endpoint land with
  regen; **all existing RTL + e2e tests still pass** (the shell wrapper is transparent to old
  testids); then commit + retro + Batch 1 kick-off.

## Carry-forward risks / watch-outs (every batch)

1. **Never transition `grid-template-columns`** with `var()`/`minmax` tracks — Chromium sticks the
   old width; the shell relies on instant collapse + an inline width var. Don't reintroduce a
   transition there.
2. **Prototype globals → ESM**: strip `lucide`/`window.*`/`ReactDOM.createRoot`/`data-lucide` on
   every port (recipe step 5).
3. **Theme is owned by `ThemeProvider`** — strip residual `data-theme` setattr from ported pages.
4. **Trailing-slash routing** (basepath `/ui`, trailing-slash always): new routes + all nav hrefs
   trailing-slashed; regenerate the route tree when adding routes.
5. **Shell adoption boundary**: keep setup/login/request-access/auth bare; verify the
   app-initializer redirects + any setup-status redirect test still pass after the old header is
   dropped.
6. **Chat is highest-risk and last**: streaming / IndexedDB / abort / MCP tools / self-managed
   scroll vs the shell's full-height grid — validate the scroll region before wiring its rail.
7. **Flag + dead-code cleanup is part of the batch** — a section isn't done until its flags are
   removed and its old code deleted.
8. **Shared files are global** — after touching the shell or shared CSS, re-verify a sample of every
   screen type that consumes them, not just the one in front of you.
