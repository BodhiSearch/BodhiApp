# Plan — Retire the ShellSlotsContext migration scaffolding

**Source brief:** `docs/claude-plans/202606/screen-v2/prompt-shell-remove.md`
**Status:** Plan ready for implementation. Structural refactor, **zero user-visible change**.

---

## Context

During the V2 screen migration, migrated and not-yet-migrated screens had to coexist under one
shell. The seam that made that possible is **`ShellSlotsContext`** (`components/shell/ShellSlotsContext.tsx`):
a screen "publishes" its chrome (breadcrumb / header actions / sidebar / detail rail / layout
overrides) *upward* through a context to a single `<AppShell>` owned by `routes/__root.tsx`'s
`RootShell`, which consumes the published slots and spreads them onto that one shell. A central
pathname-prefix switch (`components/shell/resolveShellRoute.ts`) decides bare vs fullscreen vs
app-shell. Both are explicitly self-described as **temporary migration scaffolding** (see the file
headers and `screen-v2/techdebt.md` lines 99–117).

The migration is now complete: every screen is V2, no flags remain, there is no V1/V2 coexistence.
The scaffolding still works but is non-obvious indirection every new screen must learn. This plan
removes it and replaces it with the **idiomatic end-state the migration always intended**.

### Why this is a relocate-and-lean, not a delete (the honest verdict)

The user asked for the cleanest *modular/maintainable* answer grounded in React best practices, not
just "make it work." A deep-research pass (TanStack Router docs + React docs + engineering blogs,
25 claims adversarially verified 3-0) settled the design space decisively:

