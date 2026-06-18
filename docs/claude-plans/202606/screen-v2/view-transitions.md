# UI V2 Migration — View Transitions (React-18 native)

How we add smooth motion to the V2 screens **without** a React upgrade, and the rules every screen
follows. Added during Batch 1; applies to every later batch.

## Why native, not React's `<ViewTransition>`

The app is on **React 18.3.1**. React's `<ViewTransition>` component + `addTransitionType` are
**React 19 canary only** (not exported in 18) — adopting them would mean a framework-wide upgrade,
out of scope for the per-screen migration. Instead we use the **browser's native View Transitions
API** (`document.startViewTransition`), which is exactly what React 19's component wraps — same
result, React-18-safe, framework-agnostic, graceful degradation.

Reference skill (installed): **`web-animation-view-transitions`** (`agents-inc/skills`) — native API,
React-18-compatible. (The `vercel-react-view-transitions` skill targets React 19 — do **not** follow
its `<ViewTransition>` examples here.)

## The two mechanisms

1. **Route changes → TanStack Router.** `createRouter({ defaultViewTransition: true })` in
   `crates/bodhi/src/main.tsx` wraps every navigation in `document.startViewTransition()`. Animates
   the `root` snapshot (cross-fade). React-18-safe; ignored on unsupported browsers. Covers
   list ↔ `/tokens/new/` ↔ the bare review page.
2. **In-page state changes → `useViewTransition()`** (`crates/bodhi/src/hooks/useViewTransition.ts`).
   A feature-detected, reduced-motion-aware wrapper over `document.startViewTransition`. Wrap the
   state setter whose resulting DOM change should animate:
   ```tsx
   const withViewTransition = useViewTransition();
   const selectToken = useCallback(
     (id) => withViewTransition(() => setSelectedId(id)), [withViewTransition]);
   ```
   Used for the **detail rail open/close** and **filter-tab swaps**. Not for search-as-you-type
   (too rapid) or per-keystroke updates.

CSS recipes live in `crates/bodhi/src/styles/view-transitions.css` (imported once in `__root.tsx`).

## Rules (from the native VT skill — non-negotiable)

1. **Feature-detect** before calling `startViewTransition` (`'startViewTransition' in document`). The
   hook does this; fall back to a plain synchronous update otherwise.
2. **Respect `prefers-reduced-motion`** — the hook skips the transition in JS, AND
   `view-transitions.css` has a `@media (prefers-reduced-motion: reduce)` guard that disables all
   `::view-transition-*` animations. Both layers required.
3. **Unique `view-transition-name`, and only on intent-driven elements** — duplicate names break the
   transition (only one element with a given name may be mounted at a time). We use `vt-rail` (the
   rail panel, mounted only when a row is selected). **Do NOT name a container that re-renders on
   background refetches** (e.g. a list body whose rows update after a mutation) — it races the
   refetch and breaks reads. If a named element could appear in two places at once (a component used
   in both a popover and a page), make the name conditional.
4. **Keep durations < 300ms** (vestibular safety). Use the named constants in `view-transitions.css`
   (`--vt-fade-dur` 180ms, `--vt-rail-dur` 220ms) — **no magic numbers**.
5. **Don't transition the shell grid track** — `grid-template-columns` width is intentionally not
   animated (Chromium sticks the old width; carry-forward risk #1). Animate the rail/list **content**
   via named transitions, never the grid column.
6. **Animate what communicates a spatial relationship.** If you can't say what a transition
   communicates (rail = "detail appeared", filter = "same list, new subset", route = "new place"),
   don't add it.

## What's wired (Batch 1)
- Router-level cross-fade on all navigations (`defaultViewTransition: true`).
- App Tokens list: **detail-rail open/close** (`vt-rail`, slide+fade) — the actual "side panel".

**Deliberately NOT wired: a list-body filter cross-fade (`vt-list`).** A persistent
`view-transition-name` on the always-mounted, frequently-refetching list body races status-toggle /
refetch re-renders and made E2E read stale rows (deterministic, not a flake). Rule learned:
**don't put a `view-transition-name` on a container that re-renders on background data updates** —
reserve named transitions for elements that change only on explicit, discrete user intent (the rail).
Filter swaps just update rows in place.

## Carry-forward for later batches
- Reuse `useViewTransition()` for any rail/detail-panel open-close or tab/filter content swap.
- Reuse the `vt-rail` / `vt-list` names + CSS recipes (they're generic list/rail primitives).
- New shared-element morphs (e.g. a list thumbnail → detail hero) would use a **unique per-item**
  `view-transition-name` (`vt-thing-${id}`) on both the source and target — only if both sides
  exist and the morph communicates "same thing, going deeper." Keep names unique and clean them up.
- Testing: the hook falls back to a synchronous update in jsdom (no `startViewTransition`), so RTL is
  unaffected. E2E (real Chromium) exercises the transitions as progressive enhancement — assert on
  end state via testids, never on animation frames.

## E2E gotcha (learned in Batch 1)
`defaultViewTransition: true` makes **every navigation** cross-fade (~180ms). A page object that
**reads the DOM immediately after a navigation** (e.g. scanning list rows right after returning from
a create page) can race the transition and find a stale/empty set. **Fix: auto-wait for the target
element before scanning** — `await locator.first().waitFor({ state: 'visible' })` (or a filtered
`expect(...).toBeVisible()` on the specific row) — instead of a one-shot `.count()`/`.textContent()`
scan. This is good Playwright hygiene regardless, but the route VT makes it mandatory. See
`TokensPage.findTokenByName`/`expectTokenInList` for the pattern.