- **TanStack Router has no named/multiple/parallel outlets** and its `<Outlet/>` takes **no props**
  (Discussion #605 open since 2023; RFC #6302 only a Jan-2026 draft — nothing shipped). So a child
  route **cannot** route distinct chrome into separate ancestor slots via the router.
- **`staticData` + `useMatches()`** is the correct, documented idiom — but **only for static
  chrome** (section id, breadcrumb labels, layout flag). It is fixed at route creation and
  identical for all instances; it **structurally cannot** carry live React nodes that depend on a
  screen's runtime state (e.g. a detail rail for the selected row).
- **An ancestor-owned React Context — split into value + setter, with memoized nodes — IS the
  idiomatic React answer** for dynamic, state-dependent cross-Outlet slot filling. It is *not* an
  anti-pattern; portals and slot-fill libraries (react-slot-fill, Gutenberg SlotFill) reduce to
  "context/portal under the hood" and add no value here.

**Hard constraint that pins the whole design:** `components/shell/AppShell.tsx` is **not a dumb
layout**. It owns navigation-persistent `useState` (sidebar `collapsed`, `railCollapsed`, mobile
drawers, `dragging`, `openPop`, `isMobile`), a `useEffect([])` that restores persisted column
widths from `localStorage` once on mount, and it **provides `ShellContext`** (`openRail`/`closeRail`/
`collapseRail`/`collapsed`/…) consumed by screens (`chat/-components/ChatHistorySidebar.tsx`,
`models/-components/ModelsScreenV2.tsx`, `models/explore/local/-components/LocalDiscoveryScreen.tsx`,
`models/router/-components/RouterInfoRail.tsx`). A **per-route `<AppShell>` is therefore WRONG** — it
would remount on every navigation, drop collapse/rail/width state, re-run the width-restore effect
(visual flicker), and churn the `openRail` identities that `AppShell.test.tsx` asserts are stable
(see memory `feedback_appshell_context_stability`). **The end-state must keep ONE persistent
`<AppShell>` across all app-route navigations.**

**Conclusion:** the slots context cannot be *deleted* — a child→ancestor publish is structurally
required because a screen's rail/sidebar render in AppShell columns *outside* the screen's `<Outlet/>`
subtree. It can be **relocated** (owned by a persistent `_app` layout route, not `__root`),
**leaned** (delete the coexistence-only `section`/`resizeKey` fallback indirection; move static
section/subPage to `staticData`/`useMatches()`), and **re-framed** (renamed off "scaffolding"). The
two-context split *stays* — it is the React-recommended structure, not cruft. After this, a screen
no longer publishes to "a shell it doesn't own during coexistence"; it parameterizes the idiomatic
persistent shell via the documented React mechanism.

---

## Target mechanism (chosen — revised after implementation reality-check)

**`__root`'s `RootShell` continues to render the single persistent `<AppShell>` for app routes —
no file moves, no pathless `_app` route — but the coexistence indirection is leaned out: `section`/
`subPage` come from route `staticData` via `useMatches()`, screens stop publishing `section`, and
the `slots.section ?? resolved.section` fallback is deleted.**

> **Why not a pathless `_app` route?** Investigated and rejected on cost. With the TanStack
> router-plugin (v1.167) + file-based routing, a pathless `_app` layout requires physically moving
> every app route into `routes/_app/`; on codegen the plugin rewrites each route id to `/_app/...`,
> forcing every typed `from: '/chat/'` / `getRouteApi('/models/')` / route-keyed literal across ~20
> screens to change. That churn buys only "the layout switch is router structure instead of a
> `__root` branch" — a marginal gain over a *leaned* branch. We keep `RootShell` as the single
> persistent-AppShell owner (its mount lifetime is already correct — it lives on the root route and
> never remounts across navigations) and spend the effort on removing the indirection, not on
> relocating files. This still delivers every real goal: one persistent shell, `staticData`-driven
> static chrome, a leaned+renamed context, and a shared test harness.

```
routes/
  __root.tsx → ThemeProvider > ClientProviders > NavigationProvider > ShellChromeProvider
               > { RootShell + Toaster }.
               RootShell (unchanged mount lifetime, leaned body):
                 - reads section/subPage from useMatches() staticData (NOT resolveShellRoute)
                 - useGetUser({ enabled: inAppShell }) + logout + shellUser  (gating unchanged)
                 - branch: fullscreen → <Outlet/>; bare → <BareLayout><Outlet/></BareLayout>;
                           else → <AppShell section={static} subPage={static} {...slots}><Outlet/></AppShell>
  chat/index.tsx, models/…, mcps/…, etc.  — files stay put; ids unchanged; add staticData per route.
  setup/route.tsx — fullscreen layout route, unchanged.
```

`routeTree.gen.ts` is **never hand-edited** — the Vite plugin regenerates it. We add a typed
`staticData` shape (declaration-merge `StaticDataRouteOption`) carrying `{ section?, subPage? }` and
set it per app route via `createFileRoute(...)({ staticData: { section: 'chat' }, … })`.

### Layout switch (bare/fullscreen) — kept in `__root`, leaned

`resolveShellRoute.ts`'s **bare/fullscreen predicates** (`isBareRoute`/`isFullscreenRoute`) stay —
they are the layout switch and work fine. What's **deleted** is its **section resolution**
(`resolveShellRoute()` longest-prefix match against `SHELL_NAV`) — superseded by `staticData`. So
`resolveShellRoute.ts` shrinks to the two predicates (or its section-resolver is removed); the file
is not deleted. Folding the bare switch into route-declared `staticData.layout` is an **optional
later follow-up**, not required by this effort (per user: core change first, E2E-validated).

### `useGetUser` gating — unchanged

`RootShell` keeps `useGetUser({ enabled: inAppShell })` exactly as today (pathname predicate via
`isBareRoute`/`isFullscreenRoute`). No structural change needed since we're not splitting routes.

---

## Critical files

- `routes/__root.tsx` — `RootShell` body leaned: read section/subPage from `useMatches()`/`staticData`;
  drop the `slots.section ?? resolved.section` / `slots.resizeKey ?? resolved.section` fallback
  indirection; keep the persistent `<AppShell>` mount and the bare/fullscreen branch.
- Each app route file (`chat/index.tsx`, `models/index.tsx`, …) — add `staticData: { section, subPage? }`
  to its `createFileRoute(...)` options; stop publishing `section` via `useShellChrome`.
- A typed `staticData` declaration (e.g. in `lib/` or `main.tsx`) declaration-merging
  `StaticDataRouteOption` with `{ section?: string; subPage?: string | null }`.
- `components/shell/AppShell.tsx` — the persistent-state owner. **Do not touch its internals**; it
  stays mounted by `RootShell` (mount lifetime already correct).
- `components/shell/ShellChromeContext.tsx` — drop `section` from the interface + `useShellChrome` memo
  (done in this phase); keep the value/setter split.
- `components/shell/resolveShellRoute.ts` (+ `.test.ts`) — section-resolver removed; predicates kept.
- `test-utils/shell-harness.tsx` (new) — shared harness replacing 15 per-file `SlotsConsumer` copies.
- `test-utils/router-harness.tsx` — existing `makeRouteRouter`/`RouteHarness`, reused by the new harness.

---

## Shared test harness

**Problem:** 15 route test files each copy a local `SlotsConsumer` that reads `useShellSlots()` into
`data-testid="harness-*"` divs so tests can assert the published breadcrumb / sidebar facets / rail.
(2 more — `ShellSlotsContext.test.tsx`, `AppShell.test.tsx` — are unit tests of the primitives and
stay with their own probes.)

**Design — `crates/bodhi/src/test-utils/shell-harness.tsx` (new):**

```tsx
export function ShellHarness({ children }: { children: ReactNode }): JSX.Element;
//  <ShellChromeProvider><ChromeProbe/><ShellContext.Provider value={wired}>{children}…
//  Emits the SAME testids every per-file probe used (superset):
//    harness-breadcrumb, harness-header-actions, harness-sidebar, harness-rail, harness-rail-header

export function renderScreenInShell({ path, validateSearch, Screen, initialEntries });
//  = makeRouteRouter() with the screen's route component wrapped in <ShellHarness>.
```

Two responsibilities the shared harness owns that the per-file copies only half-covered:
1. **Provide a real `ShellContext`** (wire `openRail`/`closeRail`/`collapseRail` to local state) so
   rail-consuming screens (`ModelsScreenV2`, `LocalDiscoveryScreen`, `RouterInfoRail`,
   `ChatHistorySidebar`) don't hit the default no-op `useShell()`. Makes coverage uniform.
2. **Render the chrome probe** — assertions stay byte-for-byte identical.

**Migration recipe per file:** delete the local `SlotsConsumer`/`ShellHarness`; import `ShellHarness`
from `@/test-utils/shell-harness` (replacing the `ShellSlotsProvider` import); wrap with
`<ShellHarness>` (including the auth/init wrappers); the `harness-*` assertions are unchanged.

**Migrate (15):** `settings/index`, `models/index`, `models/explore/{providers,local,api}/index`,
`models/api/{edit,new}/index`, `models/router/-components/ModelRouterForm`, `models/router/new/index`,
`mcps/index`, `mcps/explore/index`, `users/index`, `users/access-requests/index`, `tokens/index`,
`tokens/new/index`. **Do NOT migrate (2):** `ShellSlotsContext.test.tsx` (update only for rename +
`section` removal), `AppShell.test.tsx` (unchanged).

---

## Phased rollout

Gate every phase: `cd crates/bodhi && npm test` + typecheck; `make format` before each commit; suite
green between commits. Section order (per house rules + memory `feedback_phased_vertical_slice_dev`):
chat → models → mcps → settings → tokens → users. Grow ONE shell E2E spec (many `test.step`) across
phases. Set `reducedMotion:'reduce'` on rail/view-transition E2E (memory
`feedback_e2e_reduced_motion_for_view_transitions`). E2E from `lib_bodhiserver/tests-js` via
`make test.e2e` (no ui-rebuild — dev-server + Vite HMR).

**Phase 0 — Rename + de-scaffold the context (no router change).**
Rename `ShellSlotsContext.tsx` → `ShellChromeContext.tsx` (keep symbol names `useShellChrome`/
`useShellSlots`/`ShellSlots` to minimize churn). Drop the "TEMPORARY scaffolding / delete whole file"
docstring; keep the two-context split. Update barrel `components/shell/index.ts` + the unit test
import. Gate. **No E2E.** Commit.

**Phase 1 — Vertical slice: `staticData` section + chat (the core change).**
Add the typed `staticData` shape (declaration-merge `StaticDataRouteOption` → `{ section?, subPage? }`).
In `__root`'s `RootShell`: read `section`/`subPage` from `useMatches()` (last match with `staticData.section`)
instead of `resolveShellRoute()`; **delete** the `slots.section ?? resolved.section` /
`slots.resizeKey ?? resolved.section` fallback; keep `contentClass:'flush'` default and the
persistent `<AppShell>` + bare/fullscreen branch. Add `staticData:{ section:'chat' }` to
`chat/index.tsx`'s `createFileRoute`; **remove `section` from `ShellSlots`** + `useShellChrome` memo
+ chat's publish. Introduce `test-utils/shell-harness.tsx`. Verify in Chrome (`make app.run.live`):
chat rail opens/closes, sidebar resize persists, scroll/flush intact, nav highlight correct.
**Gate + chat shell E2E** (rail/sidebar/view-transition). Commit.

**Phases 2–5 — Section staticData + test-harness, one commit each:** models → mcps → settings →
tokens → users. Per section: add `staticData.{section,subPage?}` to each route's `createFileRoute`;
drop `section` publishes; **keep** real per-screen overrides where screens set them
(`railWidth`/`railScroll`/`resizeKey` — grep first per memory `feedback_plan_verification`); migrate
that section's test files to `ShellHarness`. **Gate; run E2E at any section end that touches the
rail** (models: `ModelsScreenV2.openRail`, `LocalDiscoveryScreen`, `RouterInfoRail.collapseRail`).
After the last section, remove `resolveShellRoute()`'s now-unused section-resolver (keep the
bare/fullscreen predicates).

**Phase 6 — (optional follow-up, route-declared layout).**
Only if desired after the core lands: replace `__root`'s bare/fullscreen pathname predicates with
route-declared `staticData.layout: 'app' | 'bare' | 'fullscreen'` read via `useMatches()`, removing
the `BARE_PREFIXES`/`FULLSCREEN_PREFIXES` arrays. **Gate + E2E** for login/setup/request-access/
oauth-callback; assert **zero `/bodhi/v1/user` fetch** on bare/setup (black-box, per memory
`feedback_blackbox_e2e`). Defer-able — not required for the scaffolding removal to be complete.

**Phase 7 — Tidy + full E2E.**
Purge `screen-v2/techdebt.md` lines 99–117 (mark scaffolding removed) and stale "coexistence root
shell" / "migrated screen" comments. Update `crates/bodhi/src/CLAUDE.md` "Root layout" / "Layout
routes" sections to describe `_app`/`_bare`/`staticData`. **Gate + full Playwright E2E**
(reduced-motion included). Commit.

---

## Re-render discipline (must not regress)

- **Keep the value/setter split** — publishers subscribe only to the stable `useCallback` setter;
  only `_app.tsx` reads the value context.
- **Keep `useShellChrome`'s `useMemo` + publish `useEffect`**; screens keep passing memoized nodes.
- **Keep AppShell's `openRail`/`closeRail`/`collapseRail` stable `useCallback`s** — `AppShell.test.tsx`
  asserts this; relocation doesn't touch AppShell internals, so it's preserved by construction.
- **Mount-lifetime guarantee:** TanStack keeps parent layout routes mounted across child navigations,
  swapping only `<Outlet/>`. Moving `<AppShell>` from `__root` to `_app.tsx` does **not** change its
  mount lifetime — this is the mechanism that satisfies the persistent-shell constraint. (Consult the
  `vercel-react-best-practices` skill during implementation.)

---

## Risks & mitigations

- **View-transition `InvalidStateError` carry-forward** (`techdebt.md`): keep `__root` rendering a
  single `<Outlet/>` (one `view-transition-name: root` snapshot); do NOT wrap `_app`/`_bare` outlets
  in extra animated containers. The existing `useViewTransition` wrapper already swallows the sync +
  async rejection. E2E: navigate during a rail toggle, assert no uncaught error.
- **Chat's `flush`/scroll/width overrides** (`mainScroll:false`, `railScroll:false`,
  `contentClass:'flush'`, `railWidth:360`, `sidebarWidth:260`, `resizeKey:'chat'`): stay as slot
  publishes (dynamic overrides, not `section`). `_app`'s `contentClass:'flush'` default preserves
  what non-chat screens got from `slots.contentClass ?? 'flush'`. E2E chat scroll region.
- **Resize-handle z-index deferred bug** (`techdebt.md` chat `chat.spec.mjs:72`): out of scope. This
  refactor does not touch `shell.css` or AppShell JSX — carry it forward untouched; do not "fix" here.
- **`useGetUser` gating**: now structural. A bare route mis-placed under `_app/` would fetch when
  logged out — the Phase 6 black-box network assertion catches it.
- **Reduced-motion E2E races**: keep chat's `withViewTransition` untouched; full reduced-motion E2E
  in Phase 7.
- **`routeTree.gen.ts` drift**: never hand-edit; regenerate via the Vite plugin; typecheck gate
  catches bad `staticData` types.
- **Shared-harness behavioral gap**: the new `ShellHarness` adds a real `ShellContext` some files
  omitted. Migrate one section's tests, run, diff failures before fanning out.

---

## Verification (end-to-end)

1. **Unit/typecheck per phase:** `cd crates/bodhi && npm test` + typecheck — green between every commit.
2. **Chrome manual (Phase 1 + each rail section):** `make app.run.live`; verify rail open/close,
   sidebar collapse + column-width persistence **across navigation** (models → tokens → back), chat
   scroll/flush, breadcrumbs, header actions — all visually identical to today.
3. **E2E per phase end** (rail/shell specs) and **full suite Phase 7**: `make test.e2e` from
   `crates/lib_bodhiserver`, `reducedMotion:'reduce'` on view-transition specs. Black-box only.
4. **Regression guard:** no `data-testid` changed, no visual diff — if the user sees a change, stop.
